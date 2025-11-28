use crate::ai_chat::ChatTool;
use crate::ai_tools::{ToolCallRequest, ToolResult};
use crate::tools::ToolRegistry;

pub struct ToolService;

impl ToolService {
    pub fn get_available_tools() -> Vec<ChatTool> {
        ToolRegistry::get_available_tools_global()
    }

    pub fn get_tools_by_category(category: &str) -> Vec<ChatTool> {
        ToolRegistry::get_tools_by_category_global(category)
    }

    pub fn get_tool_categories() -> Vec<&'static str> {
        ToolRegistry::get_tool_categories_global()
    }

    pub async fn execute_tool_call(
        app_handle: &tauri::AppHandle,
        request: ToolCallRequest,
    ) -> ToolResult {
        ToolRegistry::execute_tool_call_global(app_handle, &request).await
    }
}

