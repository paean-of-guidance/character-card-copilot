use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::api_config::ApiConfig;
use super::ai_tools::AITool;

/// 聊天消息角色 (为前端兼容性保留)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// 流式聊天数据块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    #[serde(rename = "system_fingerprint")]
    pub system_fingerprint: Option<String>,
    pub choices: Vec<ChatCompletionStreamChoice>,
    pub usage: Option<Usage>,
}

/// 流式聊天选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamChoice {
    pub index: u32,
    pub delta: ChatCompletionDelta,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

/// 聊天增量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionDelta {
    pub role: Option<MessageRole>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallData>>,
}

/// AI聊天服务
pub struct AIChatService;

impl AIChatService {
    /// 创建带自定义配置的客户端
    async fn create_client_with_config(api_config: &ApiConfig) -> Result<Client<OpenAIConfig>, String> {
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
    fn convert_messages_to_openai(messages: &[ChatMessage]) -> Vec<async_openai::types::ChatCompletionRequestMessage> {
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
                    let assistant_msg = ChatCompletionRequestAssistantMessageArgs::default()
                        .content(msg.content.clone())
                        .build()
                        .unwrap();
                    assistant_msg.into()
                }
                MessageRole::Tool => {
                    // 对于工具消息，暂时转换为用户消息
                    let tool_msg = ChatCompletionRequestUserMessageArgs::default()
                        .content(format!("[Tool Response] {}", msg.content))
                        .build()
                        .unwrap();
                    tool_msg.into()
                }
            };

            openai_messages.push(openai_msg);
        }

        openai_messages
    }

    /// 将 async-openai 响应转换为前端兼容格式
    fn convert_response_from_openai(response: async_openai::types::CreateChatCompletionResponse) -> ChatCompletionResponse {
        ChatCompletionResponse {
            id: response.id,
            object: response.object,
            created: response.created as u64,
            model: response.model,
            system_fingerprint: response.system_fingerprint,
            choices: response.choices.into_iter().map(|choice| {
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
                        tool_calls: None, // 暂时不处理工具调用
                        tool_call_id: None,
                    },
                    finish_reason: choice.finish_reason
                        .map(|fr| match fr {
                            async_openai::types::FinishReason::Stop => "stop".to_string(),
                            async_openai::types::FinishReason::Length => "length".to_string(),
                            async_openai::types::FinishReason::ToolCalls => "tool_calls".to_string(),
                            async_openai::types::FinishReason::FunctionCall => "function_call".to_string(),
                            async_openai::types::FinishReason::ContentFilter => "content_filter".to_string(),
                        })
                        .unwrap_or("stop".to_string()),
                }
            }).collect(),
            usage: response.usage.map(|usage| Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            }).unwrap_or(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
        }
    }

    /// 将AI工具转换为OpenAI工具格式
    pub fn convert_ai_tools_to_openai(tool: &AITool) -> ChatTool {
        let mut properties = HashMap::new();
        let mut required_params = Vec::new();

        for param in &tool.parameters {
            if param.required {
                required_params.push(param.name.clone());
            }

            let tool_param = ToolParameter {
                param_type: param.parameter_type.clone(),
                description: Some(param.description.clone()),
                enum_values: param.schema.as_ref()
                    .and_then(|s| s.get("enum"))
                    .and_then(|e| e.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    }),
                items: None,
                properties: None,
                required: if param.required {
                    Some(vec![param.name.clone()])
                } else {
                    None
                },
            };

            properties.insert(param.name.clone(), tool_param);
        }

        ChatTool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: tool.name.clone(),
                description: Some(tool.description.clone()),
                parameters: Some(ToolParameters {
                    param_type: "object".to_string(),
                    properties,
                    required: if required_params.is_empty() {
                        None
                    } else {
                        Some(required_params)
                    },
                }),
            },
        }
    }

    /// 创建聊天完成请求
    pub async fn create_chat_completion(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, String> {
        let client = Self::create_client_with_config(api_config).await?;

        // 转换消息格式
        let openai_messages = Self::convert_messages_to_openai(&request.messages);

        // 构建请求 - 使用更简单的方法
        let mut request_builder = CreateChatCompletionRequestArgs::default();

        // 必需参数
        request_builder.model(&request.model);
        request_builder.messages(openai_messages);

        // 可选参数
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

        // 暂时不处理工具调用，避免复杂的类型问题
        // TODO: 后续再实现工具调用功能

        let openai_request = request_builder.build().map_err(|e| {
            format!("构建请求失败: {}", e)
        })?;

        // 发送请求
        let response = client.chat().create(openai_request).await
            .map_err(|e| format!("API请求失败: {}", e))?;

        Ok(Self::convert_response_from_openai(response))
    }

    /// 创建流式聊天完成请求 (暂时简化实现)
    pub async fn create_streaming_chat_completion(
        api_config: &ApiConfig,
        request: &ChatCompletionRequest,
    ) -> Result<String, String> {
        // 对于流式响应，我们可以使用 async-openai 的流式功能
        // 但为了保持兼容性，暂时返回非流式结果的字符串格式
        let response = Self::create_chat_completion(api_config, request).await?;

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

    /// 构建基础聊天请求
    pub fn build_base_request(
        model: &str,
        messages: &[ChatMessage],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        tools: Option<Vec<ChatTool>>,
    ) -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            temperature: temperature.map(|t| t as f64),
            max_tokens,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: Some(false),
            tools,
            tool_choice: None,
        }
    }
}