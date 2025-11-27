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
                class="inline-flex items-center gap-1 rounded-full bg-blue-100 text-blue-800 px-3 py-1 text-sm"
            >
                <span>{{ tag }}</span>
                <button
                    type="button"
                    class="text-blue-600 hover:text-blue-800"
                    :disabled="disabled"
                    @click="removeTag(index)"
                >
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </span>
            <span v-if="!normalizedTags.length" class="text-xs text-gray-400">
                尚未添加标签
            </span>
        </div>

        <div class="flex gap-2">
            <input
                v-model="inputValue"
                type="text"
                class="flex-1 border border-gray-200 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
                :placeholder="placeholder || '输入标签后按回车'"
                :disabled="disabled"
                @keydown.enter.prevent="addTag"
            />
            <button
                type="button"
                class="rounded-full bg-blue-500 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
                :disabled="disabled"
                @click="addTag"
            >
                {{ addButtonText || "添加" }}
            </button>
        </div>
    </div>
</template>
