use super::AIToolTrait;
use crate::ai_chat::{ChatTool, ToolFunction, ToolParameter as ChatToolParameter, ToolParameters};
use crate::ai_tools::{ToolCallRequest, ToolResult};
use crate::character_storage::{CharacterBook, CharacterStorage, WorldBookEntry};
use async_trait::async_trait;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

/// 世界书条目创建工具
pub struct CreateWorldBookEntryTool;

#[async_trait]
impl AIToolTrait for CreateWorldBookEntryTool {
    fn name(&self) -> &'static str {
        "create_world_book_entry"
    }

    fn description(&self) -> &'static str {
        "为当前角色创建新的世界书条目。必填参数：keys（关键词，多个关键词用逗号分隔）、content（内容）、depth（插入深度）、comment（备注）、probability（触发概率）。选填参数：name（条目名称）、enabled（是否启用，默认true）、priority（优先级，默认10）、position（位置，默认before_char）以及extension相关参数。"
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

        // 验证必填参数
        let required_fields = vec!["keys", "content", "depth", "comment", "probability"];
        for field in &required_fields {
            if !request.parameters.contains_key(*field) {
                return ToolResult {
                    success: false,
                    data: None,
                    error: Some(format!("缺少必填参数: {}", field)),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        }

        // 获取当前角色数据
        let mut character_data =
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

        // 确保世界书存在
        let world_book = character_data
            .card
            .data
            .character_book
            .get_or_insert_with(|| CharacterBook {
                name: None,
                description: None,
                scan_depth: Some(2),
                token_budget: Some(500),
                recursive_scanning: Some(false),
                extensions: serde_json::json!({}),
                entries: Vec::new(),
            });

        // 生成新条目ID
        let new_id = if world_book.entries.is_empty() {
            1
        } else {
            world_book
                .entries
                .iter()
                .filter_map(|e| e.id)
                .max()
                .unwrap_or(0)
                + 1
        };

        // 计算插入顺序
        let insertion_order = if world_book.entries.is_empty() {
            1
        } else {
            world_book
                .entries
                .iter()
                .map(|e| e.insertion_order)
                .max()
                .unwrap_or(0)
                + 1
        };

        // 创建默认extensions
        let mut extensions = serde_json::json!({
            "automation_id": "",
            "case_sensitive": null,
            "cooldown": 0,
            "delay": 0,
            "delay_until_recursion": false,
            "depth": 5,
            "display_index": 0,
            "exclude_recursion": false,
            "group": "",
            "group_override": false,
            "group_weight": 100,
            "match_character_depth_prompt": false,
            "match_character_description": false,
            "match_character_personality": false,
            "match_creator_notes": false,
            "match_persona_description": false,
            "match_scenario": false,
            "match_whole_words": null,
            "position": 4,
            "prevent_recursion": false,
            "probability": 100,
            "role": 0,
            "scan_depth": null,
            "selectiveLogic": 0,
            "sticky": 0,
            "useProbability": true,
            "use_group_scoring": false,
            "vectorized": false,
        });

        // 解析参数
        let mut new_entry = WorldBookEntry {
            id: Some(new_id),
            name: None,
            keys: Vec::new(),
            content: String::new(),
            extensions: serde_json::json!({}),
            enabled: true,
            insertion_order,
            case_sensitive: Some(false),
            priority: Some(10),
            comment: None,
            selective: None,
            secondary_keys: None,
            constant: None,
            position: Some("before_char".to_string()),
        };

        // 处理参数
        for (field_name, field_value) in &request.parameters {
            // 忽略提示字段
            if field_name == "at_least_one_field" {
                continue;
            }

            if let Some(value_str) = field_value.as_str() {
                match field_name.as_str() {
                    "keys" => {
                        new_entry.keys = value_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "content" => {
                        new_entry.content = value_str.to_string();
                    }
                    "name" => {
                        new_entry.name = Some(value_str.to_string());
                    }
                    "comment" => {
                        new_entry.comment = Some(value_str.to_string());
                    }
                    "enabled" => {
                        new_entry.enabled = value_str.parse().unwrap_or(true);
                    }
                    "priority" => {
                        new_entry.priority = value_str.parse().ok();
                    }
                    "position" => {
                        new_entry.position = Some(value_str.to_string());
                    }
                    "depth" => {
                        if let Ok(depth_val) = value_str.parse::<i32>() {
                            extensions["depth"] = serde_json::Value::Number(depth_val.into());
                        }
                    }
                    "probability" => {
                        if let Ok(prob_val) = value_str.parse::<i32>() {
                            extensions["probability"] = serde_json::Value::Number(prob_val.into());
                        }
                    }
                    "scan_depth" => {
                        if let Ok(scan_val) = value_str.parse::<i32>() {
                            extensions["scan_depth"] = serde_json::Value::Number(scan_val.into());
                        }
                    }
                    "token_budget" => {
                        if let Ok(budget_val) = value_str.parse::<i32>() {
                            world_book.token_budget = Some(budget_val);
                        }
                    }
                    "recursive_scanning" => {
                        if let Ok(recursive_val) = value_str.parse::<bool>() {
                            world_book.recursive_scanning = Some(recursive_val);
                        }
                    }
                    _ => {
                        // 其他extension字段
                        extensions[field_name] = serde_json::Value::String(value_str.to_string());
                    }
                }
            }
        }

        // 验证必填字段
        if new_entry.keys.is_empty() {
            return ToolResult {
                success: false,
                data: None,
                error: Some("keys 参数不能为空".to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        if new_entry.content.is_empty() {
            return ToolResult {
                success: false,
                data: None,
                error: Some("content 参数不能为空".to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            };
        }

        new_entry.extensions = extensions;

        // 添加到世界书
        world_book.entries.push(new_entry.clone());

        // 保存角色数据
        match CharacterStorage::update_character(app_handle, &character_uuid, &character_data.card)
        {
            Ok(()) => {
                // 发送事件通知前端
                if let Err(e) = app_handle.emit(
                    "world-book-entry-created",
                    serde_json::json!({
                        "character_uuid": character_uuid,
                        "entry_id": new_id,
                        "entry_name": new_entry.name,
                        "keys": new_entry.keys
                    }),
                ) {
                    eprintln!("发送世界书条目创建事件失败: {}", e);
                }

                ToolResult {
                    success: true,
                    data: Some(serde_json::json!({
                        "message": "世界书条目创建成功",
                        "entry_id": new_id,
                        "entry_name": new_entry.name,
                        "keys": new_entry.keys,
                        "content_preview": if new_entry.content.len() > 50 {
                            format!("{}...", &new_entry.content[..50])
                        } else {
                            new_entry.content.clone()
                        }
                    })),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            Err(e) => ToolResult {
                success: false,
                data: None,
                error: Some(format!("保存世界书条目失败: {}", e)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }

    fn to_chat_tool(&self) -> ChatTool {
        let mut properties = HashMap::new();

        // 提示字段
        properties.insert(
            "at_least_one_field".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("必须提供至少一个要编辑的字段".to_string()),
                enum_values: Some(vec!["create_world_book_entry".to_string()]),
                items: None,
                properties: None,
                required: None,
            },
        );

        // 必填参数
        properties.insert(
            "keys".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("关键词, 多个关键词用逗号分隔, 用于触发条目".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "content".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("条目内容,尽量简短,避免浪费过多token".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "depth".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("插入深度（数字，通常1-10）".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "comment".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("备注，需要简短，格式为<function(rule/background或者其他的)>[10字/words以内概括]".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "probability".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("触发概率（数字0-100）".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        // 选填参数
        properties.insert(
            "name".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("条目名称".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "enabled".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("是否启用（true/false）".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "priority".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("优先级（数字）".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "position".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("位置（before_char/after_char）".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "scan_depth".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("扫描深度".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "token_budget".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("令牌预算".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "recursive_scanning".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("递归扫描（true/false）".to_string()),
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
                    required: Some(vec![
                        "at_least_one_field".to_string(),
                        "keys".to_string(),
                        "content".to_string(),
                        "depth".to_string(),
                        "comment".to_string(),
                        "probability".to_string(),
                    ]),
                }),
            },
        }
    }
}
