use crate::backend::domain::sessions::session::SessionInfo;
use crate::character_storage::CharacterData;
use crate::chat_history::ChatMessage;
use crate::context_builder::BuiltContextResult;
use serde::{Deserialize, Serialize};

/// 角色加载事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterLoadedPayload {
    pub uuid: String,
    pub character_data: CharacterData,
    pub timestamp: i64,
}

/// 聊天历史加载事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryLoadedPayload {
    pub uuid: String,
    pub chat_history: Vec<ChatMessage>,
    pub message_count: usize,
    pub timestamp: i64,
}

/// 消息发送事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSentPayload {
    pub uuid: String,
    pub message: ChatMessage,
    pub timestamp: i64,
}

/// 消息接收事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceivedPayload {
    pub uuid: String,
    pub message: ChatMessage,
    pub timestamp: i64,
    /// 中间消息（包括 assistant with tool_calls 和 tool results）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intermediate_messages: Option<Vec<ChatMessage>>,
}

/// 上下文构建完成事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuiltPayload {
    pub uuid: String,
    pub context_result: BuiltContextResult,
    pub timestamp: i64,
}

/// 角色更新事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterUpdatedPayload {
    pub uuid: String,
    pub character_data: CharacterData,
    pub update_type: CharacterUpdateType,
    pub timestamp: i64,
}

/// 角色更新类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterUpdateType {
    /// 基本信息更新（name, description, personality等）
    BasicInfo,
    /// 世界书更新
    Worldbook,
    /// 标签更新
    Tags,
    /// 完整角色数据更新
    FullData,
    /// 字段更新（单个或多个字段）
    Fields { fields: Vec<String> },
}

/// 工具执行事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutedPayload {
    pub uuid: String,
    pub tool_name: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: i64,
}

/// 会话卸载事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUnloadedPayload {
    pub uuid: String,
    pub session_info: SessionInfo,
    pub reason: SessionUnloadReason,
    pub timestamp: i64,
}

/// 会话卸载原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionUnloadReason {
    /// 用户主动卸载
    UserRequest,
    /// 会话过期
    Expired,
    /// 内存清理
    MemoryCleanup,
    /// 应用关闭
    AppShutdown,
    /// 错误导致卸载
    Error(String),
}

/// Token统计事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatsPayload {
    pub uuid: String,
    pub token_usage: TokenUsageStats,
    pub timestamp: i64,
}

/// Token使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageStats {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub context_tokens: usize,
    pub budget_utilization: f64, // 预算使用百分比
}

