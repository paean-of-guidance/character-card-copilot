use crate::chat_history::{ChatMessage, ChatHistoryManager};
use crate::character_storage::CharacterData;
use crate::events::{EventEmitter, CharacterUpdateType, SessionUnloadReason};
use crate::ai_chat::{AIChatService, ChatCompletionRequest, ChatMessage as AIChatMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use chrono::{DateTime, Utc};

/// Token 预算分配策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// 总限制：102400 (128k * 0.8)
    pub total_limit: usize,
    /// System 消息保留：15%
    pub system_reserved: usize,
    /// 角色核心信息保留：35%
    pub character_reserved: usize,
    /// 世界书条目保留：20%
    pub worldbook_reserved: usize,
    /// 聊天历史保留：30%
    pub history_reserved: usize,
}

impl Default for TokenBudget {
    fn default() -> Self {
        let total = 102400; // 128k * 0.8
        Self {
            total_limit: total,
            system_reserved: (total as f64 * 0.15) as usize,
            character_reserved: (total as f64 * 0.35) as usize,
            worldbook_reserved: (total as f64 * 0.20) as usize,
            history_reserved: (total as f64 * 0.30) as usize,
        }
    }
}

/// 上下文构建配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuilderOptions {
    /// Token 预算限制
    pub token_limit: usize,
    /// 是否启用智能裁剪
    pub enable_smart_truncation: bool,
    /// AI 角色定义（支持占位符）
    pub ai_role: String,
    /// AI 任务定义（支持占位符）
    pub ai_task: String,
    /// 是否优先保留聊天历史
    pub prioritize_chat_history: bool,
    /// 占位符替换映射
    pub placeholders: HashMap<String, String>,
}

impl Default for ContextBuilderOptions {
    fn default() -> Self {
        let mut placeholders = HashMap::new();
        placeholders.insert("{{ROLE}}".to_string(), "角色卡编写助手".to_string());
        placeholders.insert("{{TASK}}".to_string(), "帮助用户创作和完善角色设定".to_string());

        Self {
            token_limit: 102400,
            enable_smart_truncation: true,
            ai_role: "{{ROLE}}".to_string(),
            ai_task: "{{TASK}}".to_string(),
            prioritize_chat_history: true,
            placeholders,
        }
    }
}

/// CharacterSession - 后端状态管理的核心结构
#[derive(Debug, Clone)]
pub struct CharacterSession {
    /// 角色 UUID
    pub uuid: String,
    /// 角色数据
    pub character_data: CharacterData,
    /// 聊天历史记录
    pub chat_history: Vec<ChatMessage>,
    /// 上下文构建配置
    pub context_config: ContextBuilderOptions,
    /// Token 预算配置
    pub token_budget: TokenBudget,
    /// 上次上下文 Token 数量
    pub last_context_tokens: usize,
    /// 会话创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active: DateTime<Utc>,
    /// 会话状态
    pub status: SessionStatus,
}

/// 会话状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// 活跃状态
    Active,
    /// 暂停状态
    Paused,
    /// 加载中
    Loading,
    /// 错误状态
    Error(String),
}

impl CharacterSession {
    /// 创建新的角色会话
    pub fn new(uuid: String, character_data: CharacterData) -> Self {
        let now = Utc::now();
        Self {
            uuid,
            character_data,
            chat_history: Vec::new(),
            context_config: ContextBuilderOptions::default(),
            token_budget: TokenBudget::default(),
            last_context_tokens: 0,
            created_at: now,
            last_active: now,
            status: SessionStatus::Loading,
        }
    }

    /// 加载现有角色的会话
    pub fn load(app_handle: &AppHandle, uuid: String) -> Result<Self, String> {
        // 加载角色数据
        let character_data = crate::character_storage::CharacterStorage::get_character_by_uuid(app_handle, &uuid)?
            .ok_or_else(|| format!("角色 {} 不存在", uuid))?;

        // 加载聊天历史
        let history_manager = ChatHistoryManager::new(app_handle, &uuid);
        let chat_history = history_manager.load_history()?;

        let mut session = Self::new(uuid, character_data);
        session.chat_history = chat_history;
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
                    .as_secs() as i64
            ),
        };

        self.chat_history.push(message.clone());
        self.last_active = Utc::now();
        message
    }

    /// 添加 AI 响应消息到历史记录
    pub fn add_assistant_message(&mut self, content: String, tool_calls: Option<Vec<crate::chat_history::ToolCall>>) -> ChatMessage {
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
                    .as_secs() as i64
            ),
        };

        self.chat_history.push(message.clone());
        self.last_active = Utc::now();
        message
    }

    /// 保存聊天历史到文件
    pub async fn save_history(&self, app_handle: &AppHandle) -> Result<(), String> {
        let history_manager = ChatHistoryManager::new(app_handle, &self.uuid);

        // 保存所有消息（如果历史很长，可以考虑增量保存）
        for message in &self.chat_history {
            history_manager.save_message(message)?;
        }

        Ok(())
    }

    /// 获取最近的聊天消息
    pub fn get_recent_messages(&self, count: usize) -> Vec<ChatMessage> {
        let start = if self.chat_history.len() > count {
            self.chat_history.len() - count
        } else {
            0
        };
        self.chat_history[start..].to_vec()
    }

    /// 清空聊天历史
    pub fn clear_history(&mut self) {
        self.chat_history.clear();
        self.last_active = Utc::now();
    }

    /// 更新角色数据并发送事件
    pub fn update_character_data(&mut self, app: &AppHandle, character_data: CharacterData, update_type: CharacterUpdateType) -> Result<(), String> {
        self.character_data = character_data.clone();
        self.last_active = Utc::now();

        // 发送角色更新事件
        EventEmitter::send_character_updated(app, &self.uuid, &character_data, update_type)?;

        Ok(())
    }

    /// 更新角色数据（内部使用，不发送事件）
    pub fn update_character_data_internal(&mut self, character_data: CharacterData) {
        self.character_data = character_data;
        self.last_active = Utc::now();
    }

    /// 更新上下文配置
    pub fn update_context_config(&mut self, config: ContextBuilderOptions) {
        self.context_config = config;
        self.last_active = Utc::now();
    }

    /// 处理占位符替换
    pub fn process_placeholders(&self, template: &str) -> String {
        let mut result = template.to_string();

        // 替换基本占位符
        result = result.replace("{{ROLE}}", &self.context_config.placeholders.get("{{ROLE}}").unwrap_or(&"角色卡编写助手".to_string()));
        result = result.replace("{{TASK}}", &self.context_config.placeholders.get("{{TASK}}").unwrap_or(&"帮助用户创作和完善角色设定".to_string()));

        // 替换角色相关占位符
        result = result.replace("{{CHARACTER_NAME}}", &self.character_data.card.data.name);

        result
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

/// 会话信息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub uuid: String,
    pub character_name: Option<String>,
    pub message_count: usize,
    pub last_active: DateTime<Utc>,
    pub status: SessionStatus,
    pub last_context_tokens: usize,
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
    pub fn get_or_create_session(&self, app_handle: &AppHandle, uuid: String) -> Result<CharacterSession, String> {
        let mut sessions = self.sessions.lock()
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
        let mut sessions = self.sessions.lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        sessions.insert(session.uuid.clone(), session);
        Ok(())
    }

    /// 移除会话
    pub fn remove_session(&self, uuid: &str) -> Result<Option<CharacterSession>, String> {
        let mut sessions = self.sessions.lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        Ok(sessions.remove(uuid))
    }

    /// 获取所有活跃会话信息
    pub fn get_all_sessions_info(&self) -> Result<Vec<SessionInfo>, String> {
        let sessions = self.sessions.lock()
            .map_err(|e| format!("锁定会话失败: {}", e))?;

        Ok(sessions.values()
            .map(|session| session.get_session_info())
            .collect())
    }

    /// 清理旧的会话
    fn cleanup_old_sessions(&self, sessions: &mut HashMap<String, CharacterSession>) -> Result<(), String> {
        // 按最后活跃时间排序，移除最旧的会话
        if let Some((oldest_uuid, _)) = sessions.iter()
            .min_by_key(|(_, session)| session.last_active)
            .map(|(uuid, _)| (uuid.clone(), ())) {

            eprintln!("清理旧会话: {}", oldest_uuid);
            sessions.remove(&oldest_uuid);
        }

        Ok(())
    }

    /// 检查会话是否存在
    pub fn has_session(&self, uuid: &str) -> bool {
        let sessions = self.sessions.lock().ok();
        sessions.map(|s| s.contains_key(uuid)).unwrap_or(false)
    }

    /// 获取会话（如果存在）
    pub fn get_session(&self, uuid: &str) -> Option<CharacterSession> {
        let sessions = self.sessions.lock().ok()?;
        sessions.get(uuid).cloned()
    }
}

/// 全局会话管理器实例
lazy_static::lazy_static! {
    pub static ref SESSION_MANAGER: SessionManager = SessionManager::new(10); // 最多支持10个并发会话
}

impl SessionManager {
    /// 获取会话映射的内部引用（用于清理过期会话）
    pub fn get_sessions_map(&self) -> Result<std::sync::MutexGuard<HashMap<String, CharacterSession>>, String> {
        self.sessions.lock()
            .map_err(|e| format!("锁定会话失败: {}", e))
    }
}

// ====================== Tauri命令 ======================

/// 加载角色会话
#[tauri::command]
pub async fn load_character_session(app_handle: tauri::AppHandle, uuid: String) -> Result<SessionInfo, String> {
    let session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid)?;

    // 发送事件到前端
    let character_data = session.character_data.clone();
    let chat_history = session.chat_history.clone();

    // 发送角色加载事件
    EventEmitter::send_character_loaded(&app_handle, &session.uuid, &character_data)?;

    // 发送聊天历史加载事件
    EventEmitter::send_chat_history_loaded(&app_handle, &session.uuid, &chat_history)?;

    Ok(session.get_session_info())
}

/// 发送聊天消息
#[tauri::command]
pub async fn send_chat_message(
    app_handle: tauri::AppHandle,
    message: String,
) -> Result<(), String> {
    // 获取当前活跃角色会话
    let uuid = crate::character_state::get_active_character()
        .ok_or("没有活跃的角色会话")?;

    let mut session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid.clone())?;

    // 添加用户消息
    let user_message = session.add_user_message(message);

    // 发送用户消息事件
    EventEmitter::send_message_sent(&app_handle, &session.uuid, &user_message)?;

    // 使用上下文构建器构建完整上下文
    let context_builder = crate::context_builder::create_default_context_builder();
    let context_result = context_builder.build_full_context(
        &session.character_data,
        &session.chat_history,
        None, // 当前消息已添加到历史记录中
    ).map_err(|e| format!("构建上下文失败: {}", e))?;

    // 发送上下文构建完成事件
    EventEmitter::send_context_built(&app_handle, &session.uuid, &context_result)?;

    // 构建 AIChatMessage 格式的消息数组
    let mut ai_chat_messages = Vec::new();

    // 添加 System 消息
    for msg in context_result.system_messages {
        ai_chat_messages.push(AIChatMessage {
            role: crate::ai_chat::MessageRole::System,
            content: msg.content,
            name: msg.name,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // 添加 Assistant 消息（角色信息 + 世界书）
    for msg in context_result.assistant_messages {
        ai_chat_messages.push(AIChatMessage {
            role: crate::ai_chat::MessageRole::Assistant,
            content: msg.content,
            name: msg.name,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // 添加历史消息
    ai_chat_messages.extend(context_result.history_messages.iter().map(|msg| {
        let role = match msg.role.as_str() {
            "user" => crate::ai_chat::MessageRole::User,
            "assistant" => crate::ai_chat::MessageRole::Assistant,
            "system" => crate::ai_chat::MessageRole::System,
            _ => crate::ai_chat::MessageRole::User,
        };

        AIChatMessage {
            role,
            content: msg.content.clone(),
            name: msg.name.clone(),
            tool_calls: None, // 简化实现，暂时不转换tool_calls
            tool_call_id: msg.tool_call_id.clone(),
        }
    }));

    // 添加当前用户消息
    if let Some(current_msg) = context_result.current_user_message {
        ai_chat_messages.push(AIChatMessage {
            role: crate::ai_chat::MessageRole::User,
            content: current_msg.content,
            name: current_msg.name,
            tool_calls: None, // 暂时简化，不转换tool_calls
            tool_call_id: current_msg.tool_call_id,
        });
    }

    // 获取默认API配置
    use crate::api_config::ApiConfigService;
    use crate::api_config::ApiConfig;
    let api_config = ApiConfigService::get_default_api_config(&app_handle)?
        .ok_or("没有可用的API配置")?;

    // 记录消息数量
    let message_count = ai_chat_messages.len();

    // 构建聊天完成请求
    let request = ChatCompletionRequest {
        model: api_config.model.clone(),
        messages: ai_chat_messages,
        temperature: Some(0.7),
        max_tokens: Some(2048),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        stop: None,
        stream: Some(false),
        tools: None,
        tool_choice: None,
    };

    // 调用真实的AI服务
    let start_time = std::time::Instant::now();

    println!("发送AI请求到模型: {}", api_config.model);
    println!("消息数量: {}", message_count);

    // 调用 AIChatService 进行真实的AI API调用
    use crate::ai_chat::AIChatService;
    let ai_response_result = AIChatService::create_chat_completion(
        &api_config,
        &request,
        Some(&app_handle), // 传入 app_handle 以支持工具调用
    ).await
    .map_err(|e| format!("AI API调用失败: {}", e))?;

    let execution_time = start_time.elapsed().as_millis() as u64;

    // 提取AI响应内容
    let ai_content = ai_response_result.choices
        .first()
        .map(|choice| choice.message.content.clone())
        .unwrap_or_else(|| "AI未返回响应".to_string());

    // 提取工具调用（如果有）
    let tool_calls_data = ai_response_result.choices
        .first()
        .and_then(|choice| choice.message.tool_calls.clone());

    // 转换工具调用格式（从 ai_chat::ToolCallData 到 chat_history::ToolCall）
    let converted_tool_calls = tool_calls_data.as_ref().map(|calls| {
        calls.iter().map(|call| {
            crate::chat_history::ToolCall {
                id: call.id.clone(),
                r#type: call.call_type.clone(),
                function: crate::chat_history::ToolFunction {
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                },
            }
        }).collect::<Vec<_>>()
    });

    // 添加AI响应到历史记录
    let ai_response = session.add_assistant_message(
        ai_content.clone(),
        converted_tool_calls,
    );

    // 发送 AI 响应事件
    EventEmitter::send_message_received(&app_handle, &session.uuid, &ai_response)?;

    // 如果有工具调用，发送工具执行事件（工具已经在 ai_chat.rs 中自动执行）
    if let Some(tool_calls) = tool_calls_data {
        for tool_call in tool_calls {
            println!("工具已执行: {} (ID: {})", tool_call.function.name, tool_call.id);
            // 发送工具执行成功事件
            crate::events::EventEmitter::send_tool_executed(
                &app_handle,
                &session.uuid,
                &tool_call.function.name,
                true,
                Some(serde_json::json!({
                    "tool_call_id": tool_call.id,
                    "function_name": tool_call.function.name,
                    "arguments": tool_call.function.arguments,
                })),
                None,
                execution_time,
            )?;
        }
    }

    // 发送真实的Token统计事件
    let token_stats = crate::events::TokenUsageStats {
        prompt_tokens: ai_response_result.usage.prompt_tokens as usize,
        completion_tokens: ai_response_result.usage.completion_tokens as usize,
        total_tokens: ai_response_result.usage.total_tokens as usize,
        context_tokens: context_result.total_tokens,
        budget_utilization: (ai_response_result.usage.total_tokens as f64 / 102400.0 * 100.0), // 128k context * 0.8
    };

    EventEmitter::send_token_stats(&app_handle, &session.uuid, token_stats)?;

    // 发送整体完成进度
    EventEmitter::send_progress(
        &app_handle,
        &session.uuid,
        "ai_response",
        1.0,
        Some("AI响应完成"),
    )?;

    // 保存历史记录
    session.save_history(&app_handle).await
        .map_err(|e| format!("保存历史记录失败: {}", e))?;

    // 更新会话状态
    SESSION_MANAGER.update_session(session)?;

    Ok(())
}

/// 卸载角色会话
#[tauri::command]
pub async fn unload_character_session(app_handle: tauri::AppHandle, uuid: String) -> Result<(), String> {
    // 在卸载前保存历史记录
    if let Some(session) = SESSION_MANAGER.get_session(&uuid) {
        if let Err(e) = session.save_history(&app_handle).await {
            eprintln!("保存会话历史记录失败: {}", e);
        }
    }

    let removed_session = SESSION_MANAGER.remove_session(&uuid)?;

    if let Some(session) = removed_session {
        println!("会话 {} 已卸载", uuid);

        // 发送会话卸载事件
        let session_info = session.get_session_info();
        if let Err(e) = EventEmitter::send_session_unloaded(&app_handle, &uuid, &session_info, SessionUnloadReason::UserRequest) {
            eprintln!("发送会话卸载事件失败: {}", e);
        }
    }

    Ok(())
}

/// 获取会话信息
#[tauri::command]
pub async fn get_session_info(uuid: String) -> Result<SessionInfo, String> {
    let session = SESSION_MANAGER.get_session(&uuid)
        .ok_or_else(|| format!("会话 {} 不存在", uuid))?;

    Ok(session.get_session_info())
}

/// 获取所有活跃会话信息
#[tauri::command]
pub async fn get_all_sessions() -> Result<Vec<SessionInfo>, String> {
    SESSION_MANAGER.get_all_sessions_info()
}

/// 手动保存所有会话的历史记录
#[tauri::command]
pub async fn save_all_sessions(app_handle: tauri::AppHandle) -> Result<usize, String> {
    let sessions_info = SESSION_MANAGER.get_all_sessions_info()?;
    let mut saved_count = 0;

    for session_info in sessions_info {
        if let Some(session) = SESSION_MANAGER.get_session(&session_info.uuid) {
            match session.save_history(&app_handle).await {
                Ok(()) => saved_count += 1,
                Err(e) => eprintln!("保存会话 {} 历史记录失败: {}", session_info.uuid, e),
            }
        }
    }

    Ok(saved_count)
}

/// 清理过期会话（基于最后活跃时间）
#[tauri::command]
pub async fn cleanup_expired_sessions(max_age_hours: u64) -> Result<usize, String> {
    let mut sessions = SESSION_MANAGER.get_sessions_map()?;

    let now = chrono::Utc::now();
    let max_duration = chrono::Duration::hours(max_age_hours as i64);
    let mut removed_count = 0;

    let expired_sessions: Vec<String> = sessions.iter()
        .filter(|(_, session)| {
            now.signed_duration_since(session.last_active) > max_duration
        })
        .map(|(uuid, _)| uuid.clone())
        .collect();

    for uuid in expired_sessions {
        sessions.remove(&uuid);
        removed_count += 1;
        println!("清理过期会话: {}", uuid);
    }

    Ok(removed_count)
}