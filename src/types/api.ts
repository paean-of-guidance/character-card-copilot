/**
 * API配置相关的类型定义
 */

export interface ApiConfig {
  /** 配置名称 */
  profile: string;
  /** API链接端点 */
  endpoint: string;
  /** API密钥 */
  key: string;
  /** 使用的模型 */
  model: string;
  /** 是否为默认配置 */
  default: boolean;
  /** 是否启用 */
  enabled: boolean;
}

export interface ApiListResponse {
  apis: ApiConfig[];
}

export interface CreateApiRequest {
  profile: string;
  endpoint?: string;
  key?: string;
  model?: string;
  default?: boolean;
  enabled?: boolean;
}

export interface UpdateApiRequest extends Partial<ApiConfig> {
  /** 要更新的API配置名称 */
  profile: string;
  /** 原始的API配置名称（用于查找要更新的配置） */
  original_profile: string;
}

/**
 * API测试结果
 */
export interface ApiTestResult {
  success: boolean;
  message: string;
  error?: string;
}

/**
 * 可用模型信息
 */
export interface ModelInfo {
  id: string;
  object: string;
  created?: number;
  owned_by?: string;
}

/**
 * 模型列表响应
 */
export interface ModelsResponse {
  object: string;
  data: ModelInfo[];
}

// ====================== AI聊天相关类型 ======================

/**
 * 聊天消息角色
 */
export type MessageRole = 'system' | 'user' | 'assistant' | 'tool';

/**
 * 聊天消息
 */
export interface ChatMessage {
  role: MessageRole;
  content: string;
  name?: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
}

/**
 * 工具调用参数
 */
export interface ToolCallFunction {
  name: string;
  arguments: string; // JSON字符串
}

/**
 * 工具调用
 */
export interface ToolCall {
  id: string;
  type: 'function';
  function: ToolCallFunction;
}

/**
 * 聊天完成请求
 */
export interface ChatCompletionRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  stop?: string | string[];
  stream?: boolean;
  tools?: ChatTool[];
  tool_choice?: 'none' | 'auto' | { type: 'function'; function: { name: string } } | { type: 'auto' };
}

/**
 * 工具定义
 */
export interface ChatToolParameter {
  type: 'string' | 'number' | 'boolean' | 'object' | 'array';
  description?: string;
  enum?: string[];
  items?: ChatToolParameter;
  properties?: Record<string, ChatToolParameter>;
  required?: string[];
}

export interface ChatToolFunction {
  name: string;
  description?: string;
  parameters?: {
    type: 'object';
    properties: Record<string, ChatToolParameter>;
    required?: string[];
  };
}

export interface ChatTool {
  type: 'function';
  function: ChatToolFunction;
}

/**
 * 使用统计信息
 */
export interface Usage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

/**
 * 选择项
 */
export interface LogProb {
  token: string;
  logprob: number;
  bytes?: number[];
}

export interface TopLogProb {
  token: string;
  logprob: number;
  bytes?: number[];
  top_logprobs: LogProb[];
}

/**
 * 聊天完成选择项
 */
export interface ChatCompletionChoice {
  index: number;
  message: ChatMessage;
  finish_reason: 'stop' | 'length' | 'tool_calls' | 'content_filter';
  logprobs?: {
    content: TopLogProb[];
  };
}

/**
 * 聊天完成响应
 */
export interface ChatCompletionResponse {
  id: string;
  object: 'chat.completion';
  created: number;
  model: string;
  system_fingerprint?: string;
  choices: ChatCompletionChoice[];
  usage: Usage;
}

/**
 * 聊天流式数据块
 */
export interface ChatCompletionChunk {
  id: string;
  object: 'chat.completion.chunk';
  created: number;
  model: string;
  system_fingerprint?: string;
  choices: Array<{
    index: number;
    delta: Partial<ChatMessage>;
    finish_reason?: 'stop' | 'length' | 'tool_calls' | 'content_filter';
    logprobs?: {
      content: TopLogProb[];
    };
  }>;
  usage?: Usage;
}