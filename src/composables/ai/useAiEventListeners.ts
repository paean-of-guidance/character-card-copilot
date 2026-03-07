/*
 * AI 后端事件监听 Composable
 *
 * - 消息发送/接收
 * - 流式消息增量
 * - 工具执行状态
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
  MessageReasoningDeltaPayload,
  MessageSentPayload,
  MessageReceivedPayload,
  MessageStreamDeltaPayload,
  ContextBuiltPayload,
  CharacterUpdatedPayload,
  ToolExecutionStatusPayload,
  ToolExecutedPayload,
  SessionUnloadedPayload,
  ErrorPayload,
  TokenStatsPayload,
  ProgressPayload,
} from '@/types/events';
import type { ChatMessage } from '@/types/api';

export interface DisplayMessage extends Omit<ChatMessage, 'timestamp'> {
  id: string;
  timestamp: Date;
  isEditing?: boolean;
  isStreaming?: boolean;
  reasoningContent?: string;
  reasoningExpanded?: boolean;
  isReasoningStreaming?: boolean;
  streamTargetId?: string;
  transientKind?: 'stream-assistant' | 'tool-call-carrier' | 'tool-status';
}

export function useAiEventListeners(
  messages: Ref<DisplayMessage[]>,
  contextBuiltInfo: Ref<any>,
  isLoadingFromBackend: Ref<boolean>
) {
  const aiStore = useAiStore();
  const chatStore = useChatStore();
  const eventUnlisteners = ref<(() => void)[]>([]);
  let activeAssistantTargetId: string | null = null;

  function toDisplayMessage(message: ChatMessage, fallbackId: string): DisplayMessage {
    return {
      id: fallbackId,
      role: message.role,
      content: message.content,
      timestamp: new Date(message.timestamp || Date.now()),
      tool_calls: message.tool_calls,
      tool_call_id: message.tool_call_id,
      name: message.name,
    };
  }

  function removeTransientMessages(targetMessageId: string) {
    messages.value = messages.value.filter((message) => message.streamTargetId !== targetMessageId);
  }

  function clearReasoningState(message: DisplayMessage) {
    delete message.reasoningContent;
    delete message.reasoningExpanded;
    delete message.isReasoningStreaming;
  }

  function clearPreviousReasoning(targetMessageId: string) {
    messages.value.forEach((message) => {
      const messageTargetId = message.streamTargetId ?? message.id;

      if (messageTargetId !== targetMessageId) {
        clearReasoningState(message);
      }
    });
  }

  function beginAssistantTurn(targetMessageId: string) {
    if (activeAssistantTargetId === targetMessageId) {
      return;
    }

    clearPreviousReasoning(targetMessageId);
    activeAssistantTargetId = targetMessageId;
  }

  function findStreamingAssistantMessage(targetMessageId: string) {
    return messages.value.find(
      (message) => message.streamTargetId === targetMessageId && message.transientKind === 'stream-assistant',
    );
  }

  function ensureStreamingAssistantMessage(targetMessageId: string, timestamp: number): DisplayMessage {
    const existingMessage = findStreamingAssistantMessage(targetMessageId);

    if (existingMessage) {
      return existingMessage;
    }

    const nextMessage: DisplayMessage = {
      id: targetMessageId,
      role: 'assistant',
      content: '',
      timestamp: new Date(timestamp),
      isStreaming: true,
      streamTargetId: targetMessageId,
      transientKind: 'stream-assistant',
    };

    messages.value.push(nextMessage);
    return nextMessage;
  }

  function ensureToolCarrierMessage(payload: ToolExecutionStatusPayload): DisplayMessage {
    let carrier = messages.value.find(
      (message) => message.streamTargetId === payload.target_message_id && message.transientKind === 'tool-call-carrier',
    );

    if (!carrier) {
      carrier = {
        id: `tool-carrier_${payload.target_message_id}`,
        role: 'assistant',
        content: '',
        timestamp: new Date(payload.timestamp),
        tool_calls: [],
        streamTargetId: payload.target_message_id,
        transientKind: 'tool-call-carrier',
      };
      messages.value.push(carrier);
    }

    if (payload.tool_call) {
      const toolCalls = carrier.tool_calls ?? [];
      if (!toolCalls.some((toolCall) => toolCall.id === payload.tool_call_id)) {
        toolCalls.push(payload.tool_call);
        carrier.tool_calls = toolCalls;
      }
    }

    return carrier;
  }

  function upsertToolStatusMessage(payload: ToolExecutionStatusPayload) {
    const content = JSON.stringify(
      {
        phase: payload.phase,
        tool_name: payload.tool_name,
        result: payload.result,
        error: payload.error,
        execution_time_ms: payload.execution_time_ms,
      },
      null,
      2,
    );

    const nextMessage: DisplayMessage = {
      id: `tool-status_${payload.tool_call_id}`,
      role: 'tool',
      content,
      timestamp: new Date(payload.timestamp),
      name: payload.tool_name,
      tool_call_id: payload.tool_call_id,
      streamTargetId: payload.target_message_id,
      transientKind: 'tool-status',
    };

    const existingIndex = messages.value.findIndex(
      (message) => message.streamTargetId === payload.target_message_id && message.tool_call_id === payload.tool_call_id,
    );

    if (existingIndex >= 0) {
      messages.value[existingIndex] = {
        ...messages.value[existingIndex],
        ...nextMessage,
      };
      return;
    }

    const carrierIndex = messages.value.findIndex(
      (message) => message.streamTargetId === payload.target_message_id && message.transientKind === 'tool-call-carrier',
    );

    if (carrierIndex >= 0) {
      messages.value.splice(carrierIndex + 1, 0, nextMessage);
    } else {
      messages.value.push(nextMessage);
    }
  }

  async function setupListeners() {
    devLog('初始化后端事件监听器...');

    const unlistenCharacterLoaded = await listen<CharacterLoadedPayload>('character-loaded', (event) => {
      devLog('🎭 角色加载事件:', event.payload);
      const payload = event.payload;
      aiStore.updateSessionState(payload.uuid, true);
      isLoadingFromBackend.value = false;
    });

    const unlistenChatHistoryLoaded = await listen<ChatHistoryLoadedPayload>('chat-history-loaded', (event) => {
      devLog('📚 聊天历史加载事件:', event.payload);
      const payload = event.payload;
      activeAssistantTargetId = null;

      messages.value = payload.chat_history.map((msg, index) => ({
        id: `${msg.timestamp || index}_${payload.uuid}`,
        role: msg.role,
        content: msg.content,
        timestamp: new Date((msg.timestamp || Date.now() / 1000) * 1000),
        tool_calls: msg.tool_calls,
        tool_call_id: msg.tool_call_id,
        name: msg.name,
      }));

      chatStore.setChatHistory(payload.uuid, payload.chat_history);
      chatStore.setActiveCharacter(payload.uuid);

      devLog(`从后端加载了 ${messages.value.length} 条聊天历史记录`);
    });

    const unlistenMessageSent = await listen<MessageSentPayload>('message-sent', (event) => {
      devLog('📤 消息发送事件:', event.payload);
      const payload = event.payload;

      const existingMessage = messages.value.find(
        (message) => message.content === payload.message.content && message.role === 'user',
      );

      if (!existingMessage) {
        messages.value.push({
          id: `${payload.message.timestamp}_sent_${payload.uuid}`,
          role: 'user',
          content: payload.message.content,
          timestamp: new Date(payload.message.timestamp || Date.now()),
        });
      }
    });

    const unlistenMessageStreamDelta = await listen<MessageStreamDeltaPayload>('message-stream-delta', (event) => {
      const payload = event.payload;

      if (payload.role !== 'assistant') {
        return;
      }

      if (payload.is_aborted) {
        removeTransientMessages(payload.target_message_id);
        if (activeAssistantTargetId === payload.target_message_id) {
          activeAssistantTargetId = null;
        }
        return;
      }

      beginAssistantTurn(payload.target_message_id);

      const existingMessage = ensureStreamingAssistantMessage(payload.target_message_id, payload.timestamp);

      existingMessage.content += payload.delta;
      existingMessage.timestamp = new Date(payload.timestamp);
      existingMessage.isStreaming = !payload.is_finished;
    });

    const unlistenMessageReasoningDelta = await listen<MessageReasoningDeltaPayload>('message-reasoning-delta', (event) => {
      const payload = event.payload;

      if (payload.is_aborted) {
        const targetMessage = messages.value.find((message) => message.id === payload.target_message_id);

        if (targetMessage) {
          clearReasoningState(targetMessage);
        }

        const transientMessage = findStreamingAssistantMessage(payload.target_message_id);
        if (transientMessage) {
          clearReasoningState(transientMessage);
        }

        if (activeAssistantTargetId === payload.target_message_id) {
          activeAssistantTargetId = null;
        }

        return;
      }

      beginAssistantTurn(payload.target_message_id);

      const targetMessage = ensureStreamingAssistantMessage(payload.target_message_id, payload.timestamp);
      targetMessage.reasoningContent = `${targetMessage.reasoningContent ?? ''}${payload.delta}`;
      targetMessage.timestamp = new Date(payload.timestamp);
      targetMessage.isReasoningStreaming = !payload.is_finished;

      if (targetMessage.reasoningExpanded === undefined) {
        targetMessage.reasoningExpanded = false;
      }
    });

    const unlistenMessageReceived = await listen<MessageReceivedPayload>('message-received', (event) => {
      devLog('📥 消息接收事件:', event.payload);
      const payload = event.payload;

      const transientMessage = payload.target_message_id
        ? findStreamingAssistantMessage(payload.target_message_id)
        : undefined;

      if (payload.target_message_id) {
        removeTransientMessages(payload.target_message_id);

        if (activeAssistantTargetId === payload.target_message_id) {
          activeAssistantTargetId = payload.target_message_id;
        }
      }

      if (payload.intermediate_messages && payload.intermediate_messages.length > 0) {
        devLog(`🔄 插入 ${payload.intermediate_messages.length} 条中间消息（tool 调用流程）`);

        const intermediateDisplayMessages = payload.intermediate_messages.map((msg, index) =>
          toDisplayMessage(msg, `${msg.timestamp || Date.now()}_intermediate_${index}_${payload.uuid}`),
        );

        messages.value.push(...intermediateDisplayMessages);
      }

      const finalMessage = toDisplayMessage(
        payload.message,
        payload.target_message_id || `${payload.message.timestamp}_received_${payload.uuid}`,
      );

      if (transientMessage?.reasoningContent) {
        finalMessage.reasoningContent = transientMessage.reasoningContent;
        finalMessage.reasoningExpanded = transientMessage.reasoningExpanded ?? false;
        finalMessage.isReasoningStreaming = false;
      }

      messages.value.push(finalMessage);
    });

    const unlistenContextBuilt = await listen<ContextBuiltPayload>('context-built', (event) => {
      devLog('🔧 上下文构建完成事件:', event.payload);
      const payload = event.payload;
      contextBuiltInfo.value = payload.context_result;
    });

    const unlistenCharacterUpdated = await listen<CharacterUpdatedPayload>('character-updated', (event) => {
      devLog('🔄 角色更新事件:', event.payload);
    });

    const unlistenToolExecutionStatus = await listen<ToolExecutionStatusPayload>('tool-execution-status', (event) => {
      const payload = event.payload;
      ensureToolCarrierMessage(payload);
      upsertToolStatusMessage(payload);
    });

    const unlistenToolExecuted = await listen<ToolExecutedPayload>('tool-executed', (event) => {
      const payload = event.payload;

      if (payload.success) {
        devLog('✅ 工具执行成功:', {
          工具名称: payload.tool_name,
          执行时间: `${payload.execution_time_ms}ms`,
          结果: payload.result,
        });
      } else {
        console.error('❌ 工具执行失败:', {
          工具名称: payload.tool_name,
          错误: payload.error,
          执行时间: `${payload.execution_time_ms}ms`,
        });
      }
    });

    const unlistenSessionUnloaded = await listen<SessionUnloadedPayload>('session-unloaded', (event) => {
      devLog('🚪 会话卸载事件:', event.payload);
      const payload = event.payload;

      if (payload.uuid === aiStore.currentSessionUUID) {
        activeAssistantTargetId = null;
        aiStore.clearSessionState();
        messages.value = [];
        contextBuiltInfo.value = null;
      }
    });

    const unlistenError = await listen<ErrorPayload>('error', (event) => {
      console.error('❌ 错误事件:', event.payload);
      const payload = event.payload;

      messages.value.push({
        id: `error_${payload.timestamp}_${payload.uuid || 'unknown'}`,
        role: 'assistant',
        content: `⚠️ 系统错误 [${payload.error_code}]: ${payload.error_message}`,
        timestamp: new Date(payload.timestamp),
      });
    });

    const unlistenTokenStats = await listen<TokenStatsPayload>('token-stats', (event) => {
      devLog('📊 Token统计事件:', event.payload);
      aiStore.updateTokenStats(event.payload.token_usage);
    });

    const unlistenProgress = await listen<ProgressPayload>('progress', (event) => {
      devLog('📈 进度事件:', event.payload);
    });

    eventUnlisteners.value.push(
      unlistenCharacterLoaded,
      unlistenChatHistoryLoaded,
      unlistenMessageSent,
      unlistenMessageStreamDelta,
      unlistenMessageReasoningDelta,
      unlistenMessageReceived,
      unlistenContextBuilt,
      unlistenCharacterUpdated,
      unlistenToolExecutionStatus,
      unlistenToolExecuted,
      unlistenSessionUnloaded,
      unlistenError,
      unlistenTokenStats,
      unlistenProgress,
    );

    devLog('✅ 后端事件监听器初始化完成');
  }

  function cleanup() {
    devLog('清理事件监听器...');
    eventUnlisteners.value.forEach((unlisten) => {
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
    cleanup,
  };
}
