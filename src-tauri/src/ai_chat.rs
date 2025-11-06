use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionMessageToolCall, ChatCompletionNamedToolChoice,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestToolMessageContent,
        ChatCompletionRequestUserMessageArgs, ChatCompletionToolArgs,
        ChatCompletionToolChoiceOption, ChatCompletionToolType, CreateChatCompletionRequestArgs,
        FunctionCall, FunctionName, FunctionObject,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::api_config::ApiConfig;

/// 聊天消息角色 (为前端兼容性保留)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 聊天消息 (为前端兼容性保留)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCallData>>,
    pub tool_call_id: Option<String>,
}

/// 工具调用数据 (为前端兼容性保留)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallData {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunctionData,
}

/// 工具调用函数数据 (为前端兼容性保留)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunctionData {
    pub name: String,
    pub arguments: String,
}

/// 工具调用 (兼容性)
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

/// 停止序列 (兼容性)
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
}

/// 聊天完成请求 (兼容性)
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
    pub tools: Option<Vec<ChatTool>>,
    pub tool_choice: Option<ToolChoice>,
}

/// 工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub items: Option<Box<ToolParameter>>,
    pub properties: Option<HashMap<String, ToolParameter>>,
    pub required: Option<Vec<String>>,
}

/// 工具函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<ToolParameters>,
}

/// 工具参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, ToolParameter>,
    pub required: Option<Vec<String>>,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

/// AI聊天服务
pub struct AIChatService;

impl AIChatService {
    /// 创建带自定义配置的客户端
    async fn create_client_with_config(
        api_config: &ApiConfig,
    ) -> Result<Client<OpenAIConfig>, String> {
        // 构建基础 URL，确保以 /v1 结尾
        let base_url = if api_config.endpoint.ends_with("/v1") {
            api_config.endpoint.clone()
        } else if api_config.endpoint.ends_with("/v1/") {
            api_config.endpoint.trim_end_matches('/').to_string()
        } else {
            format!("{}/v1", api_config.endpoint.trim_end_matches('/'))
        };

        // 创建自定义配置
        let config = OpenAIConfig::new()
            .with_api_key(&api_config.key)
            .with_api_base(&base_url);

        let client = Client::with_config(config);
        Ok(client)
    }

    /// 将前端消息转换为 async-openai 消息格式
    fn convert_messages_to_openai(
        messages: &[ChatMessage],
    ) -> Vec<async_openai::types::ChatCompletionRequestMessage> {
        let mut openai_messages = Vec::new();

        for msg in messages {
            let openai_msg = match msg.role {
                MessageRole::System => {
                    let system_msg = ChatCompletionRequestSystemMessageArgs::default()
                        .content(msg.content.clone())
                        .build()
                        .unwrap();
                    system_msg.into()
                }
                MessageRole::User => {
                    let user_msg = ChatCompletionRequestUserMessageArgs::default()
                        .content(msg.content.clone())
                        .build()
                        .unwrap();
                    user_msg.into()
                }
                MessageRole::Assistant => {
                    let mut builder = ChatCompletionRequestAssistantMessageArgs::default();

                    if !msg.content.is_empty() {
                        builder.content(msg.content.clone());
                    }
                    if let Some(name) = &msg.name {
                        builder.name(name.clone());
                    }
                    if let Some(tool_calls) = &msg.tool_calls {
                        let converted_tool_calls = tool_calls
                            .iter()
                            .map(|call| ChatCompletionMessageToolCall {
                                id: call.id.clone(),
                                r#type: ChatCompletionToolType::Function,
                                function: FunctionCall {
                                    name: call.function.name.clone(),
                                    arguments: call.function.arguments.clone(),
                                },
                            })
                            .collect::<Vec<_>>();
                        builder.tool_calls(converted_tool_calls);
                    }

                    builder.build().unwrap().into()
                }
                MessageRole::Tool => {
                    // 对于工具消息，暂时转换为用户消息
                    if let Some(tool_call_id) = &msg.tool_call_id {
                        let tool_msg = ChatCompletionRequestToolMessageArgs::default()
                            .content(ChatCompletionRequestToolMessageContent::Text(
                                msg.content.clone(),
                            ))
                            .tool_call_id(tool_call_id.clone())
                            .build()
                            .unwrap();
                        tool_msg.into()
                    } else {
                        // tool_call_id 丢失时退回为用户消息以避免请求构造失败
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(format!("[Tool Response] {}", msg.content))
                            .build()
                            .unwrap()
                            .into()
                    }
                }
            };

            openai_messages.push(openai_msg);
        }

        openai_messages
    }

    fn convert_tools_to_openai(tools: &[ChatTool]) -> Vec<async_openai::types::ChatCompletionTool> {
        tools
            .iter()
            .filter_map(|tool| {
                if tool.tool_type != "function" {
                    return None;
                }

                let parameters = tool
                    .function
                    .parameters
                    .as_ref()
                    .and_then(|params| serde_json::to_value(params).ok());

                let function_object = FunctionObject {
                    name: tool.function.name.clone(),
                    description: tool.function.description.clone(),
                    parameters,
                    strict: None,
                };

                let mut builder = ChatCompletionToolArgs::default();
                builder.r#type(ChatCompletionToolType::Function);
                builder.function(function_object);
                builder.build().ok()
            })
            .collect()
    }

    fn convert_tool_choice_to_openai(
        choice: &ToolChoice,
    ) -> Option<ChatCompletionToolChoiceOption> {
        match choice {
            ToolChoice::String(value) => match value.to_lowercase().as_str() {
                "none" => Some(ChatCompletionToolChoiceOption::None),
                "auto" => Some(ChatCompletionToolChoiceOption::Auto),
                "required" => Some(ChatCompletionToolChoiceOption::Required),
                _ => None,
            },
            ToolChoice::Function {
                choice_type,
                function,
            } => {
                if choice_type.to_lowercase() != "function" {
                    return None;
                }

                Some(ChatCompletionToolChoiceOption::Named(
                    ChatCompletionNamedToolChoice {
                        r#type: ChatCompletionToolType::Function,
                        function: FunctionName {
                            name: function.name.clone(),
                        },
                    },
                ))
            }
        }
    }

    /// 将 async-openai 响应转换为前端兼容格式
    fn convert_response_from_openai(
        response: async_openai::types::CreateChatCompletionResponse,
    ) -> ChatCompletionResponse {
        ChatCompletionResponse {
            id: response.id,
            object: response.object,
            created: response.created as u64,
            model: response.model,
            system_fingerprint: response.system_fingerprint,
            choices: response
                .choices
                .into_iter()
                .map(|choice| {
                    ChatCompletionChoice {
                        index: choice.index as u32,
                        message: ChatMessage {
                            role: match choice.message.role {
                                async_openai::types::Role::System => MessageRole::System,
                                async_openai::types::Role::User => MessageRole::User,
                                async_openai::types::Role::Assistant => MessageRole::Assistant,
                                async_openai::types::Role::Tool => MessageRole::Tool,
                                async_openai::types::Role::Function => MessageRole::Tool,
                            },
                            content: choice.message.content.unwrap_or_default(),
                            name: None, // async-openai 的消息没有 name 字段
                            tool_calls: if let Some(calls) = &choice.message.tool_calls {
                                Some(
                                    calls
                                        .iter()
                                        .map(|call| ToolCallData {
                                            id: call.id.clone(),
                                            call_type: "function".to_string(),
                                            function: ToolCallFunctionData {
                                                name: call.function.name.clone(),
                                                arguments: call.function.arguments.clone(),
                                            },
                                        })
                                        .collect(),
                                )
                            } else {
                                None
                            },
                            tool_call_id: None,
                        },
                        finish_reason: choice
                            .finish_reason
                            .map(|fr| match fr {
                                async_openai::types::FinishReason::Stop => "stop".to_string(),
                                async_openai::types::FinishReason::Length => "length".to_string(),
                                async_openai::types::FinishReason::ToolCalls => {
                                    "tool_calls".to_string()
                                }
                                async_openai::types::FinishReason::FunctionCall => {
                                    "function_call".to_string()
                                }
                                async_openai::types::FinishReason::ContentFilter => {
                                    "content_filter".to_string()
                                }
                            })
                            .unwrap_or("stop".to_string()),
                    }
                })
                .collect(),
            usage: response
                .usage
                .map(|usage| Usage {
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                })
                .unwrap_or(Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                }),
        }
    }

    /// 创建聊天完成请求
    pub async fn create_chat_completion(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
        app_handle: Option<&tauri::AppHandle>,
    ) -> Result<ChatCompletionResponse, String> {
        let client = Self::create_client_with_config(api_config).await?;
        let mut messages = request.messages.clone();
        let max_iterations = 5; // 防止无限循环
        let mut iteration = 0;

        loop {
            if iteration >= max_iterations {
                return Err("工具调用循环次数超过限制".to_string());
            }
            iteration += 1;

            let openai_messages = Self::convert_messages_to_openai(&messages);
            let mut request_builder = CreateChatCompletionRequestArgs::default();

            request_builder.model(&request.model);
            request_builder.messages(openai_messages);

            if let Some(temp) = request.temperature {
                request_builder.temperature(temp as f32);
            }
            if let Some(max_tokens) = request.max_tokens {
                request_builder.max_tokens(max_tokens);
            }
            if let Some(top_p) = request.top_p {
                request_builder.top_p(top_p as f32);
            }
            if let Some(freq_penalty) = request.frequency_penalty {
                request_builder.frequency_penalty(freq_penalty as f32);
            }
            if let Some(pres_penalty) = request.presence_penalty {
                request_builder.presence_penalty(pres_penalty as f32);
            }
            if let Some(tools) = &request.tools {
                let converted_tools = Self::convert_tools_to_openai(tools);
                if !converted_tools.is_empty() {
                    request_builder.tools(converted_tools);
                }
            }
            if let Some(tool_choice) = &request.tool_choice {
                if let Some(openai_choice) = Self::convert_tool_choice_to_openai(tool_choice) {
                    request_builder.tool_choice(openai_choice);
                }
            }

            let openai_request = request_builder
                .build()
                .map_err(|e| format!("请求build错误: {}", e))?;

            let response = client
                .chat()
                .create(openai_request)
                .await
                .map_err(|e| format!("API请求失败: {}", e))?;

            let our_response = Self::convert_response_from_openai(response);

            // 检查是否有工具调用需要执行
            if let Some(choice) = our_response.choices.first() {
                if let Some(tool_calls) = &choice.message.tool_calls {
                    if !tool_calls.is_empty() {
                        // 执行工具调用
                        if let Some(app_handle) = app_handle {
                            messages.push(choice.message.clone());
                            for tool_call in tool_calls {
                                if let Some(tool_result) = Self::execute_single_tool_call(
                                    app_handle,
                                    &tool_call.function.name,
                                    &tool_call.function.arguments,
                                    &messages,
                                )
                                .await
                                {
                                    // 将工具结果添加到消息列表
                                    messages.push(ChatMessage {
                                        role: MessageRole::Tool,
                                        content: serde_json::to_string(&tool_result)
                                            .unwrap_or_default(),
                                        name: None,
                                        tool_calls: None,
                                        tool_call_id: Some(tool_call.id.clone()),
                                    });
                                } else {
                                    // 工具执行失败
                                    messages.push(ChatMessage {
                                        role: MessageRole::Tool,
                                        content: serde_json::json!({
                                            "success": false,
                                            "error": "Tool execution failed"
                                        })
                                        .to_string(),
                                        name: None,
                                        tool_calls: None,
                                        tool_call_id: Some(tool_call.id.clone()),
                                    });
                                }
                            }

                            // 继续循环，将工具结果发送回AI
                            continue;
                        }
                    }
                }
            }

            // 没有工具调用或工具调用完成，返回结果
            return Ok(our_response);
        }
    }

    /// 执行单个工具调用
    async fn execute_single_tool_call(
        app_handle: &tauri::AppHandle,
        tool_name: &str,
        arguments: &str,
        _messages: &[ChatMessage],
    ) -> Option<serde_json::Value> {
        // 解析参数
        let params: std::collections::HashMap<String, serde_json::Value> =
            match serde_json::from_str(arguments) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return Some(serde_json::json!({
                        "success": false,
                        "error": format!("Invalid tool arguments: {}", err)
                    }));
                }
            };

        // 从全局状态管理器获取当前角色UUID
        let character_uuid = crate::character_state::CHARACTER_STATE.get_current_character();

        // 创建工具调用请求
        let tool_request = crate::ai_tools::ToolCallRequest {
            tool_name: tool_name.to_string(),
            parameters: params,
            character_uuid,
            context: None, // 可以考虑添加角色上下文
        };

        // 执行工具调用
        let result =
            crate::ai_tools::AIToolService::execute_tool_call(app_handle, tool_request).await;

        if result.success {
            Some(serde_json::json!({
                "success": true,
                "data": result.data,
                "execution_time_ms": result.execution_time_ms
            }))
        } else {
            let error_message = result
                .error
                .clone()
                .unwrap_or_else(|| "Tool execution failed".to_string());

            Some(serde_json::json!({
                "success": false,
                "error": error_message,
                "data": result.data,
                "execution_time_ms": result.execution_time_ms
            }))
        }
    }

    /// 创建流式聊天完成请求 (暂时简化实现)
    pub async fn create_streaming_chat_completion(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
    ) -> Result<String, String> {
        // 对于流式响应，我们可以使用 async-openai 的流式功能
        // 但为了保持兼容性，暂时返回非流式结果的字符串格式
        let response = Self::create_chat_completion(api_config, request, None).await?;

        // 转换为 SSE 格式
        let mut result = String::new();
        for choice in &response.choices {
            let chunk = format!(
                "data: {}\n\n",
                serde_json::json!({
                    "id": response.id,
                    "object": "chat.completion.chunk",
                    "created": response.created,
                    "model": response.model,
                    "choices": [{
                        "index": choice.index,
                        "delta": {
                            "role": "assistant",
                            "content": choice.message.content
                        },
                        "finish_reason": choice.finish_reason
                    }]
                })
            );
            result.push_str(&chunk);
        }
        result.push_str("data: [DONE]\n\n");

        Ok(result)
    }
}
