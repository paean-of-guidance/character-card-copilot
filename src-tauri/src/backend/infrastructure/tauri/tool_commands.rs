use crate::ai_chat::ChatTool;
use crate::ai_tools::{ToolCallRequest, ToolResult};
use crate::backend::application::tool_service::ToolService;

#[tauri::command]
pub async fn get_available_tools() -> Result<Vec<ChatTool>, String> {
    Ok(ToolService::get_available_tools())
}

#[tauri::command]
pub async fn get_tools_by_category(category: String) -> Result<Vec<ChatTool>, String> {
    Ok(ToolService::get_tools_by_category(&category))
}

#[tauri::command]
pub async fn execute_tool_call(
    app_handle: tauri::AppHandle,
    request: ToolCallRequest,
) -> Result<ToolResult, String> {
    Ok(ToolService::execute_tool_call(&app_handle, request).await)
}

#[tauri::command]
pub async fn get_tool_categories() -> Result<Vec<&'static str>, String> {
    Ok(ToolService::get_tool_categories())
}

