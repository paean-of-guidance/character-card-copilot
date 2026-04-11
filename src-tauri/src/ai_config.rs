use super::file_utils::FileUtils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2000
}

fn default_tools_enabled() -> bool {
    true
}

fn default_context_role_template() -> String {
    "角色卡编写助手".to_string()
}

fn default_context_task_template() -> String {
    "帮助用户创作和完善角色设定, 需要从多个角度(角色动机，角色心理，角色性格，角色背景)等分析，完成角色卡。当需要局部修改某个字段中的一句话、某个 trait 或某段内容时，优先先读后写：不确定当前文本时使用 read_character_field 或 patch_character_field(dry_run=true) 预览，确认唯一命中后再执行 patch_character_field；只有当用户明确要求重写整个字段时，才使用 edit_character。当处理世界书时，先使用 list_world_book_entries 查看候选，必要时用 read_world_book_entry 读取完整条目；创建使用 create_world_book_entry，更新使用 update_world_book_entry，删除使用 delete_world_book_entry，并尽量传 entry_id 以避免误操作。".to_string()
}

fn default_context_instructions_template() -> String {
    "基于用户需求分析现有角色设定，提供建议并调用相应工具。\n始终保持角色设定的一致性和逻辑性，遵循用户的具体要求。\n如果需要局部修改角色信息，优先先用 read_character_field 或 patch_character_field(dry_run=true) 确认当前文本，再使用 patch_character_field；search 必须唯一命中，0 个或超过 1 个匹配都应视为失败。\n只有在用户明确要求重写整个字段时，才使用 edit_character 工具。\n如果需要处理世界书，先使用 list_world_book_entries，必要时再用 read_world_book_entry / update_world_book_entry / delete_world_book_entry；如果需要添加世界书条目，请使用 create_world_book_entry 工具。".to_string()
}

/// AI角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRole {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_tools_enabled")]
    pub tools_enabled: bool,
    #[serde(default = "default_context_role_template")]
    pub context_role_template: String,
    #[serde(default = "default_context_task_template")]
    pub context_task_template: String,
    #[serde(default = "default_context_instructions_template")]
    pub context_instructions_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRoleRecord {
    pub id: String,
    pub role: AIRole,
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub default_role: String,
    pub roles: HashMap<String, AIRole>,
}

/// AI配置服务
pub struct AIConfigService;

impl AIConfigService {
    /// 获取AI配置文件路径
    fn get_ai_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = FileUtils::get_app_data_dir(app_handle)?;
        Ok(app_data_dir.join("ai_config.yml"))
    }

    fn default_character_assistant_role() -> AIRole {
        AIRole {
            name: "角色设定助手".to_string(),
            description: "专门协助用户创建、编辑和优化角色设定的AI助手".to_string(),
            system_prompt: "你是一个专业的角色设定助手，擅长帮助用户创建丰富、立体的角色。你会分析用户提供的角色信息，并提供改进建议、创意灵感，帮助用户完善角色设定。请使用友好、专业的语调进行对话。".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            tools_enabled: true,
            context_role_template: "角色卡编写助手".to_string(),
                  context_task_template: "帮助用户创作和完善角色设定, 需要从多个角度(角色动机，角色心理，角色性格，角色背景)等分析，完成角色卡。当需要局部修改某个字段中的一句话、某个 trait 或某段内容时，优先先读后写：不确定当前文本时使用 read_character_field 或 patch_character_field(dry_run=true) 预览，确认唯一命中后再执行 patch_character_field；只有当用户明确要求重写整个字段时，才使用 edit_character。当处理世界书时，先使用 list_world_book_entries 查看候选，必要时用 read_world_book_entry 读取完整条目；创建使用 create_world_book_entry，更新使用 update_world_book_entry，删除使用 delete_world_book_entry，并尽量传 entry_id 以避免误操作。".to_string(),
                  context_instructions_template: "基于用户需求分析现有角色设定，提供建议并调用相应工具。\n始终保持角色设定的一致性和逻辑性，遵循用户的具体要求。\n如果需要局部修改角色信息，优先先用 read_character_field 或 patch_character_field(dry_run=true) 确认当前文本，再使用 patch_character_field；search 必须唯一命中，0 个或超过 1 个匹配都应视为失败。\n只有在用户明确要求重写整个字段时，才使用 edit_character 工具。\n如果需要处理世界书，先使用 list_world_book_entries，必要时再用 read_world_book_entry / update_world_book_entry / delete_world_book_entry；如果需要添加世界书条目，请使用 create_world_book_entry 工具。".to_string(),
        }
    }

    fn default_creative_writer_role() -> AIRole {
        AIRole {
            name: "创意写作助手".to_string(),
            description: "协助用户进行创意写作，包括剧情构思、对话生成等".to_string(),
            system_prompt: "你是一个富有创意的写作助手，能够帮助用户构思剧情、生成对话、提供写作灵感。你擅长各种文学风格的创作，能够根据角色设定提供符合其性格的对话内容。".to_string(),
            temperature: 0.8,
            max_tokens: 1500,
            tools_enabled: true,
            context_role_template: "创意写作助手".to_string(),
            context_task_template: "围绕角色卡和世界观帮助用户进行剧情构思、桥段展开、对白润色与创作延展。必要时可以调用工具同步角色卡与世界书。".to_string(),
                 context_instructions_template: "优先保持创意、多样性与角色一致性。\n如果用户要求你直接修改角色设定中的局部内容，先使用 read_character_field 或 patch_character_field(dry_run=true) 确认上下文，再使用 patch_character_field；只有明确要求整段重写时才使用 edit_character。\n如果用户要求补充或调整世界观知识，先使用 list_world_book_entries / read_world_book_entry 了解现状；新增请使用 create_world_book_entry，更新请使用 update_world_book_entry。".to_string(),
        }
    }

    fn default_character_analyst_role() -> AIRole {
        AIRole {
            name: "角色分析师".to_string(),
            description: "深度分析角色设定，提供专业的优化建议".to_string(),
            system_prompt: "你是一个专业的角色分析师，能够从心理学、文学创作等多个角度分析角色设定的合理性、深度和一致性。你会提供具体的改进建议，帮助用户创建更加立体、可信的角色。".to_string(),
            temperature: 0.6,
            max_tokens: 2500,
            tools_enabled: false,
            context_role_template: "角色分析师".to_string(),
            context_task_template: "分析角色设定的合理性、层次感、一致性与可写性，并给出结构化建议。".to_string(),
            context_instructions_template: "优先给出分析、诊断和建议，不主动调用工具。\n保持批判性但语气友好。\n当用户要求具体修改方案时，先解释原因，再给出可执行建议。".to_string(),
        }
    }

    /// 获取默认AI配置
    fn get_default_config() -> AIConfig {
        let mut roles = HashMap::new();

        roles.insert(
            "character_assistant".to_string(),
            Self::default_character_assistant_role(),
        );
        roles.insert(
            "creative_writer".to_string(),
            Self::default_creative_writer_role(),
        );
        roles.insert(
            "character_analyst".to_string(),
            Self::default_character_analyst_role(),
        );

        AIConfig {
            default_role: "character_assistant".to_string(),
            roles,
        }
    }

    fn normalize_config(mut config: AIConfig) -> AIConfig {
        if config.roles.is_empty() {
            return Self::get_default_config();
        }

        for (role_id, role) in config.roles.iter_mut() {
            if role.name.trim().is_empty() {
                role.name = role_id.clone();
            }
        }

        if !config.roles.contains_key(&config.default_role) {
            if let Some(first_role_id) = config.roles.keys().next().cloned() {
                config.default_role = first_role_id;
            }
        }

        config
    }

    fn sanitize_role_id(name: &str) -> String {
        let mut sanitized = name
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_lowercase()
                } else if ch == '-' || ch == '_' || ch.is_ascii_whitespace() {
                    '_'
                } else {
                    '_'
                }
            })
            .collect::<String>();

        while sanitized.contains("__") {
            sanitized = sanitized.replace("__", "_");
        }

        sanitized.trim_matches('_').to_string()
    }

    fn generate_role_id(config: &AIConfig, role_name: &str) -> String {
        let base = Self::sanitize_role_id(role_name);
        let base = if base.is_empty() {
            format!("ai_role_{}", &uuid::Uuid::new_v4().simple().to_string()[..8])
        } else {
            base
        };

        if !config.roles.contains_key(&base) {
            return base;
        }

        let mut index = 2;
        loop {
            let candidate = format!("{}_{}", base, index);
            if !config.roles.contains_key(&candidate) {
                return candidate;
            }
            index += 1;
        }
    }

    /// 加载AI配置
    pub fn load_config(app_handle: &tauri::AppHandle) -> Result<AIConfig, String> {
        let config_path = Self::get_ai_config_path(app_handle)?;

        if !config_path.exists() {
            let default_config = Self::get_default_config();
            Self::save_config(app_handle, &default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read AI config file: {}", e))?;

        let parsed_config = serde_yaml::from_str::<AIConfig>(&content)
            .map_err(|e| format!("Failed to parse AI config: {}", e))?;

        let normalized_config = Self::normalize_config(parsed_config);
        Self::save_config(app_handle, &normalized_config)?;
        Ok(normalized_config)
    }

    /// 保存AI配置
    pub fn save_config(app_handle: &tauri::AppHandle, config: &AIConfig) -> Result<(), String> {
        let config_path = Self::get_ai_config_path(app_handle)?;
        let normalized_config = Self::normalize_config(config.clone());

        let content = serde_yaml::to_string(&normalized_config)
            .map_err(|e| format!("Failed to serialize AI config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write AI config file: {}", e))?;

        Ok(())
    }

    /// 获取指定角色配置
    pub fn get_role(app_handle: &tauri::AppHandle, role_id: &str) -> Result<Option<AIRole>, String> {
        let config = Self::load_config(app_handle)?;
        Ok(config.roles.get(role_id).cloned())
    }

    pub fn resolve_role(
        app_handle: &tauri::AppHandle,
        requested_role_id: Option<&str>,
    ) -> Result<(String, AIRole), String> {
        let config = Self::load_config(app_handle)?;

        if let Some(role_id) = requested_role_id {
            if let Some(role) = config.roles.get(role_id).cloned() {
                return Ok((role_id.to_string(), role));
            }
        }

        let default_role_id = config.default_role.clone();
        let default_role = config
            .roles
            .get(&default_role_id)
            .cloned()
            .ok_or_else(|| format!("Role '{}' not found", default_role_id))?;

        Ok((default_role_id, default_role))
    }

    /// 更新角色配置
    pub fn update_role(
        app_handle: &tauri::AppHandle,
        role_id: &str,
        role: &AIRole,
    ) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;

        if !config.roles.contains_key(role_id) {
            return Err(format!("Role '{}' not found", role_id));
        }

        config.roles.insert(role_id.to_string(), role.clone());
        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 添加新角色，并自动生成内部ID
    pub fn add_role(app_handle: &tauri::AppHandle, role: &AIRole) -> Result<String, String> {
        let mut config = Self::load_config(app_handle)?;
        let role_id = Self::generate_role_id(&config, &role.name);

        config.roles.insert(role_id.clone(), role.clone());
        Self::save_config(app_handle, &config)?;

        Ok(role_id)
    }

    /// 删除角色
    pub fn delete_role(app_handle: &tauri::AppHandle, role_id: &str) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;

        if config.default_role == role_id {
            return Err("Cannot delete default role".to_string());
        }

        if config.roles.remove(role_id).is_none() {
            return Err(format!("Role '{}' not found", role_id));
        }

        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 设置默认角色
    pub fn set_default_role(app_handle: &tauri::AppHandle, role_id: &str) -> Result<(), String> {
        let mut config = Self::load_config(app_handle)?;

        if !config.roles.contains_key(role_id) {
            return Err(format!("Role '{}' not found", role_id));
        }

        config.default_role = role_id.to_string();
        Self::save_config(app_handle, &config)?;
        Ok(())
    }

    /// 获取所有角色列表
    pub fn get_all_roles(app_handle: &tauri::AppHandle) -> Result<Vec<AIRoleRecord>, String> {
        let config = Self::load_config(app_handle)?;
        let mut roles = config
            .roles
            .into_iter()
            .map(|(id, role)| AIRoleRecord { id, role })
            .collect::<Vec<_>>();

        roles.sort_by(|left, right| left.role.name.cmp(&right.role.name));
        Ok(roles)
    }
}
