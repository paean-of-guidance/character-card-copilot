<template>
    <div class="min-w-0 max-w-[80%] text-left">
        <button
            type="button"
            class="group inline-flex max-w-full items-center gap-1.5 rounded-md px-1 py-0.5 text-xs text-white/40 transition-colors hover:bg-white/8 hover:text-white/65"
            :aria-expanded="expanded"
            :aria-label="expanded ? '收起工具调用详情' : '展开工具调用详情'"
            @click="expanded = !expanded"
        >
            <MdChevronRight
                :class="[
                    'h-4 w-4 shrink-0 text-white/30 transition-transform duration-200',
                    expanded && 'rotate-90',
                ]"
            />
            <MdBuild class="h-3.5 w-3.5 shrink-0 text-white/30" />
            <span class="min-w-0 truncate">{{ summaryText }}</span>
        </button>

        <div
            v-if="expanded"
            class="mt-2 ml-5 w-full max-w-3xl min-w-0 rounded-2xl border border-white/10 bg-white/5 p-3 text-xs text-white/70 shadow-sm"
        >
            <div class="space-y-3">
                <section>
                    <div class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-white/35">
                        ID
                    </div>
                    <pre class="tool-block">{{ toolCallId }}</pre>
                </section>

                <section>
                    <div class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-white/35">
                        ARGS
                    </div>
                    <pre class="tool-block">{{ argsText }}</pre>
                </section>

                <section>
                    <div class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-white/35">
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

const parsedArgs = computed<unknown>(() => {
    const rawArgs = props.toolCall?.function.arguments;

    if (!rawArgs) {
        return null;
    }

    try {
        return JSON.parse(rawArgs);
    } catch {
        return rawArgs;
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
    if (!parsedArgs.value) {
        return '无参数';
    }

    return stripMarkdownFences(formatArgumentDisplay(parsedArgs.value));
});

const resultText = computed(() => {
    if (phase.value === 'started' && !parsedPayload.value?.result && !parsedPayload.value?.data && !parsedPayload.value?.error) {
        return '执行中...';
    }

    const structuredResult = parsedPayload.value?.result ?? parsedPayload.value?.data;
    if (structuredResult !== undefined) {
        return stripMarkdownFences(
            formatToolDisplay(toolName.value, structuredResult, parsedPayload.value?.error ?? null),
        );
    }

    if (parsedPayload.value?.error) {
        return String(parsedPayload.value.error);
    }

    if (props.toolResult?.content) {
        return stripMarkdownFences(props.toolResult.content);
    }

    return '暂无结果';
});

function formatArgumentDisplay(value: unknown): string {
    return formatYamlLike(value);
}

function formatToolDisplay(tool: string, value: unknown, error?: string | null): string {
    if (tool === 'patch_character_field') {
        return formatPatchDisplay(value, error);
    }

    if (tool === 'read_character_field') {
        return formatReadFieldDisplay(value, error);
    }

    if (tool === 'list_world_book_entries' || tool === 'read_world_book_entry') {
        return formatYamlLike(value);
    }

    if (error) {
        return [`failed ${tool}`, error, formatYamlLike(value)].filter(Boolean).join('\n');
    }

    return formatYamlLike(value);
}

function formatPatchDisplay(value: unknown, error?: string | null): string {
    const record = asRecord(value);

    if (!record) {
        return error ? `failed patch_character_field\n${error}` : formatYamlLike(value);
    }

    const lines: string[] = [];
    const meta = [
        record.field ? `field=${String(record.field)}` : '',
        record.operation ? `op=${String(record.operation)}` : '',
        record.match_mode ? `match=${String(record.match_mode)}` : '',
        typeof record.dry_run === 'boolean' ? `dry_run=${String(record.dry_run)}` : '',
    ].filter(Boolean);

    if (error) {
        lines.push('failed patch_character_field');
        lines.push(error);
    } else {
        lines.push('ok patch_character_field');
    }

    if (meta.length > 0) {
        lines.push(meta.join(' | '));
    }

    const matchedContext = asRecord(record.matched_context);
    const updatedContext = asRecord(record.updated_context);
    const beforeLines = buildContextDiffLines(matchedContext, 'matched_text', '-');
    const afterLines = buildContextDiffLines(updatedContext, 'selected_text', '+');

    if (beforeLines.length > 0 || afterLines.length > 0) {
        lines.push(...beforeLines, ...afterLines);
        return lines.join('\n');
    }

    return [...lines, formatYamlLike(value)].join('\n');
}

function formatReadFieldDisplay(value: unknown, error?: string | null): string {
    const record = asRecord(value);

    if (!record) {
        return error ? `failed read_character_field\n${error}` : formatYamlLike(value);
    }

    const field = typeof record.field === 'string' ? record.field : 'text';
    const start = typeof record.start === 'number' ? record.start : 0;
    const end = typeof record.end === 'number' ? record.end : 0;
    const total = typeof record.total_length === 'number' ? record.total_length : end;
    const truncated = typeof record.truncated === 'boolean' && record.truncated;
    const text = typeof record.text === 'string' ? record.text : '';

    const lines = [
        error ? 'failed read_character_field' : 'ok read_character_field',
        error || `field=${field} | range=${start}..${end}/${total}${truncated ? ' | truncated' : ''}`,
    ];

    if (text) {
        lines.push(
            text
                .split(/\r?\n/)
                .map((line, index) => `${index + 1} ${line}`)
                .join('\n'),
        );
    }

    return lines.filter(Boolean).join('\n');
}

function buildContextDiffLines(
    context: Record<string, unknown> | null,
    focusKey: 'matched_text' | 'selected_text',
    prefix: '-' | '+',
): string[] {
    if (!context) {
        return [];
    }

    const before = typeof context.context_before === 'string' ? context.context_before : '';
    const focus = typeof context[focusKey] === 'string' ? String(context[focusKey]) : '';
    const after = typeof context.context_after === 'string' ? context.context_after : '';
    const combined = `${before}${focus}${after}`.trim();

    if (!combined) {
        return [];
    }

    return combined
        .split(/\r?\n/)
        .slice(0, 6)
        .map((line) => `${prefix} ${line}`);
}

function formatYamlLike(value: unknown, indent = 0): string {
    if (typeof value === 'string') {
        return value;
    }

    if (value === null || value === undefined) {
        return String(value);
    }

    if (typeof value === 'number' || typeof value === 'boolean') {
        return String(value);
    }

    if (Array.isArray(value)) {
        return value
            .map((item) => {
                const rendered = formatYamlLike(item, indent + 2);
                if (!rendered.includes('\n')) {
                    return `${' '.repeat(indent)}- ${rendered}`;
                }

                const [firstLine, ...restLines] = rendered.split('\n');
                return [
                    `${' '.repeat(indent)}- ${firstLine}`,
                    ...restLines.map((line) => `${' '.repeat(indent + 2)}${line}`),
                ].join('\n');
            })
            .join('\n');
    }

    if (typeof value === 'object') {
        return Object.entries(value as Record<string, unknown>)
            .map(([key, nestedValue]) => {
                const rendered = formatYamlLike(nestedValue, indent + 2);
                if (!rendered.includes('\n')) {
                    return `${' '.repeat(indent)}${key}: ${rendered}`;
                }

                return `${' '.repeat(indent)}${key}:\n${rendered}`;
            })
            .join('\n');
    }

    return String(value);
}

function asRecord(value: unknown): Record<string, unknown> | null {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        return null;
    }

    return value as Record<string, unknown>;
}

function stripMarkdownFences(value: string): string {
    return value
        .split(/\r?\n/)
        .filter((line) => !line.trimStart().startsWith('```'))
        .join('\n')
        .trim();
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
    border: 1px solid rgba(255, 255, 255, 0.10);
    background: rgba(255, 255, 255, 0.04);
    color: rgba(255, 255, 255, 0.75);
    padding: 0.75rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    line-height: 1.5;
    box-sizing: border-box;
}
</style>
