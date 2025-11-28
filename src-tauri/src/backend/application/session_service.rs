use crate::backend::application::event_bus::EventBus;
use crate::backend::domain::sessions::session::SessionInfo;
use crate::character_session::{CharacterSession, SESSION_MANAGER};
use crate::events::SessionUnloadReason;
use crate::tools::ToolRegistry;
use tauri::AppHandle;

pub struct SessionService;

impl SessionService {
    pub async fn load_session(
        app_handle: &AppHandle,
        uuid: String,
    ) -> Result<SessionInfo, String> {
        let session = SESSION_MANAGER.get_or_create_session(app_handle, uuid)?;

        let character_data = session.character_data.clone();
        let chat_history = session.chat_history.clone();

        EventBus::character_loaded(app_handle, &session.uuid, &character_data)?;
        EventBus::chat_history_loaded(app_handle, &session.uuid, &chat_history)?;

        Ok(session.get_session_info())
    }

    pub async fn send_chat_message(
        app_handle: &AppHandle,
        message: String,
    ) -> Result<(), String> {
        let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

        let mut session = SESSION_MANAGER.get_or_create_session(app_handle, uuid.clone())?;

        let user_message = session.add_user_message(message);

        EventBus::message_sent(app_handle, &session.uuid, &user_message)?;

        session
            .save_history(app_handle)
            .await
            .map_err(|e| format!("保存用户消息失败: {}", e))?;

        SESSION_MANAGER.update_session(session.clone())?;

        Self::generate_ai_response(app_handle, &mut session, "chat").await
    }

    pub async fn unload_session(
        app_handle: &AppHandle,
        uuid: String,
    ) -> Result<(), String> {
        if let Some(mut session) = SESSION_MANAGER.get_session(&uuid) {
            if let Err(e) = session.save_history(app_handle).await {
                eprintln!("保存会话历史记录失败: {}", e);
            } else {
                let _ = SESSION_MANAGER.update_session(session);
            }
        }

        let removed_session = SESSION_MANAGER.remove_session(&uuid)?;

        if let Some(session) = removed_session {
            println!("会话 {} 已卸载", uuid);

            let session_info = session.get_session_info();
            if let Err(e) = EventBus::session_unloaded(
                app_handle,
                &uuid,
                &session_info,
                SessionUnloadReason::UserRequest,
            ) {
                eprintln!("发送会话卸载事件失败: {}", e);
            }
        }

        Ok(())
    }

    pub fn get_session_info(uuid: String) -> Result<SessionInfo, String> {
        let session = SESSION_MANAGER
            .get_session(&uuid)
            .ok_or_else(|| format!("会话 {} 不存在", uuid))?;

        Ok(session.get_session_info())
    }

    pub fn get_all_sessions() -> Result<Vec<SessionInfo>, String> {
        SESSION_MANAGER.get_all_sessions_info()
    }

    pub async fn save_all_sessions(app_handle: &AppHandle) -> Result<usize, String> {
        let sessions_info = SESSION_MANAGER.get_all_sessions_info()?;
        let mut saved_count = 0;

        for session_info in sessions_info {
            if let Some(mut session) = SESSION_MANAGER.get_session(&session_info.uuid) {
                match session.save_history(app_handle).await {
                    Ok(()) => {
                        saved_count += 1;
                        let _ = SESSION_MANAGER.update_session(session);
                    }
                    Err(e) => eprintln!("保存会话 {} 历史记录失败: {}", session_info.uuid, e),
                }
            }
        }

        Ok(saved_count)
    }

    pub fn cleanup_expired_sessions(max_age_hours: u64) -> Result<usize, String> {
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

    pub async fn delete_chat_message(
        app_handle: &AppHandle,
        index: usize,
    ) -> Result<(), String> {
        let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

        let mut session = SESSION_MANAGER.get_or_create_session(app_handle, uuid.clone())?;

        let deleted_message = session.delete_message(index)?;

        session.rewrite_all_history(app_handle).await?;

        SESSION_MANAGER.update_session(session)?;

        println!("删除消息 [{}]: {:?}", index, deleted_message.content);

        Ok(())
    }

    pub async fn edit_chat_message(
        app_handle: &AppHandle,
        index: usize,
        new_content: String,
    ) -> Result<(), String> {
        let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

        let mut session = SESSION_MANAGER.get_or_create_session(app_handle, uuid.clone())?;

        let edited_message = session.edit_message(index, new_content)?;

        session.rewrite_all_history(app_handle).await?;

        SESSION_MANAGER.update_session(session)?;

        println!("编辑消息 [{}]: {:?}", index, edited_message.content);

        Ok(())
    }

    pub async fn regenerate_last_message(
        app_handle: &AppHandle,
    ) -> Result<(), String> {
        let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

        let mut session = SESSION_MANAGER.get_or_create_session(app_handle, uuid.clone())?;

        if session.chat_history.is_empty() {
            return Err("聊天历史为空，无法重新生成".to_string());
        }

        let last_message = session.chat_history.last().ok_or("聊天历史为空")?;
        if last_message.role != "assistant" {
            return Err("最后一条消息不是AI回复，无法重新生成".to_string());
        }

        session.delete_last_message()?;

        session.rewrite_all_history(app_handle).await?;

        let user_message = session
            .chat_history
            .last()
            .ok_or("没有用户消息，无法重新生成")?;

        if user_message.role != "user" {
            return Err("倒数第二条消息不是用户消息，无法重新生成".to_string());
        }

        println!("重新生成消息，基于用户消息: {:?}", user_message.content);

        SESSION_MANAGER.update_session(session.clone())?;

        Self::generate_ai_response(app_handle, &mut session, "regenerate").await
    }

    pub async fn continue_chat(app_handle: &AppHandle) -> Result<(), String> {
        let uuid = crate::character_state::get_active_character().ok_or("没有活跃的角色会话")?;

        let mut session = SESSION_MANAGER.get_or_create_session(app_handle, uuid.clone())?;

        if session.chat_history.is_empty() {
            return Err("聊天历史为空，无法继续对话".to_string());
        }

        let last_message = session.chat_history.last().ok_or("聊天历史为空")?;
        if last_message.role != "user" {
            return Err("最后一条消息不是用户消息，无法继续对话".to_string());
        }

        println!("继续对话，基于最后一条用户消息: {:?}", last_message.content);

        Self::generate_ai_response(app_handle, &mut session, "continue").await
    }

    async fn generate_ai_response(
        app_handle: &AppHandle,
        session: &mut CharacterSession,
        operation_type: &str,
    ) -> Result<(), String> {
        let context_builder = crate::context_builder::create_default_context_builder();
        let context_result = context_builder
            .build_full_context(
                &session.character_data,
                &session.chat_history,
                None,
            )
            .map_err(|e| format!("构建上下文失败: {}", e))?;

        EventBus::context_built(app_handle, &session.uuid, &context_result)?;

        let mut ai_chat_messages = Vec::new();

        for msg in context_result.system_messages {
            ai_chat_messages.push(crate::ai_chat::ChatMessage {
                role: crate::ai_chat::MessageRole::System,
                content: msg.content,
                name: msg.name,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        for msg in context_result.assistant_messages {
            ai_chat_messages.push(crate::ai_chat::ChatMessage {
                role: crate::ai_chat::MessageRole::System,
                content: msg.content,
                name: msg.name,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        ai_chat_messages.extend(context_result.history_messages.iter().map(|msg| {
            let role = match msg.role.as_str() {
                "user" => crate::ai_chat::MessageRole::User,
                "assistant" => crate::ai_chat::MessageRole::Assistant,
                "system" => crate::ai_chat::MessageRole::System,
                "tool" => crate::ai_chat::MessageRole::Tool,
                _ => crate::ai_chat::MessageRole::User,
            };

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

            crate::ai_chat::ChatMessage {
                role,
                content: msg.content.clone(),
                name: msg.name.clone(),
                tool_calls: converted_tool_calls,
                tool_call_id: msg.tool_call_id.clone(),
            }
        }));

        if let Some(current_msg) = context_result.current_user_message {
            ai_chat_messages.push(crate::ai_chat::ChatMessage {
                role: crate::ai_chat::MessageRole::User,
                content: current_msg.content,
                name: current_msg.name,
                tool_calls: None,
                tool_call_id: current_msg.tool_call_id,
            });
        }

        let api_config =
            crate::api_config::ApiConfigService::get_default_api_config(app_handle)?
                .ok_or("没有可用的API配置")?;

        let chat_tools = ToolRegistry::get_available_tools_global();

        let disable_tools_for_debug = false;

        println!("=== AI 请求调试信息 ===");
        println!("模型: {}", api_config.model);
        println!("API端点: {}", api_config.endpoint);
        println!("消息数量: {}", ai_chat_messages.len());
        println!("工具数量: {}", chat_tools.len());
        if disable_tools_for_debug {
            println!("⚠️ 工具已临时禁用（调试模式）");
        }

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

        let request = crate::ai_chat::ChatCompletionRequest {
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

        let start_time = std::time::Instant::now();

        let ai_response_result = crate::ai_chat::AIChatService::create_chat_completion(
            &api_config,
            &request,
            Some(app_handle),
        )
        .await
        .map_err(|e| {
            eprintln!("❌ API调用失败详情: {}", e);
            format!("AI API调用失败: {}", e)
        })?;

        let _execution_time = start_time.elapsed().as_millis() as u64;

        let ai_content = ai_response_result
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "AI未返回响应".to_string());

        let tool_calls_data = ai_response_result
            .choices
            .first()
            .and_then(|choice| choice.message.tool_calls.clone());

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

        if let Some(intermediate_msgs) = &ai_response_result.intermediate_messages {
            for msg in intermediate_msgs {
                match msg.role {
                    crate::ai_chat::MessageRole::Assistant => {
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

        let ai_response = session.add_assistant_message(ai_content.clone(), converted_tool_calls);

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

        EventBus::message_received(
            app_handle,
            &session.uuid,
            &ai_response,
            converted_intermediate_msgs,
        )?;

        let token_stats = crate::events::TokenUsageStats {
            prompt_tokens: ai_response_result.usage.prompt_tokens as usize,
            completion_tokens: ai_response_result.usage.completion_tokens as usize,
            total_tokens: ai_response_result.usage.total_tokens as usize,
            context_tokens: context_result.total_tokens,
            budget_utilization: (ai_response_result.usage.total_tokens as f64 / 102400.0 * 100.0),
        };

        EventBus::token_stats(app_handle, &session.uuid, token_stats)?;

        EventBus::progress(
            app_handle,
            &session.uuid,
            operation_type,
            1.0,
            Some(&format!("{}操作完成", operation_type)),
        )?;

        session
            .save_history(app_handle)
            .await
            .map_err(|e| format!("保存历史记录失败: {}", e))?;

        SESSION_MANAGER.update_session(session.clone())?;

        Ok(())
    }
}
