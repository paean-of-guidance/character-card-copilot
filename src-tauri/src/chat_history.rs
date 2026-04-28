use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolFunction,
    #[serde(default)]
    pub thought_signatures: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

fn parse_history_line(line: &str) -> Result<Option<ChatMessage>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    serde_json::from_str::<ChatMessage>(trimmed)
        .map(Some)
        .map_err(|e| format!("解析聊天记录行失败: {} - {}", trimmed, e))
}

#[cfg(test)]
mod tests {
    use super::{parse_history_line, ChatMessage, ToolCall, ToolFunction};

    #[test]
    fn legacy_history_line_defaults_new_reasoning_fields() {
        let line = r#"{
            "role":"assistant",
            "content":"legacy reply",
            "tool_calls":[
                {
                    "id":"call_1",
                    "type":"function",
                    "function":{"name":"search_web","arguments":"{\"q\":\"weather\"}"}
                }
            ],
            "timestamp":1710000000
        }"#;

        let message: ChatMessage =
            serde_json::from_str(line).expect("legacy history should deserialize");

        assert_eq!(message.role, "assistant");
        assert_eq!(message.content, "legacy reply");
        assert_eq!(message.timestamp, Some(1710000000));
        assert!(message.name.is_none());
        assert!(message.reasoning_content.is_none());

        let tool_calls = message
            .tool_calls
            .expect("legacy tool calls should remain available");
        assert_eq!(tool_calls.len(), 1);
        assert!(tool_calls[0].thought_signatures.is_none());
    }

    #[test]
    fn upgraded_history_line_round_trips_reasoning_metadata() {
        let message = ChatMessage {
            role: "assistant".to_string(),
            content: String::new(),
            name: None,
            reasoning_content: Some("Need tool use before final reply".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_2".to_string(),
                r#type: "function".to_string(),
                function: ToolFunction {
                    name: "lookup".to_string(),
                    arguments: "{\"query\":\"lore\"}".to_string(),
                },
                thought_signatures: Some(vec!["sig_1".to_string(), "sig_2".to_string()]),
            }]),
            tool_call_id: None,
            timestamp: Some(1710000001),
        };

        let serialized = serde_json::to_string(&message)
            .expect("upgraded history should serialize successfully");
        let restored: ChatMessage =
            serde_json::from_str(&serialized).expect("upgraded history should deserialize");

        assert_eq!(
            restored.reasoning_content.as_deref(),
            Some("Need tool use before final reply")
        );

        let tool_calls = restored.tool_calls.expect("tool calls should round-trip");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(
            tool_calls[0].thought_signatures.as_ref(),
            Some(&vec!["sig_1".to_string(), "sig_2".to_string()])
        );
    }

    #[test]
    fn blank_history_line_is_ignored() {
        assert!(parse_history_line("   ")
            .expect("blank line should parse")
            .is_none());
    }

    #[test]
    fn malformed_history_line_reports_error_without_panicking() {
        let error = parse_history_line("{bad json").expect_err("bad line should return error");
        assert!(error.contains("解析聊天记录行失败"));
    }
}

pub struct ChatHistoryManager {
    app_handle: AppHandle,
    character_id: String,
}

impl ChatHistoryManager {
    pub fn new(app_handle: &AppHandle, character_id: &str) -> Self {
        Self {
            app_handle: app_handle.clone(),
            character_id: character_id.to_string(),
        }
    }

    fn get_history_file_path(&self) -> Result<PathBuf, String> {
        let app_dir = self
            .app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

        let character_dir = app_dir.join("character-cards").join(&self.character_id);

        // 确保目录存在
        fs::create_dir_all(&character_dir).map_err(|e| format!("创建角色目录失败: {}", e))?;

        Ok(character_dir.join("chat_history.jsonl"))
    }

    pub fn save_message(&self, message: &ChatMessage) -> Result<(), String> {
        let file_path = self.get_history_file_path()?;

        let mut message_with_timestamp = message.clone();
        if message_with_timestamp.timestamp.is_none() {
            message_with_timestamp.timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            );
        }

        let line = serde_json::to_string(&message_with_timestamp)
            .map_err(|e| format!("序列化消息失败: {}", e))?;

        // 追加写入文件
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| format!("打开历史文件失败: {}", e))?
            .write_all((line + "\n").as_bytes())
            .map_err(|e| format!("写入历史文件失败: {}", e))?;

        Ok(())
    }

    pub fn load_history(&self) -> Result<Vec<ChatMessage>, String> {
        let file_path = self.get_history_file_path();

        let Ok(file_path) = file_path else {
            return Ok(Vec::new());
        };
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&file_path).map_err(|e| format!("读取历史文件失败: {}", e))?;
        let reader = BufReader::new(file);
        let mut messages = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("读取历史文件失败: {}", e))?;
            match parse_history_line(&line) {
                Ok(Some(message)) => messages.push(message),
                Ok(None) => {}
                Err(error) => eprintln!("{}", error),
            }
        }

        Ok(messages)
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let file_path = self.get_history_file_path()?;

        if file_path.exists() {
            fs::write(&file_path, "").map_err(|e| format!("清空历史文件失败: {}", e))?;
        }

        Ok(())
    }

    pub fn save_history(&self, history: &[ChatMessage]) -> Result<(), String> {
        let file_path = self.get_history_file_path()?;

        let content = history
            .iter()
            .map(|msg| serde_json::to_string(msg).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";

        fs::write(&file_path, content).map_err(|e| format!("保存历史文件失败: {}", e))?;

        Ok(())
    }

    pub fn get_last_message(&self) -> Result<Option<ChatMessage>, String> {
        let history = self.load_history()?;
        Ok(history.last().cloned())
    }

    pub fn get_recent_messages(&self, count: usize) -> Result<Vec<ChatMessage>, String> {
        let history = self.load_history()?;
        let start = if history.len() > count {
            history.len() - count
        } else {
            0
        };
        Ok(history[start..].to_vec())
    }
}
