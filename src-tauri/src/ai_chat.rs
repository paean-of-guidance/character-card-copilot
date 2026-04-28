use crate::ai_cancellation::ActiveCancellationRequest;
use crate::ai_tools::{ToolCallRequest, ToolDefinition};
use crate::api_config::{ApiConfig, ApiProvider};
use crate::backend::application::event_bus::EventBus;
use crate::backend::domain::{ReasoningDeltaKind, ToolExecutionPhase};
use crate::tools::ToolRegistry;
use futures_util::StreamExt;
use genai::adapter::AdapterKind;
use genai::chat::{
    ChatMessage as GenAiChatMessage, ChatOptions as GenAiChatOptions,
    ChatRequest as GenAiChatRequest, ChatResponse as GenAiChatResponse,
    ChatStreamEvent as GenAiChatStreamEvent, StreamEnd as GenAiStreamEnd, Tool as GenAiTool,
    ToolCall as GenAiToolCall, ToolResponse as GenAiToolResponse,
};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 聊天消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallData>>,
    pub tool_call_id: Option<String>,
}

/// 工具调用数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallData {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunctionData,
    pub thought_signatures: Option<Vec<String>>,
}

/// 工具调用函数数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunctionData {
    pub name: String,
    pub arguments: String,
}

/// 工具调用偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    String(String),
    Function {
        #[serde(rename = "type")]
        choice_type: String,
        function: ToolTarget,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTarget {
    pub name: String,
}

/// 停止序列
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StopSequence {
    Single(String),
    Multiple(Vec<String>),
}

/// 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 聊天完成选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
}

/// 聊天完成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    #[serde(rename = "system_fingerprint")]
    pub system_fingerprint: Option<String>,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intermediate_messages: Option<Vec<ChatMessage>>,
}

/// 聊天完成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stop: Option<StopSequence>,
    pub stream: Option<bool>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone)]
struct ToolExecutionOutput {
    tool_message: ChatMessage,
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
    execution_time_ms: u64,
}

pub const AI_RESPONSE_INTERRUPTED_ERROR: &str = "AI 响应已中断";

#[derive(Debug, Clone)]
pub struct AbortedGeneration {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub intermediate_messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone)]
pub enum AIChatError {
    Aborted(AbortedGeneration),
    Failed(String),
}

impl AIChatError {
    fn failed(message: impl Into<String>) -> Self {
        Self::Failed(message.into())
    }
}

impl From<String> for AIChatError {
    fn from(value: String) -> Self {
        AIChatError::Failed(value)
    }
}

pub struct AIChatService;

impl AIChatService {
    fn normalize_endpoint_for_genai(base_url: &str) -> String {
        if base_url.ends_with('/') {
            base_url.to_string()
        } else {
            format!("{base_url}/")
        }
    }

    fn create_client_with_config(api_config: &ApiConfig) -> Client {
        let provider = api_config.provider;
        let base_url = Self::normalize_endpoint_for_genai(&api_config.base_url);
        let api_key = api_config.api_key.clone();

        let target_resolver = ServiceTargetResolver::from_resolver_fn(
            move |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
                let model_name = service_target.model.model_name;
                let endpoint = Endpoint::from_owned(base_url.clone());
                let auth = AuthData::from_single(api_key.clone());
                let model = ModelIden::new(provider.adapter_kind(), model_name);
                Ok(ServiceTarget {
                    endpoint,
                    auth,
                    model,
                })
            },
        );

        Client::builder()
            .with_service_target_resolver(target_resolver)
            .build()
    }

    fn build_options(request: &ChatCompletionRequest) -> GenAiChatOptions {
        let mut options = GenAiChatOptions::default()
            .with_capture_raw_body(true)
            .with_capture_usage(true)
            .with_capture_content(true)
            .with_capture_reasoning_content(true)
            .with_normalize_reasoning_content(true)
            .with_capture_tool_calls(true);

        if let Some(temperature) = request.temperature {
            options = options.with_temperature(temperature);
        }
        if let Some(max_tokens) = request.max_tokens {
            options = options.with_max_tokens(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            options = options.with_top_p(top_p);
        }
        if let Some(stop) = &request.stop {
            let sequences = match stop {
                StopSequence::Single(value) => vec![value.clone()],
                StopSequence::Multiple(values) => values.clone(),
            };
            options = options.with_stop_sequences(sequences);
        }

        options
    }

    fn join_system_messages(messages: &[ChatMessage]) -> Option<String> {
        let combined = messages
            .iter()
            .filter(|message| {
                message.role == MessageRole::System && !message.content.trim().is_empty()
            })
            .map(|message| message.content.trim())
            .collect::<Vec<_>>()
            .join("\n\n");

        if combined.is_empty() {
            None
        } else {
            Some(combined)
        }
    }

    fn convert_messages_to_genai(messages: &[ChatMessage]) -> Vec<GenAiChatMessage> {
        messages
            .iter()
            .filter_map(|message| match message.role {
                MessageRole::System => None,
                MessageRole::User => Some(GenAiChatMessage::user(message.content.clone())),
                MessageRole::Assistant => {
                    let mut assistant_message =
                        GenAiChatMessage::assistant(message.content.clone())
                            .with_reasoning_content(message.reasoning_content.clone());

                    if let Some(tool_calls) = &message.tool_calls {
                        let tool_calls = tool_calls
                            .iter()
                            .filter_map(Self::convert_tool_call_to_genai)
                            .collect::<Vec<_>>();

                        for tool_call in tool_calls {
                            assistant_message.content.push(tool_call);
                        }
                    }

                    Some(assistant_message)
                }
                MessageRole::Tool => message.tool_call_id.as_ref().map(|tool_call_id| {
                    GenAiChatMessage::from(GenAiToolResponse::new(
                        tool_call_id.clone(),
                        message.content.clone(),
                    ))
                }),
            })
            .collect()
    }

    fn convert_tool_definitions(tools: &[ToolDefinition]) -> Vec<GenAiTool> {
        tools
            .iter()
            .filter(|tool| tool.tool_type == "function")
            .map(|tool| {
                let mut genai_tool = GenAiTool::new(tool.function.name.clone());

                if let Some(description) = &tool.function.description {
                    genai_tool = genai_tool.with_description(description.clone());
                }

                if let Some(parameters) = &tool.function.parameters {
                    if let Ok(schema) = serde_json::to_value(parameters) {
                        genai_tool = genai_tool.with_schema(schema);
                    }
                }

                genai_tool
            })
            .collect()
    }

    fn convert_tool_call_to_genai(tool_call: &ToolCallData) -> Option<GenAiToolCall> {
        let fn_arguments = serde_json::from_str(&tool_call.function.arguments).ok()?;
        Some(GenAiToolCall {
            call_id: tool_call.id.clone(),
            fn_name: tool_call.function.name.clone(),
            fn_arguments,
            thought_signatures: tool_call.thought_signatures.clone(),
        })
    }

    fn convert_tool_call_from_genai(tool_call: &GenAiToolCall) -> ToolCallData {
        ToolCallData {
            id: tool_call.call_id.clone(),
            call_type: "function".to_string(),
            function: ToolCallFunctionData {
                name: tool_call.fn_name.clone(),
                arguments: tool_call.fn_arguments.to_string(),
            },
            thought_signatures: tool_call.thought_signatures.clone(),
        }
    }

    fn convert_usage(usage: &genai::chat::Usage) -> Usage {
        let prompt_tokens = usage.prompt_tokens.unwrap_or_default().max(0) as u32;
        let completion_tokens = usage.completion_tokens.unwrap_or_default().max(0) as u32;
        let total_tokens = usage
            .total_tokens
            .unwrap_or((prompt_tokens + completion_tokens) as i32)
            .max(0) as u32;

        Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }

    fn convert_response_from_genai(response: &GenAiChatResponse) -> ChatCompletionResponse {
        let tool_calls = response
            .tool_calls()
            .into_iter()
            .map(Self::convert_tool_call_from_genai)
            .collect::<Vec<_>>();

        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: response.first_text().unwrap_or_default().to_string(),
            name: None,
            reasoning_content: response.reasoning_content.clone(),
            tool_calls: (!tool_calls.is_empty()).then_some(tool_calls),
            tool_call_id: None,
        };

        ChatCompletionResponse {
            id: uuid::Uuid::new_v4().to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: response.model_iden.model_name.to_string(),
            system_fingerprint: None,
            choices: vec![ChatCompletionChoice {
                index: 0,
                finish_reason: if message.tool_calls.is_some() {
                    "tool_calls".to_string()
                } else {
                    "stop".to_string()
                },
                message,
            }],
            usage: Self::convert_usage(&response.usage),
            intermediate_messages: None,
        }
    }

    async fn execute_tool_call(
        app_handle: &tauri::AppHandle,
        tool_call: &ToolCallData,
    ) -> ToolExecutionOutput {
        let parameters =
            match serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments) {
                Ok(serde_json::Value::Object(map)) => map.into_iter().collect::<HashMap<_, _>>(),
                Ok(_) | Err(_) => HashMap::new(),
            };

        let request = ToolCallRequest {
            tool_name: tool_call.function.name.clone(),
            parameters,
            character_uuid: crate::character_state::CHARACTER_STATE.get_current_character(),
            context: None,
        };

        let result = ToolRegistry::execute_tool_call_global(app_handle, &request).await;
        let data = result.data.clone();
        let error = result.error.clone();
        let tool_result = Self::format_tool_message(
            &tool_call.function.name,
            result.success,
            data.as_ref(),
            error.as_deref(),
        );

        ToolExecutionOutput {
            tool_message: ChatMessage {
                role: MessageRole::Tool,
                content: tool_result,
                name: Some(tool_call.function.name.clone()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: Some(tool_call.id.clone()),
            },
            success: result.success,
            data,
            error,
            execution_time_ms: result.execution_time_ms,
        }
    }

    fn format_tool_message(
        tool_name: &str,
        success: bool,
        data: Option<&serde_json::Value>,
        error: Option<&str>,
    ) -> String {
        if !success {
            return Self::format_tool_error(tool_name, error, data);
        }

        match tool_name {
            "patch_character_field" => Self::format_patch_character_field_result(data),
            "read_character_field" => Self::format_read_character_field_result(data),
            "edit_character" => Self::format_edit_character_result(data),
            "list_world_book_entries" => Self::format_list_world_book_entries_result(data),
            "read_world_book_entry" => Self::format_read_world_book_entry_result(data),
            "create_world_book_entry" => Self::format_create_world_book_entry_result(data),
            "update_world_book_entry" => Self::format_update_world_book_entry_result(data),
            "delete_world_book_entry" => Self::format_delete_world_book_entry_result(data),
            _ => Self::format_generic_tool_result(tool_name, data),
        }
    }

    fn format_tool_error(
        tool_name: &str,
        error: Option<&str>,
        data: Option<&serde_json::Value>,
    ) -> String {
        let mut lines = vec![format!("failed {tool_name}")];

        if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
            lines.push(error.to_string());
        }

        if tool_name == "patch_character_field" {
            if let Some(details) = data
                .and_then(Self::as_object)
                .and_then(|object| object.get("details"))
            {
                let patch_lines = Self::format_patch_failure_details(details);
                if !patch_lines.is_empty() {
                    lines.extend(patch_lines);
                    return lines.join("\n");
                }
            }
        }

        if let Some(data) = data {
            let rendered = Self::render_value_lines(data);
            if !rendered.is_empty() {
                lines.extend(rendered);
            }
        }

        lines.join("\n")
    }

    fn format_patch_character_field_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data.and_then(Self::as_object) else {
            return "ok patch_character_field".to_string();
        };

        let field = Self::get_string(data, "field");
        let operation = Self::get_string(data, "operation");
        let match_mode = Self::get_string(data, "match_mode");
        let dry_run = data.get("dry_run").and_then(|value| value.as_bool());

        let mut meta = Vec::new();
        if let Some(field) = field {
            meta.push(format!("field={field}"));
        }
        if let Some(operation) = operation {
            meta.push(format!("op={operation}"));
        }
        if let Some(match_mode) = match_mode {
            meta.push(format!("match={match_mode}"));
        }
        if let Some(dry_run) = dry_run {
            meta.push(format!("dry_run={dry_run}"));
        }

        let matched_context = data.get("matched_context").and_then(Self::as_object);
        let updated_context = data.get("updated_context").and_then(Self::as_object);
        let before_lines = matched_context
            .map(|context| Self::build_context_diff_lines(context, "matched_text", "-"))
            .unwrap_or_default();
        let after_lines = updated_context
            .map(|context| Self::build_context_diff_lines(context, "selected_text", "+"))
            .unwrap_or_default();

        if !before_lines.is_empty() || !after_lines.is_empty() {
            let mut diff_lines = before_lines;
            diff_lines.extend(after_lines);
            return Self::format_change_result(
                "patch_character_field",
                meta,
                Some(Self::format_fenced_block("diff", &diff_lines.join("\n"))),
            );
        }

        let body = Self::get_string(data, "updated_preview");
        Self::format_change_result("patch_character_field", meta, body)
    }

    fn format_read_character_field_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data.and_then(Self::as_object) else {
            return "ok read_character_field".to_string();
        };

        let mut meta = Vec::new();
        let mut label = "text".to_string();
        let mut text_body = None;
        if let Some(field) = Self::get_string(data, "field") {
            label = field.clone();
            let start = data
                .get("start")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            let end = data
                .get("end")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            let total = data
                .get("total_length")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            let truncated = data
                .get("truncated")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            meta.push(format!(
                "field={field} | range={start}..{end}/{total}{}",
                if truncated { " | truncated" } else { "" }
            ));
            if let Some(text) = Self::get_string(data, "text") {
                text_body = Some(text);
            }
        } else if let Some(text) = Self::get_string(data, "text") {
            text_body = Some(text);
        }

        Self::format_read_result("read_character_field", meta, &label, text_body.as_deref())
    }

    fn format_edit_character_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data.and_then(Self::as_object) else {
            return "ok edit_character".to_string();
        };

        let updated_fields = data
            .get("updated_fields")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(Self::as_object)
                    .filter_map(|item| Self::get_string(item, "field"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if updated_fields.is_empty() {
            return "ok edit_character".to_string();
        }

        Self::format_change_result(
            "edit_character",
            vec![format!("updated={}", updated_fields.len())],
            Some(format!("fields: {}", updated_fields.join(", "))),
        )
    }

    fn format_list_world_book_entries_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data else {
            return "ok list_world_book_entries".to_string();
        };

        Self::format_structured_result("list_world_book_entries", Vec::new(), data)
    }

    fn format_read_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data else {
            return "ok read_world_book_entry".to_string();
        };

        Self::format_structured_result("read_world_book_entry", Vec::new(), data)
    }

    fn format_create_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data.and_then(Self::as_object) else {
            return "ok create_world_book_entry".to_string();
        };

        let mut meta = Vec::new();
        if let Some(entry_id) = data.get("entry_id").and_then(|value| value.as_i64()) {
            meta.push(format!("entry_id={entry_id}"));
        }
        if let Some(name) = Self::get_string(data, "entry_name") {
            meta.push(format!("name={name}"));
        }
        Self::format_change_result(
            "create_world_book_entry",
            meta,
            Some(Self::format_yaml_block(&serde_json::Value::Object(
                data.clone(),
            ))),
        )
    }

    fn format_update_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data.and_then(Self::as_object) else {
            return "ok update_world_book_entry".to_string();
        };

        let mut meta = Vec::new();
        if let Some(matched_by) = Self::get_string(data, "matched_by") {
            meta.push(format!("matched_by={matched_by}"));
        }
        if let Some(matched_value) = data.get("matched_value") {
            meta.push(format!(
                "matched_value={}",
                Self::value_to_inline(matched_value)
            ));
        }
        if let Some(updated_fields) = data
            .get("updated_fields")
            .and_then(Self::value_as_string_list)
        {
            meta.push(format!("updated={}", updated_fields.join(", ")));
        }

        Self::format_change_result(
            "update_world_book_entry",
            meta,
            Some(Self::format_yaml_block(&serde_json::Value::Object(
                data.clone(),
            ))),
        )
    }

    fn format_delete_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
        let Some(data) = data.and_then(Self::as_object) else {
            return "ok delete_world_book_entry".to_string();
        };

        let mut meta = Vec::new();
        if let Some(matched_by) = Self::get_string(data, "matched_by") {
            meta.push(format!("matched_by={matched_by}"));
        }
        if let Some(matched_value) = data.get("matched_value") {
            meta.push(format!(
                "matched_value={}",
                Self::value_to_inline(matched_value)
            ));
        }
        Self::format_change_result(
            "delete_world_book_entry",
            meta,
            Some(Self::format_yaml_block(&serde_json::Value::Object(
                data.clone(),
            ))),
        )
    }

    fn format_generic_tool_result(tool_name: &str, data: Option<&serde_json::Value>) -> String {
        let mut lines = vec![format!("ok {tool_name}")];
        if let Some(data) = data {
            let rendered = Self::render_value_lines(data);
            if !rendered.is_empty() {
                lines.extend(rendered);
            }
        }
        lines.join("\n")
    }

    fn format_patch_failure_details(details: &serde_json::Value) -> Vec<String> {
        let Some(details) = Self::as_object(details) else {
            return Vec::new();
        };

        let mut lines = Vec::new();
        if let Some(supported_fields) = details
            .get("supported_fields")
            .and_then(Self::value_as_string_list)
        {
            lines.push(format!("supported_fields: {}", supported_fields.join(", ")));
        }
        if let Some(candidates) = details.get("candidates").and_then(|value| value.as_array()) {
            for (index, candidate) in candidates.iter().enumerate() {
                if let Some(candidate) = Self::as_object(candidate) {
                    let rendered = Self::build_context_diff_lines(candidate, "matched_text", "=");
                    if !rendered.is_empty() {
                        lines.push(format!("candidate {}:", index + 1));
                        lines.extend(rendered);
                    }
                }
            }
        }
        if let Some(fragments) = details
            .get("fragment_matches")
            .and_then(|value| value.as_array())
        {
            for (index, fragment) in fragments.iter().enumerate() {
                if let Some(fragment) = Self::as_object(fragment) {
                    let line = Self::compose_context_line(
                        Self::get_string(fragment, "context_before")
                            .as_deref()
                            .unwrap_or(""),
                        Self::get_string(fragment, "fragment")
                            .as_deref()
                            .unwrap_or(""),
                        Self::get_string(fragment, "context_after")
                            .as_deref()
                            .unwrap_or(""),
                    );
                    lines.push(format!("fragment {}: {}", index + 1, line));
                }
            }
        }

        lines
    }

    fn build_context_diff_lines(
        context: &serde_json::Map<String, serde_json::Value>,
        focus_key: &str,
        prefix: &str,
    ) -> Vec<String> {
        let before = Self::get_string(context, "context_before").unwrap_or_default();
        let focus = Self::get_string(context, focus_key).unwrap_or_default();
        let after = Self::get_string(context, "context_after").unwrap_or_default();
        let combined = Self::compose_context_line(&before, &focus, &after);
        combined
            .lines()
            .take(6)
            .map(|line| format!("{prefix} {line}"))
            .collect()
    }

    fn compose_context_line(before: &str, focus: &str, after: &str) -> String {
        format!("{before}{focus}{after}")
            .replace('\r', "")
            .trim()
            .to_string()
    }

    fn render_value_lines(value: &serde_json::Value) -> Vec<String> {
        match value {
            serde_json::Value::Null => Vec::new(),
            serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {
                vec![Self::value_to_inline(value)]
            }
            serde_json::Value::Array(items) => items
                .iter()
                .enumerate()
                .flat_map(|(index, item)| {
                    let inline = Self::value_to_inline(item);
                    if inline.contains('\n') {
                        let mut lines = vec![format!("{}.", index + 1)];
                        lines.extend(
                            inline
                                .lines()
                                .map(|line| format!("  {line}"))
                                .collect::<Vec<_>>(),
                        );
                        lines
                    } else {
                        vec![format!("{}. {inline}", index + 1)]
                    }
                })
                .collect(),
            serde_json::Value::Object(map) => map
                .iter()
                .map(|(key, value)| format!("{key}: {}", Self::value_to_inline(value)))
                .collect(),
        }
    }

    fn value_to_inline(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(value) => value.to_string(),
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Array(items) => items
                .iter()
                .map(Self::value_to_inline)
                .collect::<Vec<_>>()
                .join(", "),
            serde_json::Value::Object(map) => map
                .iter()
                .map(|(key, value)| format!("{key}={}", Self::value_to_inline(value)))
                .collect::<Vec<_>>()
                .join(" | "),
        }
    }

    fn value_as_string_list(value: &serde_json::Value) -> Option<Vec<String>> {
        value.as_array().map(|items| {
            items
                .iter()
                .map(Self::value_to_inline)
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        })
    }

    fn get_string(
        object: &serde_json::Map<String, serde_json::Value>,
        key: &str,
    ) -> Option<String> {
        object.get(key).and_then(|value| match value {
            serde_json::Value::String(text) => Some(text.clone()),
            serde_json::Value::Null => None,
            _ => Some(Self::value_to_inline(value)),
        })
    }

    fn as_object(value: &serde_json::Value) -> Option<&serde_json::Map<String, serde_json::Value>> {
        value.as_object()
    }

    fn format_fenced_block(language: &str, body: &str) -> String {
        format!("```{language}\n{body}\n```")
    }

    fn format_change_result(tool_name: &str, meta: Vec<String>, body: Option<String>) -> String {
        let mut lines = vec![format!("ok {tool_name}")];
        if !meta.is_empty() {
            lines.push(meta.join(" | "));
        }
        if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
            lines.push(body);
        }
        lines.join("\n")
    }

    fn format_read_result(
        tool_name: &str,
        meta: Vec<String>,
        label: &str,
        text: Option<&str>,
    ) -> String {
        let mut lines = vec![format!("ok {tool_name}")];
        if !meta.is_empty() {
            lines.push(meta.join(" | "));
        }
        if let Some(text) = text.filter(|value| !value.trim().is_empty()) {
            lines.push(Self::format_numbered_text_block(label, text));
        }
        lines.join("\n")
    }

    fn format_structured_result(
        tool_name: &str,
        meta: Vec<String>,
        value: &serde_json::Value,
    ) -> String {
        let mut lines = vec![format!("ok {tool_name}")];
        if !meta.is_empty() {
            lines.push(meta.join(" | "));
        }
        lines.push(Self::format_yaml_block(value));
        lines.join("\n")
    }

    fn format_numbered_text_block(label: &str, text: &str) -> String {
        let numbered = text
            .lines()
            .enumerate()
            .map(|(index, line)| format!("{} {}", index + 1, line))
            .collect::<Vec<_>>()
            .join("\n");
        Self::format_fenced_block(label, &numbered)
    }

    fn format_yaml_block(value: &serde_json::Value) -> String {
        let yaml = serde_yaml::to_string(value)
            .unwrap_or_else(|_| Self::render_value_lines(value).join("\n"));
        let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml).trim_end();
        Self::format_fenced_block("yaml", yaml)
    }

    fn build_chat_request(
        messages: &[ChatMessage],
        request: &ChatCompletionRequest,
    ) -> GenAiChatRequest {
        let mut chat_request = GenAiChatRequest::default();

        if let Some(system) = Self::join_system_messages(messages) {
            chat_request = chat_request.with_system(system);
        }

        chat_request = chat_request.append_messages(Self::convert_messages_to_genai(messages));

        if let Some(tools) = &request.tools {
            let converted_tools = Self::convert_tool_definitions(tools);
            if !converted_tools.is_empty() {
                chat_request = chat_request.with_tools(converted_tools);
            }
        }

        chat_request
    }

    fn convert_stream_end_to_response(
        stream_end: GenAiStreamEnd,
        model: &str,
        intermediate_messages: Option<Vec<ChatMessage>>,
    ) -> ChatCompletionResponse {
        let text = stream_end
            .captured_first_text()
            .unwrap_or_default()
            .to_string();
        let tool_calls = stream_end
            .captured_tool_calls()
            .unwrap_or_default()
            .into_iter()
            .map(Self::convert_tool_call_from_genai)
            .collect::<Vec<_>>();

        let usage = stream_end
            .captured_usage
            .as_ref()
            .map(Self::convert_usage)
            .unwrap_or(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            });

        let message = ChatMessage {
            role: MessageRole::Assistant,
            content: text,
            name: None,
            reasoning_content: stream_end.captured_reasoning_content.clone(),
            tool_calls: (!tool_calls.is_empty()).then_some(tool_calls),
            tool_call_id: None,
        };

        ChatCompletionResponse {
            id: uuid::Uuid::new_v4().to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: model.to_string(),
            system_fingerprint: None,
            choices: vec![ChatCompletionChoice {
                index: 0,
                finish_reason: if message.tool_calls.is_some() {
                    "tool_calls".to_string()
                } else {
                    "stop".to_string()
                },
                message,
            }],
            usage,
            intermediate_messages,
        }
    }

    fn empty_assistant_message() -> ChatMessage {
        ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
            name: None,
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    fn character_uuid_for_events() -> String {
        crate::character_state::CHARACTER_STATE
            .get_current_character()
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn maybe_emit_stream_abort(
        app_handle: &tauri::AppHandle,
        character_uuid: &str,
        target_message_id: &str,
    ) {
        if let Err(error) = EventBus::message_stream_delta(
            app_handle,
            character_uuid,
            target_message_id,
            MessageRole::Assistant,
            "",
            false,
            true,
        ) {
            eprintln!("发送流式中止事件失败: {error}");
        }

        if let Err(error) = EventBus::message_reasoning_delta(
            app_handle,
            character_uuid,
            target_message_id,
            "",
            ReasoningDeltaKind::Reasoning,
            false,
            true,
        ) {
            eprintln!("发送 reasoning 中止事件失败: {error}");
        }
    }

    fn build_aborted_generation(
        content: String,
        reasoning_content: Option<String>,
        intermediate_messages: &[ChatMessage],
    ) -> AbortedGeneration {
        AbortedGeneration {
            content,
            reasoning_content: reasoning_content.filter(|value| !value.trim().is_empty()),
            intermediate_messages: intermediate_messages.to_vec(),
        }
    }

    async fn execute_tool_calls(
        app_handle: &tauri::AppHandle,
        character_uuid: &str,
        target_message_id: &str,
        tool_calls: Vec<ToolCallData>,
        reasoning_content: Option<String>,
        cancellation: Option<&ActiveCancellationRequest>,
        messages: &mut Vec<ChatMessage>,
        intermediate_messages: &mut Vec<ChatMessage>,
    ) -> Result<(), AIChatError> {
        if tool_calls.is_empty() {
            return Ok(());
        }

        let assistant_message = ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
            name: None,
            reasoning_content,
            tool_calls: Some(tool_calls.clone()),
            tool_call_id: None,
        };

        intermediate_messages.push(assistant_message.clone());
        messages.push(assistant_message);

        for tool_call in tool_calls {
            if cancellation
                .as_ref()
                .map(|request| request.is_cancelled())
                .unwrap_or(false)
            {
                Self::maybe_emit_stream_abort(app_handle, character_uuid, target_message_id);
                return Err(AIChatError::Aborted(Self::build_aborted_generation(
                    String::new(),
                    None,
                    intermediate_messages,
                )));
            }

            if let Err(error) = EventBus::tool_execution_status(
                app_handle,
                character_uuid,
                target_message_id,
                &tool_call,
                ToolExecutionPhase::Started,
                None,
                None,
                None,
            ) {
                eprintln!("发送工具开始事件失败: {error}");
            }

            let tool_result = Self::execute_tool_call(app_handle, &tool_call).await;

            let phase = if tool_result.success {
                ToolExecutionPhase::Succeeded
            } else {
                ToolExecutionPhase::Failed
            };

            if let Err(error) = EventBus::tool_execution_status(
                app_handle,
                character_uuid,
                target_message_id,
                &tool_call,
                phase,
                tool_result.data.clone(),
                tool_result.error.clone(),
                Some(tool_result.execution_time_ms),
            ) {
                eprintln!("发送工具状态事件失败: {error}");
            }

            if let Err(error_message) = EventBus::tool_executed(
                app_handle,
                character_uuid,
                &tool_call.function.name,
                tool_result.success,
                tool_result.data.clone(),
                tool_result.error.clone(),
                tool_result.execution_time_ms,
            ) {
                eprintln!("发送工具执行事件失败: {error_message}");
            }

            intermediate_messages.push(tool_result.tool_message.clone());
            messages.push(tool_result.tool_message);

            if cancellation
                .as_ref()
                .map(|request| request.is_cancelled())
                .unwrap_or(false)
            {
                Self::maybe_emit_stream_abort(app_handle, character_uuid, target_message_id);
                return Err(AIChatError::Aborted(Self::build_aborted_generation(
                    String::new(),
                    None,
                    intermediate_messages,
                )));
            }
        }

        Ok(())
    }

    pub async fn create_chat_completion_streaming(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
        app_handle: &tauri::AppHandle,
        target_message_id: &str,
        cancellation: &mut ActiveCancellationRequest,
    ) -> Result<ChatCompletionResponse, AIChatError> {
        let client = Self::create_client_with_config(api_config);
        let options = Self::build_options(request);
        let mut messages = request.messages.clone();
        let mut intermediate_messages: Vec<ChatMessage> = Vec::new();
        let character_uuid = Self::character_uuid_for_events();

        loop {
            if cancellation.is_cancelled() {
                Self::maybe_emit_stream_abort(app_handle, &character_uuid, target_message_id);
                return Err(AIChatError::Aborted(Self::build_aborted_generation(
                    String::new(),
                    None,
                    &intermediate_messages,
                )));
            }

            let chat_request = Self::build_chat_request(&messages, request);
            let stream_response = client
                .exec_chat_stream(&request.model, chat_request, Some(&options))
                .await
                .map_err(|error| AIChatError::failed(format!("AI 流式调用失败: {error}")))?;

            let mut stream = stream_response.stream;
            let mut emitted_delta = false;
            let mut emitted_reasoning_delta = false;
            let mut streamed_content = String::new();
            let mut streamed_reasoning = String::new();
            let mut stream_end: Option<GenAiStreamEnd> = None;

            loop {
                let maybe_event = tokio::select! {
                    _ = cancellation.cancelled() => {
                        Self::maybe_emit_stream_abort(app_handle, &character_uuid, target_message_id);
                        return Err(AIChatError::Aborted(Self::build_aborted_generation(
                            streamed_content,
                            (!streamed_reasoning.is_empty()).then_some(streamed_reasoning),
                            &intermediate_messages,
                        )));
                    }
                    event = stream.next() => event,
                };

                let Some(event) = maybe_event else {
                    break;
                };

                let event = match event {
                    Ok(event) => event,
                    Err(error) => {
                        Self::maybe_emit_stream_abort(
                            app_handle,
                            &character_uuid,
                            target_message_id,
                        );
                        return Err(AIChatError::failed(format!("AI 流式事件处理失败: {error}")));
                    }
                };

                match event {
                    GenAiChatStreamEvent::Chunk(chunk) => {
                        if !chunk.content.is_empty() {
                            emitted_delta = true;
                            streamed_content.push_str(&chunk.content);
                            EventBus::message_stream_delta(
                                app_handle,
                                &character_uuid,
                                target_message_id,
                                MessageRole::Assistant,
                                &chunk.content,
                                false,
                                false,
                            )?;
                        }
                    }
                    GenAiChatStreamEvent::ReasoningChunk(chunk) => {
                        if !chunk.content.is_empty() {
                            emitted_reasoning_delta = true;
                            streamed_reasoning.push_str(&chunk.content);
                            EventBus::message_reasoning_delta(
                                app_handle,
                                &character_uuid,
                                target_message_id,
                                &chunk.content,
                                ReasoningDeltaKind::Reasoning,
                                false,
                                false,
                            )?;
                        }
                    }
                    GenAiChatStreamEvent::ThoughtSignatureChunk(chunk) => {
                        if !chunk.content.is_empty() {
                            emitted_reasoning_delta = true;
                            streamed_reasoning.push_str(&chunk.content);
                            EventBus::message_reasoning_delta(
                                app_handle,
                                &character_uuid,
                                target_message_id,
                                &chunk.content,
                                ReasoningDeltaKind::ThoughtSignature,
                                false,
                                false,
                            )?;
                        }
                    }
                    GenAiChatStreamEvent::End(end) => {
                        stream_end = Some(end);
                        break;
                    }
                    GenAiChatStreamEvent::Start | GenAiChatStreamEvent::ToolCallChunk(_) => {}
                }
            }

            let Some(stream_end) = stream_end else {
                Self::maybe_emit_stream_abort(app_handle, &character_uuid, target_message_id);
                return Err(AIChatError::failed("AI 流式响应在结束前中断"));
            };

            let response = Self::convert_stream_end_to_response(
                stream_end,
                &stream_response.model_iden.model_name,
                (!intermediate_messages.is_empty()).then_some(intermediate_messages.clone()),
            );

            let assistant_message = response
                .choices
                .first()
                .map(|choice| choice.message.clone())
                .unwrap_or_else(Self::empty_assistant_message);

            let has_visible_stream_content = emitted_delta || !assistant_message.content.is_empty();

            if !assistant_message.content.is_empty() && !emitted_delta {
                EventBus::message_stream_delta(
                    app_handle,
                    &character_uuid,
                    target_message_id,
                    MessageRole::Assistant,
                    &assistant_message.content,
                    false,
                    false,
                )?;
            }

            if has_visible_stream_content {
                EventBus::message_stream_delta(
                    app_handle,
                    &character_uuid,
                    target_message_id,
                    MessageRole::Assistant,
                    "",
                    true,
                    false,
                )?;
            }

            if emitted_reasoning_delta {
                EventBus::message_reasoning_delta(
                    app_handle,
                    &character_uuid,
                    target_message_id,
                    "",
                    ReasoningDeltaKind::Reasoning,
                    true,
                    false,
                )?;
            }

            let tool_calls = assistant_message.tool_calls.clone().unwrap_or_default();
            if tool_calls.is_empty() {
                return Ok(response);
            }

            Self::execute_tool_calls(
                app_handle,
                &character_uuid,
                target_message_id,
                tool_calls,
                assistant_message.reasoning_content.clone(),
                Some(cancellation),
                &mut messages,
                &mut intermediate_messages,
            )
            .await?;
        }
    }

    pub async fn create_chat_completion(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
        app_handle: Option<&tauri::AppHandle>,
        target_message_id: Option<&str>,
    ) -> Result<ChatCompletionResponse, String> {
        let client = Self::create_client_with_config(api_config);
        let options = Self::build_options(request);
        let mut messages = request.messages.clone();
        let mut intermediate_messages: Vec<ChatMessage> = Vec::new();
        let character_uuid = app_handle.map(|_| Self::character_uuid_for_events());

        loop {
            let chat_request = Self::build_chat_request(&messages, request);

            let response = client
                .exec_chat(&request.model, chat_request, Some(&options))
                .await
                .map_err(|error| format!("AI API调用失败: {error}"))?;

            let mut converted_response = Self::convert_response_from_genai(&response);
            let assistant_message = converted_response
                .choices
                .first()
                .map(|choice| choice.message.clone())
                .unwrap_or_else(Self::empty_assistant_message);

            let tool_calls = assistant_message.tool_calls.clone().unwrap_or_default();
            if tool_calls.is_empty() || app_handle.is_none() {
                if !intermediate_messages.is_empty() {
                    converted_response.intermediate_messages = Some(intermediate_messages);
                }
                return Ok(converted_response);
            }

            let app_handle = app_handle.expect("checked above");
            let character_uuid = character_uuid.as_deref().unwrap_or("unknown");
            let target_message_id = target_message_id.unwrap_or_default();

            Self::execute_tool_calls(
                app_handle,
                character_uuid,
                target_message_id,
                tool_calls,
                assistant_message.reasoning_content.clone(),
                None,
                &mut messages,
                &mut intermediate_messages,
            )
            .await
            .map_err(|error| match error {
                AIChatError::Failed(message) => message,
                AIChatError::Aborted(_) => AI_RESPONSE_INTERRUPTED_ERROR.to_string(),
            })?;
        }
    }
}

impl ApiProvider {
    fn adapter_kind(self) -> AdapterKind {
        match self {
            ApiProvider::OpenAiCompatible => AdapterKind::OpenAI,
            ApiProvider::OpenAiResponses => AdapterKind::OpenAIResp,
            ApiProvider::Claude => AdapterKind::Anthropic,
            ApiProvider::GeminiV1Beta => AdapterKind::Gemini,
        }
    }
}
