use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Arc;

use super::command::CommandExecutor;

mod clear_command;

pub use clear_command::ClearCommand;

pub type CommandBuilder = fn() -> Arc<dyn CommandExecutor>;

pub struct BuiltinCommandDescriptor {
    pub id: &'static str,
    pub description: &'static str,
    pub builder: CommandBuilder,
}

fn build_clear_command() -> Arc<dyn CommandExecutor> {
    Arc::new(ClearCommand::new())
}

pub fn builtin_manifest() -> Vec<BuiltinCommandDescriptor> {
    vec![BuiltinCommandDescriptor {
        id: "clear",
        description: "清空当前会话历史记录",
        builder: build_clear_command,
    }]
}

static DISABLED_COMMANDS: Lazy<HashSet<String>> = Lazy::new(|| {
    std::env::var("CCC_DISABLED_COMMANDS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
});

pub fn is_enabled(id: &str) -> bool {
    !DISABLED_COMMANDS.contains(&id.to_lowercase())
}

