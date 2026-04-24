<template>
    <Teleport to="body">
        <div class="fixed top-4 right-4 z-50 pointer-events-none">
            <TransitionGroup name="toast" tag="div" class="flex flex-col gap-2">
                <div
                    v-for="toast in visibleToasts"
                    :key="toast.id"
                    :class="['pointer-events-auto', 'w-full max-w-sm', 'flex items-start gap-3 p-4', 'rounded-2xl transition-all duration-300', getLiquidClass(toast.type)]"
                    :style="getLiquidStyle(toast.type)"
                    role="alert"
                    @mouseenter="pauseTimer(toast.id)"
                    @mouseleave="resumeTimer(toast.id)"
                >
                    <!-- 图标 -->
                    <div :class="['shrink-0 flex h-8 w-8 items-center justify-center rounded-xl text-base font-bold', getIconClass(toast.type)]">
                        {{ getToastConfig(toast.type).icon }}
                    </div>

                    <!-- 内容 -->
                    <div class="min-w-0 flex-1">
                        <p v-if="toast.title" class="mb-1 text-sm font-semibold text-white/90">
                            {{ toast.title }}
                        </p>
                        <p class="text-sm text-white/65">
                            {{ toast.message }}
                        </p>
                    </div>

                    <!-- 关闭按钮 -->
                    <button
                        @click="removeToast(toast.id)"
                        class="ml-1 shrink-0 text-white/40 transition-colors hover:text-white/75"
                        aria-label="关闭通知"
                    >
                        <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
                        </svg>
                    </button>
                </div>
            </TransitionGroup>
        </div>
    </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from "vue";
import type { Toast, ToastType } from "@/utils/notification";
import { getToastConfig } from "@/utils/notification";

interface Props {
    toasts: Toast[];
}

const props = defineProps<Props>();

const MAX_VISIBLE_TOASTS = 1;
const visibleToasts = ref<Toast[]>([]);
const pausedTimers = new Set<string>();
const activeTimers = new Map<string, number>();

const filteredToasts = computed(() => props.toasts.slice(-MAX_VISIBLE_TOASTS));

function getLiquidStyle(type: ToastType): string {
    const styles: Record<ToastType, string> = {
        success: "background: rgba(52,211,153,0.14); border: 1px solid rgba(52,211,153,0.28); backdrop-filter: blur(32px) saturate(180%); -webkit-backdrop-filter: blur(32px) saturate(180%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.14), 0 8px 32px rgba(0,0,0,0.35);",
        warning: "background: rgba(251,191,36,0.14); border: 1px solid rgba(251,191,36,0.28); backdrop-filter: blur(32px) saturate(180%); -webkit-backdrop-filter: blur(32px) saturate(180%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.14), 0 8px 32px rgba(0,0,0,0.35);",
        info:    "background: rgba(99,102,241,0.16); border: 1px solid rgba(165,180,252,0.28); backdrop-filter: blur(32px) saturate(180%); -webkit-backdrop-filter: blur(32px) saturate(180%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.14), 0 8px 32px rgba(0,0,0,0.35);",
        error:   "background: rgba(239,68,68,0.14); border: 1px solid rgba(252,165,165,0.28); backdrop-filter: blur(32px) saturate(180%); -webkit-backdrop-filter: blur(32px) saturate(180%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.14), 0 8px 32px rgba(0,0,0,0.35);",
    };
    return styles[type];
}

function getLiquidClass(_type: ToastType): string {
    return "";
}

function getIconClass(type: ToastType): string {
    const classes: Record<ToastType, string> = {
        success: "bg-emerald-500/20 text-emerald-300",
        warning: "bg-yellow-500/20 text-yellow-300",
        info:    "bg-indigo-500/20 text-indigo-300",
        error:   "bg-red-500/20 text-red-300",
    };
    return classes[type];
}

function removeToast(id: string) {
    const index = visibleToasts.value.findIndex((t) => t.id === id);
    if (index > -1) visibleToasts.value.splice(index, 1);
    pausedTimers.delete(id);
    const timer = activeTimers.get(id);
    if (timer) { clearTimeout(timer); activeTimers.delete(id); }
}

function pauseTimer(id: string) { pausedTimers.add(id); }
function resumeTimer(id: string) { pausedTimers.delete(id); }

function createAutoRemoveTimer(toast: Toast) {
    const duration = toast.duration || getToastConfig(toast.type).duration;
    const timer = setTimeout(() => {
        if (!pausedTimers.has(toast.id)) {
            removeToast(toast.id);
        } else {
            const checkInterval = setInterval(() => {
                if (!pausedTimers.has(toast.id)) {
                    removeToast(toast.id);
                    clearInterval(checkInterval);
                }
            }, 100);
            setTimeout(() => { clearInterval(checkInterval); removeToast(toast.id); }, 10000);
        }
    }, duration);
    activeTimers.set(toast.id, timer as unknown as number);
}

function updateVisibleToasts() {
    const newToasts = filteredToasts.value;
    newToasts.forEach((toast) => {
        if (!visibleToasts.value.find((t) => t.id === toast.id)) {
            visibleToasts.value.forEach((old) => removeToast(old.id));
            visibleToasts.value = [toast];
            createAutoRemoveTimer(toast);
        }
    });
    visibleToasts.value = visibleToasts.value.filter((t) =>
        newToasts.find((n) => n.id === t.id),
    );
}

let timer: ReturnType<typeof setTimeout>;
watch(() => props.toasts, () => {
    clearTimeout(timer);
    timer = setTimeout(updateVisibleToasts, 50);
}, { deep: true, immediate: true });

onUnmounted(() => {
    visibleToasts.value.forEach((t) => pausedTimers.delete(t.id));
    activeTimers.forEach((id) => clearTimeout(id));
    activeTimers.clear();
    clearTimeout(timer);
});

defineExpose({ removeToast, pauseTimer, resumeTimer });
</script>

<style scoped>
.toast-enter-active {
    transition: all 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.toast-leave-active {
    transition: all 0.25s ease-in;
}
.toast-enter-from {
    transform: translateX(110%);
    opacity: 0;
}
.toast-leave-to {
    transform: translateX(110%);
    opacity: 0;
}
.toast-move {
    transition: transform 0.3s ease;
}
</style>
