use super::{error_result, AIToolTrait};
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::CharacterStorage;
use crate::tools::world_book_shared::{entry_matches_query, get_usize_parameter, summarize_entry};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use tauri::AppHandle;

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;

pub struct ListWorldBookEntriesTool;

#[async_trait]
impl AIToolTrait for ListWorldBookEntriesTool {
    fn name(&self) -> &'static str {
        "list_world_book_entries"
    }

    fn description(&self) -> &'static str {
        "列出当前角色的世界书条目摘要。适合在删除或编辑前先查看所有条目的 id、name、keys 和内容预览。"
    }

    fn category(&self) -> &'static str {
        "character"
    }

    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult {
        let start_time = std::time::Instant::now();

        let character_uuid = match &request.character_uuid {
            Some(uuid) => uuid.clone(),
            None => return error_result(start_time, "缺少角色UUID"),
        };

        let query = request
            .parameters
            .get("query")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .unwrap_or("");
        let limit = get_usize_parameter(&request.parameters, "limit")
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        let character_data =
            match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                Ok(Some(data)) => data,
                Ok(None) => return error_result(start_time, "角色不存在"),
                Err(error) => {
                    return error_result(start_time, &format!("获取角色数据失败: {}", error))
                }
            };

        let entries = character_data
            .card
            .data
            .character_book
            .as_ref()
            .map(|book| &book.entries);

        let all_entries = entries.map(|entries| entries.as_slice()).unwrap_or(&[]);
        let filtered_entries = all_entries
            .iter()
            .filter(|entry| entry_matches_query(entry, query))
            .collect::<Vec<_>>();

        let items = filtered_entries
            .iter()
            .take(limit)
            .map(|entry| summarize_entry(entry))
            .collect::<Vec<_>>();

        ToolResult {
            success: true,
            data: Some(json!({
                "message": "世界书条目列表读取成功",
                "query": if query.is_empty() { None } else { Some(query) },
                "total_count": all_entries.len(),
                "filtered_count": filtered_entries.len(),
                "returned_count": items.len(),
                "has_more": filtered_entries.len() > items.len(),
                "entries": items,
            })),
            error: None,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn to_tool_definition(&self) -> ToolDefinition {
        let mut properties = HashMap::new();
        properties.insert(
            "query".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some(
                    "可选过滤词，会在 id、name、key、comment、content 中做包含匹配".to_string(),
                ),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );
        properties.insert(
            "limit".to_string(),
            ChatToolParameter {
                param_type: "integer".to_string(),
                description: Some("最多返回多少条摘要，默认 20，上限 100".to_string()),
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
                    required: None,
                }),
            },
        }
    }
}
