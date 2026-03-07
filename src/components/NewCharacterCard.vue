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
    <div class="new-character-card overflow-hidden rounded-[24px] border border-dashed border-slate-300 bg-white/80 shadow-[0_12px_32px_rgba(148,163,184,0.16)] backdrop-blur transition-all duration-300 hover:border-blue-200 hover:shadow-[0_22px_48px_rgba(96,165,250,0.16)]">
        <div class="flex flex-col gap-3 p-3">
            <div class="rounded-[20px] border border-blue-100 bg-[radial-gradient(circle_at_top,_rgba(96,165,250,0.16),_transparent_45%),linear-gradient(180deg,_rgba(255,255,255,0.95)_0%,_rgba(241,245,249,0.92)_100%)] p-4">
                <h3 class="text-base font-semibold tracking-tight text-slate-900">开始一个新角色</h3>
                <p class="mt-2 text-sm leading-5 text-slate-600 line-clamp-2 min-h-[2.5rem]">
                    从空白模板创建，或者直接导入已有角色卡，马上进入编辑与 Copilot 工作流。
                </p>
            </div>

            <div class="grid gap-3">
                <button
                    @click="handleCreateNew"
                    class="group flex min-h-[6.25rem] flex-col items-start justify-between rounded-[20px] border border-emerald-200 bg-emerald-50/80 p-3.5 text-left transition-all hover:-translate-y-0.5 hover:border-emerald-300 hover:bg-emerald-50"
                >
                    <div class="flex h-9 w-9 items-center justify-center rounded-2xl bg-white/85 text-emerald-600 shadow-sm">
                        <MdAdd class="h-[18px] w-[18px]" />
                    </div>
                    <div>
                        <div class="text-sm font-semibold text-emerald-800">新建角色</div>
                        <div class="mt-1 line-clamp-2 text-xs leading-[18px] text-emerald-700/80">
                            创建一张全新的角色卡，并直接进入编辑器。
                        </div>
                    </div>
                </button>

                <button
                    @click="handleImport"
                    class="group flex min-h-[6.25rem] flex-col items-start justify-between rounded-[20px] border border-sky-200 bg-sky-50/85 p-3.5 text-left transition-all hover:-translate-y-0.5 hover:border-sky-300 hover:bg-sky-50"
                >
                    <div class="flex h-9 w-9 items-center justify-center rounded-2xl bg-white/85 text-sky-600 shadow-sm">
                        <MdUpload class="h-[18px] w-[18px]" />
                    </div>
                    <div>
                        <div class="text-sm font-semibold text-sky-800">导入角色</div>
                        <div class="mt-1 line-clamp-2 text-xs leading-[18px] text-sky-700/80">
                            支持 PNG、JSON、CARD，快速把已有角色接入工作台。
                        </div>
                    </div>
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.line-clamp-3 {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    overflow: hidden;
}

.new-character-card {
    border: 2px dashed #d1d5db;
    background-color: #f9fafb;
}
</style>
