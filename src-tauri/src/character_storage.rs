use super::file_utils::FileUtils;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 角色卡元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMeta {
    pub uuid: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Tavern Card V2 数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavernCardV2Data {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub creator_notes: String,
    pub system_prompt: String,
    pub post_history_instructions: String,
    pub alternate_greetings: Vec<String>,
    pub tags: Vec<String>,
    pub creator: String,
    pub character_version: String,
    pub extensions: serde_json::Value,
}

/// Tavern Card V2 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavernCardV2 {
    pub spec: String,
    pub spec_version: String,
    pub data: TavernCardV2Data,
}

/// 角色数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub uuid: String,
    pub meta: CharacterMeta,
    pub card: TavernCardV2,
    pub backgroundPath: String,
}

/// 角色卡存储服务
pub struct CharacterStorage;

impl CharacterStorage {
    /// 获取角色卡目录
    fn get_characters_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = FileUtils::get_app_data_dir(app_handle)?;
        let characters_dir = app_data_dir.join("character-cards");
        FileUtils::ensure_dir_exists(&characters_dir)?;
        Ok(characters_dir)
    }

    /// 获取角色卡文件路径
    fn get_character_file_path(
        app_handle: &tauri::AppHandle,
        uuid: &str,
    ) -> Result<PathBuf, String> {
        let characters_dir = Self::get_characters_dir(app_handle)?;
        let character_dir = characters_dir.join(uuid);
        FileUtils::ensure_dir_exists(&character_dir)?;
        Ok(character_dir.join("card.json"))
    }

    /// 获取背景图片目录
    fn get_backgrounds_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let characters_dir = Self::get_characters_dir(app_handle)?;
        let backgrounds_dir = characters_dir.join("backgrounds");
        FileUtils::ensure_dir_exists(&backgrounds_dir)?;
        Ok(backgrounds_dir)
    }

    /// 将图片路径转换为base64格式
    fn convert_image_path_to_base64(imagePath: &str) -> String {
        if imagePath.starts_with("data:") {
            // 已经是base64格式
            return imagePath.to_string();
        }

        // 如果是文件路径，转换为base64
        if let Ok(image_data) = fs::read(imagePath) {
            let base64_data = STANDARD.encode(&image_data);
            // 根据文件扩展名确定mime类型
            if let Some(extension) = std::path::Path::new(imagePath)
                .extension()
                .and_then(|s| s.to_str())
            {
                let mime_type = match extension.to_lowercase().as_str() {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "webp" => "image/webp",
                    _ => "image/png",
                };
                format!("data:{};base64,{}", mime_type, base64_data)
            } else {
                // 如果无法确定扩展名，默认为png
                format!("data:image/png;base64,{}", base64_data)
            }
        } else {
            // 如果无法读取文件，返回空字符串
            String::new()
        }
    }

    /// 获取所有角色卡列表
    pub fn get_all_characters(app_handle: &tauri::AppHandle) -> Result<Vec<CharacterData>, String> {
        let characters_dir = Self::get_characters_dir(app_handle)?;
        let mut characters = Vec::new();

        if !characters_dir.exists() {
            return Ok(characters);
        }

        for entry in fs::read_dir(&characters_dir)
            .map_err(|e| format!("Failed to read characters directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let card_file = path.join("card.json");
                if card_file.exists() {
                    match FileUtils::read_json_file::<CharacterData>(&card_file) {
                        Ok(mut character) => {
                            // 转换图片路径为base64格式
                            character.backgroundPath =
                                Self::convert_image_path_to_base64(&character.backgroundPath);
                            characters.push(character);
                        }
                        Err(e) => eprintln!(
                            "Failed to load character from {}: {}",
                            card_file.display(),
                            e
                        ),
                    }
                }
            }
        }

        Ok(characters)
    }

    /// 根据UUID获取角色卡
    pub fn get_character_by_uuid(
        app_handle: &tauri::AppHandle,
        uuid: &str,
    ) -> Result<Option<CharacterData>, String> {
        let card_file = Self::get_character_file_path(app_handle, uuid)?;

        if !card_file.exists() {
            return Ok(None);
        }

        let mut character = FileUtils::read_json_file::<CharacterData>(&card_file)?;
        // 转换图片路径为base64格式
        character.backgroundPath = Self::convert_image_path_to_base64(&character.backgroundPath);
        Ok(Some(character))
    }

    /// 创建新的角色卡
    pub fn create_character(
        app_handle: &tauri::AppHandle,
        name: &str,
    ) -> Result<CharacterData, String> {
        let uuid = FileUtils::generate_uuid();
        let now = chrono::Utc::now().to_rfc3339();

        let meta = CharacterMeta {
            uuid: uuid.clone(),
            version: "1.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        let card = TavernCardV2 {
            spec: "chara_card_v2".to_string(),
            spec_version: "2.0".to_string(),
            data: TavernCardV2Data {
                name: name.to_string(),
                description: String::new(),
                personality: String::new(),
                scenario: String::new(),
                first_mes: String::new(),
                mes_example: String::new(),
                creator_notes: String::new(),
                system_prompt: String::new(),
                post_history_instructions: String::new(),
                alternate_greetings: Vec::new(),
                tags: Vec::new(),
                creator: String::new(),
                character_version: "1.0".to_string(),
                extensions: serde_json::json!({}),
            },
        };

        let character_data = CharacterData {
            uuid: uuid.clone(),
            meta,
            card,
            backgroundPath: String::new(),
        };

        // 保存角色卡文件
        let card_file = Self::get_character_file_path(app_handle, &uuid)?;
        FileUtils::write_json_file(&card_file, &character_data)?;

        Ok(character_data)
    }

    /// 更新角色卡
    pub fn update_character(
        app_handle: &tauri::AppHandle,
        uuid: &str,
        card: &TavernCardV2,
    ) -> Result<(), String> {
        let card_file = Self::get_character_file_path(app_handle, uuid)?;

        if !card_file.exists() {
            return Err(format!("Character with UUID {} not found", uuid));
        }

        let mut character_data: CharacterData = FileUtils::read_json_file(&card_file)?;

        // 更新卡数据和修改时间
        character_data.card = card.clone();
        character_data.meta.updated_at = chrono::Utc::now().to_rfc3339();

        FileUtils::write_json_file(&card_file, &character_data)?;
        Ok(())
    }

    /// 删除角色卡
    pub fn delete_character(app_handle: &tauri::AppHandle, uuid: &str) -> Result<(), String> {
        let characters_dir = Self::get_characters_dir(app_handle)?;
        let character_dir = characters_dir.join(uuid);

        if character_dir.exists() {
            FileUtils::delete_path(&character_dir)?;
        }

        // 删除关联的背景图片
        let backgrounds_dir = Self::get_backgrounds_dir(app_handle)?;
        let background_patterns = [
            format!("{}_background.*", uuid),
            format!("{}_card.png", uuid),
        ];

        for _pattern in background_patterns {
            // 查找匹配的文件并删除
            if let Ok(entries) = backgrounds_dir.read_dir() {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if let Some(file_name) = file_path.file_name() {
                        if let Some(name_str) = file_name.to_str() {
                            if name_str.starts_with(&format!("{}_background", uuid))
                                || name_str == &format!("{}_card.png", uuid)
                            {
                                let _ = FileUtils::delete_path(&file_path);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 上传背景图片
    pub fn upload_background_image(
        app_handle: &tauri::AppHandle,
        uuid: &str,
        image_data: &[u8],
        extension: &str,
    ) -> Result<String, String> {
        let backgrounds_dir = Self::get_backgrounds_dir(app_handle)?;
        let file_name = format!("{}_background.{}", uuid, extension);
        let file_path = backgrounds_dir.join(&file_name);

        // 保存图片文件
        fs::write(&file_path, image_data)
            .map_err(|e| format!("Failed to write background image: {}", e))?;

        // 转换为base64返回给前端
        let base64_data = STANDARD.encode(image_data);
        let mime_type = match extension {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            _ => "image/png", // 默认
        };

        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }

    /// 更新角色背景图片路径
    pub fn update_character_background_path(
        app_handle: &tauri::AppHandle,
        uuid: &str,
        background_path: &str,
    ) -> Result<(), String> {
        let card_file = Self::get_character_file_path(app_handle, uuid)?;

        if !card_file.exists() {
            return Err(format!("Character with UUID {} not found", uuid));
        }

        let mut character_data: CharacterData = FileUtils::read_json_file(&card_file)?;

        // 更新背景路径为base64格式和修改时间
        character_data.backgroundPath = background_path.to_string();
        character_data.meta.updated_at = chrono::Utc::now().to_rfc3339();

        FileUtils::write_json_file(&card_file, &character_data)?;
        Ok(())
    }
}
