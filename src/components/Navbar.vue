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
    <nav class="liquid-nav sticky top-0 z-30 shrink-0 px-6 py-2">
        <div class="flex items-center justify-between">
            <!-- 左侧：返回按钮 + 页面名称 -->
            <div class="flex items-center gap-3">
                <button
                    v-if="appStore.canGoBack"
                    @click="handleBack"
                    class="rounded-full p-2 text-white/75 transition-colors hover:bg-white/10 hover:text-white/95"
                    aria-label="返回上一级"
                >
                    <MdKeyboardArrowLeft class="h-6 w-6" />
                </button>
                <h1 class="text-xl font-semibold tracking-tight text-white/90">
                    {{ appStore.pageTitle }}
                </h1>
            </div>

            <!-- 右侧：设置按钮 -->
            <button
                @click="handleSettings"
                class="rounded-full p-2 text-white/60 transition-colors hover:bg-white/10 hover:text-white/90"
                aria-label="设置"
            >
                <MdSettings class="h-6 w-6" />
            </button>
        </div>
    </nav>
</template>
