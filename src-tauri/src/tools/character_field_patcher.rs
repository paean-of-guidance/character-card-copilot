use super::AIToolTrait;
use crate::ai_tools::{
    ToolCallRequest, ToolDefinition, ToolFunction, ToolParameter as ChatToolParameter,
    ToolParameters, ToolResult,
};
use crate::backend::application::event_bus::EventBus;
use crate::backend::domain::CharacterUpdateType;
use crate::character_storage::{CharacterStorage, TavernCardV2};
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use tauri::AppHandle;

const SUPPORTED_FIELDS: &[(&str, &str)] = &[
    ("description", "角色描述"),
    ("personality", "性格特点"),
    ("scenario", "场景设定"),
    ("first_mes", "开场白"),
    ("mes_example", "对话示例"),
    ("creator_notes", "创作者笔记"),
    ("system_prompt", "系统提示词"),
    ("post_history_instructions", "历史后指令"),
];

pub struct PatchCharacterFieldTool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatchOperation {
    Replace,
    InsertBefore,
    InsertAfter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchMode {
    Exact,
    Regex,
}

#[derive(Debug, Clone)]
struct MatchSpan {
    start: usize,
    end: usize,
    matched_text: String,
}

#[derive(Debug, Clone)]
struct PatchOutput {
    updated_text: String,
    matched_text: String,
    match_start: usize,
    match_end: usize,
}

#[derive(Debug, Clone)]
struct ToolFailure {
    code: &'static str,
    message: String,
    details: Option<Value>,
}

#[async_trait]
impl AIToolTrait for PatchCharacterFieldTool {
    fn name(&self) -> &'static str {
        "patch_character_field"
    }

    fn description(&self) -> &'static str {
        "对角色卡的单个长文本字段做局部补丁编辑。支持 replace、insert_before、insert_after 三种操作，支持 exact 或 regex 搜索。搜索结果必须唯一：0 个匹配或超过 1 个匹配都会失败。适合只修改 description、personality 等字段中的某一小段内容，而不是整段重写。"
    }

    fn category(&self) -> &'static str {
        "character"
    }

    async fn execute(&self, app_handle: &AppHandle, request: &ToolCallRequest) -> ToolResult {
        let start_time = std::time::Instant::now();

        let character_uuid = match &request.character_uuid {
            Some(uuid) => uuid.clone(),
            None => {
                return failure_result(
                    start_time,
                    "missing_character_uuid",
                    "缺少角色UUID".to_string(),
                    None,
                );
            }
        };

        let field = match required_string(&request.parameters, "field") {
            Ok(value) => value,
            Err(failure) => return failure_result(start_time, failure.code, failure.message, failure.details),
        };

        let operation = match parse_operation(&request.parameters) {
            Ok(value) => value,
            Err(failure) => return failure_result(start_time, failure.code, failure.message, failure.details),
        };

        let match_mode = match parse_match_mode(&request.parameters) {
            Ok(value) => value,
            Err(failure) => return failure_result(start_time, failure.code, failure.message, failure.details),
        };

        let search = match required_string(&request.parameters, "search") {
            Ok(value) if !value.is_empty() => value,
            Ok(_) => {
                return failure_result(
                    start_time,
                    "empty_search",
                    "search 不能为空".to_string(),
                    None,
                )
            }
            Err(failure) => return failure_result(start_time, failure.code, failure.message, failure.details),
        };

        let content = match required_string(&request.parameters, "content") {
            Ok(value) => value,
            Err(failure) => return failure_result(start_time, failure.code, failure.message, failure.details),
        };

        let character_data = match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
            Ok(Some(data)) => data,
            Ok(None) => {
                return failure_result(start_time, "character_not_found", "角色不存在".to_string(), None)
            }
            Err(error) => {
                return failure_result(
                    start_time,
                    "load_character_failed",
                    format!("获取角色数据失败: {}", error),
                    None,
                )
            }
        };

        let mut tavern_card = character_data.card;
        let original_text = match get_field_value(&tavern_card, &field) {
            Some(value) => value.to_string(),
            None => {
                return failure_result(
                    start_time,
                    "unsupported_field",
                    format!("字段 '{}' 不支持 patch_character_field", field),
                    Some(serde_json::json!({
                        "field": field,
                        "supported_fields": supported_field_names(),
                    })),
                )
            }
        };

        let patch_output = match apply_patch(&original_text, operation, match_mode, &search, &content) {
            Ok(output) => output,
            Err(failure) => return failure_result(start_time, failure.code, failure.message, failure.details),
        };

        if let Err(failure) = set_field_value(&mut tavern_card, &field, patch_output.updated_text.clone()) {
            return failure_result(start_time, failure.code, failure.message, failure.details);
        }

        match CharacterStorage::update_character(app_handle, &character_uuid, &tavern_card) {
            Ok(_) => {
                let updated_character_data = match CharacterStorage::get_character_by_uuid(app_handle, &character_uuid) {
                    Ok(Some(data)) => data,
                    Ok(None) => {
                        return failure_result(
                            start_time,
                            "reload_character_failed",
                            "角色已保存，但重新加载失败".to_string(),
                            None,
                        )
                    }
                    Err(error) => {
                        return failure_result(
                            start_time,
                            "reload_character_failed",
                            format!("角色已保存，但重新加载失败: {}", error),
                            None,
                        )
                    }
                };

                if let Err(error) = EventBus::character_updated(
                    app_handle,
                    &character_uuid,
                    &updated_character_data,
                    CharacterUpdateType::Fields {
                        fields: vec![field.clone()],
                    },
                ) {
                    eprintln!("发送角色更新事件失败: {}", error);
                }

                ToolResult {
                    success: true,
                    data: Some(serde_json::json!({
                        "message": "角色字段局部更新成功",
                        "field": field,
                        "operation": operation.as_str(),
                        "match_mode": match_mode.as_str(),
                        "matched_text": patch_output.matched_text,
                        "match_start": patch_output.match_start,
                        "match_end": patch_output.match_end,
                        "updated_length": patch_output.updated_text.chars().count(),
                    })),
                    error: None,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
            Err(error) => failure_result(
                start_time,
                "save_character_failed",
                format!("保存角色数据失败: {}", error),
                None,
            ),
        }
    }

    fn to_tool_definition(&self) -> ToolDefinition {
        let mut properties = HashMap::new();

        properties.insert(
            "field".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("要修改的字段，仅支持长文本字段，如 description、personality、scenario 等".to_string()),
                enum_values: Some(supported_field_names()),
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "operation".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("补丁操作：replace、insert_before、insert_after".to_string()),
                enum_values: Some(vec![
                    "replace".to_string(),
                    "insert_before".to_string(),
                    "insert_after".to_string(),
                ]),
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "match_mode".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("匹配模式：exact 表示纯文本唯一匹配，regex 表示正则唯一匹配".to_string()),
                enum_values: Some(vec!["exact".to_string(), "regex".to_string()]),
                items: None,
                properties: None,
                required: None,
            },
        );

        properties.insert(
            "search".to_string(),
            ChatToolParameter {
                param_type: "string".to_string(),
                description: Some("要搜索的目标文本或正则表达式。必须唯一命中，否则调用失败".to_string()),
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
                description: Some("替换内容或插入内容".to_string()),
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
                        "field".to_string(),
                        "operation".to_string(),
                        "match_mode".to_string(),
                        "search".to_string(),
                        "content".to_string(),
                    ]),
                }),
            },
        }
    }
}

impl PatchOperation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Replace => "replace",
            Self::InsertBefore => "insert_before",
            Self::InsertAfter => "insert_after",
        }
    }
}

impl MatchMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Regex => "regex",
        }
    }
}

fn parse_operation(parameters: &HashMap<String, Value>) -> Result<PatchOperation, ToolFailure> {
    let raw = required_string(parameters, "operation")?;
    match raw.as_str() {
        "replace" => Ok(PatchOperation::Replace),
        "insert_before" => Ok(PatchOperation::InsertBefore),
        "insert_after" => Ok(PatchOperation::InsertAfter),
        _ => Err(ToolFailure {
            code: "invalid_operation",
            message: format!("不支持的 operation: {}", raw),
            details: Some(serde_json::json!({
                "operation": raw,
                "supported_operations": ["replace", "insert_before", "insert_after"],
            })),
        }),
    }
}

fn parse_match_mode(parameters: &HashMap<String, Value>) -> Result<MatchMode, ToolFailure> {
    let raw = required_string(parameters, "match_mode")?;
    match raw.as_str() {
        "exact" => Ok(MatchMode::Exact),
        "regex" => Ok(MatchMode::Regex),
        _ => Err(ToolFailure {
            code: "invalid_match_mode",
            message: format!("不支持的 match_mode: {}", raw),
            details: Some(serde_json::json!({
                "match_mode": raw,
                "supported_match_modes": ["exact", "regex"],
            })),
        }),
    }
}

fn required_string(parameters: &HashMap<String, Value>, key: &'static str) -> Result<String, ToolFailure> {
    match parameters.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(ToolFailure {
            code: "invalid_parameter_type",
            message: format!("参数 '{}' 必须是字符串", key),
            details: Some(serde_json::json!({ "parameter": key })),
        }),
        None => Err(ToolFailure {
            code: "missing_parameter",
            message: format!("缺少必填参数 '{}'", key),
            details: Some(serde_json::json!({ "parameter": key })),
        }),
    }
}

fn get_field_value<'a>(card: &'a TavernCardV2, field: &str) -> Option<&'a str> {
    match field {
        "description" => Some(&card.data.description),
        "personality" => Some(&card.data.personality),
        "scenario" => Some(&card.data.scenario),
        "first_mes" => Some(&card.data.first_mes),
        "mes_example" => Some(&card.data.mes_example),
        "creator_notes" => Some(&card.data.creator_notes),
        "system_prompt" => Some(&card.data.system_prompt),
        "post_history_instructions" => Some(&card.data.post_history_instructions),
        _ => None,
    }
}

fn set_field_value(card: &mut TavernCardV2, field: &str, value: String) -> Result<(), ToolFailure> {
    match field {
        "description" => card.data.description = value,
        "personality" => card.data.personality = value,
        "scenario" => card.data.scenario = value,
        "first_mes" => card.data.first_mes = value,
        "mes_example" => card.data.mes_example = value,
        "creator_notes" => card.data.creator_notes = value,
        "system_prompt" => card.data.system_prompt = value,
        "post_history_instructions" => card.data.post_history_instructions = value,
        _ => {
            return Err(ToolFailure {
                code: "unsupported_field",
                message: format!("字段 '{}' 不支持 patch_character_field", field),
                details: Some(serde_json::json!({
                    "field": field,
                    "supported_fields": supported_field_names(),
                })),
            })
        }
    }

    Ok(())
}

fn apply_patch(
    original_text: &str,
    operation: PatchOperation,
    match_mode: MatchMode,
    search: &str,
    content: &str,
) -> Result<PatchOutput, ToolFailure> {
    let matched_span = find_unique_match(original_text, match_mode, search)?;

    let updated_text = match operation {
        PatchOperation::Replace => {
            let mut next = String::with_capacity(original_text.len() - matched_span.matched_text.len() + content.len());
            next.push_str(&original_text[..matched_span.start]);
            next.push_str(content);
            next.push_str(&original_text[matched_span.end..]);
            next
        }
        PatchOperation::InsertBefore => {
            let mut next = String::with_capacity(original_text.len() + content.len());
            next.push_str(&original_text[..matched_span.start]);
            next.push_str(content);
            next.push_str(&original_text[matched_span.start..]);
            next
        }
        PatchOperation::InsertAfter => {
            let mut next = String::with_capacity(original_text.len() + content.len());
            next.push_str(&original_text[..matched_span.end]);
            next.push_str(content);
            next.push_str(&original_text[matched_span.end..]);
            next
        }
    };

    Ok(PatchOutput {
        updated_text,
        matched_text: matched_span.matched_text,
        match_start: matched_span.start,
        match_end: matched_span.end,
    })
}

fn find_unique_match(text: &str, match_mode: MatchMode, search: &str) -> Result<MatchSpan, ToolFailure> {
    let matches = match match_mode {
        MatchMode::Exact => find_exact_matches(text, search),
        MatchMode::Regex => find_regex_matches(text, search)?,
    };

    match matches.as_slice() {
        [] => Err(ToolFailure {
            code: "no_match",
            message: "search 未命中任何内容，未执行修改".to_string(),
            details: Some(serde_json::json!({
                "search": search,
                "match_mode": match_mode.as_str(),
            })),
        }),
        [single] => Ok(single.clone()),
        multiple => Err(ToolFailure {
            code: "multiple_matches",
            message: format!(
                "search 匹配到 {} 处内容，结果不唯一，未执行修改",
                multiple.len()
            ),
            details: Some(serde_json::json!({
                "search": search,
                "match_mode": match_mode.as_str(),
                "match_count": multiple.len(),
            })),
        }),
    }
}

fn find_exact_matches(text: &str, search: &str) -> Vec<MatchSpan> {
    text.match_indices(search)
        .map(|(start, matched)| MatchSpan {
            start,
            end: start + matched.len(),
            matched_text: matched.to_string(),
        })
        .collect()
}

fn find_regex_matches(text: &str, search: &str) -> Result<Vec<MatchSpan>, ToolFailure> {
    let regex = Regex::new(search).map_err(|error| ToolFailure {
        code: "invalid_regex",
        message: format!("无效的正则表达式: {}", error),
        details: Some(serde_json::json!({
            "search": search,
        })),
    })?;

    Ok(regex
        .find_iter(text)
        .map(|matched| MatchSpan {
            start: matched.start(),
            end: matched.end(),
            matched_text: matched.as_str().to_string(),
        })
        .collect())
}

fn supported_field_names() -> Vec<String> {
    SUPPORTED_FIELDS
        .iter()
        .map(|(field, _)| (*field).to_string())
        .collect()
}

fn failure_result(
    start_time: std::time::Instant,
    code: &'static str,
    message: String,
    details: Option<Value>,
) -> ToolResult {
    ToolResult {
        success: false,
        data: Some(serde_json::json!({
            "error_code": code,
            "details": details,
        })),
        error: Some(message),
        execution_time_ms: start_time.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_patch, MatchMode, PatchOperation};

    #[test]
    fn replace_exact_unique_match() {
        let output = apply_patch(
            "Proud, Dominant, Cold",
            PatchOperation::Replace,
            MatchMode::Exact,
            "Dominant",
            "Dominant, Possessive",
        )
        .expect("patch should succeed");

        assert_eq!(output.updated_text, "Proud, Dominant, Possessive, Cold");
        assert_eq!(output.matched_text, "Dominant");
    }

    #[test]
    fn insert_after_regex_unique_match() {
        let output = apply_patch(
            "Height: 175cm\nWeight: 60kg",
            PatchOperation::InsertAfter,
            MatchMode::Regex,
            r"175cm",
            "\nBuild: athletic",
        )
        .expect("patch should succeed");

        assert_eq!(output.updated_text, "Height: 175cm\nBuild: athletic\nWeight: 60kg");
    }

    #[test]
    fn no_match_returns_error() {
        let error = apply_patch(
            "Proud, Dominant, Cold",
            PatchOperation::Replace,
            MatchMode::Exact,
            "Gentle",
            "Gentle",
        )
        .expect_err("patch should fail");

        assert_eq!(error.code, "no_match");
    }

    #[test]
    fn multiple_matches_returns_error() {
        let error = apply_patch(
            "cold and cold again",
            PatchOperation::Replace,
            MatchMode::Exact,
            "cold",
            "warm",
        )
        .expect_err("patch should fail");

        assert_eq!(error.code, "multiple_matches");
    }
}
