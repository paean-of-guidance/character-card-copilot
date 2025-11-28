use crate::ai_chat::{AIChatService, ChatCompletionRequest, ChatCompletionResponse};
use crate::api_config::ApiConfig;

#[tauri::command]
pub async fn create_chat_completion(
    app: tauri::AppHandle,
    api_config: ApiConfig,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, String> {
    AIChatService::create_chat_completion(&api_config, &request, Some(&app)).await
}

#[tauri::command]
pub async fn create_streaming_chat_completion(
    api_config: ApiConfig,
    request: ChatCompletionRequest,
) -> Result<String, String> {
    AIChatService::create_streaming_chat_completion(&api_config, &request).await
}

