use super::{error_result, success_result, AIToolTrait};
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::character_storage::{CharacterBook, CharacterStorage, WorldBookEntry};
use crate::tools::world_book_shared::build_content_preview;
use async_trait::async_trait;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

const REQUIRED_FIELDS: &[&str] = &["keys", "content", "depth", "comment", "probability"];

/// 世界书条目创建工具
pub struct CreateWorldBookEntryTool;

struct EntryParameterTarget<'a> {
    entry: &'a mut WorldBookEntry,
    extensions: &'a mut serde_json::Value,
    world_book: &'a mut CharacterBook,
}

fn default_entry_extensions() -> serde_json::Value {
    serde_json::json!({
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
    })
}

fn next_entry_id(entries: &[WorldBookEntry]) -> i32 {
    entries
        .iter()
        .filter_map(|entry| entry.id)
        .max()
        .unwrap_or(0)
        + 1
}

fn next_insertion_order(entries: &[WorldBookEntry]) -> i32 {
    entries
        .iter()
        .map(|entry| entry.insertion_order)
        .max()
        .unwrap_or(0)
        + 1
}

fn create_empty_entry(id: i32, insertion_order: i32) -> WorldBookEntry {
    WorldBookEntry {
        id: Some(id),
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
    }
}

fn missing_required_parameter(
    parameters: &HashMap<String, serde_json::Value>,
) -> Option<&'static str> {
    REQUIRED_FIELDS
        .iter()
        .copied()
        .find(|field| !parameters.contains_key(*field))
}

fn get_character_uuid(
    request: &ToolCallRequest,
    start_time: std::time::Instant,
) -> Result<String, ToolResult> {
    request
        .character_uuid
        .clone()
        .ok_or_else(|| error_result(start_time, "缺少角色UUID"))
}

fn validate_required_parameters(
    parameters: &HashMap<String, serde_json::Value>,
    start_time: std::time::Instant,
) -> Result<(), ToolResult> {
    if let Some(field) = missing_required_parameter(parameters) {
        return Err(error_result(start_time, format!("缺少必填参数: {}", field)));
    }

    Ok(())
}

fn load_character_data(
    app_handle: &AppHandle,
    character_uuid: &str,
    start_time: std::time::Instant,
) -> Result<crate::character_storage::CharacterData, ToolResult> {
    match CharacterStorage::get_character_by_uuid(app_handle, character_uuid) {
        Ok(Some(data)) => Ok(data),
        Ok(None) => Err(error_result(start_time, "角色不存在")),
        Err(error) => Err(error_result(
            start_time,
            format!("获取角色数据失败: {}", error),
        )),
    }
}

fn get_or_create_world_book(
    character_data: &mut crate::character_storage::CharacterData,
) -> &mut CharacterBook {
    character_data
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
        })
}

fn apply_entry_parameters(
    entry: &mut WorldBookEntry,
    extensions: &mut serde_json::Value,
    world_book: &mut CharacterBook,
    parameters: &HashMap<String, serde_json::Value>,
) {
    let mut parameter_target = EntryParameterTarget {
        entry,
        extensions,
        world_book,
    };

    for (field_name, field_value) in parameters {
        if field_name == "at_least_one_field" {
            continue;
        }

        if let Some(value_str) = field_value.as_str() {
            apply_entry_parameter(&mut parameter_target, field_name, value_str);
        }
    }
}

fn validate_created_entry(
    entry: &WorldBookEntry,
    start_time: std::time::Instant,
) -> Result<(), ToolResult> {
    if entry.keys.is_empty() {
        return Err(error_result(start_time, "keys 参数不能为空"));
    }

    if entry.content.is_empty() {
        return Err(error_result(start_time, "content 参数不能为空"));
    }

    Ok(())
}

fn apply_entry_parameter(target: &mut EntryParameterTarget<'_>, field_name: &str, value: &str) {
    match field_name {
        "keys" => target.entry.keys = parse_keys(value),
        "content" => target.entry.content = value.to_string(),
        "name" => target.entry.name = Some(value.to_string()),
        "comment" => target.entry.comment = Some(value.to_string()),
        "enabled" => target.entry.enabled = value.parse().unwrap_or(true),
        "priority" => target.entry.priority = value.parse().ok(),
        "position" => target.entry.position = Some(value.to_string()),
        "depth" => set_extension_i32(target.extensions, "depth", value),
        "probability" => set_extension_i32(target.extensions, "probability", value),
        "scan_depth" => set_extension_i32(target.extensions, "scan_depth", value),
        "token_budget" => set_token_budget(target.world_book, value),
        "recursive_scanning" => set_recursive_scanning(target.world_book, value),
        _ => target.extensions[field_name] = serde_json::Value::String(value.to_string()),
    }
}

fn parse_keys(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_string)
        .collect()
}

fn set_extension_i32(extensions: &mut serde_json::Value, key: &str, value: &str) {
    if let Ok(number) = value.parse::<i32>() {
        extensions[key] = serde_json::Value::Number(number.into());
    }
}

fn set_token_budget(world_book: &mut CharacterBook, value: &str) {
    if let Ok(number) = value.parse::<i32>() {
        world_book.token_budget = Some(number);
    }
}

fn set_recursive_scanning(world_book: &mut CharacterBook, value: &str) {
    if let Ok(enabled) = value.parse::<bool>() {
        world_book.recursive_scanning = Some(enabled);
    }
}

fn emit_entry_created(
    app_handle: &AppHandle,
    character_uuid: &str,
    entry_id: i32,
    entry: &WorldBookEntry,
) {
    if let Err(error) = app_handle.emit(
        "world-book-entry-created",
        serde_json::json!({
            "character_uuid": character_uuid,
            "entry_id": entry_id,
            "entry_name": entry.name,
            "keys": entry.keys
        }),
    ) {
        eprintln!("发送世界书条目创建事件失败: {}", error);
    }
}

fn created_entry_result(
    start_time: std::time::Instant,
    entry_id: i32,
    entry: WorldBookEntry,
) -> ToolResult {
    let content_preview = build_content_preview(&entry.content);
    success_result(
        start_time,
        serde_json::json!({
            "message": "世界书条目创建成功",
            "entry_id": entry_id,
            "entry_name": entry.name,
            "keys": entry.keys,
            "content_preview": content_preview
        }),
    )
}

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
        let character_uuid = match get_character_uuid(request, start_time) {
            Ok(uuid) => uuid,
            Err(result) => return result,
        };
        if let Err(result) = validate_required_parameters(&request.parameters, start_time) {
            return result;
        }

        let mut character_data = match load_character_data(app_handle, &character_uuid, start_time)
        {
            Ok(data) => data,
            Err(result) => return result,
        };
        let world_book = get_or_create_world_book(&mut character_data);

        let new_id = next_entry_id(&world_book.entries);
        let insertion_order = next_insertion_order(&world_book.entries);
        let mut extensions = default_entry_extensions();
        let mut new_entry = create_empty_entry(new_id, insertion_order);

        apply_entry_parameters(
            &mut new_entry,
            &mut extensions,
            world_book,
            &request.parameters,
        );
        if let Err(result) = validate_created_entry(&new_entry, start_time) {
            return result;
        }

        new_entry.extensions = extensions;
        world_book.entries.push(new_entry.clone());

        match CharacterStorage::update_character(app_handle, &character_uuid, &character_data.card)
        {
            Ok(()) => {
                emit_entry_created(app_handle, &character_uuid, new_id, &new_entry);
                created_entry_result(start_time, new_id, new_entry)
            }
            Err(e) => error_result(start_time, format!("保存世界书条目失败: {}", e)),
        }
    }

    fn to_tool_definition(&self) -> ToolDefinition {
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
                description: Some(
                    "备注，需要简短，格式为(rule/background或者其他的)[10字/words以内概括]"
                        .to_string(),
                ),
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

        ToolDefinition {
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
