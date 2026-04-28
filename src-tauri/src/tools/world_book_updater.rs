use super::AIToolTrait;
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::{CharacterStorage, WorldBookEntry};
use crate::tools::world_book_shared::{
    detailed_entry, get_bool_parameter, get_i32_parameter, get_string_parameter, locate_entry,
    set_extension_i32,
};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

pub struct UpdateWorldBookEntryTool;

#[async_trait]
impl AIToolTrait for UpdateWorldBookEntryTool {
    fn name(&self) -> &'static str {
        "update_world_book_entry"
    }

    fn description(&self) -> &'static str {
        "更新当前角色的单个世界书条目。优先使用 entry_id 定位；只会覆盖显式传入的字段，未传字段保持不变。"
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

        let mut character_data =
            match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                Ok(Some(data)) => data,
                Ok(None) => return error_result(start_time, "角色不存在", None),
                Err(error) => {
                    return error_result(start_time, &format!("获取角色数据失败: {}", error), None)
                }
            };

        let world_book = match character_data.card.data.character_book.as_mut() {
            Some(book) => book,
            None => return error_result(start_time, "当前角色没有世界书", None),
        };

        let selection = match locate_entry(&world_book.entries, &request.parameters, "更新") {
            Ok(selection) => selection,
            Err(error) => return error_result(start_time, &error.message, Some(error.details)),
        };

        let (updated_fields, entry_snapshot) = {
            let entry = &mut world_book.entries[selection.index];
            let updated_fields = match apply_entry_updates(entry, request) {
                Ok(updated_fields) => updated_fields,
                Err(error) => {
                    return ToolResult {
                        success: false,
                        data: None,
                        error: Some(error),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                    }
                }
            };

            (updated_fields, detailed_entry(entry))
        };

        if updated_fields.is_empty() {
            return error_result(
                start_time,
                "必须提供至少一个要更新的字段",
                Some(json!({
                    "supported_fields": [
                        "name",
                        "keys",
                        "content",
                        "comment",
                        "enabled",
                        "priority",
                        "position",
                        "depth",
                        "probability"
                    ],
                })),
            );
        }

        match CharacterStorage::update_character(app_handle, &character_uuid, &character_data.card)
        {
            Ok(()) => {
                let entry_id = entry_snapshot["id"].clone();
                let entry_name = entry_snapshot["name"].clone();

                if let Err(error) = app_handle.emit(
                    "world-book-entry-updated",
                    json!({
                        "character_uuid": character_uuid,
                        "entry_id": entry_id,
                        "entry_name": entry_name,
                        "updated_fields": updated_fields.clone(),
                    }),
                ) {
                    eprintln!("发送世界书条目更新事件失败: {}", error);
                }

                ToolResult {
                    success: true,
                    data: Some(json!({
                        "message": "世界书条目更新成功",
                        "matched_by": selection.matched_by,
                        "matched_value": selection.matched_value,
                        "updated_fields": updated_fields,
                        "entry": entry_snapshot,
                    })),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            Err(error) => error_result(start_time, &format!("保存世界书变更失败: {}", error), None),
        }
    }

    fn to_tool_definition(&self) -> ToolDefinition {
        let mut properties = HashMap::new();
        properties.insert(
            "at_least_one_identifier".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("必须提供至少一个定位字段（entry_id、name 或 key）".to_string()),
                enum_values: Some(vec!["update_world_book_entry".to_string()]),
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
                description: Some(
                    "条目名称，也可作为定位字段；如果用于更新内容，请优先配合 entry_id".to_string(),
                ),
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
                description: Some("条目关键词，只有在能唯一匹配单个条目时才可用于定位".to_string()),
                enum_values: None,
                items: None,
                properties: None,
                required: None,
            },
        );

        for (field, description, param_type) in [
            ("keys", "关键词，多个关键词用逗号分隔", "string"),
            ("content", "条目内容", "string"),
            ("comment", "备注", "string"),
            ("enabled", "是否启用（true/false）", "boolean"),
            ("priority", "优先级（数字）", "integer"),
            ("position", "位置（before_char/after_char）", "string"),
            ("depth", "插入深度（数字）", "integer"),
            ("probability", "触发概率（数字0-100）", "integer"),
        ] {
            properties.insert(
                field.to_string(),
                ChatToolParameter {
                    param_type: param_type.to_string(),
                    description: Some(description.to_string()),
                    enum_values: None,
                    items: None,
                    properties: None,
                    required: None,
                },
            );
        }

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

fn apply_entry_updates(
    entry: &mut WorldBookEntry,
    request: &ToolCallRequest,
) -> Result<Vec<&'static str>, String> {
    let mut updated_fields = Vec::new();

    if let Some(keys) = get_string_parameter(&request.parameters, "keys") {
        let parsed_keys = keys
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();
        if parsed_keys.is_empty() {
            return Err("keys 参数不能为空".to_string());
        }
        entry.keys = parsed_keys;
        updated_fields.push("keys");
    }

    if let Some(content) = get_string_parameter(&request.parameters, "content") {
        entry.content = content.to_string();
        updated_fields.push("content");
    }

    if let Some(comment) = get_string_parameter(&request.parameters, "comment") {
        entry.comment = Some(comment.to_string());
        updated_fields.push("comment");
    }

    if let Some(enabled) = get_bool_parameter(&request.parameters, "enabled") {
        entry.enabled = enabled;
        updated_fields.push("enabled");
    }

    if let Some(priority) = get_i32_parameter(&request.parameters, "priority") {
        entry.priority = Some(priority);
        updated_fields.push("priority");
    }

    if let Some(position) = get_string_parameter(&request.parameters, "position") {
        entry.position = Some(position.to_string());
        updated_fields.push("position");
    }

    if let Some(depth) = get_i32_parameter(&request.parameters, "depth") {
        set_extension_i32(entry, "depth", depth);
        updated_fields.push("depth");
    }

    if let Some(probability) = get_i32_parameter(&request.parameters, "probability") {
        set_extension_i32(entry, "probability", probability);
        updated_fields.push("probability");
    }

    if should_update_name(request) {
        if let Some(name) = get_string_parameter(&request.parameters, "name") {
            entry.name = Some(name.to_string());
            updated_fields.push("name");
        }
    }

    Ok(updated_fields)
}

fn should_update_name(request: &ToolCallRequest) -> bool {
    request.parameters.contains_key("entry_id")
        || request.parameters.contains_key("key")
        || request.parameters.contains_key("content")
        || request.parameters.contains_key("keys")
        || request.parameters.contains_key("comment")
        || request.parameters.contains_key("enabled")
        || request.parameters.contains_key("priority")
        || request.parameters.contains_key("position")
        || request.parameters.contains_key("depth")
        || request.parameters.contains_key("probability")
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
