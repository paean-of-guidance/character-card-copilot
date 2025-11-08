<template>
    <div
        class="max-w-[80%] px-4 py-2 rounded-lg group relative"
        :class="
            role === 'user'
                ? 'bg-blue-500 text-white rounded-br-sm'
                : 'bg-white border border-gray-200 text-gray-800 rounded-bl-sm'
        "
    >
        <!-- 消息内容 -->
        <template v-if="!isEditing">
            <MarkdownRenderer
                v-if="role === 'assistant'"
                :content="content"
                class="text-sm"
            />
            <div v-else class="text-sm whitespace-pre-wrap">
                {{ content }}
            </div>
            <div
                class="text-xs mt-1 opacity-70"
                :class="
                    role === 'user'
                        ? 'text-blue-100'
                        : 'text-gray-500'
                "
            >
                {{ formatTime(timestamp) }}
            </div>
        </template>

        <!-- 编辑模式 -->
        <div v-else class="mt-2">
            <textarea
                v-model="editingContent"
                @keydown="handleEditKeydown"
                @blur="handleSaveEdit"
                class="w-full p-2 border border-gray-300 rounded text-sm resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                rows="3"
                placeholder="编辑消息内容..."
            ></textarea>
            <div class="flex gap-2 mt-2">
                <button
                    @click="handleSaveEdit"
                    class="text-xs bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 transition-colors"
                >
                    保存
                </button>
                <button
                    @click="handleCancelEdit"
                    class="text-xs bg-gray-300 text-gray-700 px-3 py-1 rounded hover:bg-gray-400 transition-colors"
                >
                    取消
                </button>
            </div>
        </div>

        <!-- 消息操作按钮 -->
        <div
            v-if="!loading && !isEditing"
            class="absolute -bottom-6 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1"
            :class="
                role === 'user'
                    ? 'left-0'
                    : 'right-0'
            "
        >
            <!-- 用户消息：生成AI回复按钮（仅最后一条显示） -->
            <button
                v-if="role === 'user' && isLastMessage"
                @click="$emit('continue')"
                class="p-1 bg-blue-100 hover:bg-blue-200 rounded-full transition-colors"
                title="生成AI回复"
            >
                <MdSend class="w-4 h-4 text-blue-600" />
            </button>

            <!-- AI消息：重新生成按钮 -->
            <button
                v-if="role === 'assistant' && isLastMessage"
                @click="$emit('regenerate')"
                class="p-1 bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
                title="重新生成"
            >
                <MdOutlineRefresh class="w-4 h-4 text-gray-600" />
            </button>

            <!-- 编辑按钮 -->
            <button
                @click="handleStartEdit"
                class="p-1 bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
                title="编辑消息"
            >
                <MdOutlineEdit class="w-4 h-4 text-gray-600" />
            </button>

            <!-- 删除按钮 -->
            <button
                @click="$emit('delete')"
                class="p-1 bg-gray-100 hover:bg-red-100 rounded-full transition-colors"
                title="删除消息"
            >
                <MdOutlineDelete class="w-4 h-4 text-gray-600 hover:text-red-600" />
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
    /** 消息ID */
    messageId: string;
    /** 消息角色 */
    role: 'user' | 'assistant';
    /** 消息内容 */
    content: string;
    /** 时间戳 */
    timestamp: Date;
    /** 是否正在编辑 */
    isEditing?: boolean;
    /** 是否加载中 */
    loading?: boolean;
    /** 是否最后一条消息 */
    isLastMessage?: boolean;
}

interface Emits {
    /** 继续生成AI回复 */
    continue: [];
    /** 重新生成 */
    regenerate: [];
    /** 开始编辑 */
    startEdit: [];
    /** 保存编辑 */
    saveEdit: [newContent: string];
    /** 取消编辑 */
    cancelEdit: [];
    /** 删除消息 */
    delete: [];
}

const props = withDefaults(defineProps<Props>(), {
    isEditing: false,
    loading: false,
    isLastMessage: false
});

const emit = defineEmits<Emits>();

// 编辑状态
const editingContent = ref('');

/**
 * 格式化时间
 */
function formatTime(date: Date): string {
    return date.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
    });
}

/**
 * 开始编辑
 */
function handleStartEdit() {
    editingContent.value = props.content;
    emit('startEdit');
}

/**
 * 保存编辑
 */
function handleSaveEdit() {
    const newContent = editingContent.value.trim();
    emit('saveEdit', newContent);
}

/**
 * 取消编辑
 */
function handleCancelEdit() {
    emit('cancelEdit');
}

/**
 * 处理编辑框键盘事件
 */
function handleEditKeydown(event: KeyboardEvent) {
    // Ctrl/Cmd + Enter 保存
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
        event.preventDefault();
        handleSaveEdit();
    }
    // Escape 取消
    if (event.key === 'Escape') {
        event.preventDefault();
        handleCancelEdit();
    }
}
</script>
