import { invoke } from '@tauri-apps/api/core';
import type {
  ChatCompletionRequest,
  ChatCompletionResponse,
  ChatCompletionChunk,
  ChatMessage,
  ChatTool,
  ApiConfig
} from '@/types/api';

/**
 * AI聊天完成请求选项
 */
export interface ChatCompletionOptions {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  stop?: string | string[];
  tools?: ChatTool[];
  tool_choice?: 'none' | 'auto' | { type: 'function'; function: { name: string } };
  stream?: boolean;
}

/**
 * 聊天响应
 */
export interface ChatResponse {
  id: string;
  object: 'chat.completion';
  created: number;
  model: string;
  choices: Array<{
    index: number;
    message: ChatMessage;
    finish_reason: 'stop' | 'length' | 'tool_calls' | 'content_filter';
  }>;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

/**
 * 流式聊天响应回调
 */
export type StreamCallback = (chunk: ChatCompletionChunk) => void;

/**
 * AI聊天服务 - 简化版本（上下文管理已移至后端）
 */
export class AIChatService {
  /**
   * 创建聊天完成请求
   */
  static async createChatCompletion(
    apiConfig: ApiConfig,
    options: ChatCompletionOptions
  ): Promise<ChatResponse> {
    try {
      const request: ChatCompletionRequest = {
        model: options.model,
        messages: options.messages,
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        top_p: options.top_p,
        frequency_penalty: options.frequency_penalty,
        presence_penalty: options.presence_penalty,
        stop: options.stop,
        stream: false,
        tools: options.tools,
        tool_choice: options.tool_choice,
      };

      const response = await invoke<ChatCompletionResponse>('create_chat_completion', {
        apiConfig,
        request,
      });

      return this.transformResponse(response);
    } catch (error) {
      console.error('聊天完成请求失败:', error);
      throw new Error(error as string);
    }
  }

  /**
   * 创建流式聊天完成请求
   */
  static async createStreamingChatCompletion(
    apiConfig: ApiConfig,
    options: ChatCompletionOptions,
    onChunk: StreamCallback
  ): Promise<ChatResponse> {
    try {
      const request: ChatCompletionRequest = {
        model: options.model,
        messages: options.messages,
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        top_p: options.top_p,
        frequency_penalty: options.frequency_penalty,
        presence_penalty: options.presence_penalty,
        stop: options.stop,
        stream: true,
        tools: options.tools,
        tool_choice: options.tool_choice,
      };

      const response = await invoke<string>('create_streaming_chat_completion', {
        apiConfig,
        request,
      });

      // 解析流式响应
      const chunks = this.parseStreamResponse(response);
      let finalResponse: ChatResponse | null = null;

      for (const chunk of chunks) {
        onChunk(chunk);

        // 最后一个chunk包含完整信息
        if (chunk.choices.length > 0 && chunk.choices[0].finish_reason) {
          finalResponse = this.transformChunkToResponse(chunk);
        }
      }

      if (!finalResponse) {
        throw new Error('流式响应未完成');
      }

      return finalResponse;
    } catch (error) {
      console.error('流式聊天完成请求失败:', error);
      throw new Error(error as string);
    }
  }

  /**
   * 构建聊天消息数组（已弃用 - 上下文构建现在在后端处理）
   * @deprecated 使用后端会话管理代替
   */
  static async buildMessages(
    systemPrompt: string,
    conversationHistory: Array<{ role: 'user' | 'assistant'; content: string }>,
    currentMessage: string,
    _contextData?: any,
    _options?: any
  ): Promise<ChatMessage[]> {
    console.warn('AIChatService.buildMessages() 已弃用，请使用后端会话管理');

    // 降级实现：仅构建基本的消息结构
    const messages: ChatMessage[] = [];

    // 系统消息
    if (systemPrompt) {
      messages.push({
        role: 'system',
        content: systemPrompt,
      });
    }

    // 对话历史
    conversationHistory.forEach(msg => {
      messages.push({
        role: msg.role,
        content: msg.content,
      });
    });

    // 当前用户消息
    if (currentMessage) {
      messages.push({
        role: 'user',
        content: currentMessage,
      });
    }

    return messages;
  }

  /**
   * 验证API配置
   */
  static validateApiConfig(config: ApiConfig): string[] {
    const errors: string[] = [];

    if (!config.profile) {
      errors.push('配置名称不能为空');
    }

    if (!config.endpoint) {
      errors.push('API地址不能为空');
    }

    if (!config.key) {
      errors.push('API密钥不能为空');
    }

    if (!config.model) {
      errors.push('模型名称不能为空');
    }

    // 验证URL格式
    try {
      new URL(config.endpoint);
    } catch {
      errors.push('API地址格式无效');
    }

    return errors;
  }

  /**
   * 转换响应格式
   */
  private static transformResponse(response: ChatCompletionResponse): ChatResponse {
    return {
      id: response.id,
      object: response.object,
      created: response.created,
      model: response.model,
      choices: response.choices.map(choice => ({
        index: choice.index,
        message: {
          role: choice.message.role,
          content: choice.message.content,
          name: choice.message.name,
          tool_calls: choice.message.tool_calls,
          tool_call_id: choice.message.tool_call_id,
        },
        finish_reason: choice.finish_reason,
      })),
      usage: {
        prompt_tokens: response.usage.prompt_tokens,
        completion_tokens: response.usage.completion_tokens,
        total_tokens: response.usage.total_tokens,
      },
    };
  }

  /**
   * 转换流式响应chunk为完整响应
   */
  private static transformChunkToResponse(chunk: ChatCompletionChunk): ChatResponse {
    return {
      id: chunk.id,
      object: 'chat.completion',
      created: chunk.created,
      model: chunk.model,
      choices: chunk.choices.map(choice => ({
        index: choice.index,
        message: {
          role: choice.delta.role || 'assistant',
          content: choice.delta.content || '',
          name: choice.delta.name,
          tool_calls: choice.delta.tool_calls,
          tool_call_id: choice.delta.tool_call_id,
        },
        finish_reason: choice.finish_reason || 'stop',
      })),
      usage: {
        prompt_tokens: 0, // 流式响应可能不包含usage信息
        completion_tokens: 0,
        total_tokens: 0,
      },
    };
  }

  /**
   * 解析流式响应
   */
  private static parseStreamResponse(response: string): ChatCompletionChunk[] {
    const chunks: ChatCompletionChunk[] = [];
    const lines = response.split('\n');

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6);
        if (data === '[DONE]') {
          break;
        }

        try {
          const chunk = JSON.parse(data) as ChatCompletionChunk;
          chunks.push(chunk);
        } catch (error) {
          console.error('解析流式响应chunk失败:', error);
        }
      }
    }

    return chunks;
  }
}

// ==================== 后端会话管理接口 ====================

/**
 * 后端会话管理接口 - 用于新的后端Stateful架构
 */
export class BackendSessionService {
  /**
   * 加载角色会话
   */
  static async loadCharacterSession(uuid: string): Promise<any> {
    return await invoke('load_character_session', { uuid });
  }

  /**
   * 发送聊天消息（通过后端会话）
   */
  static async sendChatMessage(message: string): Promise<void> {
    return await invoke('send_chat_message', { message });
  }

  /**
   * 卸载角色会话
   */
  static async unloadCharacterSession(uuid: string): Promise<void> {
    return await invoke('unload_character_session', { uuid });
  }

  /**
   * 获取会话信息
   */
  static async getSessionInfo(uuid: string): Promise<any> {
    return await invoke('get_session_info', { uuid });
  }

  /**
   * 获取所有活跃会话
   */
  static async getAllSessions(): Promise<any[]> {
    return await invoke('get_all_sessions');
  }

  /**
   * 保存所有会话历史
   */
  static async saveAllSessions(): Promise<number> {
    return await invoke('save_all_sessions');
  }

  /**
   * 清理过期会话
   */
  static async cleanupExpiredSessions(maxAgeHours: number): Promise<number> {
    return await invoke('cleanup_expired_sessions', { maxAgeHours });
  }
}