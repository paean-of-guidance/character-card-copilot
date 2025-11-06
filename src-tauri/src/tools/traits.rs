use async_trait::async_trait;
use tauri::AppHandle;
use crate::ai_tools::{ToolResult, ToolCallRequest};

/// AI工具特征
#[async_trait]
pub trait AIToolTrait {
    /// 工具名称
    fn name(&self) -> &'static str;

    /// 工具描述
    fn description(&self) -> &'static str;

    /// 工具分类
    fn category(&self) -> &'static str;

    /// 获取工具参数定义
    fn parameters(&self) -> Vec<crate::ai_tools::ToolParameter>;

    /// 工具是否启用
    fn enabled(&self) -> bool { true }

    /// 执行工具调用
    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult;

    /// 将工具转换为标准结构
    fn to_tool(&self) -> crate::ai_tools::AITool {
        crate::ai_tools::AITool {
            name: self.name().to_string(),
            description: self.description().to_string(),
            category: self.category().to_string(),
            parameters: self.parameters(),
            enabled: self.enabled(),
        }
    }
}