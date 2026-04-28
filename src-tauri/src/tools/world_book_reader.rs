use super::AIToolTrait;
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::CharacterStorage;
use crate::tools::world_book_shared::{detailed_entry, locate_entry};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use tauri::AppHandle;

pub struct ReadWorldBookEntryTool;

#[async_trait]
impl AIToolTrait for ReadWorldBookEntryTool {
    fn name(&self) -> &'static str {
        "read_world_book_entry"
    }

    fn description(&self) -> &'static str {
        "读取当前角色的单个世界书条目完整内容。优先使用 entry_id；如果使用 name 或 key，必须唯一命中。"
    }

    fn category(&self) -> &'static str {
        "character"
    }

    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult {
        let start_time = std::time::Instant::now();

        let character_uuid = match &request.character_uuid {
            Some(uuid) => uuid.clone(),
            None => return error_result(start_time, "缺少角色UUID", None),
        };

        let character_data =
            match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                Ok(Some(data)) => data,
                Ok(None) => return error_result(start_time, "角色不存在", None),
                Err(error) => {
                    return error_result(start_time, &format!("获取角色数据失败: {}", error), None)
                }
            };

        let world_book = match character_data.card.data.character_book.as_ref() {
            Some(book) => book,
            None => return error_result(start_time, "当前角色没有世界书", None),
        };

        let selection = match locate_entry(&world_book.entries, &request.parameters, "读取") {
            Ok(selection) => selection,
            Err(error) => return error_result(start_time, &error.message, Some(error.details)),
        };

        let entry = &world_book.entries[selection.index];

        ToolResult {
            success: true,
            data: Some(json!({
                "message": "世界书条目读取成功",
                "matched_by": selection.matched_by,
                "matched_value": selection.matched_value,
                "entry": detailed_entry(entry),
            })),
            error: None,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn to_tool_definition(&self) -> ToolDefinition {
        let mut properties = HashMap::new();
        properties.insert(
            "at_least_one_identifier".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("必须提供至少一个定位字段（entry_id、name 或 key）".to_string()),
                enum_values: Some(vec!["read_world_book_entry".to_string()]),
                items: None,
                properties: None,
                required: None,
            },
        );
        properties.insert(
            "entry_id".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("世界书条目 ID，最推荐".to_string()),
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
                description: Some("条目名称，只有在能唯一匹配单个条目时才会读取".to_string()),
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
                description: Some("条目关键词，只有在能唯一匹配单个条目时才会读取".to_string()),
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

fn error_result(
    start_time: std::time::Instant,
    message: &str,
    details: Option<serde_json::Value>,
) -> ToolResult {
    ToolResult {
        success: false,
        data: details.map(|details| json!({ "details": details })),
        error: Some(message.to_string()),
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    }
}
