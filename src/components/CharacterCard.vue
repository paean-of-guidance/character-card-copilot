<script setup lang="ts">
import { onMounted, ref, watch, onBeforeUnmount } from "vue";
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
    const img = event.target as HTMLImageElement;
    img.style.display = "none";
    console.error("Failed to load character image:", {
        backgroundPath: props.character.backgroundPath,
        thumbnailPath: props.character.thumbnailPath,
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
        },
    );
});
</script>

<template>
    <div
        class="character-card liquid-card group cursor-pointer overflow-hidden"
        @click="handleClick"
    >
        <div class="aspect-square relative overflow-hidden">
            <template v-if="imageSrc">
                <img
                    :src="imageSrc"
                    :alt="character.card.data.name"
                    class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-[1.04]"
                    @error="handleImageError"
                />
                <div class="absolute inset-0 bg-gradient-to-t from-black/70 via-black/15 to-transparent"></div>
                <div class="absolute left-4 right-4 top-4 flex items-start justify-between gap-3">
                    <span class="rounded-full border border-white/20 bg-black/30 px-3 py-1 text-[11px] font-medium text-white/90 backdrop-blur-md">
                        {{ character.card.data.character_version || "未设版本" }}
                    </span>
                    <span class="rounded-full border border-white/15 bg-black/30 px-2.5 py-1 text-[11px] text-white/80 backdrop-blur-md">
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
                class="flex h-full w-full items-center justify-center text-white/50"
                style="background: radial-gradient(circle at top, rgba(99,102,241,0.20), transparent 40%), linear-gradient(180deg, rgba(30,20,60,0.9) 0%, rgba(15,10,30,0.95) 100%);"
            >
                <div class="text-center">
                    <div class="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-2xl border border-white/15 bg-white/8 backdrop-blur">
                        <span class="text-2xl">👤</span>
                    </div>
                    <span class="text-sm font-medium text-white/50">暂无封面</span>
                </div>
            </div>
        </div>
        <div class="space-y-3 p-4">
            <p class="min-h-[4.5rem] text-sm leading-6 text-white/55 line-clamp-3">
                {{ character.card.data.description || "暂无描述" }}
            </p>
            <div class="flex items-center justify-between gap-3 border-t border-white/10 pt-3 text-xs text-white/40">
                <span class="truncate">{{ character.meta.updatedAt ? "最近更新" : "等待完善" }}</span>
                <span class="liquid-badge text-white/60">进入编辑</span>
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
.line-clamp-1 { -webkit-line-clamp: 1; }
.line-clamp-2 { -webkit-line-clamp: 2; }
.line-clamp-3 { -webkit-line-clamp: 3; }
</style>
