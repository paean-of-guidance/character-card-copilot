use super::AIToolTrait;
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::CharacterStorage;
use crate::tools::world_book_shared::{locate_entry, summarize_entry};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

/// 世界书条目删除工具
pub struct DeleteWorldBookEntryTool;

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

        let selection = match locate_entry(&world_book.entries, &request.parameters, "删除") {
            Ok(selection) => selection,
            Err(error) => {
                return ToolResult {
                    success: false,
                    data: Some(json!({
                        "error_code": error.code,
                        "details": error.details,
                    })),
                    error: Some(error.message),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let removed_entry = world_book.entries.remove(selection.index);
        let removed_entry_id = removed_entry.id.unwrap_or_default();
        let removed_entry_name = removed_entry.name.clone();
        let removed_entry_keys = removed_entry.keys.clone();

        match CharacterStorage::update_character(app_handle, &character_uuid, &character_data.card)
        {
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
                        "matched_by": selection.matched_by,
                        "matched_value": selection.matched_value,
                        "deleted_entry": {
                            "summary": summarize_entry(&removed_entry),
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
