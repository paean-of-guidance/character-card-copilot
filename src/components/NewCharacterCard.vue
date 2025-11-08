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
    <div class="new-character-card rounded-lg overflow-hidden">
        <div
            class="h-full flex flex-col justify-center items-center p-3 gap-10"
        >
            <!-- 新建角色按钮 -->
            <button
                @click="handleCreateNew"
                class="w-full h-40 flex flex-col items-center justify-center border-2 border-dashed border-green-300 rounded-lg bg-green-50 hover:bg-green-100 hover:border-green-500 transition-all group"
            >
                <MdAdd
                    class="w-8 h-8 text-green-500 group-hover:text-green-600"
                />
                <span class="mt-1 text-green-700 font-medium text-sm"
                    >新建角色</span
                >
            </button>

            <!-- 导入角色按钮 -->
            <button
                @click="handleImport"
                class="w-full h-40 flex flex-col items-center justify-center border-2 border-dashed border-blue-300 rounded-lg bg-blue-50 hover:bg-blue-100 hover:border-blue-500 transition-all group"
            >
                <MdUpload
                    class="w-8 h-8 text-blue-500 group-hover:text-blue-600"
                />
                <span class="mt-1 text-blue-700 font-medium text-sm"
                    >导入角色</span
                >
            </button>
        </div>
    </div>
</template>

<style scoped>
.new-character-card {
    border: 2px dashed #d1d5db;
    background-color: #f9fafb;
}
</style>
