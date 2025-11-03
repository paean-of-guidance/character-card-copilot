use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::Manager;

/// 通用文件操作工具
pub struct FileUtils;

impl FileUtils {
    /// 确保目录存在，如果不存在则创建
    pub fn ensure_dir_exists(dir_path: &Path) -> Result<(), String> {
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dir_path.display(), e))?;
        }
        Ok(())
    }

    /// 获取应用数据目录
    pub fn get_app_data_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;

        Self::ensure_dir_exists(&app_data_dir)?;
        Ok(app_data_dir)
    }

    /// 读取JSON文件
    pub fn read_json_file<T: for<'de> Deserialize<'de>>(file_path: &Path) -> Result<T, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", file_path.display(), e))
    }

    /// 写入JSON文件
    pub fn write_json_file<T: Serialize + ?Sized>(
        file_path: &Path,
        data: &T,
    ) -> Result<(), String> {
        // 确保父目录存在
        if let Some(parent) = file_path.parent() {
            Self::ensure_dir_exists(parent)?;
        }

        let json_content = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize data to JSON: {}", e))?;

        fs::write(file_path, json_content)
            .map_err(|e| format!("Failed to write file {}: {}", file_path.display(), e))?;

        Ok(())
    }

    /// 删除文件或目录
    pub fn delete_path(path: &Path) -> Result<(), String> {
        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|e| format!("Failed to remove directory {}: {}", path.display(), e))?;
        } else {
            fs::remove_file(path)
                .map_err(|e| format!("Failed to remove file {}: {}", path.display(), e))?;
        }
        Ok(())
    }

    /// 生成UUID v4
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
