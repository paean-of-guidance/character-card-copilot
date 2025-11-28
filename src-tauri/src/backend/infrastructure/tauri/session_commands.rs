use crate::backend::application::session_service::SessionService;
use crate::backend::domain::sessions::session::SessionInfo;

/// 加载角色会话
#[tauri::command]
pub async fn load_character_session(
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<SessionInfo, String> {
    SessionService::load_session(&app_handle, uuid).await
}

/// 发送聊天消息
#[tauri::command]
pub async fn send_chat_message(
    app_handle: tauri::AppHandle,
    message: String,
) -> Result<(), String> {
    SessionService::send_chat_message(&app_handle, message).await
}

/// 卸载角色会话
#[tauri::command]
pub async fn unload_character_session(
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<(), String> {
    SessionService::unload_session(&app_handle, uuid).await
}

/// 获取会话信息
#[tauri::command]
pub async fn get_session_info(uuid: String) -> Result<SessionInfo, String> {
    SessionService::get_session_info(uuid)
}

/// 获取所有活跃会话信息
#[tauri::command]
pub async fn get_all_sessions() -> Result<Vec<SessionInfo>, String> {
    SessionService::get_all_sessions()
}

/// 手动保存所有会话的历史记录
#[tauri::command]
pub async fn save_all_sessions(app_handle: tauri::AppHandle) -> Result<usize, String> {
    SessionService::save_all_sessions(&app_handle).await
}

/// 清理过期会话（基于最后活跃时间）
#[tauri::command]
pub async fn cleanup_expired_sessions(max_age_hours: u64) -> Result<usize, String> {
    SessionService::cleanup_expired_sessions(max_age_hours)
}

/// 删除指定索引的消息
#[tauri::command]
pub async fn delete_chat_message(
    app_handle: tauri::AppHandle,
    index: usize,
) -> Result<(), String> {
    SessionService::delete_chat_message(&app_handle, index).await
}

/// 编辑指定索引的消息
#[tauri::command]
pub async fn edit_chat_message(
    app_handle: tauri::AppHandle,
    index: usize,
    new_content: String,
) -> Result<(), String> {
    SessionService::edit_chat_message(&app_handle, index, new_content).await
}

/// 重新生成最后一条AI回复
#[tauri::command]
pub async fn regenerate_last_message(app_handle: tauri::AppHandle) -> Result<(), String> {
    SessionService::regenerate_last_message(&app_handle).await
}

/// 继续对话（当最后一条是用户消息时生成AI回复）
#[tauri::command]
pub async fn continue_chat(app_handle: tauri::AppHandle) -> Result<(), String> {
    SessionService::continue_chat(&app_handle).await
}

