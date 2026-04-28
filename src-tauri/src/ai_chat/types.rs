use crate::ai_tools::ToolDefinition;
use serde::{Deserialize, Serialize};

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
pub(super) struct ToolExecutionOutput {
    pub(super) tool_message: ChatMessage,
    pub(super) success: bool,
    pub(super) data: Option<serde_json::Value>,
    pub(super) error: Option<String>,
    pub(super) execution_time_ms: u64,
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
