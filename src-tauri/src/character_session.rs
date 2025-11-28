use crate::backend::domain::{SessionInfo, SessionStatus};
use crate::character_storage::CharacterData;
use crate::chat_history::{ChatHistoryManager, ChatMessage};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

/// CharacterSession - 后端状态管理的核心结构
#[derive(Debug, Clone)]
pub struct CharacterSession {
    /// 角色 UUID
    pub uuid: String,
    /// 角色数据
    pub character_data: CharacterData,
    /// 聊天历史记录
    pub chat_history: Vec<ChatMessage>,
    /// 上次上下文 Token 数量
    pub last_context_tokens: usize,
    /// 最后活跃时间
    pub last_active: DateTime<Utc>,
    /// 会话状态
    pub status: SessionStatus,
    /// 已保存到磁盘的消息数量（用于增量保存）
    pub last_saved_index: usize,
}

impl CharacterSession {
    /// 创建新的角色会话
    pub fn new(uuid: String, character_data: CharacterData) -> Self {
        let now = Utc::now();
        Self {
            uuid,
            character_data,
            chat_history: Vec::new(),
            last_context_tokens: 0,
            last_active: now,
            status: SessionStatus::Loading,
            last_saved_index: 0,
        }
    }

    /// 加载现有角色的会话
    pub fn load(app_handle: &AppHandle, uuid: String) -> Result<Self, String> {
        // 加载角色数据
        let character_data =
            crate::character_storage::CharacterStorage::get_character_by_uuid(app_handle, &uuid)?
                .ok_or_else(|| format!("角色 {} 不存在", uuid))?;

        // 加载聊天历史
        let history_manager = ChatHistoryManager::new(app_handle, &uuid);
        let chat_history = history_manager.load_history()?;

        let mut session = Self::new(uuid, character_data);
        let history_len = chat_history.len();
        session.chat_history = chat_history;
        session.last_saved_index = history_len; // 已加载的历史已经在磁盘上
        session.status = SessionStatus::Active;
        session.last_active = Utc::now();

        Ok(session)
    }

    /// 添加用户消息到历史记录
    pub fn add_user_message(&mut self, content: String) -> ChatMessage {
        let message = ChatMessage {
            role: "user".to_string(),
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            timestamp: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
        };

        self.chat_history.push(message.clone());
        self.last_active = Utc::now();
        message
    }

    /// 添加 AI 响应消息到历史记录
    pub fn add_assistant_message(
        &mut self,
        content: String,
        tool_calls: Option<Vec<crate::chat_history::ToolCall>>,
    ) -> ChatMessage {
        let message = ChatMessage {
            role: "assistant".to_string(),
            content,
            name: None,
            tool_calls,
            tool_call_id: None,
            timestamp: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
        };

        self.chat_history.push(message.clone());
        self.last_active = Utc::now();
        message
    }

    /// 添加工具执行结果消息到历史记录
    pub fn add_tool_message(
        &mut self,
        content: String,
        tool_call_id: String,
        name: Option<String>,
    ) -> ChatMessage {
        let message = ChatMessage {
            role: "tool".to_string(),
            content,
            name,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            timestamp: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
        };

        self.chat_history.push(message.clone());
        self.last_active = Utc::now();
        message
    }

    /// 保存聊天历史到文件（增量保存）
    pub async fn save_history(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        let history_manager = ChatHistoryManager::new(app_handle, &self.uuid);

        // 只保存新增的消息（从 last_saved_index 开始）
        let unsaved_messages = &self.chat_history[self.last_saved_index..];

        for message in unsaved_messages {
            history_manager.save_message(message)?;
        }

        // 更新已保存的索引
        self.last_saved_index = self.chat_history.len();

        Ok(())
    }

    /// 完全重写历史文件（用于删除/编辑场景）
    pub async fn rewrite_all_history(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        let history_manager = ChatHistoryManager::new(app_handle, &self.uuid);

        // 使用 ChatHistoryManager 的 save_history 方法完全重写文件
        history_manager.save_history(&self.chat_history)?;

        // 更新已保存的索引
        self.last_saved_index = self.chat_history.len();

        Ok(())
    }

    /// 清空聊天历史
    pub fn clear_history(&mut self) {
        self.chat_history.clear();
        self.last_saved_index = 0; // 重置保存索引
        self.last_active = Utc::now();
    }

    /// 删除指定索引的消息
    pub fn delete_message(&mut self, index: usize) -> Result<ChatMessage, String> {
        if index >= self.chat_history.len() {
            return Err(format!(
                "消息索引 {} 超出范围（共 {} 条消息）",
                index,
                self.chat_history.len()
            ));
        }

        let removed = self.chat_history.remove(index);
        self.last_active = Utc::now();
        Ok(removed)
    }

    /// 编辑指定索引的消息内容
    pub fn edit_message(
        &mut self,
        index: usize,
        new_content: String,
    ) -> Result<ChatMessage, String> {
        if index >= self.chat_history.len() {
            return Err(format!(
                "消息索引 {} 超出范围（共 {} 条消息）",
                index,
                self.chat_history.len()
            ));
        }

        self.chat_history[index].content = new_content;
        self.last_active = Utc::now();
        Ok(self.chat_history[index].clone())
    }

    /// 删除最后一条消息（用于重新生成）
    pub fn delete_last_message(&mut self) -> Result<ChatMessage, String> {
        if self.chat_history.is_empty() {
            return Err("聊天历史为空，无法删除".to_string());
        }

        let removed = self.chat_history.pop().unwrap();
        self.last_active = Utc::now();
        Ok(removed)
    }

    /// 获取会话信息摘要
    pub fn get_session_info(&self) -> SessionInfo {
        SessionInfo {
            uuid: self.uuid.clone(),
            character_name: Some(self.character_data.card.data.name.clone()),
            message_count: self.chat_history.len(),
            last_active: self.last_active,
            status: self.status.clone(),
            last_context_tokens: self.last_context_tokens,
        }
    }
}

/// 全局会话管理器
pub struct SessionManager {
    /// 活跃的会话映射
    sessions: Arc<Mutex<HashMap<String, CharacterSession>>>,
    /// 最大活跃会话数
    max_sessions: usize,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions,
        }
    }

    /// 获取或创建角色会话
    pub fn get_or_create_session(
        &self,
        app_handle: &AppHandle,
        uuid: String,
    ) -> Result<CharacterSession, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        // 如果会话已存在，返回现有会话
        if let Some(session) = sessions.get(&uuid) {
            return Ok(session.clone());
        }

        // 检查会话数量限制
        if sessions.len() >= self.max_sessions {
            self.cleanup_old_sessions(&mut sessions)?;
        }

        // 创建新会话
        let session = CharacterSession::load(app_handle, uuid)?;
        sessions.insert(session.uuid.clone(), session.clone());

        Ok(session)
    }

    /// 更新会话
    pub fn update_session(&self, session: CharacterSession) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        sessions.insert(session.uuid.clone(), session);
        Ok(())
    }

    /// 移除会话
    pub fn remove_session(&self, uuid: &str) -> Result<Option<CharacterSession>, String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        Ok(sessions.remove(uuid))
    }

    /// 获取所有活跃会话信息
    pub fn get_all_sessions_info(&self) -> Result<Vec<SessionInfo>, String> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        Ok(sessions
            .values()
            .map(|session| session.get_session_info())
            .collect())
    }

    /// 清理旧的会话
    fn cleanup_old_sessions(
        &self,
        sessions: &mut HashMap<String, CharacterSession>,
    ) -> Result<(), String> {
        // 按最后活跃时间排序，移除最旧的会话
        if let Some((oldest_uuid, _)) = sessions
            .iter()
            .min_by_key(|(_, session)| session.last_active)
            .map(|(uuid, _)| (uuid.clone(), ()))
        {
            eprintln!("清理旧会话: {}", oldest_uuid);
            sessions.remove(&oldest_uuid);
        }

        Ok(())
    }

    /// 获取会话（如果存在）
    pub fn get_session(&self, uuid: &str) -> Option<CharacterSession> {
        let sessions = self.sessions.lock().ok()?;
        sessions.get(uuid).cloned()
    }
}

// 全局会话管理器实例
lazy_static::lazy_static! {
    pub static ref SESSION_MANAGER: SessionManager = SessionManager::new(10); // 最多支持10个并发会话
}

impl SessionManager {
    /// 获取会话映射的内部引用（用于清理过期会话）
    pub fn get_sessions_map(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, CharacterSession>>, String> {
        self.sessions
            .lock()
            .map_err(|e| format!("锁定会话失败: {}", e))
    }
}

