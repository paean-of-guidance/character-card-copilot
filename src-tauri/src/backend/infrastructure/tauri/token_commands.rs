use crate::token_counter::{get_token_counter, TokenCountResult};

#[tauri::command]
pub async fn count_tokens(text: String) -> Result<TokenCountResult, String> {
    let counter = get_token_counter();
    Ok(counter.count_tokens(&text))
}

#[tauri::command]
pub async fn count_tokens_batch(texts: Vec<String>) -> Result<Vec<TokenCountResult>, String> {
    let counter = get_token_counter();
    Ok(counter.count_tokens_batch(&texts))
}

#[tauri::command]
pub async fn check_token_limit(text: String, limit: usize) -> Result<bool, String> {
    let counter = get_token_counter();
    Ok(counter.is_within_limit(&text, limit))
}

#[tauri::command]
pub async fn truncate_to_token_limit(text: String, limit: usize) -> Result<String, String> {
    let counter = get_token_counter();
    Ok(counter.truncate_to_limit(&text, limit))
}

