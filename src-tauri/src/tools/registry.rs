use super::AIToolTrait;
use crate::ai_tools::{AITool, ToolCallRequest, ToolResult};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::AppHandle;

/// 工具注册中心
pub struct ToolRegistry {
    pub(crate) tools: HashMap<String, Arc<dyn AIToolTrait + Send + Sync>>,
}

impl ToolRegistry {
    /// 创建新的工具注册中心
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register_tool<T: AIToolTrait + Send + Sync + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// 获取所有可用工具
    pub fn get_available_tools(&self) -> Vec<AITool> {
        self.tools
            .values()
            .filter(|tool| tool.enabled())
            .map(|tool| tool.to_tool())
            .collect()
    }

    /// 执行工具调用（从全局注册中心）
    ///
    /// 这是一个关联函数，不持有 self 引用，避免跨 await 点持有锁
    pub async fn execute_tool_call_global(
        app_handle: &AppHandle,
        request: &ToolCallRequest,
    ) -> ToolResult {
        // 克隆 tool_name 以避免借用整个 request
        let tool_name = request.tool_name.clone();

        // 在锁的作用域内获取工具的 Arc 副本，然后释放锁
        let tool_opt = {
            let registry = TOOL_REGISTRY.read().unwrap();
            registry.tools.get(&tool_name).cloned()
        };

        let start_time = std::time::Instant::now();

        // 锁已释放，可以安全地执行异步调用
        if let Some(tool) = tool_opt {
            tool.execute(app_handle, request).await
        } else {
            ToolResult {
                success: false,
                data: None,
                error: Some(format!("Unknown tool: {}", tool_name)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            }
        }
    }

    /// 获取工具分类
    pub fn get_tool_categories(&self) -> Vec<&'static str> {
        let mut categories: std::collections::HashSet<&'static str> =
            std::collections::HashSet::new();
        for tool in self.tools.values() {
            categories.insert(tool.category());
        }
        categories.into_iter().collect()
    }

    /// 根据分类获取工具
    pub fn get_tools_by_category(&self, category: &str) -> Vec<AITool> {
        self.tools
            .values()
            .filter(|tool| tool.category() == category && tool.enabled())
            .map(|tool| tool.to_tool())
            .collect()
    }

    // ========== 便捷的静态方法 ==========

    /// 获取所有可用工具（静态方法）
    pub fn get_available_tools_global() -> Vec<AITool> {
        let registry = TOOL_REGISTRY.read().unwrap();
        registry.get_available_tools()
    }

    /// 获取工具分类（静态方法）
    pub fn get_tool_categories_global() -> Vec<&'static str> {
        let registry = TOOL_REGISTRY.read().unwrap();
        registry.get_tool_categories()
    }

    /// 根据分类获取工具（静态方法）
    pub fn get_tools_by_category_global(category: &str) -> Vec<AITool> {
        let registry = TOOL_REGISTRY.read().unwrap();
        registry.get_tools_by_category(category)
    }
}

// 全局工具注册中心实例
lazy_static::lazy_static! {
    pub static ref TOOL_REGISTRY: std::sync::RwLock<ToolRegistry> = {
        let mut registry = ToolRegistry::new();
        // 注册所有工具
        registry.register_tool(super::character_editor::EditCharacterTool);
        registry.register_tool(super::world_book_creator::CreateWorldBookEntryTool);
        std::sync::RwLock::new(registry)
    };
}
