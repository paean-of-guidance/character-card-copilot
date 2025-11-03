<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import {
    getCharacterByUUID,
    updateCharacter,
} from "@/services/characterStorage";
import type { CharacterData, TavernCardV2 } from "@/types/character";
import AIPanel from "@/components/AIPanel.vue";

const appStore = useAppStore();
const route = useRoute();
const router = useRouter();
const isLoading = ref(false);
const characterUUID = ref<string>("");
const aiPanelVisible = ref(true);

// 切换AI面板显示状态
function toggleAIPanel() {
    aiPanelVisible.value = !aiPanelVisible.value;
}

// 角色数据
const characterData = ref({
    name: "",
    description: "",
    personality: "",
    scenario: "",
    first_mes: "",
    mes_example: "",
    creator_notes: "",
    system_prompt: "",
    post_history_instructions: "",
    alternate_greetings: "",
    tags: "",
    creator: "",
    character_version: "",
});

// 加载角色数据
async function loadCharacterData(uuid: string) {
    if (!uuid) return;

    isLoading.value = true;
    try {
        const character = await getCharacterByUUID(uuid);
        if (character) {
            characterUUID.value = uuid;
            // 将TavernCardV2数据映射到表单
            characterData.value = {
                name: character.card.data.name,
                description: character.card.data.description,
                personality: character.card.data.personality,
                scenario: character.card.data.scenario,
                first_mes: character.card.data.first_mes,
                mes_example: character.card.data.mes_example,
                creator_notes: character.card.data.creator_notes,
                system_prompt: character.card.data.system_prompt,
                post_history_instructions:
                    character.card.data.post_history_instructions,
                alternate_greetings:
                    character.card.data.alternate_greetings.join("\n"),
                tags: character.card.data.tags.join(", "),
                creator: character.card.data.creator,
                character_version: character.card.data.character_version,
            };
        }
    } catch (error) {
        console.error("加载角色数据失败:", error);
        alert("加载角色数据失败");
    } finally {
        isLoading.value = false;
    }
}

// 自动保存函数
async function autoSave() {
    if (!characterUUID.value) return;

    try {
        const tavernCardV2: TavernCardV2 = {
            spec: "chara_card_v2",
            spec_version: "2.0",
            data: {
                name: characterData.value.name,
                description: characterData.value.description,
                personality: characterData.value.personality,
                scenario: characterData.value.scenario,
                first_mes: characterData.value.first_mes,
                mes_example: characterData.value.mes_example,
                creator_notes: characterData.value.creator_notes,
                system_prompt: characterData.value.system_prompt,
                post_history_instructions:
                    characterData.value.post_history_instructions,
                alternate_greetings: characterData.value.alternate_greetings
                    .split("\n")
                    .filter((g) => g.trim()),
                tags: characterData.value.tags
                    .split(",")
                    .map((t) => t.trim())
                    .filter((t) => t),
                creator: characterData.value.creator,
                character_version: characterData.value.character_version,
                extensions: {},
            },
        };

        await updateCharacter(characterUUID.value, tavernCardV2);
        console.log("角色数据已自动保存");
    } catch (error) {
        console.error("自动保存失败:", error);
    }
}

// 监听路由参数变化
watch(
    () => route.params.uuid,
    (newUuid) => {
        if (newUuid && typeof newUuid === "string") {
            loadCharacterData(newUuid);
        }
    },
    { immediate: true },
);

onMounted(() => {
    appStore.setPageTitle("角色编辑器", true);
    // 检查路由参数
    const uuid = route.params.uuid as string;
    if (uuid) {
        loadCharacterData(uuid);
    }
});
</script>

<template>
    <div class="h-[calc(100vh-6rem)] bg-gray-50 p-4">
        <div class="flex h-full gap-4">
            <!-- 左侧：角色信息显示 -->
            <div
                class="card rounded-xl bg-white p-6 overflow-y-auto shadow-2xl"
                :class="aiPanelVisible ? 'w-1/2' : 'w-full'"
            >
                <!-- 加载状态 -->
                <div
                    v-if="isLoading"
                    class="flex items-center justify-center h-64"
                >
                    <div class="text-gray-600">加载角色数据中...</div>
                </div>

                <div v-else>
                    <!-- 上方：角色卡预览 + 角色名 -->
                    <div class="mb-6">
                        <div class="flex items-center gap-4 mb-4">
                            <!-- 角色卡预览 -->
                            <div
                                class="w-24 h-24 bg-gradient-to-br from-blue-400 to-purple-500 rounded-lg flex items-center justify-center shadow-lg"
                            >
                                <span class="text-white text-2xl font-bold"
                                    >角色</span
                                >
                            </div>

                            <!-- 角色名 -->
                            <div class="flex-1">
                                <label
                                    class="block text-sm font-semibold text-gray-700 mb-2"
                                    >角色名称</label
                                >
                                <input
                                    v-model="characterData.name"
                                    @blur="autoSave"
                                    type="text"
                                    class="w-full bg-white border border-gray-200 rounded-lg px-4 py-2 text-lg font-medium"
                                    placeholder="请输入角色名称"
                                />
                            </div>
                        </div>
                    </div>

                    <!-- 其他角色信息表单 -->
                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >角色描述</label
                            >
                            <textarea
                                v-model="characterData.description"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="角色的物理外观、身份和基本设定"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >性格特点</label
                            >
                            <textarea
                                v-model="characterData.personality"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="描述角色的性格特征"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >场景设定</label
                            >
                            <textarea
                                v-model="characterData.scenario"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="描述角色所处的场景和环境"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >开场白</label
                            >
                            <textarea
                                v-model="characterData.first_mes"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="角色的第一句话或开场问候"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >对话示例</label
                            >
                            <textarea
                                v-model="characterData.mes_example"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="6"
                                placeholder="示例对话格式，展示角色的说话风格"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >创作者笔记</label
                            >
                            <textarea
                                v-model="characterData.creator_notes"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="创作时的备注和说明"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >系统提示词</label
                            >
                            <textarea
                                v-model="characterData.system_prompt"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="AI系统使用的提示词"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >历史后指令</label
                            >
                            <textarea
                                v-model="
                                    characterData.post_history_instructions
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="对话历史后的处理指令"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >备用问候语</label
                            >
                            <textarea
                                v-model="characterData.alternate_greetings"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="备用开场白，用换行分隔多个问候语"
                            ></textarea>
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >标签</label
                            >
                            <input
                                v-model="characterData.tags"
                                type="text"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                                placeholder="角色标签，用逗号分隔"
                            />
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >创作者</label
                            >
                            <input
                                v-model="characterData.creator"
                                type="text"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                                placeholder="创作者名称"
                            />
                        </div>

                        <div>
                            <label
                                class="block text-sm font-semibold text-gray-700 mb-2"
                                >角色版本</label
                            >
                            <input
                                v-model="characterData.character_version"
                                type="text"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                                placeholder="角色卡版本号"
                            />
                        </div>
                    </div>
                </div>
            </div>

            <!-- 右侧：AI Panel -->
            <AIPanel
                v-if="aiPanelVisible"
                :visible="aiPanelVisible"
                panel-type="ai"
                @toggle="toggleAIPanel"
            />

            <!-- 显示/隐藏面板按钮 -->
            <div
                v-if="!aiPanelVisible"
                class="card rounded-xl bg-white p-4 shadow-2xl flex items-center justify-center cursor-pointer hover:bg-gray-50 transition-colors"
                @click="toggleAIPanel"
            >
                <div class="text-center text-gray-500">
                    <svg class="w-6 h-6 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                    </svg>
                    <span class="text-xs">显示 AI 面板</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
/* 自定义滚动条样式 */
.overflow-y-auto::-webkit-scrollbar {
    width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
    background: #f1f1f1;
    border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: #a8a8a8;
}

/* 输入框焦点样式 */
input:focus,
textarea:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
</style>
