use crate::character_storage::TavernCardV2;

pub const LONG_TEXT_FIELDS: &[(&str, &str)] = &[
    ("description", "角色描述"),
    ("personality", "性格特点"),
    ("scenario", "场景设定"),
    ("first_mes", "开场白"),
    ("mes_example", "对话示例"),
    ("creator_notes", "创作者笔记"),
    ("system_prompt", "系统提示词"),
    ("post_history_instructions", "历史后指令"),
];

pub fn long_text_field_names() -> Vec<String> {
    LONG_TEXT_FIELDS
        .iter()
        .map(|(field, _)| (*field).to_string())
        .collect()
}

pub fn get_long_text_field<'a>(card: &'a TavernCardV2, field: &str) -> Option<&'a str> {
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

pub fn set_long_text_field(
    card: &mut TavernCardV2,
    field: &str,
    value: String,
) -> Result<(), String> {
    match field {
        "description" => card.data.description = value,
        "personality" => card.data.personality = value,
        "scenario" => card.data.scenario = value,
        "first_mes" => card.data.first_mes = value,
        "mes_example" => card.data.mes_example = value,
        "creator_notes" => card.data.creator_notes = value,
        "system_prompt" => card.data.system_prompt = value,
        "post_history_instructions" => card.data.post_history_instructions = value,
        _ => return Err(format!("字段 '{}' 不支持", field)),
    }

    Ok(())
}

pub fn slice_by_chars(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

pub fn parse_alternate_greetings(value: &str) -> Vec<String> {
    value
        .split("<START_ALT>")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn parse_tags(value: &str) -> Vec<String> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{long_text_field_names, parse_alternate_greetings, parse_tags, slice_by_chars};

    #[test]
    fn long_text_fields_are_shared() {
        assert!(long_text_field_names().contains(&"description".to_string()));
        assert!(long_text_field_names().contains(&"post_history_instructions".to_string()));
    }

    #[test]
    fn slice_by_chars_respects_unicode_boundaries() {
        assert_eq!(slice_by_chars("a角色b", 1, 3), "角色");
    }

    #[test]
    fn parses_structured_list_fields() {
        assert_eq!(
            parse_alternate_greetings("<START_ALT>\nA\n<START_ALT>\nB"),
            vec!["A", "B"]
        );
        assert_eq!(parse_tags("a, b\nc"), vec!["a", "b", "c"]);
    }
}
