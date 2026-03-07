/**
 * 消息分组 Composable
 *
 * 将原始聊天消息映射为聊天时间线项：
 * 1. 普通 user / assistant 消息保持独立显示
 * 2. 工具调用改为低权重的时间线元信息行，而不是单独的大气泡卡片
 * 3. 保持严格的到达顺序，不对工具提示和助手正文做重排
 */

import { computed, type ComputedRef, type Ref } from 'vue';
import type { ToolCall } from '@/types/api';
import type { DisplayMessage } from './useAiEventListeners';

export type GroupedMessage =
  | { type: 'normal'; message: DisplayMessage }
  | {
      type: 'tool-meta';
      toolCallId: string;
      toolCall?: ToolCall;
      toolResult?: DisplayMessage;
      timestamp: Date;
    };

export function useMessageGrouping(messages: Ref<DisplayMessage[]>): ComputedRef<GroupedMessage[]> {
  return computed<GroupedMessage[]>(() => {
    const result: GroupedMessage[] = [];
    let i = 0;

    while (i < messages.value.length) {
      const msg = messages.value[i];

      if (msg.role === 'assistant' && msg.tool_calls && msg.tool_calls.length > 0) {
        const toolResultsById = new Map<string, DisplayMessage>();
        let j = i + 1;

        while (j < messages.value.length && messages.value[j].role === 'tool') {
          const toolResult = messages.value[j];

          if (toolResult.tool_call_id && !toolResultsById.has(toolResult.tool_call_id)) {
            toolResultsById.set(toolResult.tool_call_id, toolResult);
          }

          j += 1;
        }

        for (const toolCall of msg.tool_calls) {
          result.push({
            type: 'tool-meta',
            toolCallId: toolCall.id,
            toolCall,
            toolResult: toolResultsById.get(toolCall.id),
            timestamp: msg.timestamp,
          });
        }

        i = j;
        continue;
      }

      if (msg.role === 'tool') {
        result.push({
          type: 'tool-meta',
          toolCallId: msg.tool_call_id ?? msg.id,
          toolResult: msg,
          timestamp: msg.timestamp,
        });
        i += 1;
        continue;
      }

      result.push({
        type: 'normal',
        message: msg,
      });
      i += 1;
    }

    return result;
  });
}
