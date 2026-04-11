use crate::character_storage::WorldBookEntry;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

const CONTENT_PREVIEW_CHAR_LIMIT: usize = 50;

#[derive(Debug, Clone)]
pub struct EntrySelection {
    pub index: usize,
    pub matched_by: &'static str,
    pub matched_value: Value,
}

#[derive(Debug, Clone)]
pub struct EntryLookupError {
    pub code: &'static str,
    pub message: String,
    pub details: Value,
}

pub fn get_string_parameter<'a>(
    parameters: &'a HashMap<String, Value>,
    key: &str,
) -> Option<&'a str> {
    parameters.get(key).and_then(Value::as_str).map(str::trim)
}

pub fn get_i32_parameter(parameters: &HashMap<String, Value>, key: &str) -> Option<i32> {
    let value = parameters.get(key)?;

    if let Some(number) = value.as_i64() {
        return i32::try_from(number).ok();
    }

    value
        .as_str()
        .and_then(|text| text.trim().parse::<i32>().ok())
}

pub fn get_bool_parameter(parameters: &HashMap<String, Value>, key: &str) -> Option<bool> {
    let value = parameters.get(key)?;

    if let Some(boolean) = value.as_bool() {
        return Some(boolean);
    }

    value
        .as_str()
        .and_then(|text| text.trim().parse::<bool>().ok())
}

pub fn get_usize_parameter(parameters: &HashMap<String, Value>, key: &str) -> Option<usize> {
    let value = parameters.get(key)?;

    if let Some(number) = value.as_u64() {
        return usize::try_from(number).ok();
    }

    if let Some(number) = value.as_i64() {
        return usize::try_from(number).ok();
    }

    value
        .as_str()
        .and_then(|text| text.trim().parse::<usize>().ok())
}

pub fn build_content_preview(content: &str) -> String {
    if content.chars().count() > CONTENT_PREVIEW_CHAR_LIMIT {
        let truncated: String = content.chars().take(CONTENT_PREVIEW_CHAR_LIMIT).collect();
        format!("{}...", truncated)
    } else {
        content.to_string()
    }
}

pub fn editable_extensions_summary(entry: &WorldBookEntry) -> Value {
    json!({
        "depth": get_extension_i32(entry, "depth"),
        "probability": get_extension_i32(entry, "probability"),
        "scan_depth": get_extension_i32(entry, "scan_depth"),
    })
}

pub fn summarize_entry(entry: &WorldBookEntry) -> Value {
    json!({
        "id": entry.id,
        "name": entry.name,
        "keys": entry.keys,
        "comment": entry.comment,
        "enabled": entry.enabled,
        "priority": entry.priority,
        "position": entry.position,
        "content_preview": build_content_preview(&entry.content),
        "extensions": editable_extensions_summary(entry),
    })
}

pub fn detailed_entry(entry: &WorldBookEntry) -> Value {
    json!({
        "id": entry.id,
        "name": entry.name,
        "keys": entry.keys,
        "content": entry.content,
        "comment": entry.comment,
        "enabled": entry.enabled,
        "priority": entry.priority,
        "position": entry.position,
        "case_sensitive": entry.case_sensitive,
        "insertion_order": entry.insertion_order,
        "extensions": editable_extensions_summary(entry),
    })
}

pub fn locate_entry(
    entries: &[WorldBookEntry],
    parameters: &HashMap<String, Value>,
    action_label: &str,
) -> Result<EntrySelection, EntryLookupError> {
    if let Some(entry_id) = get_i32_parameter(parameters, "entry_id") {
        return entries
            .iter()
            .position(|entry| entry.id == Some(entry_id))
            .map(|index| EntrySelection {
                index,
                matched_by: "entry_id",
                matched_value: json!(entry_id),
            })
            .ok_or_else(|| EntryLookupError {
                code: "entry_not_found",
                message: format!("未找到 entry_id 为 {} 的世界书条目", entry_id),
                details: json!({
                    "locator": "entry_id",
                    "locator_value": entry_id,
                }),
            });
    }

    if let Some(name) = get_string_parameter(parameters, "name") {
        if name.is_empty() {
            return Err(EntryLookupError {
                code: "invalid_identifier",
                message: "name 不能为空".to_string(),
                details: json!({ "locator": "name" }),
            });
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
            [index] => Ok(EntrySelection {
                index: *index,
                matched_by: "name",
                matched_value: json!(name),
            }),
            [] => Err(EntryLookupError {
                code: "entry_not_found",
                message: format!("未找到名称为“{}”的世界书条目", name),
                details: json!({
                    "locator": "name",
                    "locator_value": name,
                }),
            }),
            _ => Err(ambiguous_lookup_error(
                entries,
                &matched_indexes,
                "name",
                name,
                action_label,
            )),
        };
    }

    if let Some(key) = get_string_parameter(parameters, "key") {
        if key.is_empty() {
            return Err(EntryLookupError {
                code: "invalid_identifier",
                message: "key 不能为空".to_string(),
                details: json!({ "locator": "key" }),
            });
        }

        let matched_indexes = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry
                    .keys
                    .iter()
                    .any(|entry_key| entry_key.trim().eq_ignore_ascii_case(key))
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        return match matched_indexes.as_slice() {
            [index] => Ok(EntrySelection {
                index: *index,
                matched_by: "key",
                matched_value: json!(key),
            }),
            [] => Err(EntryLookupError {
                code: "entry_not_found",
                message: format!("未找到包含关键词“{}”的世界书条目", key),
                details: json!({
                    "locator": "key",
                    "locator_value": key,
                }),
            }),
            _ => Err(ambiguous_lookup_error(
                entries,
                &matched_indexes,
                "key",
                key,
                action_label,
            )),
        };
    }

    Err(EntryLookupError {
        code: "missing_identifier",
        message: "缺少定位参数：请至少提供 entry_id、name 或 key 之一".to_string(),
        details: json!({
            "supported_identifiers": ["entry_id", "name", "key"],
        }),
    })
}

pub fn get_extension_i32(entry: &WorldBookEntry, key: &str) -> Option<i32> {
    entry.extensions.get(key).and_then(value_to_i32)
}

pub fn set_extension_i32(entry: &mut WorldBookEntry, key: &str, value: i32) {
    entry.extensions[key] = json!(value);
}

pub fn entry_matches_query(entry: &WorldBookEntry, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query_lower = query.to_ascii_lowercase();
    let mut haystacks = vec![
        entry.id.map(|id| id.to_string()).unwrap_or_default(),
        entry.name.clone().unwrap_or_default(),
        entry.comment.clone().unwrap_or_default(),
        entry.content.clone(),
        entry.position.clone().unwrap_or_default(),
    ];
    haystacks.extend(entry.keys.iter().cloned());

    haystacks
        .into_iter()
        .any(|value| value.to_ascii_lowercase().contains(&query_lower))
}

pub fn unique_fragments_from_text(text: &str, min_chars: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut fragments = Vec::new();

    for fragment in text.split_whitespace() {
        let trimmed = fragment.trim_matches(|ch: char| ch.is_ascii_punctuation());
        if trimmed.chars().count() < min_chars {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            fragments.push(trimmed.to_string());
        }
    }

    fragments
}

fn ambiguous_lookup_error(
    entries: &[WorldBookEntry],
    matched_indexes: &[usize],
    locator: &'static str,
    locator_value: &str,
    action_label: &str,
) -> EntryLookupError {
    let candidates = matched_indexes
        .iter()
        .take(5)
        .map(|index| summarize_entry(&entries[*index]))
        .collect::<Vec<_>>();

    let locator_label = match locator {
        "name" => "名称",
        "key" => "关键词",
        _ => locator,
    };

    EntryLookupError {
        code: "multiple_entries",
        message: format!(
            "找到多个{}为“{}”的世界书条目，请改用 entry_id {}。候选已返回。",
            locator_label, locator_value, action_label
        ),
        details: json!({
            "locator": locator,
            "locator_value": locator_value,
            "candidate_count": matched_indexes.len(),
            "candidates": candidates,
        }),
    }
}

fn value_to_i32(value: &Value) -> Option<i32> {
    if let Some(number) = value.as_i64() {
        return i32::try_from(number).ok();
    }

    value
        .as_str()
        .and_then(|text| text.trim().parse::<i32>().ok())
}

#[cfg(test)]
mod tests {
    use super::{locate_entry, summarize_entry};
    use crate::character_storage::WorldBookEntry;
    use serde_json::json;
    use std::collections::HashMap;

    fn sample_entry(id: i32, name: &str, key: &str) -> WorldBookEntry {
        WorldBookEntry {
            id: Some(id),
            name: Some(name.to_string()),
            keys: vec![key.to_string()],
            content: format!("content-{id}"),
            extensions: json!({ "depth": 3, "probability": 80 }),
            enabled: true,
            insertion_order: id,
            case_sensitive: Some(false),
            priority: Some(10),
            comment: Some(format!("comment-{id}")),
            selective: None,
            secondary_keys: None,
            constant: None,
            position: Some("before_char".to_string()),
        }
    }

    #[test]
    fn locate_entry_by_id() {
        let entries = vec![sample_entry(1, "Alpha", "alpha"), sample_entry(2, "Beta", "beta")];
        let mut params = HashMap::new();
        params.insert("entry_id".to_string(), json!(2));

        let selection = locate_entry(&entries, &params, "读取").expect("should locate entry");
        assert_eq!(selection.index, 1);
        assert_eq!(selection.matched_by, "entry_id");
    }

    #[test]
    fn locate_entry_reports_candidates() {
        let entries = vec![sample_entry(1, "Alpha", "tag"), sample_entry(2, "Beta", "tag")];
        let mut params = HashMap::new();
        params.insert("key".to_string(), json!("tag"));

        let error = locate_entry(&entries, &params, "更新").expect_err("should be ambiguous");
        assert_eq!(error.code, "multiple_entries");
        assert_eq!(error.details["candidates"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn summarize_entry_contains_preview() {
        let summary = summarize_entry(&sample_entry(1, "Alpha", "alpha"));
        assert_eq!(summary["id"], json!(1));
        assert!(summary["content_preview"].as_str().is_some());
    }
}
