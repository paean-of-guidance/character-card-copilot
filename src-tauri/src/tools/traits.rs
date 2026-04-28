use crate::ai_tools::ToolDefinition;
use crate::ai_tools::{ToolCallRequest, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
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

pub fn success_result(start_time: std::time::Instant, data: Value) -> ToolResult {
    ToolResult {
        success: true,
        data: Some(data),
        error: None,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    }
}

pub fn error_result(start_time: std::time::Instant, message: impl Into<String>) -> ToolResult {
    ToolResult {
        success: false,
        data: None,
        error: Some(message.into()),
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    }
}

pub fn detailed_error_result(
    start_time: std::time::Instant,
    message: impl Into<String>,
    details: Option<Value>,
) -> ToolResult {
    ToolResult {
        success: false,
        data: details.map(|details| json!({ "details": details })),
        error: Some(message.into()),
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    }
}

pub fn failure_result(
    start_time: std::time::Instant,
    code: &'static str,
    message: impl Into<String>,
    details: Option<Value>,
) -> ToolResult {
    ToolResult {
        success: false,
        data: Some(json!({
            "error_code": code,
            "details": details,
        })),
        error: Some(message.into()),
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    }
}
