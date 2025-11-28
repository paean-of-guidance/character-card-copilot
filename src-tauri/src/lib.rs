// Copyright 2025 Character Card Copilot Contributors
// SPDX-License-Identifier: Apache-2.0

mod file_utils;
mod character_storage;
mod api_config;
mod ai_config;
mod backend;
mod ai_tools;
mod ai_chat;
mod chat_history;
mod character_state;
mod character_session;
mod context_builder;
mod events;
mod png_utils;
mod token_counter;
mod tools;
mod command_system;

use character_storage::{CharacterStorage, CharacterData, TavernCardV2};
use api_config::{ApiConfigService, ApiConfig, CreateApiRequest, UpdateApiRequest, ApiTestResult, ModelInfo};
use ai_config::{AIConfigService, AIRole};
use ai_tools::{ToolCallRequest, ToolResult};
use ai_chat::{AIChatService, ChatCompletionRequest, ChatCompletionResponse, ChatTool};
use backend::application::tool_service::ToolService;
use chat_history::{ChatHistoryManager, ChatMessage};
use character_state::{set_active_character, get_active_character, clear_active_character, has_active_character};
use backend::infrastructure::tauri::session_commands::{
    load_character_session,
    send_chat_message,
    unload_character_session,
    get_session_info,
    get_all_sessions,
    save_all_sessions,
    cleanup_expired_sessions,
    delete_chat_message,
    edit_chat_message,
    regenerate_last_message,
    continue_chat,
};
use context_builder::build_context;
use token_counter::{get_token_counter, TokenCountResult};
use command_system::tauri_commands::{get_available_commands, search_commands, execute_command};

const ALTERNATE_GREETING_MARKER: &str = "<START_ALT>";

// ====================== 角色卡相关命令 ======================

#[tauri::command]
async fn get_all_characters(app_handle: tauri::AppHandle) -> Result<Vec<CharacterData>, String> {
    CharacterStorage::get_all_characters(&app_handle)
}

#[tauri::command]
async fn get_character_by_uuid(app_handle: tauri::AppHandle, uuid: String) -> Result<Option<CharacterData>, String> {
    CharacterStorage::get_character_by_uuid(&app_handle, &uuid)
}

#[tauri::command]
async fn create_character(app_handle: tauri::AppHandle, name: String) -> Result<CharacterData, String> {
    CharacterStorage::create_character(&app_handle, &name)
}

#[tauri::command]
async fn update_character(app_handle: tauri::AppHandle, uuid: String, card: TavernCardV2) -> Result<(), String> {
    CharacterStorage::update_character(&app_handle, &uuid, &card)
}

/// 更新角色的单个字段，保留其他所有数据（包括 character_book）
#[tauri::command]
async fn update_character_field(
    app_handle: tauri::AppHandle,
    uuid: String,
    field_name: String,
    field_value: String,
) -> Result<(), String> {
    use tauri::Emitter;

    // 获取当前角色数据
    let mut character_data = match CharacterStorage::get_character_by_uuid(&app_handle, &uuid)? {
        Some(data) => data,
        None => return Err(format!("角色 {} 不存在", uuid)),
    };

    // 更新指定字段
    match field_name.as_str() {
        "name" => character_data.card.data.name = field_value,
        "description" => character_data.card.data.description = field_value,
        "personality" => character_data.card.data.personality = field_value,
        "scenario" => character_data.card.data.scenario = field_value,
        "first_mes" => character_data.card.data.first_mes = field_value,
        "mes_example" => character_data.card.data.mes_example = field_value,
        "creator_notes" => character_data.card.data.creator_notes = field_value,
        "system_prompt" => character_data.card.data.system_prompt = field_value,
        "post_history_instructions" => character_data.card.data.post_history_instructions = field_value,
        "alternate_greetings" => {
            character_data.card.data.alternate_greetings = field_value
                .split(ALTERNATE_GREETING_MARKER)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "tags" => {
            character_data.card.data.tags = field_value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "creator" => character_data.card.data.creator = field_value,
        "character_version" => character_data.card.data.character_version = field_value,
        _ => return Err(format!("不支持的字段: {}", field_name)),
    }

    // 保存更新后的数据
    CharacterStorage::update_character(&app_handle, &uuid, &character_data.card)?;

    // 发送事件通知前端刷新
    if let Err(e) = app_handle.emit(
        "character-updated",
        serde_json::json!({
            "uuid": uuid,
            "character_data": character_data,
            "update_type": serde_json::json!({ "Fields": vec![field_name.clone()] })
        }),
    ) {
        eprintln!("发送角色更新事件失败: {}", e);
    }

    Ok(())
}

#[tauri::command]
async fn delete_character(app_handle: tauri::AppHandle, uuid: String) -> Result<(), String> {
    CharacterStorage::delete_character(&app_handle, &uuid)
}

#[tauri::command]
async fn upload_background_image(app_handle: tauri::AppHandle, uuid: String, image_data: Vec<u8>, extension: String) -> Result<String, String> {
    CharacterStorage::upload_background_image(&app_handle, &uuid, &image_data, &extension)
}

#[tauri::command]
async fn update_character_background_path(app_handle: tauri::AppHandle, uuid: String, background_path: String) -> Result<(), String> {
    CharacterStorage::update_character_background_path(&app_handle, &uuid, &background_path)
}

#[tauri::command]
async fn export_character_card(app_handle: tauri::AppHandle, uuid: String, output_path: String) -> Result<String, String> {
    CharacterStorage::export_character_card(&app_handle, &uuid, &output_path)
}

#[tauri::command]
async fn import_character_card(app_handle: tauri::AppHandle, file_path: String) -> Result<CharacterData, String> {
    CharacterStorage::import_character_card(&app_handle, &file_path)
}

#[tauri::command]
async fn import_character_card_from_bytes(app_handle: tauri::AppHandle, file_data: Vec<u8>, file_name: String) -> Result<CharacterData, String> {
    CharacterStorage::import_character_card_from_bytes(&app_handle, &file_data, &file_name)
}

// ====================== API配置相关命令 ======================

#[tauri::command]
async fn get_all_api_configs(app_handle: tauri::AppHandle) -> Result<Vec<ApiConfig>, String> {
    ApiConfigService::get_all_api_configs(&app_handle)
}

#[tauri::command]
async fn get_api_config_by_profile(app_handle: tauri::AppHandle, profile: String) -> Result<Option<ApiConfig>, String> {
    ApiConfigService::get_api_config_by_profile(&app_handle, &profile)
}

#[tauri::command]
async fn get_default_api_config(app_handle: tauri::AppHandle) -> Result<Option<ApiConfig>, String> {
    ApiConfigService::get_default_api_config(&app_handle)
}

#[tauri::command]
async fn create_api_config(app_handle: tauri::AppHandle, request: CreateApiRequest) -> Result<ApiConfig, String> {
    ApiConfigService::create_api_config(&app_handle, request)
}

#[tauri::command]
async fn update_api_config(app_handle: tauri::AppHandle, request: UpdateApiRequest) -> Result<(), String> {
    ApiConfigService::update_api_config(&app_handle, request)
}

#[tauri::command]
async fn delete_api_config(app_handle: tauri::AppHandle, profile: String) -> Result<(), String> {
    ApiConfigService::delete_api_config(&app_handle, &profile)
}

#[tauri::command]
async fn set_default_api_config(app_handle: tauri::AppHandle, profile: String) -> Result<(), String> {
    ApiConfigService::set_default_api_config(&app_handle, &profile)
}

#[tauri::command]
async fn toggle_api_config(app_handle: tauri::AppHandle, profile: String, enabled: bool) -> Result<(), String> {
    ApiConfigService::toggle_api_config(&app_handle, &profile, enabled)
}

#[tauri::command]
async fn test_api_connection(app_handle: tauri::AppHandle, config: ApiConfig) -> Result<ApiTestResult, String> {
    ApiConfigService::test_api_connection(&app_handle, &config).await
}

#[tauri::command]
async fn fetch_models(app_handle: tauri::AppHandle, config: ApiConfig) -> Result<Vec<ModelInfo>, String> {
    ApiConfigService::fetch_models(&app_handle, &config).await
}

// ====================== AI配置相关命令 ======================

#[tauri::command]
async fn get_ai_config(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    AIConfigService::load_config(&app_handle).map(|config| serde_json::to_value(config).unwrap())
}

#[tauri::command]
async fn get_ai_role(app_handle: tauri::AppHandle, role_name: String) -> Result<Option<AIRole>, String> {
    AIConfigService::get_role(&app_handle, &role_name)
}

#[tauri::command]
async fn update_ai_role(app_handle: tauri::AppHandle, role_name: String, role: AIRole) -> Result<(), String> {
    AIConfigService::update_role(&app_handle, &role_name, &role)
}

#[tauri::command]
async fn add_ai_role(app_handle: tauri::AppHandle, role_name: String, role: AIRole) -> Result<(), String> {
    AIConfigService::add_role(&app_handle, &role_name, &role)
}

#[tauri::command]
async fn delete_ai_role(app_handle: tauri::AppHandle, role_name: String) -> Result<(), String> {
    AIConfigService::delete_role(&app_handle, &role_name)
}

#[tauri::command]
async fn set_default_ai_role(app_handle: tauri::AppHandle, role_name: String) -> Result<(), String> {
    AIConfigService::set_default_role(&app_handle, &role_name)
}

#[tauri::command]
async fn get_all_ai_roles(app_handle: tauri::AppHandle) -> Result<Vec<(String, AIRole)>, String> {
    AIConfigService::get_all_roles(&app_handle)
}

// ====================== AI工具相关命令 ======================

#[tauri::command]
async fn get_available_tools() -> Result<Vec<ChatTool>, String> {
    Ok(ToolService::get_available_tools())
}

#[tauri::command]
async fn get_tools_by_category(category: String) -> Result<Vec<ChatTool>, String> {
    Ok(ToolService::get_tools_by_category(&category))
}

#[tauri::command]
async fn execute_tool_call(app_handle: tauri::AppHandle, request: ToolCallRequest) -> Result<ToolResult, String> {
    Ok(ToolService::execute_tool_call(&app_handle, request).await)
}

#[tauri::command]
async fn get_tool_categories() -> Result<Vec<&'static str>, String> {
    Ok(ToolService::get_tool_categories())
}

// ====================== AI聊天相关命令 ======================

#[tauri::command]
async fn create_chat_completion(
    app: tauri::AppHandle,
    api_config: ApiConfig,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, String> {
    AIChatService::create_chat_completion(&api_config, &request, Some(&app)).await
}

#[tauri::command]
async fn create_streaming_chat_completion(
    api_config: ApiConfig,
    request: ChatCompletionRequest,
) -> Result<String, String> {
    AIChatService::create_streaming_chat_completion(&api_config, &request).await
}

// ====================== 聊天历史相关命令 ======================

#[tauri::command]
async fn save_chat_message(
    app_handle: tauri::AppHandle,
    character_id: String,
    message: ChatMessage,
) -> Result<(), String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.save_message(&message)
}

#[tauri::command]
async fn load_chat_history(
    app_handle: tauri::AppHandle,
    character_id: String,
) -> Result<Vec<ChatMessage>, String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.load_history()
}

#[tauri::command]
async fn clear_chat_history(
    app_handle: tauri::AppHandle,
    character_id: String,
) -> Result<(), String> {
    // 清空磁盘文件
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.clear_history()?;

    // 如果该角色会话已加载到内存，也清空内存中的历史
    if let Some(mut session) = character_session::SESSION_MANAGER.get_session(&character_id) {
        session.clear_history();
        character_session::SESSION_MANAGER.update_session(session)?;
        println!("✅ 已清空角色 {} 的聊天历史（内存+磁盘）", character_id);
    } else {
        println!("✅ 已清空角色 {} 的聊天历史（仅磁盘）", character_id);
    }

    Ok(())
}

#[tauri::command]
async fn get_last_chat_message(
    app_handle: tauri::AppHandle,
    character_id: String,
) -> Result<Option<ChatMessage>, String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.get_last_message()
}

#[tauri::command]
async fn get_recent_chat_messages(
    app_handle: tauri::AppHandle,
    character_id: String,
    count: usize,
) -> Result<Vec<ChatMessage>, String> {
    let manager = ChatHistoryManager::new(&app_handle, &character_id);
    manager.get_recent_messages(count)
}

// ====================== Token 计数命令 ======================

#[tauri::command]
async fn count_tokens(text: String) -> Result<TokenCountResult, String> {
    let counter = get_token_counter();
    Ok(counter.count_tokens(&text))
}

#[tauri::command]
async fn count_tokens_batch(texts: Vec<String>) -> Result<Vec<TokenCountResult>, String> {
    let counter = get_token_counter();
    Ok(counter.count_tokens_batch(&texts))
}

#[tauri::command]
async fn check_token_limit(text: String, limit: usize) -> Result<bool, String> {
    let counter = get_token_counter();
    Ok(counter.is_within_limit(&text, limit))
}

#[tauri::command]
async fn truncate_to_token_limit(text: String, limit: usize) -> Result<String, String> {
    let counter = get_token_counter();
    Ok(counter.truncate_to_limit(&text, limit))
}

// ====================== 通用命令 ======================

#[tauri::command]
fn generate_uuid() -> String {
    file_utils::FileUtils::generate_uuid()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // 初始化命令系统
            tauri::async_runtime::spawn(async {
                command_system::tauri_commands::initialize_command_system().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 角色卡命令
            get_all_characters,
            get_character_by_uuid,
            create_character,
            update_character,
            update_character_field,
            delete_character,
            upload_background_image,
            update_character_background_path,
            export_character_card,
            import_character_card,
            import_character_card_from_bytes,
            // API配置命令
            get_all_api_configs,
            get_api_config_by_profile,
            get_default_api_config,
            create_api_config,
            update_api_config,
            delete_api_config,
            set_default_api_config,
            toggle_api_config,
            test_api_connection,
            fetch_models,
            // AI配置命令
            get_ai_config,
            get_ai_role,
            update_ai_role,
            add_ai_role,
            delete_ai_role,
            set_default_ai_role,
            get_all_ai_roles,
            // AI工具命令
            get_available_tools,
            get_tools_by_category,
            execute_tool_call,
            get_tool_categories,
            // AI聊天命令
            create_chat_completion,
            create_streaming_chat_completion,
            // 聊天历史命令
            save_chat_message,
            load_chat_history,
            clear_chat_history,
            get_last_chat_message,
            get_recent_chat_messages,
            // 角色状态管理命令
            set_active_character,
            get_active_character,
            clear_active_character,
            has_active_character,
            // 角色会话管理命令
            load_character_session,
            send_chat_message,
            unload_character_session,
            get_session_info,
            get_all_sessions,
            save_all_sessions,
            cleanup_expired_sessions,
            delete_chat_message,
            edit_chat_message,
            regenerate_last_message,
            continue_chat,
            // 上下文构建命令
            build_context,
            // Token 计数命令
            count_tokens,
            count_tokens_batch,
            check_token_limit,
            truncate_to_token_limit,
            // 命令系统
            get_available_commands,
            search_commands,
            execute_command,
            // 通用命令
            generate_uuid
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
