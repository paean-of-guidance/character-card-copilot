use crate::ai_tools::{ToolCallRequest, ToolDefinition, ToolResult};
use crate::tools::ToolRegistry;

#[tauri::command]
pub async fn get_available_tools() -> Result<Vec<ToolDefinition>, String> {
    Ok(ToolRegistry::get_available_tools_global())
}

#[tauri::command]
pub async fn get_tools_by_category(category: String) -> Result<Vec<ToolDefinition>, String> {
    Ok(ToolRegistry::get_tools_by_category_global(&category))
}

#[tauri::command]
pub async fn execute_tool_call(
    app_handle: tauri::AppHandle,
    request: ToolCallRequest,
) -> Result<ToolResult, String> {
    Ok(ToolRegistry::execute_tool_call_global(&app_handle, &request).await)
}

#[tauri::command]
pub async fn get_tool_categories() -> Result<Vec<&'static str>, String> {
    Ok(ToolRegistry::get_tool_categories_global())
}
