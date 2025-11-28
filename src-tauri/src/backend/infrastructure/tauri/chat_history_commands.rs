use crate::character_session::SESSION_MANAGER;
use crate::chat_history::{ChatHistoryManager, ChatMessage};

#[tauri::command]
pub async fn save_chat_message(
    app_handle: tauri::AppHandle,
    character_id: String,
    message: ChatMessage,
) -> Result<(), String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.save_message(&message)
}

#[tauri::command]
pub async fn load_chat_history(
    app_handle: tauri::AppHandle,
    character_id: String,
) -> Result<Vec<ChatMessage>, String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.load_history()
}

#[tauri::command]
pub async fn clear_chat_history(
    app_handle: tauri::AppHandle,
    character_id: String,
) -> Result<(), String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.clear_history()?;

    if let Some(mut session) = SESSION_MANAGER.get_session(&character_id) {
        session.clear_history();
        SESSION_MANAGER.update_session(session)?;
        println!("✅ 已清空角色 {} 的聊天历史（内存+磁盘）", character_id);
    } else {
        println!("✅ 已清空角色 {} 的聊天历史（仅磁盘）", character_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_last_chat_message(
    app_handle: tauri::AppHandle,
    character_id: String,
) -> Result<Option<ChatMessage>, String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.get_last_message()
}

#[tauri::command]
pub async fn get_recent_chat_messages(
    app_handle: tauri::AppHandle,
    character_id: String,
    count: usize,
) -> Result<Vec<ChatMessage>, String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.get_recent_messages(count)
}

