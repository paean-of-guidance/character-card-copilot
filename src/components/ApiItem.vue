<script setup lang="ts">
import { ref } from "vue";
import type { ApiConfig, ApiTestResult } from "@/types/api";
import { MdContentCopy, MdDelete } from "vue-icons-plus/md";

const props = defineProps<{
    api: ApiConfig;
}>();

const emit = defineEmits<{
    select: [api: ApiConfig];
    delete: [profile: string];
    setDefault: [profile: string];
    toggleEnabled: [profile: string, enabled: boolean];
    testConnection: [result: ApiTestResult];
    copy: [api: ApiConfig];
}>();

const testResult = ref<ApiTestResult | null>(null);

function handleSelect() {
    emit("select", props.api);
}

function handleDelete() {
    emit("delete", props.api.profile);
}

function handleCopyConfig() {
    emit("copy", props.api);
}
</script>

<template>
    <div
        class="api-item border border-gray-200 rounded-lg p-3 cursor-pointer hover:bg-gray-50"
        :class="{
            'border-blue-500 bg-blue-50': api.default,
            'opacity-60': !api.enabled,
        }"
        @click="handleSelect"
    >
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
                <!-- 状态指示器 -->
                <div
                    class="w-8 h-8 rounded-full flex items-center justify-center"
                    :class="{
                        'bg-green-100': api.enabled,
                        'bg-gray-100': !api.enabled,
                    }"
                >
                    <div
                        class="w-2 h-2 rounded-full"
                        :class="{
                            'bg-green-500': api.enabled,
                            'bg-gray-400': !api.enabled,
                        }"
                    />
                </div>

                <!-- API信息 -->
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                        <h3 class="font-medium text-gray-800 text-sm truncate">
                            {{ api.profile }}
                        </h3>
                        <span
                            v-if="api.default"
                            class="text-xs bg-blue-100 text-blue-600 px-2 py-0.5 rounded-full"
                        >
                            默认
                        </span>
                    </div>
                    <div class="text-xs text-gray-600 mt-0.5 truncate">
                        {{ api.model || "未设置模型" }}
                    </div>
                </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-1 ml-2">
                <!-- 复制配置按钮 -->
                <button
                    class="p-2 hover:bg-gray-200 rounded-full transition-colors text-gray-600"
                    @click.stop="handleCopyConfig"
                    title="复制配置"
                >
                    <MdContentCopy class="w-4 h-4" />
                </button>

                <!-- 删除按钮 -->
                <button
                    class="p-2 hover:bg-red-100 rounded-full transition-colors text-red-600"
                    @click.stop="handleDelete"
                    title="删除"
                >
                    <MdDelete class="w-4 h-4" />
                </button>
            </div>
        </div>

        <!-- 测试结果显示 -->
        <div v-if="testResult" class="test-result mt-2 ml-10">
            <div
                class="test-message text-xs px-2 py-1 rounded"
                :class="{
                    'bg-green-100 text-green-700': testResult.success,
                    'bg-red-100 text-red-700': !testResult.success,
                }"
            >
                {{ testResult.message }}
            </div>
        </div>
    </div>
</template>

<style scoped>
.api-item {
    transition: all 0.2s ease;
}
</style>
