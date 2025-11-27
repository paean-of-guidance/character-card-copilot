use super::AIToolTrait;
use crate::ai_tools::{ToolCallRequest, ToolResult};
use crate::ai_chat::{ChatTool, ToolFunction, ToolParameters, ToolParameter as ChatToolParameter};
use crate::character_storage::CharacterStorage;
use async_trait::async_trait;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

const ALTERNATE_GREETING_MARKER: &str = "<START_ALT>";

/// 角色编辑工具
pub struct EditCharacterTool;

#[async_trait]
impl AIToolTrait for EditCharacterTool {
    fn name(&self) -> &'static str {
        "edit_character"
    }

    fn description(&self) -> &'static str {
        "直接编辑角色卡字段。使用方法：将要更新的字段作为参数传入，例如要更新description字段，就直接传入description参数。不需要指定角色名称，系统会自动使用当前角色。支持的参数：name, description, personality, scenario, first_mes, mes_example, creator_notes, system_prompt, post_history_instructions, alternate_greetings(使用<START_ALT>标记每段), tags(逗号分隔), creator, character_version"
    }

    fn category(&self) -> &'static str {
        "character"
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
                            .split(ALTERNATE_GREETING_MARKER)
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
                // 重新加载完整的角色数据
                let updated_character_data =
                    match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                        Ok(Some(data)) => data,
                        Ok(None) => {
                            return ToolResult {
                                success: false,
                                data: None,
                                error: Some(format!("重新加载角色数据失败：角色不存在")),
                                execution_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                        }
                        Err(e) => {
                            return ToolResult {
                                success: false,
                                data: None,
                                error: Some(format!("重新加载角色数据失败: {}", e)),
                                execution_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                        }
                    };

                // ✅ 使用标准事件发送方法（包含完整的 character_data）
                if let Err(e) = crate::events::EventEmitter::send_character_updated(
                    app_handle,
                    &character_uuid,
                    &updated_character_data,
                    crate::events::CharacterUpdateType::BasicInfo,
                ) {
                    eprintln!("发送角色更新事件失败: {}", e);
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

    fn to_chat_tool(&self) -> ChatTool {
        let mut properties = HashMap::new();

        // 添加所有参数到 properties
        properties.insert(
            "at_least_one_field".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("必须提供至少一个要编辑的字段（如description, personality等）".to_string()),
                enum_values: Some(vec!["edit_character".to_string()]),
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "name".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("角色名称".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "description".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("角色描述".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "personality".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("性格特点".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "scenario".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("场景设定".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "first_mes".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("开场白".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "mes_example".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("对话示例".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "creator_notes".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("创作者笔记".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "system_prompt".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("系统提示词".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "post_history_instructions".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("历史后指令".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "alternate_greetings".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("备用问候语，使用 <START_ALT> 标记每段开头".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "tags".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("标签，多个标签用逗号分隔".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "creator".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("创作者".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "character_version".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("角色版本".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        ChatTool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: self.name().to_string(),
                description: Some(self.description().to_string()),
                parameters: Some(ToolParameters {
                    param_type: "object".to_string(),
                    properties,
                    required: Some(vec!["at_least_one_field".to_string()]),
                }),
            },
        }
    }
}
