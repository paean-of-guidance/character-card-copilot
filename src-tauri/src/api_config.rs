use super::file_utils::FileUtils;
use crate::ai_chat::{AIChatService, ChatCompletionRequest, ChatMessage, MessageRole};
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiProvider {
    OpenAiCompatible,
    OpenAiResponses,
    Claude,
    GeminiV1Beta,
}

impl Default for ApiProvider {
    fn default() -> Self {
        Self::OpenAiCompatible
    }
}

impl ApiProvider {
    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "https://api.openai.com/v1",
            Self::OpenAiResponses => "https://api.openai.com/v1",
            Self::Claude => "https://api.anthropic.com",
            Self::GeminiV1Beta => "https://generativelanguage.googleapis.com/v1beta",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "OpenAI Compatible",
            Self::OpenAiResponses => "OpenAI Responses",
            Self::Claude => "Claude",
            Self::GeminiV1Beta => "Gemini v1beta",
        }
    }
}

fn default_api_max_tokens() -> u32 {
    8192
}

fn default_context_window() -> u32 {
    65534
}

fn deserialize_max_tokens_with_default<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<u32>::deserialize(deserializer)?.unwrap_or_else(default_api_max_tokens))
}

fn deserialize_context_window_with_default<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<u32>::deserialize(deserializer)?.unwrap_or_else(default_context_window))
}

/// API配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub profile: String,
    #[serde(default)]
    pub provider: ApiProvider,
    #[serde(default, skip_serializing_if = "is_false")]
    pub provider_explicit: bool,
    #[serde(alias = "endpoint")]
    pub base_url: String,
    #[serde(alias = "key")]
    pub api_key: String,
    pub model: String,
    #[serde(
        default = "default_api_max_tokens",
        deserialize_with = "deserialize_max_tokens_with_default"
    )]
    pub max_tokens: u32,
    #[serde(
        default = "default_context_window",
        deserialize_with = "deserialize_context_window_with_default"
    )]
    pub context_window: u32,
    pub default: bool,
    pub enabled: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// 创建API请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiRequest {
    pub profile: String,
    pub provider: Option<ApiProvider>,
    #[serde(alias = "endpoint")]
    pub base_url: Option<String>,
    #[serde(alias = "key")]
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub context_window: Option<u32>,
    pub default: Option<bool>,
    pub enabled: Option<bool>,
}

/// 更新API请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApiRequest {
    pub profile: String,
    pub original_profile: String,
    pub provider: Option<ApiProvider>,
    #[serde(alias = "endpoint")]
    pub base_url: Option<String>,
    #[serde(alias = "key")]
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub context_window: Option<u32>,
    pub default: Option<bool>,
    pub enabled: Option<bool>,
}

/// API测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u32>,
}

fn normalize_profile(profile: &str) -> String {
    profile.trim().to_string()
}

fn normalize_optional_text(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_string()
}

fn validate_profile(profile: &str) -> Result<(), String> {
    if profile.is_empty() {
        return Err("API配置名称不能为空".to_string());
    }

    if profile.chars().count() < 2 {
        return Err("API配置名称至少需要2个字符".to_string());
    }

    if profile.chars().count() > 50 {
        return Err("API配置名称不能超过50个字符".to_string());
    }

    Ok(())
}

fn ensure_unique_profile(
    configs: &[ApiConfig],
    profile: &str,
    exclude_profile: Option<&str>,
) -> Result<(), String> {
    let duplicated = configs.iter().any(|config| {
        config.profile == profile && exclude_profile != Some(config.profile.as_str())
    });

    if duplicated {
        return Err(format!("API配置 '{}' 已存在", profile));
    }

    Ok(())
}

fn normalize_base_url(provider: ApiProvider, base_url: String) -> String {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        provider.default_base_url().to_string()
    } else {
        trimmed.trim_end_matches('/').to_string()
    }
}

fn value_to_u32(value: &serde_json::Value) -> Option<u32> {
    value.as_u64().and_then(|raw| u32::try_from(raw).ok())
}

fn extract_model_limit(model: &serde_json::Value, candidate_keys: &[&str]) -> Option<u32> {
    match model {
        serde_json::Value::Object(map) => {
            for key in candidate_keys {
                if let Some(limit) = map.get(*key).and_then(value_to_u32) {
                    return Some(limit);
                }
            }

            map.values()
                .find_map(|value| extract_model_limit(value, candidate_keys))
        }
        serde_json::Value::Array(items) => items
            .iter()
            .find_map(|value| extract_model_limit(value, candidate_keys)),
        _ => None,
    }
}

fn normalize_test_reply(content: &str) -> String {
    content
        .trim()
        .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '!' | '.' | ',' | '，' | '。' | '！'))
        .trim()
        .to_uppercase()
}

fn migrate_config(config: ApiConfig) -> ApiConfig {
    let provider = if config.provider_explicit {
        config.provider
    } else {
        match config.provider {
            ApiProvider::OpenAiResponses => ApiProvider::OpenAiCompatible,
            provider => provider,
        }
    };

    ApiConfig {
        profile: normalize_profile(&config.profile),
        provider,
        provider_explicit: true,
        base_url: normalize_base_url(provider, config.base_url),
        api_key: config.api_key.trim().to_string(),
        model: config.model.trim().to_string(),
        max_tokens: config.max_tokens,
        context_window: config.context_window,
        default: config.default,
        enabled: config.enabled,
    }
}

fn build_new_config(
    configs: &mut Vec<ApiConfig>,
    request: CreateApiRequest,
) -> Result<ApiConfig, String> {
    let profile = normalize_profile(&request.profile);
    validate_profile(&profile)?;
    ensure_unique_profile(configs, &profile, None)?;

    let provider = request.provider.unwrap_or_default();
    let enabled = request.enabled.unwrap_or(false);
    let default = request.default.unwrap_or(false);

    if default && !enabled {
        return Err("默认配置必须先启用".to_string());
    }

    if default {
        for config in configs.iter_mut() {
            config.default = false;
        }
    }

    Ok(ApiConfig {
        profile,
        provider,
        provider_explicit: true,
        base_url: normalize_base_url(provider, normalize_optional_text(request.base_url)),
        api_key: normalize_optional_text(request.api_key),
        model: normalize_optional_text(request.model),
        max_tokens: request.max_tokens.unwrap_or_else(default_api_max_tokens),
        context_window: request.context_window.unwrap_or_else(default_context_window),
        default,
        enabled,
    })
}

fn update_config_in_configs(
    configs: &mut [ApiConfig],
    request: UpdateApiRequest,
) -> Result<(), String> {
    let config_index = configs
        .iter()
        .position(|config| config.profile == request.original_profile)
        .ok_or_else(|| format!("未找到配置 '{}'", request.original_profile))?;

    let profile = normalize_profile(&request.profile);
    validate_profile(&profile)?;
    ensure_unique_profile(configs, &profile, Some(request.original_profile.as_str()))?;

    let mut updated_config = configs[config_index].clone();
    updated_config.profile = profile;

    if let Some(provider) = request.provider {
        updated_config.provider = provider;
        updated_config.provider_explicit = true;
        if request.base_url.is_none() {
            updated_config.base_url = normalize_base_url(provider, updated_config.base_url.clone());
        }
    }
    if let Some(base_url) = request.base_url {
        updated_config.base_url = normalize_base_url(updated_config.provider, base_url);
    }
    if let Some(api_key) = request.api_key {
        updated_config.api_key = api_key.trim().to_string();
    }
    if let Some(model) = request.model {
        updated_config.model = model.trim().to_string();
    }
    if let Some(max_tokens) = request.max_tokens {
        updated_config.max_tokens = max_tokens;
    }
    if let Some(context_window) = request.context_window {
        updated_config.context_window = context_window;
    }
    if let Some(enabled) = request.enabled {
        updated_config.enabled = enabled;
        if !enabled {
            updated_config.default = false;
        }
    }

    if let Some(default) = request.default {
        if default {
            if !updated_config.enabled {
                return Err("启用后的配置才能设为默认".to_string());
            }

            for config in configs.iter_mut() {
                config.default = false;
            }
            updated_config.default = true;
        } else {
            updated_config.default = false;
        }
    }

    configs[config_index] = updated_config;
    Ok(())
}

fn set_default_in_configs(configs: &mut [ApiConfig], profile: &str) -> Result<(), String> {
    let target_index = configs
        .iter()
        .position(|config| config.profile == profile)
        .ok_or_else(|| format!("未找到配置 '{}'", profile))?;

    if !configs[target_index].enabled {
        return Err("启用后的配置才能设为默认".to_string());
    }

    for config in configs.iter_mut() {
        config.default = false;
    }
    configs[target_index].default = true;

    Ok(())
}

fn toggle_enabled_in_configs(
    configs: &mut [ApiConfig],
    profile: &str,
    enabled: bool,
) -> Result<(), String> {
    let config = configs
        .iter_mut()
        .find(|config| config.profile == profile)
        .ok_or_else(|| format!("未找到配置 '{}'", profile))?;

    config.enabled = enabled;
    if !enabled {
        config.default = false;
    }

    Ok(())
}

pub struct ApiConfigService;

impl ApiConfigService {
    fn get_api_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = FileUtils::get_app_data_dir(app_handle)?;
        Ok(app_data_dir.join("api_configs.json"))
    }

    fn load_configs(app_handle: &tauri::AppHandle) -> Result<Vec<ApiConfig>, String> {
        let file_path = Self::get_api_config_path(app_handle)?;
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let configs = FileUtils::read_json_file::<Vec<ApiConfig>>(&file_path)?
            .into_iter()
            .map(migrate_config)
            .collect::<Vec<_>>();

        Ok(configs)
    }

    fn save_configs(app_handle: &tauri::AppHandle, configs: &[ApiConfig]) -> Result<(), String> {
        let file_path = Self::get_api_config_path(app_handle)?;
        FileUtils::write_json_file(&file_path, configs)
    }

    pub fn get_all_api_configs(app_handle: &tauri::AppHandle) -> Result<Vec<ApiConfig>, String> {
        let configs = Self::load_configs(app_handle)?;
        Self::save_configs(app_handle, &configs)?;
        Ok(configs)
    }

    pub fn get_api_config_by_profile(
        app_handle: &tauri::AppHandle,
        profile: &str,
    ) -> Result<Option<ApiConfig>, String> {
        let configs = Self::load_configs(app_handle)?;
        Ok(configs.into_iter().find(|config| config.profile == profile))
    }

    pub fn get_default_api_config(
        app_handle: &tauri::AppHandle,
    ) -> Result<Option<ApiConfig>, String> {
        let configs = Self::load_configs(app_handle)?;
        Ok(configs
            .into_iter()
            .find(|config| config.default && config.enabled))
    }

    pub fn create_api_config(
        app_handle: &tauri::AppHandle,
        request: CreateApiRequest,
    ) -> Result<ApiConfig, String> {
        let mut configs = Self::load_configs(app_handle)?;
        let new_config = build_new_config(&mut configs, request)?;
        configs.push(new_config.clone());
        Self::save_configs(app_handle, &configs)?;
        Ok(new_config)
    }

    pub fn update_api_config(
        app_handle: &tauri::AppHandle,
        request: UpdateApiRequest,
    ) -> Result<(), String> {
        let mut configs = Self::load_configs(app_handle)?;
        update_config_in_configs(&mut configs, request)?;
        Self::save_configs(app_handle, &configs)
    }

    pub fn delete_api_config(app_handle: &tauri::AppHandle, profile: &str) -> Result<(), String> {
        let mut configs = Self::load_configs(app_handle)?;
        let original_len = configs.len();
        configs.retain(|config| config.profile != profile);

        if configs.len() == original_len {
            return Err(format!("未找到配置 '{}'", profile));
        }

        Self::save_configs(app_handle, &configs)
    }

    pub fn set_default_api_config(
        app_handle: &tauri::AppHandle,
        profile: &str,
    ) -> Result<(), String> {
        let mut configs = Self::load_configs(app_handle)?;
        set_default_in_configs(&mut configs, profile)?;
        Self::save_configs(app_handle, &configs)
    }

    pub fn toggle_api_config(
        app_handle: &tauri::AppHandle,
        profile: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let mut configs = Self::load_configs(app_handle)?;
        toggle_enabled_in_configs(&mut configs, profile, enabled)?;
        Self::save_configs(app_handle, &configs)
    }

    pub async fn test_api_connection(
        _app_handle: &tauri::AppHandle,
        config: &ApiConfig,
    ) -> Result<ApiTestResult, String> {
        if config.base_url.is_empty() || config.api_key.is_empty() || config.model.is_empty() {
            return Ok(ApiTestResult {
                success: false,
                message: "API Base URL、密钥和模型不能为空".to_string(),
                error: Some("Missing required fields".to_string()),
            });
        }

        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Reply with exactly one short word: PONG".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            temperature: Some(0.0),
            max_tokens: Some(4096),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };

        let result = match AIChatService::create_chat_completion(config, &request, None, None).await
        {
            Ok(response) => {
                let reply = response
                    .choices
                    .first()
                    .map(|choice| choice.message.content.as_str())
                    .unwrap_or_default();
                let normalized_reply = normalize_test_reply(reply);

                if normalized_reply.contains("PONG") {
                    ApiTestResult {
                        success: true,
                        message: format!("{} 连通性测试成功", config.provider.label()),
                        error: None,
                    }
                } else if !normalized_reply.is_empty() {
                    ApiTestResult {
                        success: true,
                        message: format!(
                            "{} 已成功响应，虽然未严格返回 PONG，但连通性测试通过",
                            config.provider.label()
                        ),
                        error: Some(format!("Model reply: {}", reply.trim())),
                    }
                } else {
                    ApiTestResult {
                        success: false,
                        message: "模型调用成功，但返回了空文本".to_string(),
                        error: Some(
                            "Unexpected reply: <empty>. 这通常是推理模型在较低 max_tokens 下只消耗了隐藏推理预算。"
                                .to_string(),
                        ),
                    }
                }
            }
            Err(error) => ApiTestResult {
                success: false,
                message: "真实推理测试失败".to_string(),
                error: Some(error),
            },
        };

        Ok(result)
    }

    pub async fn fetch_models(
        _app_handle: &tauri::AppHandle,
        config: &ApiConfig,
    ) -> Result<Vec<ModelInfo>, String> {
        if config.base_url.is_empty() || config.api_key.is_empty() {
            return Err("API Base URL 和密钥不能为空".to_string());
        }

        let client = reqwest::Client::new();
        let response = match config.provider {
            ApiProvider::OpenAiCompatible => {
                client
                    .get(format!("{}/models", config.base_url.trim_end_matches('/')))
                    .bearer_auth(&config.api_key)
                    .header("Content-Type", "application/json")
                    .send()
                    .await
            }
            ApiProvider::OpenAiResponses => {
                client
                    .get(format!("{}/models", config.base_url.trim_end_matches('/')))
                    .bearer_auth(&config.api_key)
                    .header("Content-Type", "application/json")
                    .send()
                    .await
            }
            ApiProvider::Claude => {
                client
                    .get(format!(
                        "{}/v1/models",
                        config.base_url.trim_end_matches('/')
                    ))
                    .header("x-api-key", &config.api_key)
                    .header("anthropic-version", "2023-06-01")
                    .send()
                    .await
            }
            ApiProvider::GeminiV1Beta => {
                client
                    .get(format!("{}/models", config.base_url.trim_end_matches('/')))
                    .query(&[("key", config.api_key.clone())])
                    .send()
                    .await
            }
        }
        .map_err(|error| format!("发送请求失败: {}", error))?;

        if !response.status().is_success() {
            return Err(format!("获取模型列表失败: {}", response.status()));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("解析响应失败: {}", error))?;

        let models = match config.provider {
            ApiProvider::OpenAiCompatible | ApiProvider::OpenAiResponses | ApiProvider::Claude => {
                response_json
                    .get("data")
                    .and_then(|value| value.as_array())
                    .map(|data| {
                        data.iter()
                            .filter_map(|model| {
                                let id = model.get("id")?.as_str()?.to_string();
                                let object = model
                                    .get("object")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or("model")
                                    .to_string();
                                let owned_by = model
                                    .get("owned_by")
                                    .and_then(|value| value.as_str())
                                    .map(|value| value.to_string())
                                    .or_else(|| match config.provider {
                                        ApiProvider::Claude => Some("anthropic".to_string()),
                                        _ => None,
                                    });
                                let max_tokens = extract_model_limit(
                                    model,
                                    &["max_output_tokens", "max_completion_tokens", "max_tokens", "output_token_limit"],
                                );
                                let context_window = extract_model_limit(
                                    model,
                                    &["context_window", "contextWindow", "input_token_limit", "inputTokenLimit"],
                                );

                                Some(ModelInfo {
                                    id,
                                    object,
                                    owned_by,
                                    max_tokens,
                                    context_window,
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            }
            ApiProvider::GeminiV1Beta => response_json
                .get("models")
                .and_then(|value| value.as_array())
                .map(|data| {
                    data.iter()
                        .filter_map(|model| {
                            let id = model
                                .get("baseModelId")
                                .and_then(|value| value.as_str())
                                .map(|value| value.to_string())
                                .or_else(|| {
                                    model.get("name").and_then(|value| value.as_str()).map(
                                        |value| value.trim_start_matches("models/").to_string(),
                                    )
                                })?;

                            Some(ModelInfo {
                                id,
                                object: "model".to_string(),
                                owned_by: Some("google".to_string()),
                                max_tokens: extract_model_limit(
                                    model,
                                    &["outputTokenLimit", "output_token_limit", "maxOutputTokens", "max_output_tokens"],
                                ),
                                context_window: extract_model_limit(
                                    model,
                                    &["inputTokenLimit", "input_token_limit", "contextWindow", "context_window"],
                                ),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        };

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_configs() -> Vec<ApiConfig> {
        vec![
            ApiConfig {
                profile: "Primary".to_string(),
                provider: ApiProvider::OpenAiCompatible,
                provider_explicit: true,
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: "key-1".to_string(),
                model: "gpt-4.1".to_string(),
                max_tokens: 16_384,
                context_window: 128_000,
                default: true,
                enabled: true,
            },
            ApiConfig {
                profile: "Backup".to_string(),
                provider: ApiProvider::Claude,
                provider_explicit: true,
                base_url: "https://api.anthropic.com".to_string(),
                api_key: "key-2".to_string(),
                model: "claude-3-5-sonnet-latest".to_string(),
                max_tokens: 8_192,
                context_window: 200_000,
                default: false,
                enabled: true,
            },
        ]
    }

    #[test]
    fn create_config_rejects_disabled_default() {
        let mut configs = sample_configs();
        let result = build_new_config(
            &mut configs,
            CreateApiRequest {
                profile: " Draft ".to_string(),
                provider: Some(ApiProvider::GeminiV1Beta),
                base_url: Some(" https://generativelanguage.googleapis.com/v1beta ".to_string()),
                api_key: Some(" key-3 ".to_string()),
                model: Some(" gemini-2.0-flash ".to_string()),
                max_tokens: Some(8_192),
                context_window: Some(1_048_576),
                default: Some(true),
                enabled: Some(false),
            },
        );

        assert_eq!(result.unwrap_err(), "默认配置必须先启用");
    }

    #[test]
    fn update_config_rejects_duplicate_profile() {
        let mut configs = sample_configs();
        let result = update_config_in_configs(
            &mut configs,
            UpdateApiRequest {
                profile: "Backup".to_string(),
                original_profile: "Primary".to_string(),
                provider: None,
                base_url: None,
                api_key: None,
                model: None,
                max_tokens: None,
                context_window: None,
                default: None,
                enabled: None,
            },
        );

        assert_eq!(result.unwrap_err(), "API配置 'Backup' 已存在");
    }

    #[test]
    fn create_config_uses_conservative_model_limits_by_default() {
        let mut configs = sample_configs();

        let created = build_new_config(
            &mut configs,
            CreateApiRequest {
                profile: "Draft".to_string(),
                provider: Some(ApiProvider::OpenAiCompatible),
                base_url: Some("https://api.example.com/v1".to_string()),
                api_key: Some("key-3".to_string()),
                model: Some("custom-model".to_string()),
                max_tokens: None,
                context_window: None,
                default: Some(false),
                enabled: Some(false),
            },
        )
        .unwrap();

        assert_eq!(created.max_tokens, default_api_max_tokens());
        assert_eq!(created.context_window, default_context_window());
    }

    #[test]
    fn update_config_updates_model_limits() {
        let mut configs = sample_configs();

        update_config_in_configs(
            &mut configs,
            UpdateApiRequest {
                profile: "Primary".to_string(),
                original_profile: "Primary".to_string(),
                provider: None,
                base_url: None,
                api_key: None,
                model: Some("custom-model".to_string()),
                max_tokens: Some(12_288),
                context_window: Some(131_072),
                default: None,
                enabled: None,
            },
        )
        .unwrap();

        assert_eq!(configs[0].model, "custom-model");
        assert_eq!(configs[0].max_tokens, 12_288);
        assert_eq!(configs[0].context_window, 131_072);
    }

    #[test]
    fn set_default_requires_enabled_config() {
        let mut configs = sample_configs();
        configs[1].enabled = false;

        let result = set_default_in_configs(&mut configs, "Backup");

        assert_eq!(result.unwrap_err(), "启用后的配置才能设为默认");
        assert!(configs[0].default);
        assert!(!configs[1].default);
    }

    #[test]
    fn disabling_default_config_clears_default_flag() {
        let mut configs = sample_configs();
        toggle_enabled_in_configs(&mut configs, "Primary", false).unwrap();

        assert!(!configs[0].enabled);
        assert!(!configs[0].default);
    }

    #[test]
    fn migration_maps_legacy_fields() {
        let legacy_json = serde_json::json!({
            "profile": "Legacy",
            "endpoint": "https://api.openai.com/v1/",
            "key": "sk-legacy",
            "model": "gpt-4.1-mini",
            "default": false,
            "enabled": true
        });

        let config: ApiConfig = serde_json::from_value(legacy_json).unwrap();
        let migrated = migrate_config(config);

        assert_eq!(migrated.provider, ApiProvider::OpenAiCompatible);
        assert_eq!(migrated.base_url, "https://api.openai.com/v1");
        assert_eq!(migrated.api_key, "sk-legacy");
        assert_eq!(migrated.max_tokens, default_api_max_tokens());
        assert_eq!(migrated.context_window, default_context_window());
        assert!(migrated.provider_explicit);
    }

    #[test]
    fn migration_maps_null_model_limits_to_defaults() {
        let legacy_json = serde_json::json!({
            "profile": "Legacy Null",
            "endpoint": "https://api.openai.com/v1/",
            "key": "sk-null",
            "model": "gpt-4.1-mini",
            "max_tokens": null,
            "context_window": null,
            "default": false,
            "enabled": true
        });

        let config: ApiConfig = serde_json::from_value(legacy_json).unwrap();
        let migrated = migrate_config(config);

        assert_eq!(migrated.max_tokens, default_api_max_tokens());
        assert_eq!(migrated.context_window, default_context_window());
    }

    #[test]
    fn migration_preserves_explicit_openai_responses_provider() {
        let config = ApiConfig {
            profile: "Responses".to_string(),
            provider: ApiProvider::OpenAiResponses,
            provider_explicit: true,
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "sk-live".to_string(),
            model: "gpt-5-mini".to_string(),
            max_tokens: 4_096,
            context_window: 128_000,
            default: false,
            enabled: true,
        };

        let migrated = migrate_config(config);

        assert_eq!(migrated.provider, ApiProvider::OpenAiResponses);
    }
}
