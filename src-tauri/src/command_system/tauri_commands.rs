use crate::backend::application::command_service::CommandService;
use crate::command_system::command::{CommandMetadata, CommandResult};
use crate::command_system::loader;

/// 初始化命令系统
/// 在应用启动时调用，注册所有内置命令
pub async fn initialize_command_system() {
    CommandService::initialize().await;
    let count = loader::register_builtin_commands().await;
    println!("✅ 命令系统初始化完成，已注册 {} 个内置命令", count);
}

/// 获取可用命令列表
#[tauri::command]
pub async fn get_available_commands(app_handle: tauri::AppHandle) -> Result<Vec<CommandMetadata>, String> {
    CommandService::get_available_commands(&app_handle).await
}

/// 搜索命令
#[tauri::command]
pub async fn search_commands(
    app_handle: tauri::AppHandle,
    query: String,
) -> Result<Vec<CommandMetadata>, String> {
    CommandService::search_commands(&app_handle, query).await
}

/// 执行命令
#[tauri::command]
pub async fn execute_command(
    app_handle: tauri::AppHandle,
    command_id: String,
    _user_input: Option<String>,
) -> Result<CommandResult, String> {
    CommandService::execute_command(&app_handle, command_id, _user_input).await
}
