<script setup lang="ts">
import { onMounted, onUnmounted, watch, computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { storeToRefs } from "pinia";
import { useAppStore } from "@/stores/app";
import { useCharacterStore } from "@/stores/character";
import { useNotification } from "@/composables/useNotification";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import CharacterCard from "@/components/CharacterCard.vue";
import NewCharacterCard from "@/components/NewCharacterCard.vue";

const appStore = useAppStore();
const characterStore = useCharacterStore();
const router = useRouter();
const route = useRoute();
const { loading } = storeToRefs(characterStore);
const { showSuccessToast, showErrorToast } = useNotification();

// 过滤掉无效的角色数据（安全防护）
const characters = computed(() => {
    return characterStore.characters.filter(c => c && c.uuid);
});

const characterCount = computed(() => characters.value.length);

onMounted(async () => {
    appStore.setPageTitle("首页", false);
    await characterStore.loadAllCharacters(true); // 强制加载
});

// 监听路由变化，当从编辑器返回时重新加载角色数据
watch(
    () => route.path,
    (newPath, oldPath) => {
        // 当从编辑器页面返回首页时，重新加载角色数据
        if (newPath === "/" && oldPath?.startsWith("/editor/")) {
            characterStore.loadAllCharacters();
        }
    },
);

// 添加页面可见性监听，当页面重新获得焦点时重新加载数据
onMounted(() => {
    document.addEventListener("visibilitychange", handleVisibilityChange);
});

function handleVisibilityChange() {
    if (!document.hidden && route.path === "/") {
        characterStore.loadAllCharacters(true); // 强制重新加载
    }
}

// 清理事件监听器
onUnmounted(() => {
    document.removeEventListener("visibilitychange", handleVisibilityChange);
});

function handleCharacterClick(uuid: string) {
    router.push(`/editor/${uuid}`);
}

async function handleNewCharacter() {
    try {
        const newCharacter = await characterStore.createCharacter("新角色");
        showSuccessToast("角色创建成功", "创建完成");
        router.push(`/editor/${newCharacter.uuid}`);
    } catch (error) {
        console.error("创建角色卡失败:", error);
        showErrorToast("创建角色失败，请重试", "创建失败");
    }
}

async function handleImportCharacter() {
    try {
        // 打开文件选择对话框
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "角色卡文件",
                    extensions: ["png", "json", "card"],
                },
            ],
        });

        if (!selected || typeof selected !== "string") {
            // 用户取消了选择
            return;
        }

        // 使用 Tauri fs 插件读取文件内容
        const fileData = await readFile(selected);
        const fileName = selected.split(/[\\/]/).pop() || "character.png";

        // 调用导入API
        const importedCharacter = await characterStore.importCharacterCardFromBytes(
            fileData,
            fileName,
        );

        showSuccessToast("角色导入成功", "导入完成");

        // 跳转到导入的角色编辑页面
        router.push(`/editor/${importedCharacter.uuid}`);
    } catch (error) {
        console.error("导入角色失败:", error);
        showErrorToast("导入角色失败，请检查文件格式", "导入失败");
    }
}
</script>

<template>
    <div class="home h-full min-h-0 w-full overflow-y-auto bg-[radial-gradient(circle_at_top,_rgba(59,130,246,0.10),_transparent_32%),linear-gradient(180deg,_#f8fafc_0%,_#eef2ff_100%)]">
        <div class="mx-auto flex min-h-full w-full max-w-7xl flex-col gap-6 px-4 py-6 lg:px-6 lg:py-8">
            <section class="rounded-[28px] border border-white/70 bg-white/75 p-6 shadow-[0_16px_50px_rgba(148,163,184,0.16)] backdrop-blur-xl lg:p-8">
                <div class="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
                    <div class="max-w-3xl space-y-3">
                        <div class="inline-flex items-center gap-2 rounded-full border border-blue-200 bg-blue-50 px-3 py-1 text-xs font-medium text-blue-700">
                            Character Workspace
                        </div>
                        <div class="space-y-2">
                            <h1 class="text-2xl font-semibold tracking-tight text-slate-900 lg:text-3xl">
                                你的角色库工作台
                            </h1>
                            <p class="max-w-2xl text-sm leading-6 text-slate-600 lg:text-base">
                                在这里集中管理角色卡、快速创建新角色，并继续进入编辑与 Copilot 协作流程。
                            </p>
                        </div>
                    </div>

                    <div class="grid gap-3 lg:min-w-[10rem]">
                        <div class="rounded-2xl border border-slate-200 bg-white/85 px-4 py-3 shadow-sm">
                            <div class="text-xs font-medium uppercase tracking-[0.18em] text-slate-400">
                                总角色数
                            </div>
                            <div class="mt-2 text-2xl font-semibold text-slate-900">
                                {{ characterCount }}
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <div v-if="loading" class="character-grid">
                <div
                    v-for="index in 6"
                    :key="index"
                    class="overflow-hidden rounded-[24px] border border-white/60 bg-white/70 shadow-[0_12px_32px_rgba(148,163,184,0.14)] backdrop-blur"
                >
                    <div class="aspect-square animate-pulse bg-slate-200/80"></div>
                    <div class="space-y-3 p-4">
                        <div class="h-5 w-2/3 animate-pulse rounded-full bg-slate-200/80"></div>
                        <div class="h-4 w-full animate-pulse rounded-full bg-slate-200/70"></div>
                        <div class="h-4 w-5/6 animate-pulse rounded-full bg-slate-200/70"></div>
                    </div>
                </div>
            </div>

            <section
                v-else
                class="rounded-[28px] border border-white/65 bg-white/55 p-4 shadow-[0_16px_40px_rgba(148,163,184,0.14)] backdrop-blur-xl lg:p-5"
            >
                <div class="mb-4 flex items-center justify-between gap-3 px-2">
                    <div>
                        <h2 class="text-lg font-semibold text-slate-900">角色列表</h2>
                        <p class="mt-1 text-sm text-slate-500">
                            点击任意角色卡片，继续编辑设定与 AI 协作。
                        </p>
                    </div>
                </div>

                <div class="character-grid">
                    <CharacterCard
                        v-for="character in characters"
                        :key="character.uuid"
                        :character="character"
                        @click="handleCharacterClick"
                    />
                    <NewCharacterCard
                        @create-new="handleNewCharacter"
                        @import="handleImportCharacter"
                    />
                </div>
            </section>
        </div>
    </div>
</template>

<style scoped>
.home {
    scrollbar-gutter: stable;
}

.character-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1.25rem;
    align-content: start;
}
</style>
