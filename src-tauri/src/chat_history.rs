use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
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
        let app_dir = self.app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

        let character_dir = app_dir.join("character-cards").join(&self.character_id);

        // 确保目录存在
        fs::create_dir_all(&character_dir)
            .map_err(|e| format!("创建角色目录失败: {}", e))?;

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
                    .as_secs() as i64
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

        if file_path.is_err() || !file_path.as_ref().unwrap().exists() {
            return Ok(Vec::new());
        }

        let file_path = file_path.unwrap();
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("读取历史文件失败: {}", e))?;

        let lines: Vec<&str> = content.trim().split('\n').collect();
        let mut messages = Vec::new();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<ChatMessage>(line) {
                Ok(message) => messages.push(message),
                Err(e) => eprintln!("解析聊天记录行失败: {} - {}", line, e),
            }
        }

        Ok(messages)
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let file_path = self.get_history_file_path()?;

        if file_path.exists() {
            fs::write(&file_path, "")
                .map_err(|e| format!("清空历史文件失败: {}", e))?;
        }

        Ok(())
    }

    pub fn delete_message(&self, index: usize) -> Result<(), String> {
        let mut history = self.load_history()?;

        if index < history.len() {
            history.remove(index);
            self.save_history(&history)?;
        }

        Ok(())
    }

    pub fn update_message(&self, index: usize, new_message: &ChatMessage) -> Result<(), String> {
        let mut history = self.load_history()?;

        if index < history.len() {
            let mut updated_message = new_message.clone();
            updated_message.timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64
            );

            history[index] = updated_message;
            self.save_history(&history)?;
        }

        Ok(())
    }

    pub fn save_history(&self, history: &[ChatMessage]) -> Result<(), String> {
        let file_path = self.get_history_file_path()?;

        let content = history
            .iter()
            .map(|msg| serde_json::to_string(msg).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n") + "\n";

        fs::write(&file_path, content)
            .map_err(|e| format!("保存历史文件失败: {}", e))?;

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