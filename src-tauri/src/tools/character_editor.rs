use super::AIToolTrait;
use crate::ai_tools::{ToolCallRequest, ToolParameter, ToolResult};
use crate::character_storage::CharacterStorage;
use async_trait::async_trait;
use tauri::{AppHandle, Emitter};

/// 角色编辑工具
pub struct EditCharacterTool;

#[async_trait]
impl AIToolTrait for EditCharacterTool {
    fn name(&self) -> &'static str {
        "edit_character"
    }

    fn description(&self) -> &'static str {
        "直接编辑角色卡字段。使用方法：将要更新的字段作为参数传入，例如要更新description字段，就直接传入description参数。不需要指定角色名称，系统会自动使用当前角色。支持的参数：name, description, personality, scenario, first_mes, mes_example, creator_notes, system_prompt, post_history_instructions, alternate_greetings(换行分隔), tags(逗号分隔), creator, character_version"
    }

    fn category(&self) -> &'static str {
        "character"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            // 添加一个提示字段，说明至少需要传入一个字段
            ToolParameter {
                name: "at_least_one_field".to_string(),
                description: "必须提供至少一个要编辑的字段（如description, personality等）"
                    .to_string(),
                parameter_type: "string".to_string(),
                required: true,
                schema: Some(serde_json::json!({
                    "type": "string",
                    "enum": ["edit_character"]
                })),
            },
            // 基础字段
            ToolParameter {
                name: "name".to_string(),
                description: "角色名称".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "description".to_string(),
                description: "角色描述".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "personality".to_string(),
                description: "性格特点".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "scenario".to_string(),
                description: "场景设定".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "first_mes".to_string(),
                description: "开场白".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "mes_example".to_string(),
                description: "对话示例".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "creator_notes".to_string(),
                description: "创作者笔记".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "system_prompt".to_string(),
                description: "系统提示词".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "post_history_instructions".to_string(),
                description: "历史后指令".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            // 数组字段
            ToolParameter {
                name: "alternate_greetings".to_string(),
                description: "备用问候语，多个问候语用换行分隔".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "tags".to_string(),
                description: "标签，多个标签用逗号分隔".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            // 元信息字段
            ToolParameter {
                name: "creator".to_string(),
                description: "创作者".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
            ToolParameter {
                name: "character_version".to_string(),
                description: "角色版本".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                schema: None,
            },
        ]
    }

    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult {
        let start_time = std::time::Instant::now();

        // 获取角色UUID
        let character_uuid = match &request.character_uuid {
            Some(uuid) => uuid.clone(),
            None => {
                return ToolResult {
                    success: false,
                    data: None,
                    error: Some("缺少角色UUID".to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        // 获取当前角色数据
        let character_data =
            match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                Ok(Some(data)) => data,
                Ok(None) => {
                    return ToolResult {
                        success: false,
                        data: None,
                        error: Some("角色不存在".to_string()),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                }
                Err(e) => {
                    return ToolResult {
                        success: false,
                        data: None,
                        error: Some(format!("获取角色数据失败: {}", e)),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                }
            };

        let mut tavern_card = character_data.card;
        let mut updated_fields = Vec::new();

        // 遍历所有参数，更新对应的字段（忽略提示字段）
        for (field_name, field_value) in &request.parameters {
            // 忽略提示字段
            if field_name == "at_least_one_field" {
                continue;
            }

            if let Some(value_str) = field_value.as_str() {
                match field_name.as_str() {
                    "name" => {
                        tavern_card.data.name = value_str.to_string();
                        updated_fields.push(("name", "角色名称"));
                    }
                    "description" => {
                        tavern_card.data.description = value_str.to_string();
                        updated_fields.push(("description", "角色描述"));
                    }
                    "personality" => {
                        tavern_card.data.personality = value_str.to_string();
                        updated_fields.push(("personality", "性格特点"));
                    }
                    "scenario" => {
                        tavern_card.data.scenario = value_str.to_string();
                        updated_fields.push(("scenario", "场景设定"));
                    }
                    "first_mes" => {
                        tavern_card.data.first_mes = value_str.to_string();
                        updated_fields.push(("first_mes", "开场白"));
                    }
                    "mes_example" => {
                        tavern_card.data.mes_example = value_str.to_string();
                        updated_fields.push(("mes_example", "对话示例"));
                    }
                    "creator_notes" => {
                        tavern_card.data.creator_notes = value_str.to_string();
                        updated_fields.push(("creator_notes", "创作者笔记"));
                    }
                    "system_prompt" => {
                        tavern_card.data.system_prompt = value_str.to_string();
                        updated_fields.push(("system_prompt", "系统提示词"));
                    }
                    "post_history_instructions" => {
                        tavern_card.data.post_history_instructions = value_str.to_string();
                        updated_fields.push(("post_history_instructions", "历史后指令"));
                    }
                    "alternate_greetings" => {
                        tavern_card.data.alternate_greetings = value_str
                            .split('\n')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        updated_fields.push(("alternate_greetings", "备用问候语"));
                    }
                    "tags" => {
                        tavern_card.data.tags = value_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        updated_fields.push(("tags", "标签"));
                    }
                    "creator" => {
                        tavern_card.data.creator = value_str.to_string();
                        updated_fields.push(("creator", "创作者"));
                    }
                    "character_version" => {
                        tavern_card.data.character_version = value_str.to_string();
                        updated_fields.push(("character_version", "角色版本"));
                    }
                    _ => {
                        // 忽略未知字段，但记录警告
                        eprintln!("警告: 未知字段名 '{}' 被忽略", field_name);
                    }
                }
            }
        }

        // 检查是否有字段被更新
        if updated_fields.is_empty() {
            return ToolResult {
                success: false,
                data: None,
                error: Some("没有提供有效的字段参数".to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        // 保存更新后的角色数据
        match CharacterStorage::update_character(app_handle, &character_uuid, &tavern_card) {
            Ok(()) => {
                // 发送事件通知前端刷新角色数据
                if let Err(e) = app_handle.emit(
                    "character-updated",
                    serde_json::json!({
                        "character_uuid": character_uuid,
                        "updated_fields": updated_fields.iter().map(|(k, _)| k).collect::<Vec<_>>()
                    }),
                ) {
                    eprintln!("发送角色更新事件失败: {}", e);
                }

                // 发送工具调用成功事件，用于调试
                if let Err(e) = app_handle.emit(
                    "tool-executed",
                    serde_json::json!({
                        "tool_name": "edit_character",
                        "character_uuid": character_uuid,
                        "updated_fields": updated_fields.iter().map(|(k, v)| serde_json::json!({
                            "field": k,
                            "description": v
                        })).collect::<Vec<_>>(),
                        "update_count": updated_fields.len()
                    }),
                ) {
                    eprintln!("发送工具执行事件失败: {}", e);
                }

                ToolResult {
                    success: true,
                    data: Some(serde_json::json!({
                        "message": "角色字段更新成功",
                        "updated_fields": updated_fields.iter().map(|(k, v)| serde_json::json!({
                            "field": k,
                            "description": v
                        })).collect::<Vec<_>>(),
                        "update_count": updated_fields.len()
                    })),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            Err(e) => ToolResult {
                success: false,
                data: None,
                error: Some(format!("保存角色数据失败: {}", e)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }
}
