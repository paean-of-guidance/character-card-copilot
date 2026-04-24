<template>
    <div
        class="min-w-0 rounded-2xl px-4 py-2.5 group relative"
        :class="
            role === 'user'
                ? 'rounded-br-sm max-w-[82%]'
                : 'rounded-bl-sm max-w-[95%]'
        "
        :style="
            role === 'user'
                ? 'background: rgba(99,102,241,0.25); border: 1px solid rgba(165,180,252,0.25);'
                : 'background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.10); backdrop-filter: blur(12px);'
        "
    >
        <!-- 消息内容 -->
        <template v-if="!isEditing">
            <!-- 思考过程 -->
            <div
                v-if="role === 'assistant' && (reasoningContent || reasoningLoading)"
                class="mb-3 rounded-xl border border-white/10 bg-white/5"
            >
                <button
                    class="flex w-full items-center justify-between px-3 py-2 text-left text-xs text-white/50 transition-colors hover:bg-white/5"
                    @click="$emit('toggleReasoning')"
                >
                    <span>{{ reasoningExpanded ? '▼' : '▶' }} 思考过程</span>
                    <span v-if="reasoningLoading" class="text-violet-400">思考中...</span>
                </button>
                <div v-if="reasoningExpanded" class="whitespace-pre-wrap px-3 pb-3 text-xs text-white/40">
                    {{ reasoningContent }}
                </div>
            </div>

            <MarkdownRenderer
                v-if="role === 'assistant' && content"
                :content="content"
                class="text-sm text-white/85"
            />
            <div v-else class="whitespace-pre-wrap text-sm" :class="role === 'user' ? 'text-white/90' : 'text-white/85'">
                {{ content }}
            </div>

            <div class="mt-1 text-xs opacity-60" :class="role === 'user' ? 'text-indigo-200' : 'text-white/40'">
                {{ formatTime(timestamp) }}
            </div>
            <div v-if="role === 'assistant' && loading" class="mt-1 text-xs text-violet-400">
                生成中...
            </div>
        </template>

        <!-- 编辑模式 -->
        <div v-else class="mt-2">
            <textarea
                v-model="editingContent"
                @keydown="handleEditKeydown"
                @blur="handleSaveEdit"
                class="liquid-textarea w-full resize-none text-sm"
                rows="3"
                placeholder="编辑消息内容..."
            ></textarea>
            <div class="mt-2 flex gap-2">
                <button @click="handleSaveEdit" class="glass-btn glass-btn--primary text-xs">保存</button>
                <button @click="handleCancelEdit" class="glass-btn glass-btn--neutral text-xs">取消</button>
            </div>
        </div>

        <!-- 消息操作按钮 -->
        <div
            v-if="!loading && !isEditing"
            class="absolute -bottom-6 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100"
            :class="role === 'user' ? 'left-0' : 'right-0'"
        >
            <button
                v-if="role === 'user' && isLastMessage"
                @click="$emit('continue')"
                class="rounded-full bg-indigo-500/20 p-1 text-indigo-300 transition-colors hover:bg-indigo-500/35"
                title="生成AI回复"
            >
                <MdSend class="h-4 w-4" />
            </button>
            <button
                v-if="role === 'assistant' && isLastMessage"
                @click="$emit('regenerate')"
                class="rounded-full bg-white/10 p-1 text-white/50 transition-colors hover:bg-white/18 hover:text-white/75"
                title="重新生成"
            >
                <MdOutlineRefresh class="h-4 w-4" />
            </button>
            <button
                @click="handleStartEdit"
                class="rounded-full bg-white/10 p-1 text-white/50 transition-colors hover:bg-white/18 hover:text-white/75"
                title="编辑消息"
            >
                <MdOutlineEdit class="h-4 w-4" />
            </button>
            <button
                @click="$emit('delete')"
                class="rounded-full bg-white/10 p-1 text-white/50 transition-colors hover:bg-red-500/25 hover:text-red-300"
                title="删除消息"
            >
                <MdOutlineDelete class="h-4 w-4" />
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import MarkdownRenderer from '../MarkdownRenderer.vue';
import {
    MdSend,
    MdOutlineRefresh,
    MdOutlineEdit,
    MdOutlineDelete,
} from 'vue-icons-plus/md';

interface Props {
    messageId: string;
    role: 'user' | 'assistant';
    content: string;
    reasoningContent?: string;
    reasoningExpanded?: boolean;
    reasoningLoading?: boolean;
    timestamp: Date;
    isEditing?: boolean;
    loading?: boolean;
    isLastMessage?: boolean;
}

interface Emits {
    continue: [];
    regenerate: [];
    startEdit: [];
    saveEdit: [newContent: string];
    cancelEdit: [];
    delete: [];
    toggleReasoning: [];
}

const props = withDefaults(defineProps<Props>(), {
    reasoningContent: '',
    reasoningExpanded: false,
    reasoningLoading: false,
    isEditing: false,
    loading: false,
    isLastMessage: false
});

const emit = defineEmits<Emits>();

const editingContent = ref('');

function formatTime(date: Date): string {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
}

function handleStartEdit() {
    editingContent.value = props.content;
    emit('startEdit');
}

function handleSaveEdit() {
    emit('saveEdit', editingContent.value.trim());
}

function handleCancelEdit() {
    emit('cancelEdit');
}

function handleEditKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
        event.preventDefault();
        handleSaveEdit();
    }
    if (event.key === 'Escape') {
        event.preventDefault();
        handleCancelEdit();
    }
}
</script>
