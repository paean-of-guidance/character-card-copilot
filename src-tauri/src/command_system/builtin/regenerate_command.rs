use async_trait::async_trait;

use crate::backend::application::session_service::SessionService;
use crate::backend::domain::{CommandCategory, CommandMetadata, CommandResult};
use crate::character_session::SESSION_MANAGER;
use crate::command_system::command::{CommandContext, CommandExecutor};

pub struct RegenerateCommand {
    metadata: CommandMetadata,
}

impl RegenerateCommand {
    pub fn new() -> Self {
        Self {
            metadata: CommandMetadata {
                id: "regenerate".to_string(),
                name: "/regenerate".to_string(),
                description: "重新生成最后一条 AI 回复".to_string(),
                icon: None,
                category: Some(CommandCategory::Chat),
                priority: 2,
                requires_confirmation: false,
                confirmation_message: None,
            },
        }
    }
}

#[async_trait]
impl CommandExecutor for RegenerateCommand {
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    async fn is_available(&self, context: &CommandContext) -> bool {
        let Some(uuid) = &context.session_uuid else {
            return false;
        };

        let Some(session) = SESSION_MANAGER.get_session(uuid) else {
            return false;
        };

        matches!(session.chat_history.last(), Some(message) if message.role == "assistant")
    }

    async fn execute(&self, context: CommandContext) -> Result<CommandResult, String> {
        SessionService::regenerate_last_message(&context.app_handle).await?;

        Ok(CommandResult {
            success: true,
            message: Some("已触发重新生成".to_string()),
            error: None,
            data: None,
        })
    }
}
