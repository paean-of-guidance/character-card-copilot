<script setup lang="ts">
const emit = defineEmits<{
    toggle: [];
}>();
</script>

<template>
    <div class="flex items-center justify-between gap-3 px-1 pt-1">
        <div class="inline-flex items-center gap-2 rounded-full border border-blue-200/80 bg-[linear-gradient(180deg,_rgba(219,234,254,0.92)_0%,_rgba(191,219,254,0.7)_100%)] px-3 py-1 text-[11px] font-medium uppercase tracking-[0.18em] text-blue-700 shadow-[inset_0_1px_0_rgba(255,255,255,0.9)]">
            <span class="inline-block h-2 w-2 rounded-full bg-blue-500"></span>
            Workspace Copilot
        </div>

        <button
            class="flex h-9 w-9 items-center justify-center rounded-2xl border border-white/70 bg-white/58 text-slate-400 shadow-[inset_0_1px_0_rgba(255,255,255,0.84)] backdrop-blur transition-colors hover:text-slate-600"
            title="隐藏面板"
            @click="emit('toggle')"
        >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
        </button>
    </div>
</template>
