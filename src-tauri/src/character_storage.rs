use super::file_utils::FileUtils;
use super::png_utils::PngMetadataUtils;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 角色卡元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMeta {
    pub uuid: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

// 默认 extensions 值
fn default_extensions() -> serde_json::Value {
    serde_json::json!({})
}

/// 世界书条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBookEntry {
    pub keys: Vec<String>,
    pub content: String,
    #[serde(default = "default_extensions")]
    pub extensions: serde_json::Value,
    pub enabled: bool,
    pub insertion_order: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_sensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selective: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
}

/// 世界书（Character Book）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_depth: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive_scanning: Option<bool>,
    #[serde(default = "default_extensions")]
    pub extensions: serde_json::Value,
    pub entries: Vec<WorldBookEntry>,
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
    #[serde(default = "default_extensions")]
    pub extensions: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_book: Option<CharacterBook>,
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
    #[serde(rename = "backgroundPath")]
    pub background_path: String,
    #[serde(rename = "thumbnailPath", default)]
    pub thumbnail_path: String,
}

const CARD_FILE_NAME: &str = "card.png";
const THUMBNAIL_FILE_NAME: &str = "thumbnail.png";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePaths {
    #[serde(rename = "backgroundPath")]
    pub background_path: String,
    #[serde(rename = "thumbnailPath")]
    pub thumbnail_path: String,
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

    /// 获取角色目录
    fn get_character_dir(app_handle: &tauri::AppHandle, uuid: &str) -> Result<PathBuf, String> {
        let characters_dir = Self::get_characters_dir(app_handle)?;
        let character_dir = characters_dir.join(uuid);
        FileUtils::ensure_dir_exists(&character_dir)?;
        Ok(character_dir)
    }

    /// 获取角色卡文件路径
    fn get_character_file_path(
        app_handle: &tauri::AppHandle,
        uuid: &str,
    ) -> Result<PathBuf, String> {
        let character_dir = Self::get_character_dir(app_handle, uuid)?;
        Ok(character_dir.join("character.json"))
    }

    /// 兼容旧文件名 card.json（仅用于迁移）
    fn get_legacy_character_file_path(
        app_handle: &tauri::AppHandle,
        uuid: &str,
    ) -> Result<PathBuf, String> {
        let character_dir = Self::get_character_dir(app_handle, uuid)?;
        Ok(character_dir.join("card.json"))
    }

    /// 获取角色原始图片路径（固定文件名）
    fn get_card_image_path(app_handle: &tauri::AppHandle, uuid: &str) -> Result<PathBuf, String> {
        let character_dir = Self::get_character_dir(app_handle, uuid)?;
        Ok(character_dir.join(CARD_FILE_NAME))
    }

    /// 获取角色缩略图路径（固定文件名）
    fn get_thumbnail_image_path(
        app_handle: &tauri::AppHandle,
        uuid: &str,
    ) -> Result<PathBuf, String> {
        let character_dir = Self::get_character_dir(app_handle, uuid)?;
        Ok(character_dir.join(THUMBNAIL_FILE_NAME))
    }

    /// 获取背景图片目录
    fn get_backgrounds_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let characters_dir = Self::get_characters_dir(app_handle)?;
        let backgrounds_dir = characters_dir.join("backgrounds");
        FileUtils::ensure_dir_exists(&backgrounds_dir)?;
        Ok(backgrounds_dir)
    }

    /// 将 data URL 解码为字节
    fn decode_data_url(data_url: &str) -> Result<Vec<u8>, String> {
        if !data_url.starts_with("data:") {
            return Err("无效的 data URL".to_string());
        }

        let parts: Vec<&str> = data_url.split(',').collect();
        if parts.len() != 2 {
            return Err("无效的图片数据格式".to_string());
        }

        STANDARD
            .decode(parts[1])
            .map_err(|e| format!("解码图片数据失败: {}", e))
    }

    /// 保存 card.png 与 thumbnail.png（均为 PNG）
    fn write_card_and_thumbnail(
        card_path: &Path,
        thumbnail_path: &Path,
        image_bytes: &[u8],
    ) -> Result<(), String> {
        let image =
            image::load_from_memory(image_bytes).map_err(|e| format!("解析图片失败: {}", e))?;

        let mut card_file =
            fs::File::create(card_path).map_err(|e| format!("写入背景图片失败: {}", e))?;
        image
            .write_to(&mut card_file, ImageFormat::Png)
            .map_err(|e| format!("写入背景图片失败: {}", e))?;

        Self::write_thumbnail(&image, thumbnail_path)
    }

    /// 写入缩略图
    fn write_thumbnail(image: &DynamicImage, thumbnail_path: &Path) -> Result<(), String> {
        let resized = image.resize(320, 320, FilterType::Triangle);
        let mut thumb_file =
            fs::File::create(thumbnail_path).map_err(|e| format!("写入缩略图失败: {}", e))?;
        resized
            .write_to(&mut thumb_file, ImageFormat::Png)
            .map_err(|e| format!("写入缩略图失败: {}", e))
    }

    /// 如果缩略图缺失，从 card.png 生成
    fn ensure_thumbnail_from_card(card_path: &Path, thumbnail_path: &Path) -> Result<(), String> {
        if thumbnail_path.exists() || !card_path.exists() {
            return Ok(());
        }
        let image = image::open(card_path).map_err(|e| format!("读取背景图片失败: {}", e))?;
        Self::write_thumbnail(&image, thumbnail_path)
    }

    /// 将存储中的相对路径转换为绝对路径（返回给前端时使用）
    fn apply_absolute_paths(
        app_handle: &tauri::AppHandle,
        character_data: &mut CharacterData,
    ) -> Result<(), String> {
        let character_dir = Self::get_character_dir(app_handle, &character_data.uuid)?;

        if !character_data.background_path.is_empty() {
            let path = Path::new(&character_data.background_path);
            if !path.is_absolute() {
                character_data.background_path =
                    character_dir.join(path).to_string_lossy().to_string();
            }
        }

        if !character_data.thumbnail_path.is_empty() {
            let path = Path::new(&character_data.thumbnail_path);
            if !path.is_absolute() {
                character_data.thumbnail_path =
                    character_dir.join(path).to_string_lossy().to_string();
            }
        }

        Ok(())
    }

    /// 迁移旧格式（base64 / 旧路径）到新文件结构，并在需要时回写 JSON
    fn migrate_character_assets(
        app_handle: &tauri::AppHandle,
        character_file: &Path,
        character_data: &mut CharacterData,
    ) -> Result<(), String> {
        let card_path = Self::get_card_image_path(app_handle, &character_data.uuid)?;
        let thumbnail_path = Self::get_thumbnail_image_path(app_handle, &character_data.uuid)?;
        let mut updated = false;

        if !character_data.background_path.is_empty()
            && character_data.background_path.starts_with("data:")
        {
            let image_bytes = Self::decode_data_url(&character_data.background_path)?;
            Self::write_card_and_thumbnail(&card_path, &thumbnail_path, &image_bytes)?;
            character_data.background_path = CARD_FILE_NAME.to_string();
            character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
            updated = true;
        } else if !character_data.background_path.is_empty() {
            let path = Path::new(&character_data.background_path);
            // 兼容旧的绝对路径或相对路径，统一迁移到固定文件名
            let resolved_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                Self::get_character_dir(app_handle, &character_data.uuid)?.join(path)
            };

            if resolved_path.exists() && resolved_path != card_path {
                let image_bytes =
                    fs::read(&resolved_path).map_err(|e| format!("读取背景图片失败: {}", e))?;
                Self::write_card_and_thumbnail(&card_path, &thumbnail_path, &image_bytes)?;
                character_data.background_path = CARD_FILE_NAME.to_string();
                character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
                updated = true;
            }
        } else if card_path.exists() {
            // JSON 缺失路径但文件已存在
            character_data.background_path = CARD_FILE_NAME.to_string();
            updated = true;
        }

        // 缩略图缺失时尝试生成
        if card_path.exists() && !thumbnail_path.exists() {
            Self::ensure_thumbnail_from_card(&card_path, &thumbnail_path)?;
        }

        if character_data.thumbnail_path.is_empty() && thumbnail_path.exists() {
            character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
            updated = true;
        }

        if updated {
            character_data.meta.updated_at = chrono::Utc::now().to_rfc3339();
            FileUtils::write_json_file(character_file, character_data)?;
        }

        Ok(())
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
                let card_file = path.join("character.json");
                let legacy_card_file = path.join("card.json");

                let active_card_file = if card_file.exists() {
                    card_file.clone()
                } else if legacy_card_file.exists() {
                    // 尝试迁移旧文件名
                    match fs::rename(&legacy_card_file, &card_file) {
                        Ok(_) => card_file.clone(),
                        Err(err) => {
                            eprintln!(
                                "迁移 card.json -> character.json 失败，继续使用旧文件: {}",
                                err
                            );
                            legacy_card_file.clone()
                        }
                    }
                } else {
                    continue;
                };

                match FileUtils::read_json_file::<CharacterData>(&active_card_file) {
                    Ok(mut character) => {
                        // 迁移旧数据并生成缩略图（写入新文件名）
                        Self::migrate_character_assets(app_handle, &card_file, &mut character)?;

                        // 返回给前端时使用绝对路径
                        let mut response_character = character.clone();
                        Self::apply_absolute_paths(app_handle, &mut response_character)?;
                        characters.push(response_character);
                    }
                    Err(e) => eprintln!(
                        "Failed to load character from {}: {}",
                        active_card_file.display(),
                        e
                    ),
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
        let mut card_file = Self::get_character_file_path(app_handle, uuid)?;

        if !card_file.exists() {
            let legacy_card_file = Self::get_legacy_character_file_path(app_handle, uuid)?;
            if legacy_card_file.exists() {
                if let Err(err) = fs::rename(&legacy_card_file, &card_file) {
                    eprintln!(
                        "迁移 card.json -> character.json 失败，继续使用旧文件: {}",
                        err
                    );
                    card_file = legacy_card_file;
                }
            } else {
                return Ok(None);
            }
        }

        let mut character = FileUtils::read_json_file::<CharacterData>(&card_file)?;
        Self::migrate_character_assets(app_handle, &card_file, &mut character)?;

        let mut response = character.clone();
        Self::apply_absolute_paths(app_handle, &mut response)?;
        Ok(Some(response))
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
                character_book: None,
            },
        };

        let character_data = CharacterData {
            uuid: uuid.clone(),
            meta,
            card,
            background_path: String::new(),
            thumbnail_path: String::new(),
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
                                || name_str == format!("{}_card.png", uuid)
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
        _extension: &str,
    ) -> Result<ImagePaths, String> {
        let card_path = Self::get_card_image_path(app_handle, uuid)?;
        let thumbnail_path = Self::get_thumbnail_image_path(app_handle, uuid)?;

        // 始终以 PNG 格式写入固定文件
        Self::write_card_and_thumbnail(&card_path, &thumbnail_path, image_data)?;

        // 更新 JSON 中的路径为固定文件名
        let card_file = Self::get_character_file_path(app_handle, uuid)?;
        if card_file.exists() {
            let mut character_data: CharacterData = FileUtils::read_json_file(&card_file)?;
            character_data.background_path = CARD_FILE_NAME.to_string();
            character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
            character_data.meta.updated_at = chrono::Utc::now().to_rfc3339();
            FileUtils::write_json_file(&card_file, &character_data)?;
        }

        Ok(ImagePaths {
            background_path: card_path.to_string_lossy().to_string(),
            thumbnail_path: thumbnail_path.to_string_lossy().to_string(),
        })
    }

    /// 更新角色背景图片路径
    pub fn update_character_background_path(
        app_handle: &tauri::AppHandle,
        uuid: &str,
        _background_path: &str,
    ) -> Result<(), String> {
        let card_file = Self::get_character_file_path(app_handle, uuid)?;

        if !card_file.exists() {
            return Err(format!("Character with UUID {} not found", uuid));
        }

        let mut character_data: CharacterData = FileUtils::read_json_file(&card_file)?;

        // 统一使用固定文件名，忽略传入的路径，保证一致性
        let card_path = Self::get_card_image_path(app_handle, uuid)?;
        let thumbnail_path = Self::get_thumbnail_image_path(app_handle, uuid)?;
        if card_path.exists() {
            character_data.background_path = CARD_FILE_NAME.to_string();
            // 确保缩略图存在
            let _ = Self::ensure_thumbnail_from_card(&card_path, &thumbnail_path);
        } else {
            character_data.background_path.clear();
        }

        if thumbnail_path.exists() {
            character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
        } else {
            character_data.thumbnail_path.clear();
        }

        character_data.meta.updated_at = chrono::Utc::now().to_rfc3339();

        FileUtils::write_json_file(&card_file, &character_data)?;
        Ok(())
    }

    /// 导出角色卡
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄
    /// * `uuid` - 角色 UUID
    /// * `output_path` - 输出文件路径
    ///
    /// # 返回
    /// * `Ok(String)` - 导出的文件类型（"json" 或 "png"）
    pub fn export_character_card(
        app_handle: &tauri::AppHandle,
        uuid: &str,
        output_path: &str,
    ) -> Result<String, String> {
        // 读取角色数据
        let character = Self::get_character_by_uuid(app_handle, uuid)?
            .ok_or_else(|| format!("角色 {} 不存在", uuid))?;

        // 序列化 TavernCardV2 为 JSON
        let card_json = serde_json::to_string_pretty(&character.card)
            .map_err(|e| format!("序列化角色卡失败: {}", e))?;

        let card_image_path = Self::get_card_image_path(app_handle, uuid)?;

        if card_image_path.exists() {
            let image_data =
                fs::read(&card_image_path).map_err(|e| format!("读取背景图片失败: {}", e))?;

            // 将角色卡数据写入 PNG
            let output_bytes =
                PngMetadataUtils::write_character_data_to_bytes(&image_data, &card_json)
                    .map_err(|e| format!("写入 PNG 元数据失败: {}", e))?;

            // 保存到文件
            fs::write(output_path, output_bytes)
                .map_err(|e| format!("保存 PNG 文件失败: {}", e))?;

            Ok("png".to_string())
        } else {
            // 没有图片，直接导出 JSON
            fs::write(output_path, card_json).map_err(|e| format!("保存 JSON 文件失败: {}", e))?;

            Ok("json".to_string())
        }
    }

    /// 从 PNG 或 JSON 导入角色卡
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄
    /// * `file_path` - 导入文件路径
    ///
    /// # 返回
    /// * `Ok(CharacterData)` - 导入的角色数据
    pub fn import_character_card(
        app_handle: &tauri::AppHandle,
        file_path: &str,
    ) -> Result<CharacterData, String> {
        // 读取文件
        let file_data = fs::read(file_path).map_err(|e| format!("读取文件失败: {}", e))?;

        // 判断文件类型
        let is_png = file_path.ends_with(".png");

        // 尝试解析为 PNG
        let card_json = if is_png {
            // 从 PNG 中提取角色卡数据
            PngMetadataUtils::read_character_data_from_bytes(&file_data)
                .map_err(|e| format!("从 PNG 读取角色卡数据失败: {}", e))?
        } else {
            // 作为 JSON 解析
            String::from_utf8(file_data.clone())
                .map_err(|e| format!("读取 JSON 文件失败: {}", e))?
        };

        // 解析 TavernCardV2
        let card: TavernCardV2 =
            serde_json::from_str(&card_json).map_err(|e| format!("解析角色卡数据失败: {}", e))?;

        // 生成新的 UUID 和元数据
        let uuid = FileUtils::generate_uuid();
        let now = chrono::Utc::now().to_rfc3339();

        let meta = CharacterMeta {
            uuid: uuid.clone(),
            version: "1.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        let mut character_data = CharacterData {
            uuid: uuid.clone(),
            meta,
            card,
            background_path: String::new(),
            thumbnail_path: String::new(),
        };

        // 保存角色卡及图片
        let card_file = Self::get_character_file_path(app_handle, &uuid)?;

        if is_png {
            let card_path = Self::get_card_image_path(app_handle, &uuid)?;
            let thumbnail_path = Self::get_thumbnail_image_path(app_handle, &uuid)?;
            Self::write_card_and_thumbnail(&card_path, &thumbnail_path, &file_data)?;
            character_data.background_path = CARD_FILE_NAME.to_string();
            character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
        }

        FileUtils::write_json_file(&card_file, &character_data)?;

        let mut response = character_data.clone();
        Self::apply_absolute_paths(app_handle, &mut response)?;

        Ok(response)
    }

    /// 从字节数据导入角色卡
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄
    /// * `file_data` - 文件字节数据
    /// * `file_name` - 文件名（用于判断类型）
    ///
    /// # 返回
    /// * `Ok(CharacterData)` - 导入的角色数据
    pub fn import_character_card_from_bytes(
        app_handle: &tauri::AppHandle,
        file_data: &[u8],
        file_name: &str,
    ) -> Result<CharacterData, String> {
        // 尝试解析为 PNG
        let card_json = if file_name.ends_with(".png") {
            // 从 PNG 中提取角色卡数据
            PngMetadataUtils::read_character_data_from_bytes(file_data)
                .map_err(|e| format!("从 PNG 读取角色卡数据失败: {}", e))?
        } else {
            // 作为 JSON 解析
            String::from_utf8(file_data.to_vec())
                .map_err(|e| format!("读取 JSON 文件失败: {}", e))?
        };

        // 解析 TavernCardV2
        let card: TavernCardV2 =
            serde_json::from_str(&card_json).map_err(|e| format!("解析角色卡数据失败: {}", e))?;

        // 生成新的 UUID 和元数据
        let uuid = FileUtils::generate_uuid();
        let now = chrono::Utc::now().to_rfc3339();

        let meta = CharacterMeta {
            uuid: uuid.clone(),
            version: "1.0".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        let mut character_data = CharacterData {
            uuid: uuid.clone(),
            meta,
            card,
            background_path: String::new(),
            thumbnail_path: String::new(),
        };

        // 保存角色卡
        let card_file = Self::get_character_file_path(app_handle, &uuid)?;

        if file_name.ends_with(".png") {
            let card_path = Self::get_card_image_path(app_handle, &uuid)?;
            let thumbnail_path = Self::get_thumbnail_image_path(app_handle, &uuid)?;
            Self::write_card_and_thumbnail(&card_path, &thumbnail_path, file_data)?;
            character_data.background_path = CARD_FILE_NAME.to_string();
            character_data.thumbnail_path = THUMBNAIL_FILE_NAME.to_string();
        }

        FileUtils::write_json_file(&card_file, &character_data)?;

        let mut response = character_data.clone();
        Self::apply_absolute_paths(app_handle, &mut response)?;

        Ok(response)
    }
}
