use crate::ai_tools::ToolDefinition;
use crate::ai_tools::{ToolCallRequest, ToolResult};
use async_trait::async_trait;
use tauri::AppHandle;

/// AI工具特征
#[async_trait]
pub trait AIToolTrait {
    /// 工具名称
    fn name(&self) -> &'static str;

    /// 工具描述
    fn description(&self) -> &'static str;

    /// 工具分类
    fn category(&self) -> &'static str;

    /// 工具是否启用
    fn enabled(&self) -> bool {
        true
    }

    /// 执行工具调用
    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult;

    /// 将工具转换为通用 ToolDefinition 格式
    fn to_tool_definition(&self) -> ToolDefinition;
}
