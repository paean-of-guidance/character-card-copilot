import { invoke } from '@tauri-apps/api/core';

/**
 * OpenAI ChatTool 格式 - 单个参数定义
 */
export interface ToolParameter {
  type: string; // "string", "number", "boolean", "object", "array"
  description?: string;
  enum?: string[]; // 枚举值
  items?: ToolParameter; // 数组元素类型
  properties?: Record<string, ToolParameter>; // 对象属性
  required?: string[]; // 必填字段列表
}

/**
 * OpenAI ChatTool 格式 - 参数集合
 */
export interface ToolParameters {
  type: 'object';
  properties: Record<string, ToolParameter>;
  required?: string[];
}

/**
 * OpenAI ChatTool 格式 - 函数定义
 */
export interface ToolFunction {
  name: string;
  description?: string;
  parameters?: ToolParameters;
}

/**
 * OpenAI ChatTool 格式 - 完整工具定义
 */
export interface ChatTool {
  type: 'function';
  function: ToolFunction;
}

/**
 * 旧版 AITool 类型 (已废弃,仅用于兼容性)
 * @deprecated 使用 ChatTool 代替
 */
export interface AITool {
  name: string;
  description: string;
  category: string;
  parameters: any[];
  enabled: boolean;
}

export interface ToolCallRequest {
  tool_name: string;
  parameters: Record<string, any>;
  character_uuid?: string; // 角色UUID
  context?: any; // CharacterData or other context
}

export interface ToolResult {
  success: boolean;
  data?: any;
  error?: string;
  execution_time_ms: number;
}

export class AIToolsService {
  /**
   * 获取所有可用工具 (OpenAI ChatTool 格式)
   */
  static async getAvailableTools(): Promise<ChatTool[]> {
    return await invoke<ChatTool[]>('get_available_tools');
  }

  /**
   * 根据分类获取工具 (OpenAI ChatTool 格式)
   */
  static async getToolsByCategory(category: string): Promise<ChatTool[]> {
    return await invoke<ChatTool[]>('get_tools_by_category', { category });
  }

  /**
   * 执行工具调用
   */
  static async executeToolCall(request: ToolCallRequest): Promise<ToolResult> {
    return await invoke<ToolResult>('execute_tool_call', { request });
  }

  /**
   * 获取工具分类列表
   */
  static async getToolCategories(): Promise<string[]> {
    return await invoke<string[]>('get_tool_categories');
  }

  /**
   * 编辑角色工具 - 主要的AI编辑功能
   * 参数对应TavernCardV2字段名
   */
  static async editCharacter(
    characterUuid: string,
    fields: {
      name?: string;
      description?: string;
      personality?: string;
      scenario?: string;
      first_mes?: string;
      mes_example?: string;
      creator_notes?: string;
      system_prompt?: string;
      post_history_instructions?: string;
      alternate_greetings?: string; // 使用 <START_ALT> 标记每段
      tags?: string; // 用逗号分隔
      creator?: string;
      character_version?: string;
    },
    context?: any
  ): Promise<ToolResult> {
    return this.executeToolCall({
      tool_name: 'edit_character',
      parameters: fields,
      character_uuid: characterUuid,
      context,
    });
  }

  /**
   * 根据工具名称获取工具
   */
  static async getToolByName(toolName: string): Promise<ChatTool | null> {
    const tools = await this.getAvailableTools();
    return tools.find(tool => tool.function.name === toolName) || null;
  }

  /**
   * 获取所有工具的名称列表
   */
  static async getToolNames(): Promise<string[]> {
    const tools = await this.getAvailableTools();
    return tools.map(tool => tool.function.name);
  }

  /**
   * 检查工具是否存在
   */
  static async hasToolByName(toolName: string): Promise<boolean> {
    const tool = await this.getToolByName(toolName);
    return tool !== null;
  }
}

// 工具分类映射
export const TOOL_CATEGORIES = {
  character: '角色管理',
  content: '内容生成',
  analysis: '分析评估',
  utility: '实用工具',
} as const;

// 工具分类颜色映射
export const TOOL_CATEGORY_COLORS = {
  character: 'bg-blue-100 text-blue-800',
  content: 'bg-green-100 text-green-800',
  analysis: 'bg-purple-100 text-purple-800',
  utility: 'bg-gray-100 text-gray-800',
} as const;
