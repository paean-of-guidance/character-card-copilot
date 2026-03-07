<script setup lang="ts">
import { useAppStore } from "@/stores/app";
import { useRouter } from "vue-router";
import { MdKeyboardArrowLeft, MdSettings } from "vue-icons-plus/md";

const appStore = useAppStore();
const router = useRouter();

function handleBack() {
    if (appStore.canGoBack) {
        appStore.goBack();
    }
}

function handleSettings() {
    router.push("/settings");
}
</script>

<template>
    <nav class="sticky top-0 z-30 shrink-0 border-b border-gray-200 bg-white px-6 py-2 shadow-md">
        <div class="flex items-center justify-between">
            <!-- 左侧：返回按钮 + 页面名称 -->
            <div class="flex items-center gap-3">
                <button
                    v-if="appStore.canGoBack"
                    @click="handleBack"
                    class="p-2 hover:bg-gray-100 rounded-full transition-colors"
                    aria-label="返回上一级"
                >
                    <MdKeyboardArrowLeft class="w-6 h-6 text-gray-700" />
                </button>
                <h1 class="text-xl font-semibold text-gray-800">
                    {{ appStore.pageTitle }}
                </h1>
            </div>

            <!-- 右侧：设置按钮 -->
            <button
                @click="handleSettings"
                class="p-2 hover:bg-gray-100 rounded-full transition-colors"
                aria-label="设置"
            >
                <MdSettings class="w-6 h-6 text-gray-700" />
            </button>
        </div>
    </nav>
</template>
