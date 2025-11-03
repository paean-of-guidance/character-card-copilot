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
 * AI聊天服务
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
   * 构建聊天消息数组
   */
  static buildMessages(
    systemPrompt: string,
    conversationHistory: Array<{ role: 'user' | 'assistant'; content: string }>,
    currentMessage: string,
    contextData?: any
  ): ChatMessage[] {
    const messages: ChatMessage[] = [];

    // 系统消息
    if (systemPrompt) {
      messages.push({
        role: 'system',
        content: systemPrompt,
      });
    }

    // 上下文信息
    if (contextData) {
      const contextMessage = this.buildContextMessage(contextData);
      if (contextMessage) {
        messages.push({
          role: 'system',
          content: contextMessage,
        });
      }
    }

    // 对话历史
    conversationHistory.forEach(msg => {
      messages.push({
        role: msg.role,
        content: msg.content,
      });
    });

    // 当前用户消息
    messages.push({
      role: 'user',
      content: currentMessage,
    });

    return messages;
  }

  /**
   * 构建上下文消息
   */
  private static buildContextMessage(contextData: any): string {
    try {
      if (contextData && typeof contextData === 'object') {
        return `以下是当前的角色数据信息，请基于此信息进行回复：\n\n${JSON.stringify(contextData, null, 2)}`;
      }
      return '';
    } catch (error) {
      console.error('构建上下文消息失败:', error);
      return '';
    }
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
        message: choice.message,
        finish_reason: choice.finish_reason,
      })),
      usage: response.usage,
    };
  }

  /**
   * 将流式chunk转换为响应格式
   */
  private static transformChunkToResponse(chunk: ChatCompletionChunk): ChatResponse {
    const choice = chunk.choices[0];
    if (!choice) {
      throw new Error('无效的流式响应chunk');
    }

    return {
      id: chunk.id,
      object: 'chat.completion',
      created: chunk.created,
      model: chunk.model,
      choices: [{
        index: choice.index,
        message: choice.delta as ChatMessage,
        finish_reason: choice.finish_reason || 'stop',
      }],
      usage: chunk.usage || {
        prompt_tokens: 0,
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
          console.warn('解析流式数据失败:', error, data);
        }
      }
    }

    return chunks;
  }

  /**
   * 计算token数量（粗略估算）
   */
  static estimateTokens(text: string): number {
    // 简单的token估算：大约4个字符=1个token
    return Math.ceil(text.length / 4);
  }

  /**
   * 验证API配置
   */
  static validateApiConfig(config: ApiConfig): string[] {
    const errors: string[] = [];

    if (!config.endpoint) {
      errors.push('API端点不能为空');
    } else if (!this.isValidUrl(config.endpoint)) {
      errors.push('API端点格式无效');
    }

    if (!config.key) {
      errors.push('API密钥不能为空');
    }

    if (!config.model) {
      errors.push('模型名称不能为空');
    }

    return errors;
  }

  /**
   * 验证URL格式
   */
  private static isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 构建API端点URL
   */
  static buildApiEndpoint(baseUrl: string): string {
    const cleanUrl = baseUrl.replace(/\/$/, ''); // 移除末尾斜杠
    return `${cleanUrl}/chat/completions`;
  }
}