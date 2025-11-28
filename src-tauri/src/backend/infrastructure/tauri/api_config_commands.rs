use crate::api_config::{
    ApiConfig, ApiConfigService, ApiTestResult, CreateApiRequest, ModelInfo, UpdateApiRequest,
};

#[tauri::command]
pub async fn get_all_api_configs(app_handle: tauri::AppHandle) -> Result<Vec<ApiConfig>, String> {
    ApiConfigService::get_all_api_configs(&app_handle)
}

#[tauri::command]
pub async fn get_api_config_by_profile(
    app_handle: tauri::AppHandle,
    profile: String,
) -> Result<Option<ApiConfig>, String> {
    ApiConfigService::get_api_config_by_profile(&app_handle, &profile)
}

#[tauri::command]
pub async fn get_default_api_config(
    app_handle: tauri::AppHandle,
) -> Result<Option<ApiConfig>, String> {
    ApiConfigService::get_default_api_config(&app_handle)
}

#[tauri::command]
pub async fn create_api_config(
    app_handle: tauri::AppHandle,
    request: CreateApiRequest,
) -> Result<ApiConfig, String> {
    ApiConfigService::create_api_config(&app_handle, request)
}

#[tauri::command]
pub async fn update_api_config(
    app_handle: tauri::AppHandle,
    request: UpdateApiRequest,
) -> Result<(), String> {
    ApiConfigService::update_api_config(&app_handle, request)
}

#[tauri::command]
pub async fn delete_api_config(
    app_handle: tauri::AppHandle,
    profile: String,
) -> Result<(), String> {
    ApiConfigService::delete_api_config(&app_handle, &profile)
}

#[tauri::command]
pub async fn set_default_api_config(
    app_handle: tauri::AppHandle,
    profile: String,
) -> Result<(), String> {
    ApiConfigService::set_default_api_config(&app_handle, &profile)
}

#[tauri::command]
pub async fn toggle_api_config(
    app_handle: tauri::AppHandle,
    profile: String,
    enabled: bool,
) -> Result<(), String> {
    ApiConfigService::toggle_api_config(&app_handle, &profile, enabled)
}

#[tauri::command]
pub async fn test_api_connection(
    app_handle: tauri::AppHandle,
    config: ApiConfig,
) -> Result<ApiTestResult, String> {
    ApiConfigService::test_api_connection(&app_handle, &config).await
}

#[tauri::command]
pub async fn fetch_models(
    app_handle: tauri::AppHandle,
    config: ApiConfig,
) -> Result<Vec<ModelInfo>, String> {
    ApiConfigService::fetch_models(&app_handle, &config).await
}

