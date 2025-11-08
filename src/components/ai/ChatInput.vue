<template>
    <div class="flex gap-3">
        <textarea
            ref="textareaRef"
            v-model="userInput"
            @input="handleInput"
            @keydown="handleKeydown"
            :disabled="disabled"
            placeholder="输入消息... (Enter发送，Shift+Enter换行)"
            class="flex-1 resize-none border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed overflow-hidden"
            style="
                height: 40px;
                min-height: 40px;
                max-height: 120px;
                line-height: 24px;
            "
        ></textarea>

        <button
            @click="handleSend"
            :disabled="!userInput.trim() || disabled"
            class="bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 text-white px-4 py-2 rounded-lg transition-colors flex items-center justify-center self-end"
            title="发送消息"
            style="height: 40px"
        >
            <svg
                v-if="!loading"
                class="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                />
            </svg>
            <div
                v-else
                class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
            ></div>
        </button>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';

interface Props {
    /** 是否禁用输入 */
    disabled?: boolean;
    /** 是否加载中 */
    loading?: boolean;
    /** 命令面板是否打开 */
    commandPaletteOpen?: boolean;
}

interface Emits {
    /** 发送消息 */
    send: [message: string];
    /** 打开命令面板 */
    openCommandPalette: [];
    /** 键盘事件（用于命令面板导航） */
    keydown: [event: KeyboardEvent];
    /** 输入变化 */
    input: [value: string];
}

const props = withDefaults(defineProps<Props>(), {
    disabled: false,
    loading: false,
    commandPaletteOpen: false
});

const emit = defineEmits<Emits>();

// 状态
const userInput = ref('');
const textareaRef = ref<HTMLTextAreaElement>();
const inputRows = ref(1);

/**
 * 自动调整输入框高度
 */
function adjustTextareaHeight() {
    nextTick(() => {
        if (textareaRef.value) {
            const textarea = textareaRef.value;
            const lineHeight = 24; // 行高24px
            const maxRows = 5;
            const maxHeight = lineHeight * maxRows;

            // 先重置高度为最小高度
            textarea.style.height = '40px';

            // 获取实际需要的行数
            const lines = textarea.value.split('\n').length;

            // 只有当内容包含换行符或者内容长度超过一行时才调整高度
            if (lines > 1 || textarea.value.length > 60) {
                const scrollHeight = textarea.scrollHeight;
                const newHeight = Math.min(scrollHeight, maxHeight);
                textarea.style.height = newHeight + 'px';
                inputRows.value = Math.min(lines, maxRows);
            } else {
                // 保持最小高度
                textarea.style.height = '40px';
                inputRows.value = 1;
            }
        }
    });
}

/**
 * 处理用户输入
 */
function handleInput() {
    // 发送输入变化事件
    emit('input', userInput.value);

    // 只有当输入内容包含换行符时才调整高度
    if (userInput.value.includes('\n') || userInput.value.length > 80) {
        adjustTextareaHeight();
    } else {
        // 如果没有换行符且长度较短，保持最小高度
        if (textareaRef.value) {
            textareaRef.value.style.height = '40px';
        }
        inputRows.value = 1;
    }
}

/**
 * 处理键盘事件
 */
function handleKeydown(event: KeyboardEvent) {
    // 如果命令面板打开，将键盘事件委托给父组件处理
    if (props.commandPaletteOpen) {
        if (
            ['ArrowUp', 'ArrowDown', 'Enter', 'Tab', ' ', 'Escape'].includes(
                event.key
            )
        ) {
            emit('keydown', event);
            return;
        }
    }

    // 检测"/"键触发命令面板
    // 当且仅当输入框完全为空时，按下"/"才触发命令面板
    if (event.key === '/' && userInput.value === '') {
        event.preventDefault();
        emit('openCommandPalette');
        return;
    }

    // 普通发送消息逻辑（Shift+Enter换行，Enter发送）
    if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        handleSend();
    }
}

/**
 * 发送消息
 */
function handleSend() {
    const message = userInput.value.trim();
    if (!message || props.disabled) return;

    emit('send', message);

    // 清空输入框并重置高度
    userInput.value = '';
    if (textareaRef.value) {
        textareaRef.value.style.height = '40px';
    }
    inputRows.value = 1;
}

/**
 * 设置输入框值（用于命令面板）
 */
function setValue(value: string) {
    userInput.value = value;
    if (textareaRef.value) {
        textareaRef.value.focus();
    }
}

/**
 * 清空输入框
 */
function clear() {
    userInput.value = '';
    if (textareaRef.value) {
        textareaRef.value.style.height = '40px';
    }
    inputRows.value = 1;
}

// 暴露方法供父组件调用
defineExpose({
    setValue,
    clear
});
</script>

<style scoped>
textarea {
    font-family: inherit;
}

textarea:focus {
    outline: none;
}
</style>
