use super::file_utils::FileUtils;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// AI角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRole {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub tools_enabled: bool,
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub default_role: String,
    pub roles: std::collections::HashMap<String, AIRole>,
}

/// AI配置服务
pub struct AIConfigService;

impl AIConfigService {
    /// 获取AI配置文件路径
    fn get_ai_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = FileUtils::get_app_data_dir(app_handle)?;
        Ok(app_data_dir.join("ai_config.yml"))
    }

    /// 获取默认AI配置
    fn get_default_config() -> AIConfig {
        let mut roles = std::collections::HashMap::new();

        // 默认角色助手
        roles.insert("character_assistant".to_string(), AIRole {
            name: "角色设定助手".to_string(),
            description: "专门协助用户创建、编辑和优化角色设定的AI助手".to_string(),
            system_prompt: "你是一个专业的角色设定助手，擅长帮助用户创建丰富、立体的角色。你会分析用户提供的角色信息，并提供改进建议、创意灵感，帮助用户完善角色设定。请使用友好、专业的语调进行对话。".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            tools_enabled: true,
        });

        // 创意写作助手
        roles.insert("creative_writer".to_string(), AIRole {
            name: "创意写作助手".to_string(),
            description: "协助用户进行创意写作，包括剧情构思、对话生成等".to_string(),
            system_prompt: "你是一个富有创意的写作助手，能够帮助用户构思剧情、生成对话、提供写作灵感。你擅长各种文学风格的创作，能够根据角色设定提供符合其性格的对话内容。".to_string(),
            temperature: 0.8,
            max_tokens: 1500,
            tools_enabled: true,
        });

        // 角色分析师
        roles.insert("character_analyst".to_string(), AIRole {
            name: "角色分析师".to_string(),
            description: "深度分析角色设定，提供专业的优化建议".to_string(),
            system_prompt: "你是一个专业的角色分析师，能够从心理学、文学创作等多个角度分析角色设定的合理性、深度和一致性。你会提供具体的改进建议，帮助用户创建更加立体、可信的角色。".to_string(),
            temperature: 0.6,
            max_tokens: 2500,
            tools_enabled: false,
        });

        AIConfig {
            default_role: "character_assistant".to_string(),
            roles,
        }
    }

    /// 加载AI配置
    pub fn load_config(app_handle: &tauri::AppHandle) -> Result<AIConfig, String> {
        let config_path = Self::get_ai_config_path(app_handle)?;

        if !config_path.exists() {
            // 如果配置文件不存在，创建默认配置
            let default_config = Self::get_default_config();
            Self::save_config(app_handle, &default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read AI config file: {}", e))?;

        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse AI config: {}", e))
    }

    /// 保存AI配置
    pub fn save_config(app_handle: &tauri::AppHandle, config: &AIConfig) -> Result<(), String> {
        let config_path = Self::get_ai_config_path(app_handle)?;

        let content = serde_yaml::to_string(config)
            .map_err(|e| format!("Failed to serialize AI config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write AI config file: {}", e))?;

        Ok(())
    }

    /// 获取指定角色配置
    pub fn get_role(
        app_handle: &tauri::AppHandle,
        role_name: &str,
    ) -> Result<Option<AIRole>, String> {
        let config = Self::load_config(app_handle)?;
        Ok(config.roles.get(role_name).cloned())
    }

    /// 更新角色配置
    pub fn update_role(
        app_handle: &tauri::AppHandle,
        role_name: &str,
        role: &AIRole,
    ) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;
        config.roles.insert(role_name.to_string(), role.clone());
        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 添加新角色
    pub fn add_role(
        app_handle: &tauri::AppHandle,
        role_name: &str,
        role: &AIRole,
    ) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;
        config.roles.insert(role_name.to_string(), role.clone());
        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 删除角色
    pub fn delete_role(app_handle: &tauri::AppHandle, role_name: &str) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;

        // 不允许删除默认角色
        if config.default_role == role_name {
            return Err("Cannot delete default role".to_string());
        }

        config.roles.remove(role_name);
        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 设置默认角色
    pub fn set_default_role(app_handle: &tauri::AppHandle, role_name: &str) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;

        // 确保角色存在
        if !config.roles.contains_key(role_name) {
            return Err(format!("Role '{}' not found", role_name));
        }

        config.default_role = role_name.to_string();
        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 获取所有角色列表
    pub fn get_all_roles(app_handle: &tauri::AppHandle) -> Result<Vec<(String, AIRole)>, String> {
        let config = Self::load_config(app_handle)?;
        Ok(config.roles.into_iter().collect())
    }
}
