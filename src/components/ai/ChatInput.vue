<template>
    <div class="composer-shell">
        <div class="composer-input-wrap">
            <textarea
                ref="textareaRef"
                v-model="userInput"
                :disabled="disabled"
                placeholder="输入消息…"
                class="chat-input-textarea"
                @input="handleInput"
                @keydown="handleKeydown"
            ></textarea>
            <button
                :disabled="isActionDisabled"
                class="send-button"
                :class="{ 'send-button--stop': canStop && loading }"
                :title="canStop && loading ? '停止生成' : '发送消息'"
                @click="handlePrimaryAction"
            >
                <svg
                    v-if="!loading"
                    class="h-4 w-4"
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
                <svg
                    v-else-if="canStop && !stopping"
                    class="h-4 w-4"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                >
                    <rect x="7" y="7" width="10" height="10" rx="2" />
                </svg>
                <div
                    v-else
                    class="h-4 w-4 rounded-full border-2 border-white border-t-transparent animate-spin"
                ></div>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useTextareaAutosize } from '@vueuse/core';
import { computed, nextTick, onMounted, ref, watch } from 'vue';

interface Props {
    disabled?: boolean;
    loading?: boolean;
    canStop?: boolean;
    stopping?: boolean;
    commandPaletteOpen?: boolean;
}

interface Emits {
    send: [message: string];
    stop: [];
    openCommandPalette: [];
    keydown: [event: KeyboardEvent];
    input: [value: string];
}

const props = withDefaults(defineProps<Props>(), {
    disabled: false,
    loading: false,
    canStop: false,
    stopping: false,
    commandPaletteOpen: false,
});

const emit = defineEmits<Emits>();

const userInput = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const MIN_TEXTAREA_HEIGHT = 48;
const LINE_HEIGHT = 24;
const MAX_ROWS = 5;
let heightAnimationFrame: number | null = null;
const { triggerResize } = useTextareaAutosize({
    element: textareaRef,
    input: userInput,
});

function resolveTextareaMaxHeight(textarea: HTMLTextAreaElement): number {
    const computedStyle = window.getComputedStyle(textarea);
    const paddingTop = Number.parseFloat(computedStyle.paddingTop || '0');
    const paddingBottom = Number.parseFloat(computedStyle.paddingBottom || '0');

    return LINE_HEIGHT * MAX_ROWS + paddingTop + paddingBottom;
}

function syncTextareaLayout() {
    nextTick(() => {
        if (!textareaRef.value) {
            return;
        }

        const textarea = textareaRef.value;
        const maxHeight = resolveTextareaMaxHeight(textarea);
        const previousHeight = textarea.getBoundingClientRect().height || MIN_TEXTAREA_HEIGHT;

        textarea.style.height = 'auto';
        textarea.style.minHeight = `${MIN_TEXTAREA_HEIGHT}px`;
        textarea.style.maxHeight = `${maxHeight}px`;
        const nextHeight = Math.max(MIN_TEXTAREA_HEIGHT, Math.min(textarea.scrollHeight, maxHeight));

        if (heightAnimationFrame !== null) {
            cancelAnimationFrame(heightAnimationFrame);
        }

        if (Math.abs(previousHeight - nextHeight) > 1) {
            textarea.style.height = `${previousHeight}px`;
            heightAnimationFrame = requestAnimationFrame(() => {
                textarea.style.height = `${nextHeight}px`;
                heightAnimationFrame = null;
            });
        } else {
            textarea.style.height = `${nextHeight}px`;
        }

        textarea.style.overflowY = textarea.scrollHeight > maxHeight ? 'auto' : 'hidden';
    });
}

function resetTextarea() {
    triggerResize();
    syncTextareaLayout();
}

function handleInput() {
    emit('input', userInput.value);
}

function handleKeydown(event: KeyboardEvent) {
    if (props.loading && props.canStop && event.key === 'Escape') {
        event.preventDefault();
        emit('stop');
        return;
    }

    if (props.commandPaletteOpen) {
        if (['ArrowUp', 'ArrowDown', 'Enter', 'Tab', ' ', 'Escape'].includes(event.key)) {
            emit('keydown', event);
            return;
        }
    }

    if (event.key === '/' && userInput.value === '') {
        event.preventDefault();
        emit('openCommandPalette');
        return;
    }

    if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        handlePrimaryAction();
    }
}

function handleSend() {
    const message = userInput.value.trim();
    if (!message || props.disabled) {
        return;
    }

    emit('send', message);
    userInput.value = '';
    resetTextarea();
}

const isActionDisabled = computed(() => {
    if (props.loading) {
        return !props.canStop || props.stopping;
    }

    return !userInput.value.trim() || props.disabled;
});

function handlePrimaryAction() {
    if (props.loading && props.canStop) {
        emit('stop');
        return;
    }

    handleSend();
}

function setValue(value: string) {
    userInput.value = value;
    triggerResize();
    syncTextareaLayout();
    textareaRef.value?.focus();
}

function clear() {
    userInput.value = '';
    resetTextarea();
}

defineExpose({
    setValue,
    clear,
});

watch(userInput, () => {
    triggerResize();
    syncTextareaLayout();
});

onMounted(() => {
    triggerResize();
    syncTextareaLayout();
});
</script>

<style scoped>
.composer-shell {
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 1.5rem;
    background: rgba(255, 255, 255, 0.06);
    box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.14),
        0 8px 24px rgba(0, 0, 0, 0.25);
    padding: 0.5rem;
    backdrop-filter: blur(24px);
}

.composer-input-wrap {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: end;
    gap: 0.625rem;
    width: 100%;
    min-width: 0;
    border-radius: 1.125rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 0.3125rem;
}

.chat-input-textarea {
    width: 100%;
    resize: none;
    overflow-y: hidden;
    border: none;
    background: transparent;
    padding: 0.625rem 0.75rem;
    color: rgba(255, 255, 255, 0.90);
    font-size: 0.95rem;
    transition: height 0.18s ease;
    display: block;
    box-sizing: border-box;
    line-height: 24px;
    min-height: 48px;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
}

.chat-input-textarea::placeholder {
    color: rgba(255, 255, 255, 0.28);
}

.chat-input-textarea:focus {
    outline: none;
}

.chat-input-textarea:disabled {
    cursor: not-allowed;
    opacity: 0.5;
}

.send-button {
    display: inline-flex;
    height: 2.75rem;
    width: 2.75rem;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(165, 180, 252, 0.35);
    border-radius: 999px;
    background: linear-gradient(180deg, rgba(99, 102, 241, 0.35) 0%, rgba(139, 92, 246, 0.25) 100%);
    color: rgba(196, 181, 253, 0.95);
    box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.16),
        0 8px 20px rgba(99, 102, 241, 0.25);
    backdrop-filter: blur(16px);
    align-self: end;
    transition:
        transform 0.15s ease,
        box-shadow 0.15s ease,
        opacity 0.15s ease,
        color 0.15s ease,
        background 0.15s ease;
}

.send-button:hover:not(:disabled) {
    transform: translateY(-1px);
    color: rgba(255, 255, 255, 0.98);
    background: linear-gradient(180deg, rgba(99, 102, 241, 0.55) 0%, rgba(139, 92, 246, 0.42) 100%);
    border-color: rgba(165, 180, 252, 0.55);
    box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.22),
        0 12px 28px rgba(99, 102, 241, 0.35);
}

.send-button--stop {
    border-color: rgba(252, 165, 165, 0.35);
    background: linear-gradient(180deg, rgba(239, 68, 68, 0.32) 0%, rgba(220, 38, 38, 0.22) 100%);
    color: rgba(252, 165, 165, 0.95);
    box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.12),
        0 8px 20px rgba(239, 68, 68, 0.20);
}

.send-button--stop:hover:not(:disabled) {
    color: rgba(255, 255, 255, 0.98);
    background: linear-gradient(180deg, rgba(239, 68, 68, 0.50) 0%, rgba(220, 38, 38, 0.38) 100%);
    border-color: rgba(252, 165, 165, 0.55);
}

.send-button:disabled {
    cursor: not-allowed;
    border-color: rgba(255, 255, 255, 0.10);
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.25);
    box-shadow: none;
}

.chat-input-textarea::-webkit-scrollbar {
    width: 8px;
}

.chat-input-textarea::-webkit-scrollbar-track {
    background: transparent;
}

.chat-input-textarea::-webkit-scrollbar-thumb {
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.15);
    border: 2px solid transparent;
    background-clip: padding-box;
}

@media (max-width: 640px) {
    .composer-input-wrap {
        gap: 0.5rem;
    }

    .send-button {
        height: 2.5rem;
        width: 2.5rem;
    }
}
</style>
