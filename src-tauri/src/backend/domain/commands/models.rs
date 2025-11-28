use serde::{Deserialize, Serialize};

/// 命令元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// 命令唯一标识符
    pub id: String,
    /// 命令名称（用于显示）
    pub name: String,
    /// 命令描述
    pub description: String,
    /// 命令图标（可选）
    pub icon: Option<String>,
    /// 命令分类（可选）
    pub category: Option<CommandCategory>,
    /// 优先级（数值越小优先级越高）
    pub priority: i32,
    /// 是否需要确认
    pub requires_confirmation: bool,
    /// 确认提示消息（可选）
    pub confirmation_message: Option<String>,
}

/// 命令分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandCategory {
    Chat,
    History,
    Export,
    Settings,
    Other,
}

/// 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// 执行是否成功
    pub success: bool,
    /// 成功消息（可选）
    pub message: Option<String>,
    /// 错误消息（可选）
    pub error: Option<String>,
    /// 返回数据（可选）
    pub data: Option<serde_json::Value>,
}

