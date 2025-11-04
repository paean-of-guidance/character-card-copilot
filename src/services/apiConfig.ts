import type { ApiConfig, CreateApiRequest, UpdateApiRequest, ApiTestResult, ModelInfo } from '@/types/api';
import { invoke } from '@tauri-apps/api/core';

/**
 * API配置存储服务
 * 管理API配置的文件系统存储和读取
 */

/**
 * 获取所有API配置
 */
export async function getAllApiConfigs(): Promise<ApiConfig[]> {
  try {
    const configs = await invoke<ApiConfig[]>('get_all_api_configs');
    return configs;
  } catch (error) {
    console.error('获取API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 根据配置名称获取API配置
 * @param profile 配置名称
 */
export async function getApiConfigByProfile(profile: string): Promise<ApiConfig | null> {
  try {
    const config = await invoke<ApiConfig | null>('get_api_config_by_profile', { profile });
    return config;
  } catch (error) {
    console.error('获取API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 获取默认API配置
 */
export async function getDefaultApiConfig(): Promise<ApiConfig | null> {
  try {
    const config = await invoke<ApiConfig | null>('get_default_api_config');
    return config;
  } catch (error) {
    console.error('获取默认API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 创建新的API配置
 * @param config API配置数据
 */
export async function createApiConfig(config: CreateApiRequest): Promise<ApiConfig> {
  try {
    const newConfig = await invoke<ApiConfig>('create_api_config', { request: config });
    return newConfig;
  } catch (error) {
    console.error('创建API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 更新API配置
 * @param config 更新后的API配置数据
 */
export async function updateApiConfig(config: UpdateApiRequest): Promise<void> {
  try {
    await invoke('update_api_config', { request: config });
  } catch (error) {
    console.error('更新API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 删除API配置
 * @param profile 配置名称
 */
export async function deleteApiConfig(profile: string): Promise<void> {
  try {
    await invoke('delete_api_config', { profile });
  } catch (error) {
    console.error('删除API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 设置默认API配置
 * @param profile 配置名称
 */
export async function setDefaultApiConfig(profile: string): Promise<void> {
  try {
    await invoke('set_default_api_config', { profile });
  } catch (error) {
    console.error('设置默认API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 启用/禁用API配置
 * @param profile 配置名称
 * @param enabled 是否启用
 */
export async function toggleApiConfig(profile: string, enabled: boolean): Promise<void> {
  try {
    await invoke('toggle_api_config', { profile, enabled });
  } catch (error) {
    console.error('启用/禁用API配置失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 测试API连接
 * @param config API配置
 */
export async function testApiConnection(config: ApiConfig): Promise<ApiTestResult> {
  if (!config.endpoint || !config.key) {
    return {
      success: false,
      message: 'API端点和密钥不能为空',
      error: 'Missing required fields'
    };
  }

  try {
    const result = await invoke<ApiTestResult>('test_api_connection', { config });
    return result;
  } catch (error) {
    console.error('测试API连接失败:', error);
    return {
      success: false,
      message: '连接测试失败',
      error: error as string
    };
  }
}

/**
 * 获取可用模型列表
 * @param config API配置
 */
export async function fetchModels(config: ApiConfig): Promise<ModelInfo[]> {
  if (!config.endpoint || !config.key) {
    throw new Error('API端点和密钥不能为空');
  }

  try {
    const models = await invoke<ModelInfo[]>('fetch_models', { config });
    return models;
  } catch (error) {
    console.error('获取模型列表失败:', error);
    throw new Error(error as string);
  }
}

/**
 * 复制API配置
 * @param api 要复制的API配置
 */
export async function copyApiConfig(api: ApiConfig): Promise<ApiConfig> {
  const copyConfig: CreateApiRequest = {
    profile: `${api.profile} (copy)`,
    endpoint: api.endpoint,
    key: api.key,
    model: api.model,
    default: false, // 复制的配置不能是默认
    enabled: api.enabled, // 保持原有的启用状态
  };

  return await createApiConfig(copyConfig);
}
