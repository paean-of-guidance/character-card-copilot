<script setup lang="ts">
const emit = defineEmits<{
    (e: "toggle"): void;
}>();

function onToggle() {
    emit("toggle");
}
</script>

<template>
    <div
        class="liquid-card flex h-full min-h-[12rem] w-[4.5rem] cursor-pointer items-center justify-center px-3 py-5 transition-all duration-300 hover:-translate-x-0.5"
        @click="onToggle"
    >
        <div class="flex flex-col items-center gap-3 text-white/40">
            <svg
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 19l-7-7 7-7"
                />
            </svg>
            <span class="text-[11px] font-medium tracking-[0.24em] [writing-mode:vertical-rl]">
                COPILOT
            </span>
        </div>
    </div>
</template>
