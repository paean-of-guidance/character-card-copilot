<script setup lang="ts">
import { onMounted } from "vue";
import type { CharacterData } from "@/types/character";

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
        startsWithData: props.character.backgroundPath.startsWith("data:"),
        length: props.character.backgroundPath.length,
    });
}

onMounted(() => {
    console.log(
        `CharacterCard mounted for ${props.character.card.data.name}:`,
        {
            backgroundPath: props.character.backgroundPath,
            isBase64: props.character.backgroundPath.startsWith("data:"),
        },
    );
});
</script>

<template>
    <div
        class="character-card bg-white rounded-lg shadow-md overflow-hidden cursor-pointer transition-transform hover:scale-105"
        @click="handleClick"
    >
        <div class="aspect-square relative bg-gray-200">
            <img
                v-if="character.backgroundPath"
                :src="
                    character.backgroundPath.startsWith('data:')
                        ? character.backgroundPath
                        : `file://${character.backgroundPath}`
                "
                :alt="character.card.data.name"
                class="w-full h-full object-cover"
                @error="handleImageError"
            />
            <div
                v-else
                class="w-full h-full flex items-center justify-center text-gray-400"
            >
                <div class="text-center">
                    <div class="text-4xl mb-2">👤</div>
                    <span class="text-sm">暂无图片</span>
                </div>
            </div>
        </div>
        <div class="p-4">
            <h3 class="text-lg font-semibold text-gray-800 truncate">
                {{ character.card.data.name || "未命名角色" }}
            </h3>
            <p class="text-sm text-gray-600 mt-1 line-clamp-2">
                {{ character.card.data.description || "暂无描述" }}
            </p>
            <p class="text-xs text-gray-500 mt-1 line-clamp-2">
                {{ character.card.data.character_version || "暂无版本" }}
            </p>
        </div>
    </div>
</template>

<style scoped>
.line-clamp-2 {
    display: -webkit-box;
    -line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}
</style>
