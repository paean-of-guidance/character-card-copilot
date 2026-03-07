<template>
    <div class="min-w-0 max-w-[80%] text-left">
        <button
            type="button"
            class="group inline-flex max-w-full items-center gap-1.5 rounded-md px-1 py-0.5 text-xs text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700"
            :aria-expanded="expanded"
            :aria-label="expanded ? '收起工具调用详情' : '展开工具调用详情'"
            @click="expanded = !expanded"
        >
            <MdChevronRight
                :class="[
                    'h-4 w-4 shrink-0 text-gray-400 transition-transform duration-200',
                    expanded && 'rotate-90',
                ]"
            />
            <MdBuild class="h-3.5 w-3.5 shrink-0 text-gray-400" />
            <span class="min-w-0 truncate">{{ summaryText }}</span>
        </button>

        <div
            v-if="expanded"
            class="mt-2 ml-5 w-full max-w-3xl min-w-0 rounded-2xl border border-gray-200 bg-gray-100/90 p-3 text-xs text-gray-700 shadow-sm"
        >
            <div class="space-y-3">
                <section>
                    <div class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-gray-500">
                        ID
                    </div>
                    <pre class="tool-block">{{ toolCallId }}</pre>
                </section>

                <section>
                    <div class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-gray-500">
                        ARGS
                    </div>
                    <pre class="tool-block">{{ argsText }}</pre>
                </section>

                <section>
                    <div class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-gray-500">
                        RESULT
                    </div>
                    <pre class="tool-block">{{ resultText }}</pre>
                </section>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { MdBuild, MdChevronRight } from 'vue-icons-plus/md';
import type { ToolCall } from '@/types/api';
import type { DisplayMessage } from '@/composables/ai/useAiEventListeners';

interface ToolPayload {
    phase?: 'started' | 'succeeded' | 'failed';
    tool_name?: string;
    result?: unknown;
    data?: unknown;
    error?: string | null;
    execution_time_ms?: number;
    success?: boolean;
}

interface Props {
    toolCall?: ToolCall;
    toolResult?: DisplayMessage;
    timestamp?: Date;
}

const props = defineProps<Props>();

const expanded = ref(false);

const parsedPayload = computed<ToolPayload | null>(() => {
    const content = props.toolResult?.content;

    if (!content) {
        return null;
    }

    try {
        return JSON.parse(content) as ToolPayload;
    } catch {
        return null;
    }
});

const toolName = computed(() => {
    return (
        props.toolCall?.function.name ||
        props.toolResult?.name ||
        parsedPayload.value?.tool_name ||
        '工具'
    );
});

const toolCallId = computed(() => {
    return props.toolCall?.id || props.toolResult?.tool_call_id || 'unknown';
});

const phase = computed<'started' | 'succeeded' | 'failed'>(() => {
    const payload = parsedPayload.value;

    if (payload?.phase === 'started' || payload?.phase === 'succeeded' || payload?.phase === 'failed') {
        return payload.phase;
    }

    if (typeof payload?.success === 'boolean') {
        return payload.success ? 'succeeded' : 'failed';
    }

    const content = props.toolResult?.content?.toLowerCase() || '';

    if (content.includes('"phase": "started"')) {
        return 'started';
    }

    if (content.includes('error') || content.includes('failed') || content.includes('exception')) {
        return 'failed';
    }

    return 'succeeded';
});

const durationText = computed(() => {
    const durationMs = parsedPayload.value?.execution_time_ms;

    if (typeof durationMs !== 'number' || Number.isNaN(durationMs) || durationMs < 0) {
        return '';
    }

    if (durationMs < 1000) {
        return `${durationMs}ms`;
    }

    return `${(durationMs / 1000).toFixed(durationMs >= 10_000 ? 0 : 1)}s`;
});

const summaryText = computed(() => {
    const parts = [
        phase.value === 'started'
            ? `正在使用 ${toolName.value} 工具`
            : `已使用 ${toolName.value} 工具`,
    ];

    if (durationText.value) {
        parts.push(durationText.value);
    }

    if (phase.value === 'failed') {
        parts.push('失败');
    }

    return parts.join(' · ');
});

const argsText = computed(() => {
    const rawArgs = props.toolCall?.function.arguments;

    if (!rawArgs) {
        return '无参数';
    }

    return formatValue(rawArgs);
});

const resultText = computed(() => {
    if (phase.value === 'started' && !parsedPayload.value?.result && !parsedPayload.value?.data && !parsedPayload.value?.error) {
        return '执行中...';
    }

    if (parsedPayload.value?.result !== undefined) {
        return formatValue(parsedPayload.value.result);
    }

    if (parsedPayload.value?.data !== undefined) {
        return formatValue(parsedPayload.value.data);
    }

    if (parsedPayload.value?.error) {
        return String(parsedPayload.value.error);
    }

    if (props.toolResult?.content) {
        return formatValue(props.toolResult.content);
    }

    return '暂无结果';
});

function formatValue(value: unknown): string {
    if (typeof value === 'string') {
        try {
            return JSON.stringify(JSON.parse(value), null, 2);
        } catch {
            return value;
        }
    }

    if (value === null || value === undefined) {
        return String(value);
    }

    try {
        return JSON.stringify(value, null, 2);
    } catch {
        return String(value);
    }
}
</script>

<style scoped>
.tool-block {
    width: 100%;
    max-width: 100%;
    max-height: 16rem;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
    word-wrap: break-word;
    border-radius: 0.875rem;
    border: 1px solid #e5e7eb;
    background: rgba(255, 255, 255, 0.75);
    padding: 0.75rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    line-height: 1.5;
    box-sizing: border-box;
}
</style>
