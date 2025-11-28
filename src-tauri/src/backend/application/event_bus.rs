use crate::backend::domain::{
    CharacterUpdateType,
    SessionInfo,
    SessionUnloadReason,
    TokenUsageStats,
};
use crate::character_storage::CharacterData;
use crate::chat_history::ChatMessage;
use crate::context_builder::BuiltContextResult;
use crate::events::EventEmitter;
use serde_json::Value;

pub struct EventBus;

impl EventBus {
    pub fn character_loaded(
        app: &tauri::AppHandle,
        uuid: &str,
        data: &CharacterData,
    ) -> Result<(), String> {
        EventEmitter::send_character_loaded(app, uuid, data)
    }

    pub fn chat_history_loaded(
        app: &tauri::AppHandle,
        uuid: &str,
        history: &[ChatMessage],
    ) -> Result<(), String> {
        EventEmitter::send_chat_history_loaded(app, uuid, history)
    }

    pub fn message_sent(
        app: &tauri::AppHandle,
        uuid: &str,
        message: &ChatMessage,
    ) -> Result<(), String> {
        EventEmitter::send_message_sent(app, uuid, message)
    }

    pub fn session_unloaded(
        app: &tauri::AppHandle,
        uuid: &str,
        info: &SessionInfo,
        reason: SessionUnloadReason,
    ) -> Result<(), String> {
        EventEmitter::send_session_unloaded(app, uuid, info, reason)
    }

    pub fn context_built(
        app: &tauri::AppHandle,
        uuid: &str,
        result: &BuiltContextResult,
    ) -> Result<(), String> {
        EventEmitter::send_context_built(app, uuid, result)
    }

    pub fn message_received(
        app: &tauri::AppHandle,
        uuid: &str,
        message: &ChatMessage,
        intermediates: Option<Vec<ChatMessage>>,
    ) -> Result<(), String> {
        EventEmitter::send_message_received(app, uuid, message, intermediates)
    }

    pub fn token_stats(
        app: &tauri::AppHandle,
        uuid: &str,
        stats: TokenUsageStats,
    ) -> Result<(), String> {
        EventEmitter::send_token_stats(app, uuid, stats)
    }

    pub fn progress(
        app: &tauri::AppHandle,
        uuid: &str,
        operation: &str,
        progress: f64,
        message: Option<&str>,
    ) -> Result<(), String> {
        EventEmitter::send_progress(app, uuid, operation, progress, message)
    }

    pub fn character_updated(
        app: &tauri::AppHandle,
        uuid: &str,
        data: &CharacterData,
        update_type: CharacterUpdateType,
    ) -> Result<(), String> {
        EventEmitter::send_character_updated(app, uuid, data, update_type)
    }

    pub fn tool_executed(
        app: &tauri::AppHandle,
        uuid: &str,
        tool_name: &str,
        success: bool,
        result: Option<Value>,
        error: Option<String>,
        execution_time_ms: u64,
    ) -> Result<(), String> {
        EventEmitter::send_tool_executed(
            app,
            uuid,
            tool_name,
            success,
            result,
            error,
            execution_time_ms,
        )
    }
}
