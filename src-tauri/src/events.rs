use crate::ai_chat::{MessageRole, ToolCallData};
use crate::backend::domain::{
    CharacterLoadedPayload, CharacterUpdateType, CharacterUpdatedPayload, ChatHistoryLoadedPayload,
    ContextBuiltPayload, MessageReasoningDeltaPayload, MessageReceivedPayload, MessageSentPayload,
    MessageStreamDeltaPayload, ReasoningDeltaKind, SessionInfo, SessionUnloadReason,
    SessionUnloadedPayload, TokenStatsPayload, TokenUsageStats, ToolExecutedPayload,
    ToolExecutionPhase, ToolExecutionStatusPayload,
};
use crate::character_storage::CharacterData;
use crate::chat_history::ChatMessage;
use crate::context_builder::BuiltContextResult;
use tauri::{AppHandle, Emitter};

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
        target_message_id: Option<&str>,
        intermediate_messages: Option<Vec<ChatMessage>>,
    ) -> Result<(), String> {
        let payload = MessageReceivedPayload {
            uuid: uuid.to_string(),
            message: message.clone(),
            target_message_id: target_message_id.map(|value| value.to_string()),
            timestamp: chrono::Utc::now().timestamp(),
            intermediate_messages,
        };

        app.emit("message-received", &payload)
            .map_err(|e| format!("发送接收消息事件失败: {}", e))?;

        Ok(())
    }

    pub fn send_message_stream_delta(
        app: &AppHandle,
        uuid: &str,
        target_message_id: &str,
        role: MessageRole,
        delta: &str,
        is_finished: bool,
        is_aborted: bool,
    ) -> Result<(), String> {
        let payload = MessageStreamDeltaPayload {
            uuid: uuid.to_string(),
            role,
            target_message_id: target_message_id.to_string(),
            delta: delta.to_string(),
            is_finished,
            is_aborted,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("message-stream-delta", &payload)
            .map_err(|e| format!("发送流式消息事件失败: {}", e))?;

        Ok(())
    }

    pub fn send_message_reasoning_delta(
        app: &AppHandle,
        uuid: &str,
        target_message_id: &str,
        delta: &str,
        kind: ReasoningDeltaKind,
        is_finished: bool,
        is_aborted: bool,
    ) -> Result<(), String> {
        let payload = MessageReasoningDeltaPayload {
            uuid: uuid.to_string(),
            target_message_id: target_message_id.to_string(),
            delta: delta.to_string(),
            kind,
            is_finished,
            is_aborted,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("message-reasoning-delta", &payload)
            .map_err(|e| format!("发送 reasoning 流式消息事件失败: {}", e))?;

        Ok(())
    }

    pub fn send_tool_execution_status(
        app: &AppHandle,
        uuid: &str,
        target_message_id: &str,
        tool_call: &ToolCallData,
        phase: ToolExecutionPhase,
        result: Option<serde_json::Value>,
        error: Option<String>,
        execution_time_ms: Option<u64>,
    ) -> Result<(), String> {
        let payload = ToolExecutionStatusPayload {
            uuid: uuid.to_string(),
            target_message_id: target_message_id.to_string(),
            tool_call_id: tool_call.id.clone(),
            tool_name: tool_call.function.name.clone(),
            phase,
            tool_call: Some(tool_call.clone()),
            result,
            error,
            execution_time_ms,
            timestamp: chrono::Utc::now().timestamp(),
        };

        app.emit("tool-execution-status", &payload)
            .map_err(|e| format!("发送工具状态事件失败: {}", e))?;

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
