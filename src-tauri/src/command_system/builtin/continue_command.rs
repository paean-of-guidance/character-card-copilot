use async_trait::async_trait;

use crate::backend::application::session_service::SessionService;
use crate::backend::domain::{CommandCategory, CommandMetadata, CommandResult};
use crate::character_session::SESSION_MANAGER;
use crate::command_system::command::{CommandContext, CommandExecutor};

pub struct ContinueCommand {
    metadata: CommandMetadata,
}

impl ContinueCommand {
    pub fn new() -> Self {
        Self {
            metadata: CommandMetadata {
                id: "continue".to_string(),
                name: "/continue".to_string(),
                description: "基于最后一条用户消息继续生成回复".to_string(),
                icon: None,
                category: Some(CommandCategory::Chat),
                priority: 3,
                requires_confirmation: false,
                confirmation_message: None,
            },
        }
    }
}

#[async_trait]
impl CommandExecutor for ContinueCommand {
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

        matches!(session.chat_history.last(), Some(message) if message.role == "user")
    }

    async fn execute(&self, context: CommandContext) -> Result<CommandResult, String> {
        SessionService::continue_chat(&context.app_handle, None).await?;

        Ok(CommandResult {
            success: true,
            message: Some("已触发继续对话".to_string()),
            error: None,
            data: None,
        })
    }
}
