import type { CharacterData } from './character'
import type { ChatMessage } from './api'

// 角色加载事件载荷
export interface CharacterLoadedPayload {
  uuid: string
  character_data: CharacterData
  timestamp: number
}

// 聊天历史加载事件载荷
export interface ChatHistoryLoadedPayload {
  uuid: string
  chat_history: ChatMessage[]
  message_count: number
  timestamp: number
}

// 消息发送事件载荷
export interface MessageSentPayload {
  uuid: string
  message: ChatMessage
  timestamp: number
}

// 消息接收事件载荷
export interface MessageReceivedPayload {
  uuid: string
  message: ChatMessage
  timestamp: number
  /** 中间消息（包括 assistant with tool_calls 和 tool results） */
  intermediate_messages?: ChatMessage[]
}

// 上下文构建完成事件载荷
export interface ContextBuiltPayload {
  uuid: string
  context_result: BuiltContextResult
  timestamp: number
}

// 角色更新事件载荷
export interface CharacterUpdatedPayload {
  uuid: string
  character_data: CharacterData
  update_type: CharacterUpdateType
  timestamp: number
}

// 角色更新类型
export type CharacterUpdateType =
  | 'BasicInfo'
  | 'Worldbook'
  | 'Tags'
  | 'FullData'
  | { Fields: { fields: string[] } }

// 工具执行事件载荷
export interface ToolExecutedPayload {
  uuid: string
  tool_name: string
  success: boolean
  result?: any
  error?: string
  execution_time_ms: number
  timestamp: number
}

// 会话卸载事件载荷
export interface SessionUnloadedPayload {
  uuid: string
  session_info: SessionInfo
  reason: SessionUnloadReason
  timestamp: number
}

// 会话卸载原因
export type SessionUnloadReason =
  | 'UserRequest'
  | 'Expired'
  | 'MemoryCleanup'
  | 'AppShutdown'
  | { Error: string }

// 错误事件载荷
export interface ErrorPayload {
  uuid?: string
  error_code: string
  error_message: string
  error_context?: any
  timestamp: number
}

// Token统计事件载荷
export interface TokenStatsPayload {
  uuid: string
  token_usage: TokenUsageStats
  timestamp: number
}

// Token使用统计
export interface TokenUsageStats {
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
  context_tokens: number
  budget_utilization: number
}

// 进度事件载荷
export interface ProgressPayload {
  uuid: string
  operation: string
  progress: number // 0.0 - 1.0
  message?: string
  timestamp: number
}

// 会话信息
export interface SessionInfo {
  uuid: string
  character_name?: string
  message_count: number
  last_active: string
  status: SessionStatus
  last_context_tokens: number
}

// 会话状态
export type SessionStatus = 'Active' | 'Paused' | 'Loading' | { Error: string }

// Token分配详情
export interface TokenAllocation {
  character: number
  worldbook: number
  system: number
  history: number
}

// 构建完成的上下文结果
export interface BuiltContextResult {
  system_messages: any[] // OpenAIMessage[]
  assistant_messages: any[] // OpenAIMessage[]
  history_messages: any[] // OpenAIMessage[]
  current_user_message?: any // OpenAIMessage
  total_tokens: number
  token_allocation: TokenAllocation
  was_truncated: boolean
}

// OpenAI消息结构
export interface OpenAIMessage {
  role: string
  content: string
  name?: string
  tool_calls?: any[] // ToolCall[]
  tool_call_id?: string
}