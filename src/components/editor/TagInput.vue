<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
    modelValue: string[];
    placeholder?: string;
    disabled?: boolean;
    addButtonText?: string;
}>();

const emit = defineEmits<{
    (e: "update:modelValue", value: string[]): void;
    (e: "change", value: string[]): void;
}>();

const inputValue = ref("");

const normalizedTags = computed(() => props.modelValue || []);

function emitValue(next: string[]) {
    emit("update:modelValue", next);
    emit("change", next);
}

function addTag() {
    if (props.disabled) return;
    const value = inputValue.value.trim();
    if (!value || normalizedTags.value.includes(value)) {
        inputValue.value = "";
        return;
    }
    emitValue([...normalizedTags.value, value]);
    inputValue.value = "";
}

function removeTag(index: number) {
    if (props.disabled) return;
    const next = normalizedTags.value.filter((_, i) => i !== index);
    emitValue(next);
}
</script>

<template>
    <div class="space-y-2">
        <div class="flex flex-wrap gap-2">
            <span
                v-for="(tag, index) in normalizedTags"
                :key="tag + index"
                class="inline-flex items-center gap-1 rounded-full border border-indigo-400/25 bg-indigo-500/18 px-3 py-1 text-sm text-indigo-300"
            >
                <span>{{ tag }}</span>
                <button
                    type="button"
                    class="text-indigo-400/60 hover:text-indigo-200"
                    :disabled="disabled"
                    @click="removeTag(index)"
                >
                    <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </span>
            <span v-if="!normalizedTags.length" class="text-xs text-white/30">尚未添加标签</span>
        </div>

        <div class="flex gap-2">
            <input
                v-model="inputValue"
                type="text"
                class="liquid-input flex-1"
                :placeholder="placeholder || '输入标签后按回车'"
                :disabled="disabled"
                @keydown.enter.prevent="addTag"
            />
            <button
                type="button"
                class="glass-btn glass-btn--primary disabled:cursor-not-allowed disabled:opacity-40"
                :disabled="disabled"
                @click="addTag"
            >
                {{ addButtonText || "添加" }}
            </button>
        </div>
    </div>
</template>
