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
                    if let Some(tool_calls) = &message.tool_calls {
                        let tool_calls = tool_calls
                            .iter()
                            .filter_map(Self::convert_tool_call_to_genai)
                            .collect::<Vec<_>>();

                        if !tool_calls.is_empty() {
                            Some(GenAiChatMessage::from(tool_calls))
                        } else {
                            Some(GenAiChatMessage::assistant(message.content.clone()))
                        }
                    } else {
                        Some(GenAiChatMessage::assistant(message.content.clone()))
                    }
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
            thought_signatures: None,
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
        let tool_result = serde_json::json!({
            "success": result.success,
            "data": data,
            "error": error,
            "execution_time_ms": result.execution_time_ms,
        });

        ToolExecutionOutput {
            tool_message: ChatMessage {
                role: MessageRole::Tool,
                content: tool_result.to_string(),
                name: Some(tool_call.function.name.clone()),
                tool_calls: None,
                tool_call_id: Some(tool_call.id.clone()),
            },
            success: result.success,
            data,
            error,
            execution_time_ms: result.execution_time_ms,
        }
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

    async fn execute_tool_calls(
        app_handle: &tauri::AppHandle,
        character_uuid: &str,
        target_message_id: &str,
        tool_calls: Vec<ToolCallData>,
        messages: &mut Vec<ChatMessage>,
        intermediate_messages: &mut Vec<ChatMessage>,
    ) {
        if tool_calls.is_empty() {
            return;
        }

        let assistant_message = ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
            name: None,
            tool_calls: Some(tool_calls.clone()),
            tool_call_id: None,
        };

        intermediate_messages.push(assistant_message.clone());
        messages.push(assistant_message);

        for tool_call in tool_calls {
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
        }
    }

    pub async fn create_chat_completion_streaming(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
        app_handle: &tauri::AppHandle,
        target_message_id: &str,
    ) -> Result<ChatCompletionResponse, String> {
        let client = Self::create_client_with_config(api_config);
        let options = Self::build_options(request);
        let mut messages = request.messages.clone();
        let mut intermediate_messages: Vec<ChatMessage> = Vec::new();
        let character_uuid = Self::character_uuid_for_events();

        for _ in 0..5 {
            let chat_request = Self::build_chat_request(&messages, request);
            let stream_response = client
                .exec_chat_stream(&request.model, chat_request, Some(&options))
                .await
                .map_err(|error| format!("AI 流式调用失败: {error}"))?;

            let mut stream = stream_response.stream;
            let mut emitted_delta = false;
            let mut emitted_reasoning_delta = false;
            let mut stream_end: Option<GenAiStreamEnd> = None;

            while let Some(event) = stream.next().await {
                let event = match event {
                    Ok(event) => event,
                    Err(error) => {
                        Self::maybe_emit_stream_abort(
                            app_handle,
                            &character_uuid,
                            target_message_id,
                        );
                        return Err(format!("AI 流式事件处理失败: {error}"));
                    }
                };

                match event {
                    GenAiChatStreamEvent::Chunk(chunk) => {
                        if !chunk.content.is_empty() {
                            emitted_delta = true;
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
                    GenAiChatStreamEvent::Start
                    | GenAiChatStreamEvent::ToolCallChunk(_) => {}
                }
            }

            let Some(stream_end) = stream_end else {
                Self::maybe_emit_stream_abort(app_handle, &character_uuid, target_message_id);
                return Err("AI 流式响应在结束前中断".to_string());
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
                &mut messages,
                &mut intermediate_messages,
            )
            .await;
        }

        Err("工具调用循环次数超过限制".to_string())
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

        for _ in 0..5 {
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
                &mut messages,
                &mut intermediate_messages,
            )
            .await;
        }

        Err("工具调用循环次数超过限制".to_string())
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
