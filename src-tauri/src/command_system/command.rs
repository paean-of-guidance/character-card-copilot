use async_trait::async_trait;
use crate::backend::domain::{CommandMetadata, CommandResult};

/// 命令执行上下文
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// 当前会话UUID（可选）
    pub session_uuid: Option<String>,
    /// Tauri应用句柄
    pub app_handle: tauri::AppHandle,
}

/// 命令执行器特征
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// 获取命令元数据
    fn metadata(&self) -> &CommandMetadata;

    /// 检查命令是否可用
    /// 默认实现：总是可用
    async fn is_available(&self, _context: &CommandContext) -> bool {
        true
    }

    /// 执行命令
    async fn execute(&self, context: CommandContext) -> Result<CommandResult, String>;
}
