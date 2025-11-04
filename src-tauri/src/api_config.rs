use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use super::file_utils::FileUtils;

/// API配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub profile: String,
    pub endpoint: String,
    pub key: String,
    pub model: String,
    pub default: bool,
    pub enabled: bool,
}

/// 创建API请求
#[derive(Debug, Deserialize)]
pub struct CreateApiRequest {
    pub profile: String,
    pub endpoint: Option<String>,
    pub key: Option<String>,
    pub model: Option<String>,
    pub default: Option<bool>,
    pub enabled: Option<bool>,
}

/// 更新API请求
#[derive(Debug, Deserialize)]
pub struct UpdateApiRequest {
    pub profile: String,
    pub original_profile: String,
    pub endpoint: Option<String>,
    pub key: Option<String>,
    pub model: Option<String>,
    pub default: Option<bool>,
    pub enabled: Option<bool>,
}

/// API测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTestResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// 模型信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
}

/// API配置服务
pub struct ApiConfigService;

impl ApiConfigService {
    /// 获取API配置文件路径
    fn get_api_config_file(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = FileUtils::get_app_data_dir(app_handle)?;
        let api_dir = app_data_dir.join("api");
        FileUtils::ensure_dir_exists(&api_dir)?;
        Ok(api_dir.join("apis.json"))
    }

    /// 读取所有API配置
    fn read_api_configs(app_handle: &tauri::AppHandle) -> Result<Vec<ApiConfig>, String> {
        let config_file = Self::get_api_config_file(app_handle)?;

        if !config_file.exists() {
            return Ok(Vec::new());
        }

        FileUtils::read_json_file::<Vec<ApiConfig>>(&config_file)
    }

    /// 写入API配置
    fn write_api_configs(app_handle: &tauri::AppHandle, configs: &[ApiConfig]) -> Result<(), String> {
        let config_file = Self::get_api_config_file(app_handle)?;
        FileUtils::write_json_file(&config_file, configs)
    }

    /// 获取所有API配置
    pub fn get_all_api_configs(app_handle: &tauri::AppHandle) -> Result<Vec<ApiConfig>, String> {
        Self::read_api_configs(app_handle)
    }

    /// 根据配置名称获取API配置
    pub fn get_api_config_by_profile(app_handle: &tauri::AppHandle, profile: &str) -> Result<Option<ApiConfig>, String> {
        let configs = Self::read_api_configs(app_handle)?;
        Ok(configs.into_iter().find(|config| config.profile == profile))
    }

    /// 获取默认API配置
    pub fn get_default_api_config(app_handle: &tauri::AppHandle) -> Result<Option<ApiConfig>, String> {
        let configs = Self::read_api_configs(app_handle)?;
        Ok(configs.into_iter().find(|config| config.default))
    }

    /// 创建新的API配置
    pub fn create_api_config(app_handle: &tauri::AppHandle, request: CreateApiRequest) -> Result<ApiConfig, String> {
        let mut configs = Self::read_api_configs(app_handle)?;

        // 检查配置名称是否已存在
        if configs.iter().any(|config| config.profile == request.profile) {
            return Err(format!("API配置 '{}' 已存在", request.profile));
        }

        let new_config = ApiConfig {
            profile: request.profile,
            endpoint: request.endpoint.unwrap_or_default(),
            key: request.key.unwrap_or_default(),
            model: request.model.unwrap_or_default(),
            default: request.default.unwrap_or(false),
            enabled: request.enabled.unwrap_or(false),
        };

        // 如果设置为默认，清除其他默认配置
        if new_config.default {
            for config in &mut configs {
                config.default = false;
            }
        }

        configs.push(new_config.clone());
        Self::write_api_configs(app_handle, &configs)?;

        Ok(new_config)
    }

    /// 更新API配置
    pub fn update_api_config(app_handle: &tauri::AppHandle, request: UpdateApiRequest) -> Result<(), String> {
        let mut configs = Self::read_api_configs(app_handle)?;

        let config_index = configs.iter()
            .position(|config| config.profile == request.original_profile)
            .ok_or_else(|| format!("未找到配置 '{}'", request.original_profile))?;

        // 克隆当前配置以避免借用冲突
        let mut updated_config = configs[config_index].clone();

        // 更新profile名称
        updated_config.profile = request.profile;

        // 更新其他字段
        if let Some(endpoint) = request.endpoint {
            updated_config.endpoint = endpoint;
        }
        if let Some(key) = request.key {
            updated_config.key = key;
        }
        if let Some(model) = request.model {
            updated_config.model = model;
        }
        if let Some(enabled) = request.enabled {
            updated_config.enabled = enabled;
        }

        // 处理默认设置
        if let Some(default) = request.default {
            if default && !updated_config.default {
                // 设置为默认，清除其他默认配置
                for config in &mut configs {
                    config.default = false;
                }
                updated_config.default = true;
            } else if !default && updated_config.default {
                updated_config.default = false;
            }
        }

        // 替换配置
        configs[config_index] = updated_config;

        Self::write_api_configs(app_handle, &configs)?;
        Ok(())
    }

    /// 删除API配置
    pub fn delete_api_config(app_handle: &tauri::AppHandle, profile: &str) -> Result<(), String> {
        let mut configs = Self::read_api_configs(app_handle)?;

        let original_len = configs.len();
        configs.retain(|config| config.profile != profile);

        if configs.len() == original_len {
            return Err(format!("未找到配置 '{}'", profile));
        }

        Self::write_api_configs(app_handle, &configs)?;
        Ok(())
    }

    /// 设置默认API配置
    pub fn set_default_api_config(app_handle: &tauri::AppHandle, profile: &str) -> Result<(), String> {
        let mut configs = Self::read_api_configs(app_handle)?;

        let mut config_found = false;
        for config in configs.iter_mut() {
            if config.profile == profile {
                config.default = true;
                config_found = true;
            } else {
                config.default = false;
            }
        }

        if !config_found {
            return Err(format!("未找到配置 '{}'", profile));
        }

        Self::write_api_configs(app_handle, &configs)?;
        Ok(())
    }

    /// 启用/禁用API配置
    pub fn toggle_api_config(app_handle: &tauri::AppHandle, profile: &str, enabled: bool) -> Result<(), String> {
        let mut configs = Self::read_api_configs(app_handle)?;

        let config = configs.iter_mut()
            .find(|config| config.profile == profile)
            .ok_or_else(|| format!("未找到配置 '{}'", profile))?;

        config.enabled = enabled;

        Self::write_api_configs(app_handle, &configs)?;
        Ok(())
    }

    /// 测试API连接
    pub async fn test_api_connection(_app_handle: &tauri::AppHandle, config: &ApiConfig) -> Result<ApiTestResult, String> {
        if config.endpoint.is_empty() || config.key.is_empty() {
            return Ok(ApiTestResult {
                success: false,
                message: "API端点和密钥不能为空".to_string(),
                error: Some("Missing required fields".to_string()),
            });
        }

        // 构建测试请求URL
        let models_url = if config.endpoint.ends_with('/') {
            format!("{}models", config.endpoint)
        } else {
            format!("{}/models", config.endpoint)
        };

        // 创建HTTP客户端
        let client = reqwest::Client::new();

        let result = match client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", config.key))
            .header("Content-Type", "application/json")
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(_) => ApiTestResult {
                            success: true,
                            message: "连接测试成功".to_string(),
                            error: None,
                        },
                        Err(e) => ApiTestResult {
                            success: false,
                            message: "响应格式错误".to_string(),
                            error: Some(format!("解析响应失败: {}", e)),
                        },
                    }
                } else {
                    ApiTestResult {
                        success: false,
                        message: format!("连接失败: {}", response.status()),
                        error: Some(format!("HTTP错误: {}", response.status())),
                    }
                }
            }
            Err(e) => ApiTestResult {
                success: false,
                message: "网络连接失败".to_string(),
                error: Some(format!("网络错误: {}", e)),
            },
        };

        Ok(result)
    }

    /// 获取可用模型列表
    pub async fn fetch_models(_app_handle: &tauri::AppHandle, config: &ApiConfig) -> Result<Vec<ModelInfo>, String> {
        if config.endpoint.is_empty() || config.key.is_empty() {
            return Err("API端点和密钥不能为空".to_string());
        }

        // 构建模型请求URL
        let models_url = if config.endpoint.ends_with('/') {
            format!("{}models", config.endpoint)
        } else {
            format!("{}/models", config.endpoint)
        };

        // 创建HTTP客户端
        let client = reqwest::Client::new();

        let response = client
            .get(&models_url)
            .header("Authorization", format!("Bearer {}", config.key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("发送请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("获取模型列表失败: {}", response.status()));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        // 解析模型列表（OpenAI格式）
        let models = if let Some(data) = response_json.get("data").and_then(|d| d.as_array()) {
            data.iter()
                .filter_map(|model| {
                    let id = model.get("id")?.as_str()?.to_string();
                    let object = model.get("object")
                        .and_then(|o| o.as_str())
                        .unwrap_or("model")
                        .to_string();
                    Some(ModelInfo { id, object })
                })
                .collect()
        } else {
            // 如果不是标准格式，返回空列表
            Vec::new()
        };

        Ok(models)
    }
}