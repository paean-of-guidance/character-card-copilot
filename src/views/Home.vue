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

const characters = computed(() => {
    return characterStore.characters.filter(c => c && c.uuid);
});

const characterCount = computed(() => characters.value.length);

onMounted(async () => {
    appStore.setPageTitle("首页", false);
    await characterStore.loadAllCharacters(true);
});

watch(
    () => route.path,
    (newPath, oldPath) => {
        if (newPath === "/" && oldPath?.startsWith("/editor/")) {
            characterStore.loadAllCharacters();
        }
    },
);

onMounted(() => {
    document.addEventListener("visibilitychange", handleVisibilityChange);
});

function handleVisibilityChange() {
    if (!document.hidden && route.path === "/") {
        characterStore.loadAllCharacters(true);
    }
}

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
            return;
        }

        const fileData = await readFile(selected);
        const fileName = selected.split(/[\\/]/).pop() || "character.png";

        const importedCharacter = await characterStore.importCharacterCardFromBytes(
            fileData,
            fileName,
        );

        showSuccessToast("角色导入成功", "导入完成");
        router.push(`/editor/${importedCharacter.uuid}`);
    } catch (error) {
        console.error("导入角色失败:", error);
        showErrorToast("导入角色失败，请检查文件格式", "导入失败");
    }
}
</script>

<template>
    <div class="home h-full min-h-0 w-full overflow-y-auto">
        <div class="mx-auto flex min-h-full w-full max-w-7xl flex-col gap-6 px-4 py-6 lg:px-6 lg:py-8">

            <!-- Hero 区域 -->
            <section class="liquid-panel p-6 lg:p-8">
                <div class="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
                    <div class="max-w-3xl space-y-3">
                        <div class="liquid-badge liquid-badge--primary">
                            Character Workspace
                        </div>
                        <div class="space-y-2">
                            <h1 class="text-2xl font-semibold tracking-tight text-white/90 lg:text-3xl">
                                你的角色库工作台
                            </h1>
                            <p class="max-w-2xl text-sm leading-6 text-white/55 lg:text-base">
                                在这里集中管理角色卡、快速创建新角色，并继续进入编辑与 Copilot 协作流程。
                            </p>
                        </div>
                    </div>

                    <div class="grid gap-3 lg:min-w-[10rem]">
                        <div class="liquid-panel-elevated rounded-2xl px-4 py-3">
                            <div class="text-xs font-medium uppercase tracking-[0.18em] text-white/40">
                                总角色数
                            </div>
                            <div class="mt-2 text-2xl font-semibold text-white/90">
                                {{ characterCount }}
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <!-- 加载骨架 -->
            <div v-if="loading" class="character-grid">
                <div
                    v-for="index in 6"
                    :key="index"
                    class="liquid-card overflow-hidden"
                >
                    <div class="aspect-square animate-pulse bg-white/10"></div>
                    <div class="space-y-3 p-4">
                        <div class="h-5 w-2/3 animate-pulse rounded-full bg-white/10"></div>
                        <div class="h-4 w-full animate-pulse rounded-full bg-white/8"></div>
                        <div class="h-4 w-5/6 animate-pulse rounded-full bg-white/8"></div>
                    </div>
                </div>
            </div>

            <!-- 角色列表区 -->
            <section
                v-else
                class="liquid-panel p-4 lg:p-5"
            >
                <div class="mb-4 flex items-center justify-between gap-3 px-2">
                    <div>
                        <h2 class="text-lg font-semibold text-white/90">角色列表</h2>
                        <p class="mt-1 text-sm text-white/45">
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
