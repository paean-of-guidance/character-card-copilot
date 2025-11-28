use crate::ai_config::{AIConfigService, AIRole};

#[tauri::command]
pub async fn get_ai_config(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    AIConfigService::load_config(&app_handle).map(|config| serde_json::to_value(config).unwrap())
}

#[tauri::command]
pub async fn get_ai_role(
    app_handle: tauri::AppHandle,
    role_name: String,
) -> Result<Option<AIRole>, String> {
    AIConfigService::get_role(&app_handle, &role_name)
}

#[tauri::command]
pub async fn update_ai_role(
    app_handle: tauri::AppHandle,
    role_name: String,
    role: AIRole,
) -> Result<(), String> {
    AIConfigService::update_role(&app_handle, &role_name, &role)
}

#[tauri::command]
pub async fn add_ai_role(
    app_handle: tauri::AppHandle,
    role_name: String,
    role: AIRole,
) -> Result<(), String> {
    AIConfigService::add_role(&app_handle, &role_name, &role)
}

#[tauri::command]
pub async fn delete_ai_role(
    app_handle: tauri::AppHandle,
    role_name: String,
) -> Result<(), String> {
    AIConfigService::delete_role(&app_handle, &role_name)
}

#[tauri::command]
pub async fn set_default_ai_role(
    app_handle: tauri::AppHandle,
    role_name: String,
) -> Result<(), String> {
    AIConfigService::set_default_role(&app_handle, &role_name)
}

#[tauri::command]
pub async fn get_all_ai_roles(
    app_handle: tauri::AppHandle,
) -> Result<Vec<(String, AIRole)>, String> {
    AIConfigService::get_all_roles(&app_handle)
}

