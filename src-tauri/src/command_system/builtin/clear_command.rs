use async_trait::async_trait;
use crate::backend::application::event_bus::EventBus;
use crate::command_system::command::*;
use crate::character_session::SESSION_MANAGER;
use crate::chat_history::ChatHistoryManager;

/// /clear 命令 - 清空所有对话记录
pub struct ClearCommand {
    metadata: CommandMetadata,
}

impl ClearCommand {
    pub fn new() -> Self {
        Self {
            metadata: CommandMetadata {
                id: "clear".to_string(),
                name: "/clear".to_string(),
                description: "清空所有对话记录".to_string(),
                icon: Some("MdOutlineDelete".to_string()),
                category: Some(CommandCategory::History),
                priority: 1,
                requires_confirmation: true,
                confirmation_message: Some("确定要清空所有对话记录吗？此操作不可撤销。".to_string()),
            },
        }
    }
}

#[async_trait]
impl CommandExecutor for ClearCommand {
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    async fn is_available(&self, context: &CommandContext) -> bool {
        // 检查是否有活跃会话且有聊天记录
        if let Some(uuid) = &context.session_uuid {
            if let Some(session) = SESSION_MANAGER.get_session(uuid) {
                return !session.chat_history.is_empty();
            }
        }
        false
    }

    async fn execute(&self, context: CommandContext) -> Result<CommandResult, String> {
        let uuid = context
            .session_uuid
            .ok_or("没有活跃的会话")?;

        // 获取会话
        let mut session = SESSION_MANAGER
            .get_session(&uuid)
            .ok_or("会话不存在")?;

        // 清空聊天历史（内存）
        session.clear_history();

        // 删除持久化的历史记录
        let history_manager = ChatHistoryManager::new(&context.app_handle, &uuid);
        history_manager.clear_history()?;

        // 更新会话
        SESSION_MANAGER.update_session(session)?;

        // 发送事件通知前端
        EventBus::chat_history_loaded(&context.app_handle, &uuid, &[])?;

        Ok(CommandResult {
            success: true,
            message: Some("已清空所有对话记录".to_string()),
            error: None,
            data: None,
        })
    }
}
