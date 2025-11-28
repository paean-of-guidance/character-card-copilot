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

use backend::infrastructure::tauri::{
    add_ai_role,
    check_token_limit,
    clear_chat_history,
    cleanup_expired_sessions,
    continue_chat,
    count_tokens,
    count_tokens_batch,
    create_api_config,
    create_character,
    create_chat_completion,
    create_streaming_chat_completion,
    delete_ai_role,
    delete_api_config,
    delete_character,
    delete_chat_message,
    edit_chat_message,
    execute_tool_call,
    export_character_card,
    fetch_models,
    generate_uuid,
    get_ai_config,
    get_ai_role,
    get_all_api_configs,
    get_all_ai_roles,
    get_all_characters,
    get_all_sessions,
    get_api_config_by_profile,
    get_available_tools,
    get_character_by_uuid,
    get_default_api_config,
    get_last_chat_message,
    get_recent_chat_messages,
    get_session_info,
    get_tool_categories,
    get_tools_by_category,
    import_character_card,
    import_character_card_from_bytes,
    load_character_session,
    load_chat_history,
    regenerate_last_message,
    save_all_sessions,
    save_chat_message,
    send_chat_message,
    set_default_ai_role,
    set_default_api_config,
    test_api_connection,
    toggle_api_config,
    truncate_to_token_limit,
    unload_character_session,
    update_ai_role,
    update_api_config,
    update_character,
    update_character_background_path,
    update_character_field,
    upload_background_image,
};
use character_state::{set_active_character, get_active_character, clear_active_character, has_active_character};
use context_builder::build_context;
use command_system::tauri_commands::{get_available_commands, search_commands, execute_command};

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
