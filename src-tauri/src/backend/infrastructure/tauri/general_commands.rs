#[tauri::command]
pub fn generate_uuid() -> String {
    crate::file_utils::FileUtils::generate_uuid()
}

