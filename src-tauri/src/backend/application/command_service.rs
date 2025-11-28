use crate::backend::application::event_bus::EventBus;
use crate::command_system::builtin::ClearCommand;
use crate::command_system::command::{CommandContext, CommandMetadata, CommandResult};
use crate::command_system::registry::COMMAND_REGISTRY;
use std::sync::Arc;

pub struct CommandService;

impl CommandService {
    pub async fn initialize() {
        COMMAND_REGISTRY
            .register(Arc::new(ClearCommand::new()))
            .await;

        println!("✅ 命令系统初始化完成，已注册 1 个命令");
    }

    fn build_context(app_handle: &tauri::AppHandle) -> CommandContext {
        let session_uuid = crate::character_state::get_active_character();

        CommandContext {
            session_uuid,
            app_handle: app_handle.clone(),
        }
    }

    pub async fn get_available_commands(
        app_handle: &tauri::AppHandle,
    ) -> Result<Vec<CommandMetadata>, String> {
        let context = Self::build_context(app_handle);
        Ok(COMMAND_REGISTRY.get_available_commands(&context).await)
    }

    pub async fn search_commands(
        app_handle: &tauri::AppHandle,
        query: String,
    ) -> Result<Vec<CommandMetadata>, String> {
        let context = Self::build_context(app_handle);
        Ok(COMMAND_REGISTRY
            .search_commands(&query, &context)
            .await)
    }

    pub async fn execute_command(
        app_handle: &tauri::AppHandle,
        command_id: String,
        _user_input: Option<String>,
    ) -> Result<CommandResult, String> {
        let session_uuid = crate::character_state::get_active_character();

        let context = CommandContext {
            session_uuid: session_uuid.clone(),
            app_handle: app_handle.clone(),
        };

        if let Some(ref uuid) = context.session_uuid {
            EventBus::progress(
                app_handle,
                uuid,
                &format!("command:{}", command_id),
                0.0,
                Some("命令开始执行"),
            )?;
        }

        let result = COMMAND_REGISTRY
            .execute_command(&command_id, context)
            .await?;

        if let Some(uuid) = session_uuid {
            let message = if result.success {
                "命令执行成功"
            } else {
                "命令执行失败"
            };

            EventBus::progress(
                app_handle,
                &uuid,
                &format!("command:{}", command_id),
                1.0,
                Some(message),
            )?;
        }

        Ok(result)
    }
}
