use crate::chat_history::ChatMessage;
use crate::character_storage::CharacterData;
use crate::character_session::{SessionInfo, TokenBudget};
use crate::context_builder::BuiltContextResult;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

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

/// 错误事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub uuid: Option<String>,
    pub error_code: String,
    pub error_message: String,
    pub error_context: Option<serde_json::Value>,
    pub timestamp: i64,
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

/// 事件发送器 - 提供统一的事件发送接口
pub struct EventEmitter;

impl EventEmitter {
    /// 发送角色加载事件
    pub fn send_character_loaded(
        app: &AppHandle,
        uuid: &str,
        character_data: &CharacterData,
    ) -> Result<(), String> {
        let payload = CharacterLoadedPayload {
            uuid: uuid.to_string(),
            character_data: character_data.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("character-loaded", &payload)
            .map_err(|e| format!("发送角色加载事件失败: {}", e))?;

        Ok(())
    }

    /// 发送聊天历史加载事件
    pub fn send_chat_history_loaded(
        app: &AppHandle,
        uuid: &str,
        chat_history: &[ChatMessage],
    ) -> Result<(), String> {
        let payload = ChatHistoryLoadedPayload {
            uuid: uuid.to_string(),
            chat_history: chat_history.to_vec(),
            message_count: chat_history.len(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("chat-history-loaded", &payload)
            .map_err(|e| format!("发送聊天历史加载事件失败: {}", e))?;

        Ok(())
    }

    /// 发送消息发送事件
    pub fn send_message_sent(
        app: &AppHandle,
        uuid: &str,
        message: &ChatMessage,
    ) -> Result<(), String> {
        let payload = MessageSentPayload {
            uuid: uuid.to_string(),
            message: message.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("message-sent", &payload)
            .map_err(|e| format!("发送消息事件失败: {}", e))?;

        Ok(())
    }

    /// 发送消息接收事件
    pub fn send_message_received(
        app: &AppHandle,
        uuid: &str,
        message: &ChatMessage,
        intermediate_messages: Option<Vec<ChatMessage>>,
    ) -> Result<(), String> {
        let payload = MessageReceivedPayload {
            uuid: uuid.to_string(),
            message: message.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            intermediate_messages,
        };

        app.emit("message-received", &payload)
            .map_err(|e| format!("发送接收消息事件失败: {}", e))?;

        Ok(())
    }

    /// 发送上下文构建完成事件
    pub fn send_context_built(
        app: &AppHandle,
        uuid: &str,
        context_result: &BuiltContextResult,
    ) -> Result<(), String> {
        let payload = ContextBuiltPayload {
            uuid: uuid.to_string(),
            context_result: context_result.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("context-built", &payload)
            .map_err(|e| format!("发送上下文构建事件失败: {}", e))?;

        Ok(())
    }

    /// 发送角色更新事件
    pub fn send_character_updated(
        app: &AppHandle,
        uuid: &str,
        character_data: &CharacterData,
        update_type: CharacterUpdateType,
    ) -> Result<(), String> {
        let payload = CharacterUpdatedPayload {
            uuid: uuid.to_string(),
            character_data: character_data.clone(),
            update_type,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("character-updated", &payload)
            .map_err(|e| format!("发送角色更新事件失败: {}", e))?;

        Ok(())
    }

    /// 发送工具执行事件
    pub fn send_tool_executed(
        app: &AppHandle,
        uuid: &str,
        tool_name: &str,
        success: bool,
        result: Option<serde_json::Value>,
        error: Option<String>,
        execution_time_ms: u64,
    ) -> Result<(), String> {
        let payload = ToolExecutedPayload {
            uuid: uuid.to_string(),
            tool_name: tool_name.to_string(),
            success,
            result,
            error,
            execution_time_ms,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("tool-executed", &payload)
            .map_err(|e| format!("发送工具执行事件失败: {}", e))?;

        Ok(())
    }

    /// 发送会话卸载事件
    pub fn send_session_unloaded(
        app: &AppHandle,
        uuid: &str,
        session_info: &SessionInfo,
        reason: SessionUnloadReason,
    ) -> Result<(), String> {
        let payload = SessionUnloadedPayload {
            uuid: uuid.to_string(),
            session_info: session_info.clone(),
            reason,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("session-unloaded", &payload)
            .map_err(|e| format!("发送会话卸载事件失败: {}", e))?;

        Ok(())
    }

    /// 发送错误事件
    pub fn send_error(
        app: &AppHandle,
        uuid: Option<&str>,
        error_code: &str,
        error_message: &str,
        error_context: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let payload = ErrorPayload {
            uuid: uuid.map(|u| u.to_string()),
            error_code: error_code.to_string(),
            error_message: error_message.to_string(),
            error_context,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("error", &payload)
            .map_err(|e| format!("发送错误事件失败: {}", e))?;

        Ok(())
    }

    /// 发送Token统计事件
    pub fn send_token_stats(
        app: &AppHandle,
        uuid: &str,
        token_usage: TokenUsageStats,
    ) -> Result<(), String> {
        let payload = TokenStatsPayload {
            uuid: uuid.to_string(),
            token_usage,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("token-stats", &payload)
            .map_err(|e| format!("发送Token统计事件失败: {}", e))?;

        Ok(())
    }

    /// 发送通用进度事件（用于长时间操作）
    pub fn send_progress(
        app: &AppHandle,
        uuid: &str,
        operation: &str,
        progress: f64, // 0.0 - 1.0
        message: Option<&str>,
    ) -> Result<(), String> {
        let payload = serde_json::json!({
            "uuid": uuid,
            "operation": operation,
            "progress": progress,
            "message": message,
            "timestamp": chrono::Utc::now().timestamp()
        });

        app.emit("progress", &payload)
            .map_err(|e| format!("发送进度事件失败: {}", e))?;

        Ok(())
    }
}

/// 便捷宏，用于发送错误事件
#[macro_export]
macro_rules! emit_error {
    ($app:expr, $uuid:expr, $error_code:expr, $error_message:expr) => {
        $crate::events::EventEmitter::send_error(
            $app,
            Some($uuid),
            $error_code,
            $error_message,
            None,
        ).unwrap_or_else(|e| eprintln!("发送错误事件失败: {}", e));
    };
    ($app:expr, $uuid:expr, $error_code:expr, $error_message:expr, $context:expr) => {
        $crate::events::EventEmitter::send_error(
            $app,
            Some($uuid),
            $error_code,
            $error_message,
            Some($context),
        ).unwrap_or_else(|e| eprintln!("发送错误事件失败: {}", e));
    };
}

/// 便捷宏，用于发送工具执行成功事件
#[macro_export]
macro_rules! emit_tool_success {
    ($app:expr, $uuid:expr, $tool_name:expr, $result:expr, $time_ms:expr) => {
        $crate::events::EventEmitter::send_tool_executed(
            $app,
            $uuid,
            $tool_name,
            true,
            Some($result),
            None,
            $time_ms,
        ).unwrap_or_else(|e| eprintln!("发送工具执行事件失败: {}", e));
    };
}

/// 便捷宏，用于发送工具执行失败事件
#[macro_export]
macro_rules! emit_tool_error {
    ($app:expr, $uuid:expr, $tool_name:expr, $error:expr, $time_ms:expr) => {
        $crate::events::EventEmitter::send_tool_executed(
            $app,
            $uuid,
            $tool_name,
            false,
            None,
            Some($error.to_string()),
            $time_ms,
        ).unwrap_or_else(|e| eprintln!("发送工具执行事件失败: {}", e));
    };
}