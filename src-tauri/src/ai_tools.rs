use crate::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tauri::AppHandle;

/// AI工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: String, // "string", "number", "boolean", "object", "array"
    pub required: bool,
    pub schema: Option<Value>, // JSON Schema for validation
}

/// AI工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITool {
    pub name: String,
    pub description: String,
    pub category: String, // "character", "content", "analysis", "utility"
    pub parameters: Vec<ToolParameter>,
    pub enabled: bool,
}

/// AI工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// AI工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub tool_name: String,
    pub parameters: HashMap<String, Value>,
    pub character_uuid: Option<String>, // 角色UUID
    pub context: Option<Value>,         // CharacterData or other context
}

/// AI工具服务 - 对工具注册中心的简单封装
///
/// 这个服务层提供统一的对外接口，所有方法都委托给工具注册中心
pub struct AIToolService;

impl AIToolService {
    /// 获取所有可用工具
    pub fn get_available_tools() -> Vec<AITool> {
        ToolRegistry::get_available_tools_global()
    }

    /// 执行工具调用
    pub async fn execute_tool_call(app_handle: &AppHandle, request: ToolCallRequest) -> ToolResult {
        ToolRegistry::execute_tool_call_global(app_handle, &request).await
    }

    /// 获取工具分类
    pub fn get_tool_categories() -> Vec<&'static str> {
        ToolRegistry::get_tool_categories_global()
    }

    /// 根据分类获取工具
    pub fn get_tools_by_category(category: &str) -> Vec<AITool> {
        ToolRegistry::get_tools_by_category_global(category)
    }
}
