<script setup lang="ts">
import { MdAdd, MdUpload } from "vue-icons-plus/md";

const emit = defineEmits<{
    "create-new": [];
    import: [];
}>();

function handleCreateNew() {
    emit("create-new");
}

function handleImport() {
    emit("import");
}
</script>

<template>
    <div class="liquid-card overflow-hidden border-dashed">
        <div class="flex flex-col gap-3 p-3">
            <!-- 头部说明 -->
            <div
                class="rounded-[20px] p-4"
                style="background: radial-gradient(circle at top, rgba(99,102,241,0.18), transparent 50%), rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.10);"
            >
                <h3 class="text-base font-semibold tracking-tight text-white/90">开始一个新角色</h3>
                <p class="mt-2 min-h-[2.5rem] text-sm leading-5 text-white/50 line-clamp-2">
                    从空白模板创建，或者直接导入已有角色卡，马上进入编辑与 Copilot 工作流。
                </p>
            </div>

            <div class="grid gap-3">
                <!-- 新建按钮 -->
                <button
                    @click="handleCreateNew"
                    class="group flex min-h-[6.25rem] flex-col items-start justify-between rounded-[20px] p-3.5 text-left transition-all hover:-translate-y-0.5"
                    style="background: rgba(52,211,153,0.10); border: 1px solid rgba(52,211,153,0.22);"
                    @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'rgba(52,211,153,0.18)'"
                    @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'rgba(52,211,153,0.10)'"
                >
                    <div class="flex h-9 w-9 items-center justify-center rounded-2xl bg-emerald-400/20 text-emerald-300">
                        <MdAdd class="h-[18px] w-[18px]" />
                    </div>
                    <div>
                        <div class="text-sm font-semibold text-emerald-300">新建角色</div>
                        <div class="mt-1 text-xs leading-[18px] text-emerald-400/70 line-clamp-2">
                            创建一张全新的角色卡，并直接进入编辑器。
                        </div>
                    </div>
                </button>

                <!-- 导入按钮 -->
                <button
                    @click="handleImport"
                    class="group flex min-h-[6.25rem] flex-col items-start justify-between rounded-[20px] p-3.5 text-left transition-all hover:-translate-y-0.5"
                    style="background: rgba(99,102,241,0.12); border: 1px solid rgba(165,180,252,0.22);"
                    @mouseenter="(e) => (e.currentTarget as HTMLElement).style.background = 'rgba(99,102,241,0.20)'"
                    @mouseleave="(e) => (e.currentTarget as HTMLElement).style.background = 'rgba(99,102,241,0.12)'"
                >
                    <div class="flex h-9 w-9 items-center justify-center rounded-2xl bg-indigo-400/20 text-indigo-300">
                        <MdUpload class="h-[18px] w-[18px]" />
                    </div>
                    <div>
                        <div class="text-sm font-semibold text-indigo-300">导入角色</div>
                        <div class="mt-1 text-xs leading-[18px] text-indigo-400/70 line-clamp-2">
                            支持 PNG、JSON、CARD，快速把已有角色接入工作台。
                        </div>
                    </div>
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.line-clamp-2 {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    overflow: hidden;
}
</style>
