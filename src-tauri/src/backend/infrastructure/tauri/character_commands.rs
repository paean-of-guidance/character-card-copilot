use crate::backend::application::event_bus::EventBus;
use crate::character_storage::{CharacterData, CharacterStorage, TavernCardV2};
use crate::events::CharacterUpdateType;

const ALTERNATE_GREETING_MARKER: &str = "<START_ALT>";

#[tauri::command]
pub async fn get_all_characters(app_handle: tauri::AppHandle) -> Result<Vec<CharacterData>, String> {
    CharacterStorage::get_all_characters(&app_handle)
}

#[tauri::command]
pub async fn get_character_by_uuid(
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<Option<CharacterData>, String> {
    CharacterStorage::get_character_by_uuid(&app_handle, &uuid)
}

#[tauri::command]
pub async fn create_character(
    app_handle: tauri::AppHandle,
    name: String,
) -> Result<CharacterData, String> {
    CharacterStorage::create_character(&app_handle, &name)
}

#[tauri::command]
pub async fn update_character(
    app_handle: tauri::AppHandle,
    uuid: String,
    card: TavernCardV2,
) -> Result<(), String> {
    CharacterStorage::update_character(&app_handle, &uuid, &card)
}

#[tauri::command]
pub async fn update_character_field(
    app_handle: tauri::AppHandle,
    uuid: String,
    field_name: String,
    field_value: String,
) -> Result<(), String> {
    let mut character_data = match CharacterStorage::get_character_by_uuid(&app_handle, &uuid)? {
        Some(data) => data,
        None => return Err(format!("角色 {} 不存在", uuid)),
    };

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

    CharacterStorage::update_character(&app_handle, &uuid, &character_data.card)?;

    EventBus::character_updated(
        &app_handle,
        &uuid,
        &character_data,
        CharacterUpdateType::Fields {
            fields: vec![field_name],
        },
    )?;

    Ok(())
}

#[tauri::command]
pub async fn delete_character(app_handle: tauri::AppHandle, uuid: String) -> Result<(), String> {
    CharacterStorage::delete_character(&app_handle, &uuid)
}

#[tauri::command]
pub async fn upload_background_image(
    app_handle: tauri::AppHandle,
    uuid: String,
    image_data: Vec<u8>,
    extension: String,
) -> Result<String, String> {
    CharacterStorage::upload_background_image(&app_handle, &uuid, &image_data, &extension)
}

#[tauri::command]
pub async fn update_character_background_path(
    app_handle: tauri::AppHandle,
    uuid: String,
    background_path: String,
) -> Result<(), String> {
    CharacterStorage::update_character_background_path(&app_handle, &uuid, &background_path)
}

#[tauri::command]
pub async fn export_character_card(
    app_handle: tauri::AppHandle,
    uuid: String,
    output_path: String,
) -> Result<String, String> {
    CharacterStorage::export_character_card(&app_handle, &uuid, &output_path)
}

#[tauri::command]
pub async fn import_character_card(
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<CharacterData, String> {
    CharacterStorage::import_character_card(&app_handle, &file_path)
}

#[tauri::command]
pub async fn import_character_card_from_bytes(
    app_handle: tauri::AppHandle,
    file_data: Vec<u8>,
    file_name: String,
) -> Result<CharacterData, String> {
    CharacterStorage::import_character_card_from_bytes(&app_handle, &file_data, &file_name)
}

