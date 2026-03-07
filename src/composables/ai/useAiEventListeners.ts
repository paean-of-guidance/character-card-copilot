/**
 * AI 后端事件监听 Composable
 *
 * 封装所有后端事件监听逻辑，包括：
 * - 角色加载/更新/卸载
 * - 聊天历史加载
 * - 消息发送/接收
 * - 工具执行
 * - 上下文构建
 * - 错误处理
 * - Token 统计
 * - 进度更新
 */

import { ref, type Ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useAiStore } from '@/stores/ai';
import { useChatStore } from '@/stores/chat';
import { devLog } from '@/utils/logger';
import type {
  CharacterLoadedPayload,
  ChatHistoryLoadedPayload,
  MessageSentPayload,
  MessageReceivedPayload,
  ContextBuiltPayload,
  CharacterUpdatedPayload,
  ToolExecutedPayload,
  SessionUnloadedPayload,
  ErrorPayload,
  TokenStatsPayload,
  ProgressPayload
} from '@/types/events';
import type { ChatMessage } from '@/types/api';

/**
 * 前端显示消息类型
 */
export interface DisplayMessage extends Omit<ChatMessage, 'timestamp'> {
  id: string;
  timestamp: Date;
  isEditing?: boolean;
}

/**
 * 使用 AI 事件监听器
 *
 * @param messages - 消息列表响应式引用
 * @param contextBuiltInfo - 上下文构建信息响应式引用
 * @param isLoadingFromBackend - 后端加载状态响应式引用
 */
export function useAiEventListeners(
  messages: Ref<DisplayMessage[]>,
  contextBuiltInfo: Ref<any>,
  isLoadingFromBackend: Ref<boolean>
) {
  const aiStore = useAiStore();
  const chatStore = useChatStore();
  const eventUnlisteners = ref<(() => void)[]>([]);

  /**
   * 初始化所有后端事件监听器
   */
  async function setupListeners() {
    devLog('初始化后端事件监听器...');

    // 角色加载事件
    const unlistenCharacterLoaded = await listen<CharacterLoadedPayload>('character-loaded', (event) => {
      devLog('🎭 角色加载事件:', event.payload);
      const payload = event.payload;
      aiStore.updateSessionState(payload.uuid, true);
      isLoadingFromBackend.value = false;
    });

    // 聊天历史加载事件
    const unlistenChatHistoryLoaded = await listen<ChatHistoryLoadedPayload>('chat-history-loaded', (event) => {
      devLog('📚 聊天历史加载事件:', event.payload);
      const payload = event.payload;

      // 转换为前端消息格式
      messages.value = payload.chat_history.map((msg, index) => ({
        id: `${msg.timestamp || index}_${payload.uuid}`,
        role: msg.role,
        content: msg.content,
        timestamp: new Date((msg.timestamp || Date.now() / 1000) * 1000),
        tool_calls: msg.tool_calls,
        tool_call_id: msg.tool_call_id,
        name: msg.name,
      }));

      // 同步到 store
      chatStore.setChatHistory(payload.uuid, payload.chat_history);
      chatStore.setActiveCharacter(payload.uuid);

      devLog(`从后端加载了 ${messages.value.length} 条聊天历史记录`);
    });

    // 消息发送事件
    const unlistenMessageSent = await listen<MessageSentPayload>('message-sent', (event) => {
      devLog('📤 消息发送事件:', event.payload);
      const payload = event.payload;

      // 如果消息不在前端列表中，添加它
      const existingMessage = messages.value.find(msg =>
        msg.content === payload.message.content && msg.role === 'user'
      );

      if (!existingMessage) {
        const userMessageObj = {
          id: `${payload.message.timestamp}_sent_${payload.uuid}`,
          role: 'user' as const,
          content: payload.message.content,
          timestamp: new Date(payload.message.timestamp || Date.now()),
        };
        messages.value.push(userMessageObj);
      }
    });

    // 消息接收事件
    const unlistenMessageReceived = await listen<MessageReceivedPayload>('message-received', (event) => {
      devLog('📥 消息接收事件:', event.payload);
      const payload = event.payload;

      // 如果有中间消息（工具调用流程），先插入它们
      if (payload.intermediate_messages && payload.intermediate_messages.length > 0) {
        devLog(`🔄 插入 ${payload.intermediate_messages.length} 条中间消息（tool 调用流程）`);

        const intermediateDisplayMessages = payload.intermediate_messages.map((msg, index) => ({
          id: `${msg.timestamp || Date.now()}_intermediate_${index}_${payload.uuid}`,
          role: msg.role,
          content: msg.content,
          timestamp: new Date(msg.timestamp || Date.now()),
          tool_calls: msg.tool_calls,
          tool_call_id: msg.tool_call_id,
          name: msg.name,
        }));

        messages.value.push(...intermediateDisplayMessages);
      }

      // 添加最终的 AI 回复消息
      const aiMessageObj: DisplayMessage = {
        id: `${payload.message.timestamp}_received_${payload.uuid}`,
        role: 'assistant',
        content: payload.message.content,
        timestamp: new Date(payload.message.timestamp || Date.now()),
        tool_calls: payload.message.tool_calls,
        tool_call_id: payload.message.tool_call_id,
        name: payload.message.name,
      };
      messages.value.push(aiMessageObj);
    });

    // 上下文构建完成事件
    const unlistenContextBuilt = await listen<ContextBuiltPayload>('context-built', (event) => {
      devLog('🔧 上下文构建完成事件:', event.payload);
      const payload = event.payload;
      contextBuiltInfo.value = payload.context_result;
    });

    // 角色更新事件
    const unlistenCharacterUpdated = await listen<CharacterUpdatedPayload>('character-updated', (event) => {
      devLog('🔄 角色更新事件:', event.payload);
    });

    // 工具执行事件
    const unlistenToolExecuted = await listen<ToolExecutedPayload>('tool-executed', (event) => {
      const payload = event.payload;

      if (payload.success) {
        devLog('✅ 工具执行成功:', {
          工具名称: payload.tool_name,
          执行时间: `${payload.execution_time_ms}ms`,
          结果: payload.result
        });
      } else {
        console.error('❌ 工具执行失败:', {
          工具名称: payload.tool_name,
          错误: payload.error,
          执行时间: `${payload.execution_time_ms}ms`
        });
      }
    });

    // 会话卸载事件
    const unlistenSessionUnloaded = await listen<SessionUnloadedPayload>('session-unloaded', (event) => {
      devLog('🚪 会话卸载事件:', event.payload);
      const payload = event.payload;

      if (payload.uuid === aiStore.currentSessionUUID) {
        aiStore.clearSessionState();
        messages.value = [];
        contextBuiltInfo.value = null;
      }
    });

    // 错误事件
    const unlistenError = await listen<ErrorPayload>('error', (event) => {
      console.error('❌ 错误事件:', event.payload);
      const payload = event.payload;

      const errorMessageObj = {
        id: `error_${payload.timestamp}_${payload.uuid || 'unknown'}`,
        role: 'assistant' as const,
        content: `⚠️ 系统错误 [${payload.error_code}]: ${payload.error_message}`,
        timestamp: new Date(payload.timestamp),
      };

      messages.value.push(errorMessageObj);
    });

    // Token统计事件
    const unlistenTokenStats = await listen<TokenStatsPayload>('token-stats', (event) => {
      devLog('📊 Token统计事件:', event.payload);
      aiStore.updateTokenStats(event.payload.token_usage);
    });

    // 进度事件
    const unlistenProgress = await listen<ProgressPayload>('progress', (event) => {
      devLog('📈 进度事件:', event.payload);
    });

    // 保存所有清理函数
    eventUnlisteners.value.push(
      unlistenCharacterLoaded,
      unlistenChatHistoryLoaded,
      unlistenMessageSent,
      unlistenMessageReceived,
      unlistenContextBuilt,
      unlistenCharacterUpdated,
      unlistenToolExecuted,
      unlistenSessionUnloaded,
      unlistenError,
      unlistenTokenStats,
      unlistenProgress,
    );

    devLog('✅ 后端事件监听器初始化完成');
  }

  /**
   * 清理所有事件监听器
   */
  function cleanup() {
    devLog('清理事件监听器...');
    eventUnlisteners.value.forEach(unlisten => {
      try {
        unlisten();
      } catch (error) {
        console.error('清理事件监听器失败:', error);
      }
    });
    eventUnlisteners.value = [];
    devLog('✅ 事件监听器清理完成');
  }

  return {
    setupListeners,
    cleanup
  };
}
