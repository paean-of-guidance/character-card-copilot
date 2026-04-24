<script setup lang="ts">
const emit = defineEmits<{
    toggle: [];
}>();
</script>

<template>
    <div class="flex items-center justify-between gap-3 px-1 pt-1">
        <div class="liquid-badge liquid-badge--primary inline-flex items-center gap-2 uppercase tracking-[0.18em]">
            <span class="inline-block h-2 w-2 rounded-full bg-violet-400"></span>
            Workspace Copilot
        </div>

        <button
            class="flex h-9 w-9 items-center justify-center rounded-2xl border border-white/12 bg-white/8 text-white/40 backdrop-blur transition-colors hover:bg-white/14 hover:text-white/70"
            title="隐藏面板"
            @click="emit('toggle')"
        >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
        </button>
    </div>
</template>
