pub(crate) fn format_tool_message(
    tool_name: &str,
    success: bool,
    data: Option<&serde_json::Value>,
    error: Option<&str>,
) -> String {
    if !success {
        return format_tool_error(tool_name, error, data);
    }

    match tool_name {
        "patch_character_field" => format_patch_character_field_result(data),
        "read_character_field" => format_read_character_field_result(data),
        "edit_character" => format_edit_character_result(data),
        "list_world_book_entries" => format_list_world_book_entries_result(data),
        "read_world_book_entry" => format_read_world_book_entry_result(data),
        "create_world_book_entry" => format_create_world_book_entry_result(data),
        "update_world_book_entry" => format_update_world_book_entry_result(data),
        "delete_world_book_entry" => format_delete_world_book_entry_result(data),
        _ => format_generic_tool_result(tool_name, data),
    }
}

fn format_tool_error(
    tool_name: &str,
    error: Option<&str>,
    data: Option<&serde_json::Value>,
) -> String {
    let mut lines = vec![format!("failed {tool_name}")];

    if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
        lines.push(error.to_string());
    }

    if tool_name == "patch_character_field" {
        if let Some(details) = data
            .and_then(as_object)
            .and_then(|object| object.get("details"))
        {
            let patch_lines = format_patch_failure_details(details);
            if !patch_lines.is_empty() {
                lines.extend(patch_lines);
                return lines.join("\n");
            }
        }
    }

    if let Some(data) = data {
        let rendered = render_value_lines(data);
        if !rendered.is_empty() {
            lines.extend(rendered);
        }
    }

    lines.join("\n")
}

fn format_patch_character_field_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data.and_then(as_object) else {
        return "ok patch_character_field".to_string();
    };

    let field = get_string(data, "field");
    let operation = get_string(data, "operation");
    let match_mode = get_string(data, "match_mode");
    let dry_run = data.get("dry_run").and_then(|value| value.as_bool());

    let mut meta = Vec::new();
    if let Some(field) = field {
        meta.push(format!("field={field}"));
    }
    if let Some(operation) = operation {
        meta.push(format!("op={operation}"));
    }
    if let Some(match_mode) = match_mode {
        meta.push(format!("match={match_mode}"));
    }
    if let Some(dry_run) = dry_run {
        meta.push(format!("dry_run={dry_run}"));
    }

    let matched_context = data.get("matched_context").and_then(as_object);
    let updated_context = data.get("updated_context").and_then(as_object);
    let before_lines = matched_context
        .map(|context| build_context_diff_lines(context, "matched_text", "-"))
        .unwrap_or_default();
    let after_lines = updated_context
        .map(|context| build_context_diff_lines(context, "selected_text", "+"))
        .unwrap_or_default();

    if !before_lines.is_empty() || !after_lines.is_empty() {
        let mut diff_lines = before_lines;
        diff_lines.extend(after_lines);
        return format_change_result(
            "patch_character_field",
            meta,
            Some(format_fenced_block("diff", &diff_lines.join("\n"))),
        );
    }

    let body = get_string(data, "updated_preview");
    format_change_result("patch_character_field", meta, body)
}

fn format_read_character_field_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data.and_then(as_object) else {
        return "ok read_character_field".to_string();
    };

    let mut meta = Vec::new();
    let mut label = "text".to_string();
    let mut text_body = None;
    if let Some(field) = get_string(data, "field") {
        label = field.clone();
        let start = data
            .get("start")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let end = data
            .get("end")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let total = data
            .get("total_length")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let truncated = data
            .get("truncated")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        meta.push(format!(
            "field={field} | range={start}..{end}/{total}{}",
            if truncated { " | truncated" } else { "" }
        ));
        if let Some(text) = get_string(data, "text") {
            text_body = Some(text);
        }
    } else if let Some(text) = get_string(data, "text") {
        text_body = Some(text);
    }

    format_read_result("read_character_field", meta, &label, text_body.as_deref())
}

fn format_edit_character_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data.and_then(as_object) else {
        return "ok edit_character".to_string();
    };

    let updated_fields = data
        .get("updated_fields")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(as_object)
                .filter_map(|item| get_string(item, "field"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if updated_fields.is_empty() {
        return "ok edit_character".to_string();
    }

    format_change_result(
        "edit_character",
        vec![format!("updated={}", updated_fields.len())],
        Some(format!("fields: {}", updated_fields.join(", "))),
    )
}

fn format_list_world_book_entries_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data else {
        return "ok list_world_book_entries".to_string();
    };

    format_structured_result("list_world_book_entries", Vec::new(), data)
}

fn format_read_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data else {
        return "ok read_world_book_entry".to_string();
    };

    format_structured_result("read_world_book_entry", Vec::new(), data)
}

fn format_create_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data.and_then(as_object) else {
        return "ok create_world_book_entry".to_string();
    };

    let mut meta = Vec::new();
    if let Some(entry_id) = data.get("entry_id").and_then(|value| value.as_i64()) {
        meta.push(format!("entry_id={entry_id}"));
    }
    if let Some(name) = get_string(data, "entry_name") {
        meta.push(format!("name={name}"));
    }
    format_change_result(
        "create_world_book_entry",
        meta,
        Some(format_yaml_block(&serde_json::Value::Object(data.clone()))),
    )
}

fn format_update_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data.and_then(as_object) else {
        return "ok update_world_book_entry".to_string();
    };

    let mut meta = Vec::new();
    if let Some(matched_by) = get_string(data, "matched_by") {
        meta.push(format!("matched_by={matched_by}"));
    }
    if let Some(matched_value) = data.get("matched_value") {
        meta.push(format!("matched_value={}", value_to_inline(matched_value)));
    }
    if let Some(updated_fields) = data.get("updated_fields").and_then(value_as_string_list) {
        meta.push(format!("updated={}", updated_fields.join(", ")));
    }

    format_change_result(
        "update_world_book_entry",
        meta,
        Some(format_yaml_block(&serde_json::Value::Object(data.clone()))),
    )
}

fn format_delete_world_book_entry_result(data: Option<&serde_json::Value>) -> String {
    let Some(data) = data.and_then(as_object) else {
        return "ok delete_world_book_entry".to_string();
    };

    let mut meta = Vec::new();
    if let Some(matched_by) = get_string(data, "matched_by") {
        meta.push(format!("matched_by={matched_by}"));
    }
    if let Some(matched_value) = data.get("matched_value") {
        meta.push(format!("matched_value={}", value_to_inline(matched_value)));
    }
    format_change_result(
        "delete_world_book_entry",
        meta,
        Some(format_yaml_block(&serde_json::Value::Object(data.clone()))),
    )
}

fn format_generic_tool_result(tool_name: &str, data: Option<&serde_json::Value>) -> String {
    let mut lines = vec![format!("ok {tool_name}")];
    if let Some(data) = data {
        let rendered = render_value_lines(data);
        if !rendered.is_empty() {
            lines.extend(rendered);
        }
    }
    lines.join("\n")
}

fn format_patch_failure_details(details: &serde_json::Value) -> Vec<String> {
    let Some(details) = as_object(details) else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    if let Some(supported_fields) = details
        .get("supported_fields")
        .and_then(value_as_string_list)
    {
        lines.push(format!("supported_fields: {}", supported_fields.join(", ")));
    }
    if let Some(candidates) = details.get("candidates").and_then(|value| value.as_array()) {
        for (index, candidate) in candidates.iter().enumerate() {
            if let Some(candidate) = as_object(candidate) {
                let rendered = build_context_diff_lines(candidate, "matched_text", "=");
                if !rendered.is_empty() {
                    lines.push(format!("candidate {}:", index + 1));
                    lines.extend(rendered);
                }
            }
        }
    }
    if let Some(fragments) = details
        .get("fragment_matches")
        .and_then(|value| value.as_array())
    {
        for (index, fragment) in fragments.iter().enumerate() {
            if let Some(fragment) = as_object(fragment) {
                let line = compose_context_line(
                    get_string(fragment, "context_before")
                        .as_deref()
                        .unwrap_or(""),
                    get_string(fragment, "fragment").as_deref().unwrap_or(""),
                    get_string(fragment, "context_after")
                        .as_deref()
                        .unwrap_or(""),
                );
                lines.push(format!("fragment {}: {}", index + 1, line));
            }
        }
    }

    lines
}

fn build_context_diff_lines(
    context: &serde_json::Map<String, serde_json::Value>,
    focus_key: &str,
    prefix: &str,
) -> Vec<String> {
    let before = get_string(context, "context_before").unwrap_or_default();
    let focus = get_string(context, focus_key).unwrap_or_default();
    let after = get_string(context, "context_after").unwrap_or_default();
    let combined = compose_context_line(&before, &focus, &after);
    combined
        .lines()
        .take(6)
        .map(|line| format!("{prefix} {line}"))
        .collect()
}

fn compose_context_line(before: &str, focus: &str, after: &str) -> String {
    format!("{before}{focus}{after}")
        .replace('\r', "")
        .trim()
        .to_string()
}

fn render_value_lines(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {
            vec![value_to_inline(value)]
        }
        serde_json::Value::Array(items) => items
            .iter()
            .enumerate()
            .flat_map(|(index, item)| {
                let inline = value_to_inline(item);
                if inline.contains('\n') {
                    let mut lines = vec![format!("{}.", index + 1)];
                    lines.extend(
                        inline
                            .lines()
                            .map(|line| format!("  {line}"))
                            .collect::<Vec<_>>(),
                    );
                    lines
                } else {
                    vec![format!("{}. {inline}", index + 1)]
                }
            })
            .collect(),
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(key, value)| format!("{key}: {}", value_to_inline(value)))
            .collect(),
    }
}

fn value_to_inline(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .map(value_to_inline)
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(key, value)| format!("{key}={}", value_to_inline(value)))
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

fn value_as_string_list(value: &serde_json::Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items
            .iter()
            .map(value_to_inline)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    })
}

fn get_string(object: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
    object.get(key).and_then(|value| match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Null => None,
        _ => Some(value_to_inline(value)),
    })
}

fn as_object(value: &serde_json::Value) -> Option<&serde_json::Map<String, serde_json::Value>> {
    value.as_object()
}

fn format_fenced_block(language: &str, body: &str) -> String {
    format!("```{language}\n{body}\n```")
}

fn format_change_result(tool_name: &str, meta: Vec<String>, body: Option<String>) -> String {
    let mut lines = vec![format!("ok {tool_name}")];
    if !meta.is_empty() {
        lines.push(meta.join(" | "));
    }
    if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        lines.push(body);
    }
    lines.join("\n")
}

fn format_read_result(
    tool_name: &str,
    meta: Vec<String>,
    label: &str,
    text: Option<&str>,
) -> String {
    let mut lines = vec![format!("ok {tool_name}")];
    if !meta.is_empty() {
        lines.push(meta.join(" | "));
    }
    if let Some(text) = text.filter(|value| !value.trim().is_empty()) {
        lines.push(format_numbered_text_block(label, text));
    }
    lines.join("\n")
}

fn format_structured_result(
    tool_name: &str,
    meta: Vec<String>,
    value: &serde_json::Value,
) -> String {
    let mut lines = vec![format!("ok {tool_name}")];
    if !meta.is_empty() {
        lines.push(meta.join(" | "));
    }
    lines.push(format_yaml_block(value));
    lines.join("\n")
}

fn format_numbered_text_block(label: &str, text: &str) -> String {
    let numbered = text
        .lines()
        .enumerate()
        .map(|(index, line)| format!("{} {}", index + 1, line))
        .collect::<Vec<_>>()
        .join("\n");
    format_fenced_block(label, &numbered)
}

fn format_yaml_block(value: &serde_json::Value) -> String {
    let yaml =
        serde_yaml::to_string(value).unwrap_or_else(|_| render_value_lines(value).join("\n"));
    let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml).trim_end();
    format_fenced_block("yaml", yaml)
}
