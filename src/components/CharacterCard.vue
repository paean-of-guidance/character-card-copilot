<script setup lang="ts">
import {  onMounted, ref, watch, onBeforeUnmount } from "vue";
import { readFile } from "@tauri-apps/plugin-fs";
import type { CharacterData } from "@/types/character";
import { devLog } from "@/utils/logger";

const props = defineProps<{
    character: CharacterData;
}>();

const emit = defineEmits<{
    click: [uuid: string];
}>();

function handleClick() {
    emit("click", props.character.uuid);
}

function handleImageError(event: Event) {
    // 图片加载失败时隐藏图片，显示默认占位符
    const img = event.target as HTMLImageElement;
    img.style.display = "none";

    // 添加调试信息
    console.error("Failed to load character image:", {
        backgroundPath: props.character.backgroundPath,
        thumbnailPath: props.character.thumbnailPath,
        startsWithData: props.character.backgroundPath.startsWith("data:"),
        length: props.character.backgroundPath.length,
    });
}

const imageSrc = ref("");
let revokeUrl: string | null = null;

function revokeImageUrl() {
    if (revokeUrl) {
        URL.revokeObjectURL(revokeUrl);
        revokeUrl = null;
    }
}

async function loadImage(path: string) {
    if (!path) {
        imageSrc.value = "";
        revokeImageUrl();
        return;
    }

    if (path.startsWith("data:")) {
        revokeImageUrl();
        imageSrc.value = path;
        return;
    }

    try {
        const normalized = path.replace(/\\/g, "/");
        const data = await readFile(normalized);
        const blob = new Blob([data], { type: "image/png" });
        revokeImageUrl();
        const url = URL.createObjectURL(blob);
        imageSrc.value = url;
        revokeUrl = url;
    } catch (error) {
        console.error("读取角色图片失败", error);
        revokeImageUrl();
        imageSrc.value = "";
    }
}

watch(
    () => [props.character.thumbnailPath, props.character.backgroundPath],
    ([thumb, bg]) => {
        loadImage(thumb || bg || "");
    },
    { immediate: true },
);

onBeforeUnmount(() => {
    revokeImageUrl();
});

onMounted(() => {
    devLog(
        `CharacterCard mounted for ${props.character.card.data.name}:`,
        {
            backgroundPath: props.character.backgroundPath,
            thumbnailPath: props.character.thumbnailPath,
            isBase64: props.character.backgroundPath.startsWith("data:"),
        },
    );
});
</script>

<template>
    <div
        class="character-card group overflow-hidden rounded-[24px] border border-white/70 bg-white/85 shadow-[0_12px_32px_rgba(148,163,184,0.18)] backdrop-blur transition-all duration-300 hover:-translate-y-1 hover:border-blue-200 hover:shadow-[0_22px_50px_rgba(96,165,250,0.16)]"
        @click="handleClick"
    >
        <div class="aspect-square relative overflow-hidden bg-slate-200">
            <template v-if="imageSrc">
                <img
                    :src="imageSrc"
                    :alt="character.card.data.name"
                    class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-[1.03]"
                    @error="handleImageError"
                />
                <div class="absolute inset-0 bg-gradient-to-t from-slate-950/55 via-slate-900/10 to-transparent"></div>
                <div class="absolute left-4 right-4 top-4 flex items-start justify-between gap-3">
                    <span class="rounded-full border border-white/20 bg-white/15 px-3 py-1 text-[11px] font-medium text-white/90 backdrop-blur-md">
                        {{ character.card.data.character_version || "未设版本" }}
                    </span>
                    <span class="rounded-full border border-white/15 bg-slate-950/25 px-2.5 py-1 text-[11px] text-white/85 backdrop-blur-md">
                        角色卡
                    </span>
                </div>
                <div class="absolute inset-x-0 bottom-0 p-4 text-white">
                    <h3 class="text-lg font-semibold tracking-tight drop-shadow-sm line-clamp-1">
                        {{ character.card.data.name || "未命名角色" }}
                    </h3>
                </div>
            </template>
            <div
                v-else
                class="flex h-full w-full items-center justify-center bg-[radial-gradient(circle_at_top,_rgba(148,163,184,0.24),_transparent_30%),linear-gradient(180deg,_#e2e8f0_0%,_#cbd5e1_100%)] text-slate-500"
            >
                <div class="text-center">
                    <div class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-2xl border border-white/50 bg-white/45 shadow-sm backdrop-blur">
                        <span class="text-2xl">👤</span>
                    </div>
                    <span class="text-sm font-medium">暂无封面</span>
                </div>
            </div>
        </div>
        <div class="space-y-3 p-4">
            <p class="text-sm leading-6 text-slate-600 line-clamp-3 min-h-[4.5rem]">
                {{ character.card.data.description || "暂无描述" }}
            </p>
            <div class="flex items-center justify-between gap-3 border-t border-slate-100 pt-3 text-xs text-slate-500">
                <span class="truncate">{{ character.meta.updatedAt ? "最近更新" : "等待完善" }}</span>
                <span class="rounded-full bg-slate-100 px-2.5 py-1 font-medium text-slate-600">进入编辑</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.line-clamp-1,
.line-clamp-2,
.line-clamp-3 {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.line-clamp-1 {
    -webkit-line-clamp: 1;
}

.line-clamp-2 {
    -webkit-line-clamp: 2;
}

.line-clamp-3 {
    -webkit-line-clamp: 3;
}
</style>
