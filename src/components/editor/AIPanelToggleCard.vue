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
        class="card flex h-full min-h-[12rem] w-[4.5rem] cursor-pointer items-center justify-center rounded-[24px] border border-white/70 bg-white/80 px-3 py-5 shadow-[0_18px_36px_rgba(148,163,184,0.18)] backdrop-blur-xl transition-all duration-300 hover:-translate-x-0.5 hover:border-blue-200 hover:bg-white/90 hover:shadow-[0_24px_48px_rgba(96,165,250,0.14)]"
        @click="onToggle"
    >
        <div class="flex flex-col items-center gap-3 text-slate-500">
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
