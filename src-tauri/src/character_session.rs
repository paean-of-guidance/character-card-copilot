use crate::ai_chat::{ChatCompletionRequest, ChatMessage as AIChatMessage};
use crate::character_storage::CharacterData;
use crate::chat_history::{ChatHistoryManager, ChatMessage};
use crate::events::{EventEmitter, SessionUnloadReason};
use crate::tools::ToolRegistry;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

pub use crate::backend::domain::sessions::config::{ContextBuilderOptions, TokenBudget};
pub use crate::backend::domain::sessions::session::{SessionInfo, SessionStatus};

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
    async fn rewrite_all_history(&mut self, app_handle: &AppHandle) -> Result<(), String> {
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

// ====================== Tauri命令 ======================

/// 加载角色会话
#[tauri::command]
pub async fn load_character_session(
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<SessionInfo, String> {
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

/// 内部函数：生成AI回复（公共逻辑）
///
/// 参数：
/// - app_handle: Tauri应用句柄
/// - session: 可变的角色会话引用
/// - operation_type: 操作类型（用于日志和进度事件），如 "chat", "regenerate", "continue"
async fn generate_ai_response(
    app_handle: &tauri::AppHandle,
    session: &mut CharacterSession,
    operation_type: &str,
) -> Result<(), String> {
    // 使用上下文构建器构建完整上下文
    let context_builder = crate::context_builder::create_default_context_builder();
    let context_result = context_builder
        .build_full_context(
            &session.character_data,
            &session.chat_history,
            None, // 当前消息已添加到历史记录中
        )
        .map_err(|e| format!("构建上下文失败: {}", e))?;

    // 发送上下文构建完成事件
    EventEmitter::send_context_built(app_handle, &session.uuid, &context_result)?;

    // ==================== 按照标准顺序构建消息 ====================
    // 1️⃣ System / Role Prompt （定义模型身份、语气、核心目标）
    // 2️⃣ Task / Objective      （本次会话目标、任务说明）
    // 3️⃣ Character_Information （角色卡：背景、性格、偏好、知识、记忆）
    // 4️⃣ History               （过去的 user / assistant 对话）
    // 5️⃣ User Reply            （当前用户输入）
    // 注：Tools 通过 request.tools 参数传递，不放在消息中
    let mut ai_chat_messages = Vec::new();

    // 1️⃣ System / Role Prompt + 2️⃣ Task / Objective
    for msg in context_result.system_messages {
        ai_chat_messages.push(AIChatMessage {
            role: crate::ai_chat::MessageRole::System,
            content: msg.content,
            name: msg.name,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // 3️⃣ Character_Information（角色信息 + 世界书）
    // 使用 System 角色而非 Assistant，避免破坏对话时间线
    for msg in context_result.assistant_messages {
        ai_chat_messages.push(AIChatMessage {
            role: crate::ai_chat::MessageRole::System,
            content: msg.content,
            name: msg.name,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // 4️⃣ History（历史对话）
    ai_chat_messages.extend(context_result.history_messages.iter().map(|msg| {
        let role = match msg.role.as_str() {
            "user" => crate::ai_chat::MessageRole::User,
            "assistant" => crate::ai_chat::MessageRole::Assistant,
            "system" => crate::ai_chat::MessageRole::System,
            "tool" => crate::ai_chat::MessageRole::Tool,
            _ => crate::ai_chat::MessageRole::User,
        };

        // 转换 tool_calls（如果存在）
        let converted_tool_calls = msg.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|tc| crate::ai_chat::ToolCallData {
                    id: tc.id.clone(),
                    call_type: tc.r#type.clone(),
                    function: crate::ai_chat::ToolCallFunctionData {
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    },
                })
                .collect()
        });

        AIChatMessage {
            role,
            content: msg.content.clone(),
            name: msg.name.clone(),
            tool_calls: converted_tool_calls,
            tool_call_id: msg.tool_call_id.clone(),
        }
    }));

    // 5️⃣ User Reply / Current Input（当前用户输入）
    if let Some(current_msg) = context_result.current_user_message {
        ai_chat_messages.push(AIChatMessage {
            role: crate::ai_chat::MessageRole::User,
            content: current_msg.content,
            name: current_msg.name,
            tool_calls: None,
            tool_call_id: current_msg.tool_call_id,
        });
    }

    // 获取默认API配置
    use crate::api_config::ApiConfigService;
    let api_config =
        ApiConfigService::get_default_api_config(app_handle)?.ok_or("没有可用的API配置")?;

    // 获取可用工具定义
    let chat_tools = ToolRegistry::get_available_tools_global();

    // 🔧 临时禁用工具进行调试
    // 某些模型（如 GLM-4.6）或某些 API 端点可能不支持 function calling
    // 设置为 true 可以暂时禁用工具，测试基础对话是否正常
    let disable_tools_for_debug = false;

    // ===== 调试信息打印（在移动 ai_chat_messages 之前） =====
    println!("=== AI 请求调试信息 ===");
    println!("模型: {}", api_config.model);
    println!("API端点: {}", api_config.endpoint);
    println!("消息数量: {}", ai_chat_messages.len());
    println!("工具数量: {}", chat_tools.len());
    if disable_tools_for_debug {
        println!("⚠️ 工具已临时禁用（调试模式）");
    }

    // 打印消息详情
    for (idx, msg) in ai_chat_messages.iter().enumerate() {
        let role_str = match msg.role {
            crate::ai_chat::MessageRole::System => "system",
            crate::ai_chat::MessageRole::User => "user",
            crate::ai_chat::MessageRole::Assistant => "assistant",
            crate::ai_chat::MessageRole::Tool => "tool",
        };
        println!(
            "消息[{}] role={}, content_len={}, has_tool_calls={}, tool_call_id={:?}",
            idx,
            role_str,
            msg.content.len(),
            msg.tool_calls.is_some(),
            msg.tool_call_id
        );
        if msg.content.is_empty() && msg.tool_calls.is_none() {
            println!("⚠️ 警告: 消息[{}]内容为空且没有tool_calls", idx);
        }
    }
    println!("=====================");

    // 构建聊天完成请求（移动 ai_chat_messages）
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
        tools: if disable_tools_for_debug {
            None
        } else {
            Some(chat_tools)
        },
        tool_choice: if disable_tools_for_debug {
            None
        } else {
            Some(crate::ai_chat::ToolChoice::String("auto".to_string()))
        },
    };

    // 调用真实的AI服务
    let start_time = std::time::Instant::now();

    // 调用 AIChatService 进行真实的AI API调用
    use crate::ai_chat::AIChatService;
    let ai_response_result = AIChatService::create_chat_completion(
        &api_config,
        &request,
        Some(app_handle), // 传入 app_handle 以支持工具调用
    )
    .await
    .map_err(|e| {
        eprintln!("❌ API调用失败详情: {}", e);
        format!("AI API调用失败: {}", e)
    })?;

    let _execution_time = start_time.elapsed().as_millis() as u64;

    // 提取AI响应内容
    let ai_content = ai_response_result
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .unwrap_or_else(|| "AI未返回响应".to_string());

    // 提取工具调用（如果有）
    let tool_calls_data = ai_response_result
        .choices
        .first()
        .and_then(|choice| choice.message.tool_calls.clone());

    // 转换工具调用格式（从 ai_chat::ToolCallData 到 chat_history::ToolCall）
    let converted_tool_calls = tool_calls_data.as_ref().map(|calls| {
        calls
            .iter()
            .map(|call| crate::chat_history::ToolCall {
                id: call.id.clone(),
                r#type: call.call_type.clone(),
                function: crate::chat_history::ToolFunction {
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                },
            })
            .collect::<Vec<_>>()
    });

    // 处理中间消息（工具调用和工具结果）
    if let Some(intermediate_msgs) = &ai_response_result.intermediate_messages {
        for msg in intermediate_msgs {
            match msg.role {
                crate::ai_chat::MessageRole::Assistant => {
                    // 保存带 tool_calls 的 assistant 消息
                    if msg.tool_calls.is_some() {
                        let converted_calls = msg.tool_calls.as_ref().map(|calls| {
                            calls
                                .iter()
                                .map(|call| crate::chat_history::ToolCall {
                                    id: call.id.clone(),
                                    r#type: call.call_type.clone(),
                                    function: crate::chat_history::ToolFunction {
                                        name: call.function.name.clone(),
                                        arguments: call.function.arguments.clone(),
                                    },
                                })
                                .collect::<Vec<_>>()
                        });
                        session.add_assistant_message(msg.content.clone(), converted_calls);
                    }
                }
                crate::ai_chat::MessageRole::Tool => {
                    // 保存工具结果
                    if let Some(tool_call_id) = &msg.tool_call_id {
                        session.add_tool_message(
                            msg.content.clone(),
                            tool_call_id.clone(),
                            msg.name.clone(),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // 添加最终AI响应到历史记录，并附带本次响应的工具调用
    let ai_response = session.add_assistant_message(ai_content.clone(), converted_tool_calls);

    // 转换中间消息为 ChatMessage 格式
    let converted_intermediate_msgs =
        ai_response_result
            .intermediate_messages
            .as_ref()
            .map(|msgs| {
                msgs.iter()
                    .map(|msg| crate::chat_history::ChatMessage {
                        role: match msg.role {
                            crate::ai_chat::MessageRole::User => "user".to_string(),
                            crate::ai_chat::MessageRole::Assistant => "assistant".to_string(),
                            crate::ai_chat::MessageRole::System => "system".to_string(),
                            crate::ai_chat::MessageRole::Tool => "tool".to_string(),
                        },
                        content: msg.content.clone(),
                        timestamp: Some(chrono::Utc::now().timestamp_millis()),
                        tool_calls: msg.tool_calls.as_ref().map(|calls| {
                            calls
                                .iter()
                                .map(|call| crate::chat_history::ToolCall {
                                    id: call.id.clone(),
                                    r#type: call.call_type.clone(),
                                    function: crate::chat_history::ToolFunction {
                                        name: call.function.name.clone(),
                                        arguments: call.function.arguments.clone(),
                                    },
                                })
                                .collect()
                        }),
                        tool_call_id: msg.tool_call_id.clone(),
                        name: msg.name.clone(),
                    })
                    .collect()
            });

    // 发送 AI 响应事件（包含中间消息）
    EventEmitter::send_message_received(
        app_handle,
        &session.uuid,
        &ai_response,
        converted_intermediate_msgs,
    )?;

    // 注：工具执行事件已在 ai_chat.rs 中的工具执行时发送，无需在此重复发送

    // 发送真实的Token统计事件
    let token_stats = crate::events::TokenUsageStats {
        prompt_tokens: ai_response_result.usage.prompt_tokens as usize,
        completion_tokens: ai_response_result.usage.completion_tokens as usize,
        total_tokens: ai_response_result.usage.total_tokens as usize,
        context_tokens: context_result.total_tokens,
        budget_utilization: (ai_response_result.usage.total_tokens as f64 / 102400.0 * 100.0), // 128k context * 0.8
    };

    EventEmitter::send_token_stats(app_handle, &session.uuid, token_stats)?;

    // 发送整体完成进度
    EventEmitter::send_progress(
        app_handle,
        &session.uuid,
        operation_type,
        1.0,
        Some(&format!("{}操作完成", operation_type)),
    )?;

    // 保存历史记录
    session
        .save_history(app_handle)
        .await
        .map_err(|e| format!("保存历史记录失败: {}", e))?;

    // 更新会话状态
    SESSION_MANAGER.update_session(session.clone())?;

    Ok(())
}

/// 发送聊天消息
#[tauri::command]
pub async fn send_chat_message(
    app_handle: tauri::AppHandle,
    message: String,
) -> Result<(), String> {
    // 获取当前活跃角色会话
    let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

    let mut session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid.clone())?;

    // 添加用户消息
    let user_message = session.add_user_message(message);

    // 发送用户消息事件
    EventEmitter::send_message_sent(&app_handle, &session.uuid, &user_message)?;

    // 先保存用户消息，避免后续 AI 回复失败导致历史不同步
    session
        .save_history(&app_handle)
        .await
        .map_err(|e| format!("保存用户消息失败: {}", e))?;

    // 保存成功后更新会话状态（更新 last_saved_index）
    SESSION_MANAGER.update_session(session.clone())?;

    // 调用公共的AI生成逻辑
    generate_ai_response(&app_handle, &mut session, "chat").await
}

/// 卸载角色会话
#[tauri::command]
pub async fn unload_character_session(
    app_handle: tauri::AppHandle,
    uuid: String,
) -> Result<(), String> {
    // 在卸载前保存历史记录
    if let Some(mut session) = SESSION_MANAGER.get_session(&uuid) {
        if let Err(e) = session.save_history(&app_handle).await {
            eprintln!("保存会话历史记录失败: {}", e);
        } else {
            // 保存成功后更新会话（更新 last_saved_index）
            let _ = SESSION_MANAGER.update_session(session);
        }
    }

    let removed_session = SESSION_MANAGER.remove_session(&uuid)?;

    if let Some(session) = removed_session {
        println!("会话 {} 已卸载", uuid);

        // 发送会话卸载事件
        let session_info = session.get_session_info();
        if let Err(e) = EventEmitter::send_session_unloaded(
            &app_handle,
            &uuid,
            &session_info,
            SessionUnloadReason::UserRequest,
        ) {
            eprintln!("发送会话卸载事件失败: {}", e);
        }
    }

    Ok(())
}

/// 获取会话信息
#[tauri::command]
pub async fn get_session_info(uuid: String) -> Result<SessionInfo, String> {
    let session = SESSION_MANAGER
        .get_session(&uuid)
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
        if let Some(mut session) = SESSION_MANAGER.get_session(&session_info.uuid) {
            match session.save_history(&app_handle).await {
                Ok(()) => {
                    saved_count += 1;
                    // 保存成功后更新会话（更新 last_saved_index）
                    let _ = SESSION_MANAGER.update_session(session);
                }
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

    let expired_sessions: Vec<String> = sessions
        .iter()
        .filter(|(_, session)| now.signed_duration_since(session.last_active) > max_duration)
        .map(|(uuid, _)| uuid.clone())
        .collect();

    for uuid in expired_sessions {
        sessions.remove(&uuid);
        removed_count += 1;
        println!("清理过期会话: {}", uuid);
    }

    Ok(removed_count)
}

/// 删除指定索引的消息
#[tauri::command]
pub async fn delete_chat_message(app_handle: tauri::AppHandle, index: usize) -> Result<(), String> {
    // 获取当前活跃角色会话
    let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

    let mut session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid.clone())?;

    // 删除消息
    let deleted_message = session.delete_message(index)?;

    // 删除后需要完全重写历史文件
    session.rewrite_all_history(&app_handle).await?;

    // 更新会话
    SESSION_MANAGER.update_session(session)?;

    println!("删除消息 [{}]: {:?}", index, deleted_message.content);

    Ok(())
}

/// 编辑指定索引的消息
#[tauri::command]
pub async fn edit_chat_message(
    app_handle: tauri::AppHandle,
    index: usize,
    new_content: String,
) -> Result<(), String> {
    // 获取当前活跃角色会话
    let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

    let mut session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid.clone())?;

    // 编辑消息
    let edited_message = session.edit_message(index, new_content)?;

    // 编辑后需要完全重写历史文件
    session.rewrite_all_history(&app_handle).await?;

    // 更新会话
    SESSION_MANAGER.update_session(session)?;

    println!("编辑消息 [{}]: {:?}", index, edited_message.content);

    Ok(())
}

/// 重新生成最后一条AI回复
#[tauri::command]
pub async fn regenerate_last_message(app_handle: tauri::AppHandle) -> Result<(), String> {
    // 获取当前活跃角色会话
    let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

    let mut session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid.clone())?;

    // 检查历史记录是否为空
    if session.chat_history.is_empty() {
        return Err("聊天历史为空，无法重新生成".to_string());
    }

    // 检查最后一条消息是否是AI回复
    let last_message = session.chat_history.last().ok_or("聊天历史为空")?;
    if last_message.role != "assistant" {
        return Err("最后一条消息不是AI回复，无法重新生成".to_string());
    }

    // 删除最后一条AI回复
    session.delete_last_message()?;

    // 删除后需要完全重写历史文件
    session.rewrite_all_history(&app_handle).await?;

    // 获取倒数第二条消息（应该是用户消息）
    let user_message = session
        .chat_history
        .last()
        .ok_or("没有用户消息，无法重新生成")?;

    if user_message.role != "user" {
        return Err("倒数第二条消息不是用户消息，无法重新生成".to_string());
    }

    println!("重新生成消息，基于用户消息: {:?}", user_message.content);

    // 更新会话（删除消息后）
    SESSION_MANAGER.update_session(session.clone())?;

    // 调用公共的AI生成逻辑
    generate_ai_response(&app_handle, &mut session, "regenerate").await
}

/// 继续对话（当最后一条是用户消息时生成AI回复）
#[tauri::command]
pub async fn continue_chat(app_handle: tauri::AppHandle) -> Result<(), String> {
    // 获取当前活跃角色会话
    let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

    let mut session = SESSION_MANAGER.get_or_create_session(&app_handle, uuid.clone())?;

    // 检查历史记录是否为空
    if session.chat_history.is_empty() {
        return Err("聊天历史为空，无法继续对话".to_string());
    }

    // 检查最后一条消息是否是用户消息
    let last_message = session.chat_history.last().ok_or("聊天历史为空")?;
    if last_message.role != "user" {
        return Err("最后一条消息不是用户消息，无法继续对话".to_string());
    }

    println!("继续对话，基于最后一条用户消息: {:?}", last_message.content);

    // 调用公共的AI生成逻辑
    generate_ai_response(&app_handle, &mut session, "continue").await
}
