use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 会话状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// 活跃状态
    Active,
    /// 暂停状态
    Paused,
    /// 加载中
    Loading,
    /// 错误状态
    Error(String),
}

/// 会话信息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub uuid: String,
    pub character_name: Option<String>,
    pub message_count: usize,
    pub last_active: DateTime<Utc>,
    pub status: SessionStatus,
    pub last_context_tokens: usize,
}

