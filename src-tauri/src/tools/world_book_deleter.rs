use super::AIToolTrait;
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::{CharacterStorage, WorldBookEntry};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

/// 世界书条目删除工具
pub struct DeleteWorldBookEntryTool;

impl DeleteWorldBookEntryTool {
    fn get_string_parameter<'a>(request: &'a ToolCallRequest, key: &str) -> Option<&'a str> {
        request.parameters.get(key).and_then(Value::as_str).map(str::trim)
    }

    fn get_i32_parameter(request: &ToolCallRequest, key: &str) -> Option<i32> {
        let value = request.parameters.get(key)?;

        if let Some(number) = value.as_i64() {
            return i32::try_from(number).ok();
        }

        value
            .as_str()
            .and_then(|text| text.trim().parse::<i32>().ok())
    }

    fn format_entry_summary(entry: &WorldBookEntry) -> String {
        let entry_id = entry.id.unwrap_or_default();
        let entry_name = entry.name.as_deref().unwrap_or("<未命名>");
        let keys = if entry.keys.is_empty() {
            "<无关键词>".to_string()
        } else {
            entry.keys.join(", ")
        };

        format!("id={entry_id}, name={entry_name}, keys=[{keys}]")
    }

    fn find_entry_index(entries: &[WorldBookEntry], request: &ToolCallRequest) -> Result<usize, String> {
        if let Some(entry_id) = Self::get_i32_parameter(request, "entry_id") {
            return entries
                .iter()
                .position(|entry| entry.id == Some(entry_id))
                .ok_or_else(|| format!("未找到 entry_id 为 {} 的世界书条目", entry_id));
        }

        if let Some(name) = Self::get_string_parameter(request, "name") {
            if name.is_empty() {
                return Err("name 不能为空".to_string());
            }

            let matched_indexes = entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry
                        .name
                        .as_deref()
                        .map(|entry_name| entry_name.trim().eq_ignore_ascii_case(name))
                        .unwrap_or(false)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            return match matched_indexes.as_slice() {
                [index] => Ok(*index),
                [] => Err(format!("未找到名称为“{}”的世界书条目", name)),
                _ => {
                    let candidates = matched_indexes
                        .iter()
                        .take(5)
                        .map(|index| Self::format_entry_summary(&entries[*index]))
                        .collect::<Vec<_>>()
                        .join("; ");
                    Err(format!(
                        "找到多个名称为“{}”的世界书条目，请改用 entry_id 删除。候选：{}",
                        name, candidates
                    ))
                }
            };
        }

        if let Some(key) = Self::get_string_parameter(request, "key") {
            if key.is_empty() {
                return Err("key 不能为空".to_string());
            }

            let matched_indexes = entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.keys.iter().any(|entry_key| entry_key.trim().eq_ignore_ascii_case(key)))
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            return match matched_indexes.as_slice() {
                [index] => Ok(*index),
                [] => Err(format!("未找到包含关键词“{}”的世界书条目", key)),
                _ => {
                    let candidates = matched_indexes
                        .iter()
                        .take(5)
                        .map(|index| Self::format_entry_summary(&entries[*index]))
                        .collect::<Vec<_>>()
                        .join("; ");
                    Err(format!(
                        "找到多个包含关键词“{}”的世界书条目，请改用 entry_id 删除。候选：{}",
                        key, candidates
                    ))
                }
            };
        }

        Err("缺少定位参数：请至少提供 entry_id、name 或 key 之一".to_string())
    }
}

#[async_trait]
impl AIToolTrait for DeleteWorldBookEntryTool {
    fn name(&self) -> &'static str {
        "delete_world_book_entry"
    }

    fn description(&self) -> &'static str {
        "删除当前角色的世界书条目。优先使用 entry_id 精确删除；如果没有 entry_id，也可以使用 name 或 key 精确匹配单个条目。若匹配到多个条目，工具会返回候选并要求改用 entry_id。"
    }

    fn category(&self) -> &'static str {
        "character"
    }

    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult {
        let start_time = std::time::Instant::now();

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

        let mut character_data = match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
            Ok(Some(data)) => data,
            Ok(None) => {
                return ToolResult {
                    success: false,
                    data: None,
                    error: Some("角色不存在".to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
            Err(error) => {
                return ToolResult {
                    success: false,
                    data: None,
                    error: Some(format!("获取角色数据失败: {}", error)),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let world_book = match character_data.card.data.character_book.as_mut() {
            Some(world_book) => world_book,
            None => {
                return ToolResult {
                    success: false,
                    data: None,
                    error: Some("当前角色没有世界书".to_string()),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let entry_index = match Self::find_entry_index(&world_book.entries, request) {
            Ok(index) => index,
            Err(error) => {
                return ToolResult {
                    success: false,
                    data: None,
                    error: Some(error),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let removed_entry = world_book.entries.remove(entry_index);
        let removed_entry_id = removed_entry.id.unwrap_or_default();
        let removed_entry_name = removed_entry.name.clone();
        let removed_entry_keys = removed_entry.keys.clone();

        match CharacterStorage::update_character(app_handle, &character_uuid, &character_data.card) {
            Ok(()) => {
                if let Err(error) = app_handle.emit(
                    "world-book-entry-deleted",
                    json!({
                        "character_uuid": character_uuid,
                        "entry_id": removed_entry_id,
                        "entry_name": removed_entry_name,
                        "keys": removed_entry_keys,
                    }),
                ) {
                    eprintln!("发送世界书条目删除事件失败: {}", error);
                }

                ToolResult {
                    success: true,
                    data: Some(json!({
                        "message": "世界书条目删除成功",
                        "deleted_entry": {
                            "id": removed_entry_id,
                            "name": removed_entry.name,
                            "keys": removed_entry.keys,
                            "comment": removed_entry.comment,
                        }
                    })),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            Err(error) => ToolResult {
                success: false,
                data: None,
                error: Some(format!("保存世界书变更失败: {}", error)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }

    fn to_tool_definition(&self) -> ToolDefinition {
        let mut properties = HashMap::new();

        properties.insert(
            "at_least_one_identifier".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("必须提供至少一个定位字段（entry_id、name 或 key）".to_string()),
                enum_values: Some(vec!["delete_world_book_entry".to_string()]),
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "entry_id".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("世界书条目 ID，最推荐，能精确删除单个条目".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "name".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("条目名称，只有在能唯一匹配单个条目时才会删除".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "key".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("条目关键词，只有在能唯一匹配单个条目时才会删除".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        ToolDefinition {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: self.name().to_string(),
                description: Some(self.description().to_string()),
                parameters: Some(ToolParameters {
                    param_type: "object".to_string(),
                    properties,
                    required: Some(vec!["at_least_one_identifier".to_string()]),
                }),
            },
        }
    }
}
