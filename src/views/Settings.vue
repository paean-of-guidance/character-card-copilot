<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "@/stores/app";
import { useNotification } from "@/composables/useNotification";
import type { ApiConfig, ApiTestResult } from "@/types/api";
import ApiList from "@/components/ApiList.vue";
import ModelSelect from "@/components/ModelSelect.vue";
import {
    copyApiConfig,
    updateApiConfig,
    setDefaultApiConfig,
    testApiConnection,
    getAllApiConfigs,
} from "@/services/apiConfig";

const appStore = useAppStore();
const { showSuccessToast, showErrorToast, showWarningToast } = useNotification();
const selectedApi = ref<ApiConfig | null>(null);
const editingApi = ref<ApiConfig | null>(null);
const lastTestResult = ref<ApiTestResult | null>(null);

// 用于强制刷新ApiList组件的key
const apiListKey = ref(0);

// 测试连接状态
const testing = ref(false);

// 存储原始的配置名称
const originalProfile = ref<string>("");

// 更新API列表
async function updateApiList() {
    try {
        const configs = await getAllApiConfigs();

        if (selectedApi.value) {
            const updated = configs.find(
                (api) => api.profile === selectedApi.value?.profile,
            );
            if (updated) {
                selectedApi.value = { ...updated };
                if (editingApi.value) {
                    editingApi.value = { ...updated };
                    originalProfile.value = updated.profile;
                }
            }
        }

        // ����ǿ��ˢ��ApiList�����key
        apiListKey.value++;
    } catch (error) {
        console.error("����API�б�ʧ��:", error);
    }
}

// 自动保存函数
async function autoSave() {
    if (!editingApi.value || !selectedApi.value) return;

    try {
        await updateApiConfig({
            profile: editingApi.value.profile,
            original_profile: originalProfile.value,
            endpoint: editingApi.value.endpoint,
            key: editingApi.value.key,
            model: editingApi.value.model,
            default: editingApi.value.default,
            enabled: editingApi.value.enabled,
        });

        // 更新selectedApi以反映最新保存的状态
        selectedApi.value = { ...editingApi.value };

        // 更新原始profile名称为当前名称
        originalProfile.value = editingApi.value.profile;

        // 触发左侧列表更新
        await updateApiList();
    } catch (error) {
        console.error("自动保存失败:", error);
    }
}

onMounted(() => {
    appStore.setPageTitle("设置", true);
});

function handleSelectApi(api: ApiConfig) {
    selectedApi.value = api;
    editingApi.value = { ...api }; // 创建副本用于编辑
    originalProfile.value = api.profile; // 存储原始配置名称
    lastTestResult.value = null; // 重置测试结果
}

function updateApiModel(model: string) {
    if (editingApi.value) {
        editingApi.value.model = model;
        autoSave();
    }
}

async function handleToggleEnabled() {
    if (!selectedApi.value || !editingApi.value) return;

    if (!selectedApi.value.enabled) {
        // 如果要启用，检查是否有成功的测试结果
        if (lastTestResult.value?.success) {
            editingApi.value.enabled = true;
            autoSave();
        } else {
            showWarningToast("请先测试连接成功后再启用此配置", "无法启用配置");
        }
    } else {
        editingApi.value.enabled = false;
        autoSave();
    }
}

async function handleSetDefault() {
    if (selectedApi.value && !selectedApi.value.default && editingApi.value) {
        try {
            await setDefaultApiConfig(selectedApi.value.profile);
            selectedApi.value.default = true;
            editingApi.value.default = true;
            // 更新左侧列表显示
            await updateApiList();
            showSuccessToast("设为默认配置成功！", "操作成功");
        } catch (error) {
            console.error("设为默认失败:", error);
            showErrorToast("设为默认失败，请重试", "操作失败");
        }
    }
}

async function handleCopyConfig(api: ApiConfig) {
    try {
        const newApi = await copyApiConfig(api);
        // 重新加载API列表 - 通过改变key强制刷新ApiList组件
        apiListKey.value++;
        console.log("复制配置成功:", newApi);
        showSuccessToast("复制配置成功！", "操作完成");
    } catch (error) {
        console.error("复制配置失败:", error);
        showErrorToast("复制配置失败，请重试", "操作失败");
    }
}

async function handleTestConnection() {
    if (!editingApi.value) return;

    testing.value = true;
    lastTestResult.value = null;

    try {
        const result = await testApiConnection(editingApi.value);
        lastTestResult.value = result;

        // 更新selectedApi的测试结果
        if (selectedApi.value) {
            selectedApi.value = { ...selectedApi.value };
        }

        // 如果测试成功，更新editingApi
        if (result.success && editingApi.value) {
            editingApi.value = { ...editingApi.value };
        }
    } catch (error) {
        console.error("测试连接失败:", error);
        lastTestResult.value = {
            success: false,
            message: "测试连接失败",
            error: error as string,
        };
    } finally {
        testing.value = false;
    }
}
</script>

<template>
    <div class="bg-gray-50">
        <div class="max-w-7xl mx-auto">
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
                <!-- 左侧：API列表 -->
                <div class="lg:col-span-1">
                    <div class="bg-white rounded-xl shadow-lg p-4 h-fit">
                        <div class="mb-4">
                            <h2 class="text-lg font-bold text-gray-900 mb-1">
                                API配置
                            </h2>
                            <p class="text-xs text-gray-600">
                                管理AI服务的API配置
                            </p>
                        </div>

                        <ApiList
                            :key="apiListKey"
                            @select="handleSelectApi"
                            @testConnection="handleTestConnection"
                            @copy="handleCopyConfig"
                        />
                    </div>
                </div>

                <!-- 右侧：配置详情 -->
                <div class="lg:col-span-2">
                    <div
                        v-if="selectedApi && editingApi"
                        class="bg-white rounded-xl shadow-lg p-4"
                    >
                        <div class="mb-4">
                            <h3 class="text-xl font-bold text-gray-900 mb-1">
                                {{ selectedApi.profile }} - 配置详情
                            </h3>
                            <div class="h-1 w-16 bg-blue-500 rounded"></div>
                        </div>

                        <div class="space-y-2">
                            <div class="bg-gray-50 rounded-lg p-3">
                                <label
                                    class="block text-sm font-semibold text-gray-700 mb-1"
                                    >配置名称</label
                                >
                                <input
                                    v-model="editingApi.profile"
                                    @blur="autoSave"
                                    type="text"
                                    class="bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm w-full"
                                    placeholder="请输入配置名称"
                                />
                            </div>

                            <div class="bg-gray-50 rounded-lg p-3">
                                <label
                                    class="block text-sm font-semibold text-gray-700 mb-1"
                                    >链接端点</label
                                >
                                <input
                                    v-model="editingApi.endpoint"
                                    @blur="autoSave"
                                    type="text"
                                    class="bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm w-full"
                                    placeholder="请输入API端点URL"
                                />
                            </div>

                            <div class="bg-gray-50 rounded-lg p-3">
                                <label
                                    class="block text-sm font-semibold text-gray-700 mb-1"
                                    >API密钥</label
                                >
                                <input
                                    v-model="editingApi.key"
                                    @blur="autoSave"
                                    type="password"
                                    class="bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm w-full"
                                    placeholder="请输入API密钥"
                                />
                            </div>

                            <div class="bg-gray-50 rounded-lg p-3">
                                <label
                                    class="block text-sm font-semibold text-gray-700 mb-1"
                                    >使用模型</label
                                >
                                <ModelSelect
                                    v-if="editingApi"
                                    :api-config="editingApi"
                                    :model-value="editingApi.model"
                                    @update:modelValue="updateApiModel"
                                />
                                <div
                                    v-else
                                    class="bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm"
                                >
                                    {{ selectedApi.model || "未设置" }}
                                </div>
                            </div>

                            <div class="bg-gray-50 rounded-lg p-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700 mb-1"
                                    >启用/禁用和默认设置</label
                                >

                                <!-- 测试结果显示 -->
                                <div v-if="lastTestResult" class="mb-3">
                                    <div
                                        class="text-xs px-3 py-2 rounded-lg"
                                        :class="{
                                            'bg-green-100 text-green-800 border border-green-200':
                                                lastTestResult.success,
                                            'bg-red-100 text-red-800 border border-red-200':
                                                !lastTestResult.success,
                                        }"
                                    >
                                        {{ lastTestResult.message }}
                                    </div>
                                </div>

                                <!-- 控制按钮 -->
                                <div class="flex flex-wrap gap-2">
                                    <!-- 启用/禁用按钮 -->
                                    <button
                                        class="font-bold py-1.5 px-4 rounded-full text-sm transition-colors"
                                        :class="{
                                            'bg-green-500 hover:bg-green-600 text-white':
                                                selectedApi.enabled,
                                            'bg-blue-500 hover:bg-blue-600 text-white':
                                                !selectedApi.enabled,
                                            'opacity-50 cursor-not-allowed':
                                                !selectedApi.enabled &&
                                                !lastTestResult?.success,
                                        }"
                                        @click="handleToggleEnabled"
                                        :disabled="
                                            !selectedApi.enabled &&
                                            !lastTestResult?.success
                                        "
                                    >
                                        {{
                                            selectedApi.enabled
                                                ? "禁用"
                                                : "启用"
                                        }}
                                    </button>

                                    <!-- 设为默认按钮 -->
                                    <button
                                        v-if="!selectedApi.default"
                                        class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-1.5 px-4 rounded-full text-sm transition-colors"
                                        @click="handleSetDefault"
                                    >
                                        设为默认
                                    </button>
                                </div>
                            </div>
                        </div>

                        <!-- 操作按钮区域 -->
                        <div class="mt-6 pt-4 border-t border-gray-200">
                            <div class="flex justify-end gap-3">
                                <button
                                    class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded-full text-sm transition-colors"
                                    :disabled="testing"
                                    @click="handleTestConnection"
                                >
                                    {{ testing ? "测试中..." : "测试连接" }}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div v-else class="bg-white rounded-xl shadow-lg p-8">
                        <div class="text-center">
                            <div class="mb-4">
                                <span class="text-6xl text-gray-300">🔧</span>
                            </div>
                            <h3 class="text-xl font-bold text-gray-800 mb-3">
                                选择API配置
                            </h3>
                            <p class="text-gray-600 text-sm max-w-sm mx-auto">
                                请从左侧选择一个API配置进行查看和编辑，<br />
                                或创建新的API配置。
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped></style>

