<script setup lang="ts">
const props = defineProps<{
    mode: "character" | "worldBook";
}>();

const emit = defineEmits<{
    (e: "toggle-mode"): void;
}>();

function onToggle() {
    emit("toggle-mode");
}
</script>

<template>
    <button
        @click="onToggle"
        class="bg-purple-500 hover:bg-purple-700 text-white text-sm font-medium py-1.5 px-3 rounded-full flex items-center gap-1.5"
    >
        <svg
            class="w-3.5 h-3.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
            />
        </svg>
        {{
            props.mode === "character" ? "世界书编辑" : "角色编辑"
        }}
    </button>
</template>
