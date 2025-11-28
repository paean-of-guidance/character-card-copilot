use crate::backend::domain::{CommandMetadata, CommandResult};
use super::command::{CommandContext, CommandExecutor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 全局命令注册表
pub struct CommandRegistry {
    commands: Arc<RwLock<HashMap<String, Arc<dyn CommandExecutor>>>>,
}

impl CommandRegistry {
    /// 创建新的命令注册表
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册命令
    pub async fn register(&self, executor: Arc<dyn CommandExecutor>) {
        let mut commands = self.commands.write().await;
        let id = executor.metadata().id.clone();
        commands.insert(id, executor);
    }

    /// 获取所有可用命令元数据
    pub async fn get_available_commands(&self, context: &CommandContext) -> Vec<CommandMetadata> {
        let commands = self.commands.read().await;
        let mut metadata_list = Vec::new();

        for executor in commands.values() {
            if executor.is_available(context).await {
                metadata_list.push(executor.metadata().clone());
            }
        }

        // 按优先级排序
        metadata_list.sort_by_key(|m| m.priority);
        metadata_list
    }

    /// 搜索命令
    pub async fn search_commands(
        &self,
        query: &str,
        context: &CommandContext,
    ) -> Vec<CommandMetadata> {
        let commands = self.get_available_commands(context).await;

        if query.is_empty() {
            return commands;
        }

        let normalized_query = query
            .to_lowercase()
            .trim_start_matches('/')
            .to_string();

        commands
            .into_iter()
            .filter(|cmd| {
                cmd.id.to_lowercase().contains(&normalized_query)
                    || cmd.name.to_lowercase().contains(&normalized_query)
                    || cmd.description.to_lowercase().contains(&normalized_query)
            })
            .collect()
    }

    /// 执行命令
    pub async fn execute_command(
        &self,
        command_id: &str,
        context: CommandContext,
    ) -> Result<CommandResult, String> {
        let commands = self.commands.read().await;

        if let Some(executor) = commands.get(command_id) {
            if !executor.is_available(&context).await {
                return Ok(CommandResult {
                    success: false,
                    error: Some(format!("命令 {} 当前不可用", command_id)),
                    message: None,
                    data: None,
                });
            }

            executor.execute(context).await
        } else {
            Err(format!("命令 {} 不存在", command_id))
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// 全局单例
lazy_static::lazy_static! {
    pub static ref COMMAND_REGISTRY: CommandRegistry = CommandRegistry::new();
}
