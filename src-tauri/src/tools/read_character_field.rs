use super::{error_result, success_result, AIToolTrait};
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::CharacterStorage;
use crate::tools::character_fields::{get_long_text_field, long_text_field_names, slice_by_chars};
use crate::tools::world_book_shared::get_usize_parameter;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use tauri::AppHandle;

const DEFAULT_MAX_CHARS: usize = 1200;
const MAX_MAX_CHARS: usize = 4000;
pub struct ReadCharacterFieldTool;

#[async_trait]
impl AIToolTrait for ReadCharacterFieldTool {
    fn name(&self) -> &'static str {
        "read_character_field"
    }

    fn description(&self) -> &'static str {
        "读取当前角色的长文本字段内容。适合在 patch_character_field 前先查看 description、personality 等字段的真实内容，也支持分页读取。"
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

        let field = match request
            .parameters
            .get("field")
            .and_then(|value| value.as_str())
        {
            Some(value) if long_text_field_names().iter().any(|field| field == value) => value,
            Some(value) => {
                return ToolResult {
                    success: false,
                    data: Some(json!({
                            "supported_fields": long_text_field_names(),
                            "field": value,
                    })),
                    error: Some(format!("字段 '{}' 不支持 read_character_field", value)),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            None => return error_result(start_time, "缺少必填参数 'field'"),
        };

        let start = get_usize_parameter(&request.parameters, "start").unwrap_or(0);
        let max_chars = get_usize_parameter(&request.parameters, "max_chars")
            .unwrap_or(DEFAULT_MAX_CHARS)
            .min(MAX_MAX_CHARS);

        let character_data =
            match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                Ok(Some(data)) => data,
                Ok(None) => return error_result(start_time, "角色不存在"),
                Err(error) => {
                    return error_result(start_time, &format!("获取角色数据失败: {}", error))
                }
            };

        let text = match get_long_text_field(&character_data.card, field) {
            Some(value) => value,
            None => return error_result(start_time, "读取字段失败"),
        };

        let total_length = text.chars().count();
        let start = start.min(total_length);
        let end = (start + max_chars).min(total_length);
        let content = slice_by_chars(text, start, end);

        success_result(
            start_time,
            json!({
                "message": "角色字段读取成功",
                "field": field,
                "text": content,
            "start": start,
            "end": end,
            "total_length": total_length,
            "truncated": end < total_length,
            }),
        )
    }

    fn to_tool_definition(&self) -> ToolDefinition {
        let mut properties = HashMap::new();
        properties.insert(
            "field".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("要读取的字段，仅支持长文本字段".to_string()),
                enum_values: Some(long_text_field_names()),
                items: None,
                properties: None,
                required: None,
            },
        );
        properties.insert(
            "start".to_string(),
            ChatToolParameter {
                param_type: "integer".to_string(),
                description: Some("起始字符偏移，默认 0".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );
        properties.insert(
            "max_chars".to_string(),
            ChatToolParameter {
                param_type: "integer".to_string(),
                description: Some("最多返回多少字符，默认 1200，上限 4000".to_string()),
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
                    required: Some(vec!["field".to_string()]),
                }),
            },
        }
    }
}
