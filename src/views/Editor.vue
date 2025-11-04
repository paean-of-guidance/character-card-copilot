<script setup lang="ts">
import { onMounted, ref, watch, nextTick, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import {
    getCharacterByUUID,
    updateCharacter,
    deleteCharacter as deleteCharacterByUUID,
} from "@/services/characterStorage";
import type { TavernCardV2 } from "@/types/character";
import AIPanel from "@/components/AIPanel.vue";
import {
    uploadBackgroundImage,
    updateCharacterBackgroundPath,
} from "@/services/characterStorage";
import { CharacterStateService } from "@/services/characterState";
import { listen } from "@tauri-apps/api/event";
import { tokenCounter } from "@/utils/tokenCounter";
import { useNotification } from "@/composables/useNotification";
import { useModal } from "@/composables/useModal";

const appStore = useAppStore();
const route = useRoute();
const router = useRouter();
const { showSuccessToast, showErrorToast, showWarningToast } = useNotification();
const { showAlertModal } = useModal();
const isLoading = ref(false);
const characterUUID = ref<string>("");
const aiPanelVisible = ref(true);
const backgroundPath = ref<string>("");
const isUploading = ref(false);

// 编辑器容器引用
const editorContainerRef = ref<HTMLElement>();

// Token计数数据
const tokenCounts = ref<Record<string, number>>({});

// 切换AI面板显示状态
function toggleAIPanel() {
    aiPanelVisible.value = !aiPanelVisible.value;
}

// 头像上传功能
async function handleAvatarClick() {
    if (!characterUUID.value) return;

    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/png,image/jpeg,image/jpg,image/webp";

    input.onchange = async (event) => {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) return;

        // 检查文件大小 (限制为5MB)
        if (file.size > 5 * 1024 * 1024) {
            showWarningToast("图片文件大小不能超过5MB", "文件过大");
            return;
        }

        isUploading.value = true;

        try {
            const uploadedPath = await uploadBackgroundImage(
                characterUUID.value,
                file,
            );
            backgroundPath.value = uploadedPath;

            // 更新角色的background_path字段
            await updateCharacterBackgroundPath(
                characterUUID.value,
                uploadedPath,
            );
            console.log("头像上传成功:", uploadedPath);

            // 触发一次自动保存，确保数据同步
            await autoSave();
        } catch (error) {
            console.error("头像上传失败:", error);
            showErrorToast("头像上传失败，请重试", "上传失败");
        } finally {
            isUploading.value = false;
        }
    };

    input.click();
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
            backgroundPath.value = character.backgroundPath || "";
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
        showErrorToast("加载角色数据失败", "加载失败");
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
                    .filter((g: string) => g.trim()),
                tags: characterData.value.tags
                    .split(",")
                    .map((t: string) => t.trim())
                    .filter((t: string) => t),
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
    (newUuid: string | string[]) => {
        if (newUuid && typeof newUuid === "string") {
            loadCharacterData(newUuid);
        }
    },
    { immediate: true },
);

onMounted(async () => {
    appStore.setPageTitle("角色编辑器", true);

    // 页面加载时滚动到顶部
    nextTick(() => {
        if (editorContainerRef.value) {
            editorContainerRef.value.scrollTop = 0;
        }
    });

    // 检查路由参数
    const uuid = route.params.uuid as string;
    if (uuid) {
        await loadCharacterData(uuid);
        // 设置当前活跃角色
        await CharacterStateService.setActiveCharacter(uuid);
    }

    // 监听角色更新事件
    await listen("character-updated", (event) => {
        console.log("收到角色更新事件:", event.payload);
        // 检查事件是否针对当前角色
        if (
            event.payload &&
            typeof event.payload === "object" &&
            "character_uuid" in event.payload &&
            event.payload.character_uuid === characterUUID.value
        ) {
            console.log("刷新当前角色数据");
            // 重新加载角色数据
            loadCharacterData(characterUUID.value);
        }
    });
});

// 计算tokens的函数
function updateTokenCount(fieldName: string, text: string) {
    const count = tokenCounter.countTokens(text)
    tokenCounts.value[fieldName] = count
}

// 监听字段变化更新token计数
watch([
    () => characterData.value.description,
    () => characterData.value.personality,
    () => characterData.value.scenario,
    () => characterData.value.first_mes,
    () => characterData.value.mes_example,
    () => characterData.value.creator_notes,
    () => characterData.value.system_prompt,
    () => characterData.value.post_history_instructions,
    () => characterData.value.alternate_greetings,
    () => characterData.value.tags
], () => {
    updateTokenCount('description', characterData.value.description)
    updateTokenCount('personality', characterData.value.personality)
    updateTokenCount('scenario', characterData.value.scenario)
    updateTokenCount('first_mes', characterData.value.first_mes)
    updateTokenCount('mes_example', characterData.value.mes_example)
    updateTokenCount('creator_notes', characterData.value.creator_notes)
    updateTokenCount('system_prompt', characterData.value.system_prompt)
    updateTokenCount('post_history_instructions', characterData.value.post_history_instructions)
    updateTokenCount('alternate_greetings', characterData.value.alternate_greetings)
    updateTokenCount('tags', characterData.value.tags)
}, { immediate: true })

// 删除角色功能
async function deleteCharacter() {
    if (!characterUUID.value) return

    try {
        const confirmed = await showAlertModal(
            `确定要删除"${characterData.value.name || '这个角色'}"吗？此操作不可撤销。`,
            async () => {
                // 调用删除角色的API
                await deleteCharacterByUUID(characterUUID.value)
                console.log('角色删除成功')
            },
            {
                title: '删除确认',
                type: 'danger',
                confirmText: '确认删除',
                cancelText: '取消'
            }
        )

        if (confirmed) {
            showSuccessToast('角色删除成功', '操作完成')
            // 等待Toast显示一下再跳转
            setTimeout(() => {
                router.push('/')
            }, 500)
        }
    } catch (error) {
        console.error('删除角色失败:', error)
        showErrorToast('删除角色失败，请重试', '删除失败')
    }
}

// 导入角色功能
async function importCharacter() {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json,.png,.card'

    input.onchange = async (event) => {
        const file = (event.target as HTMLInputElement).files?.[0]
        if (!file) return

        try {
            // 这里需要实现角色导入逻辑
            // const importedData = await parseCharacterFile(file)
            // await saveImportedCharacter(importedData)
            showSuccessToast('角色导入成功', '导入完成')
        } catch (error) {
            console.error('导入角色失败:', error)
            showErrorToast('导入角色失败，请检查文件格式', '导入失败')
        }
    }

    input.click()
}

// 组件卸载时清除活跃角色状态
onUnmounted(async () => {
    await CharacterStateService.clearActiveCharacter();
});
</script>

<template>
    <div class="h-[calc(100vh-6rem)] bg-gray-50 p-4">
        <div class="flex h-full gap-4">
            <!-- 左侧：角色信息显示 -->
            <div
                ref="editorContainerRef"
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
                                class="w-24 h-24 rounded-lg flex items-center justify-center shadow-lg overflow-hidden cursor-pointer hover:opacity-80 transition-opacity relative"
                                @click="handleAvatarClick"
                                :class="isUploading ? 'opacity-50' : ''"
                            >
                                <!-- 上传中的加载状态 -->
                                <div
                                    v-if="isUploading"
                                    class="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center"
                                >
                                    <div class="text-white text-xs">
                                        上传中...
                                    </div>
                                </div>

                                <!-- 显示上传的图片 -->
                                <img
                                    v-if="backgroundPath"
                                    :src="
                                        backgroundPath.startsWith('data:')
                                            ? backgroundPath
                                            : `file://${backgroundPath}`
                                    "
                                    alt="角色头像"
                                    class="w-full h-full object-cover"
                                />

                                <!-- 默认头像 -->
                                <div
                                    v-else
                                    class="w-full h-full bg-linear-to-br from-blue-400 to-purple-500 flex items-center justify-center"
                                >
                                    <span class="text-white text-2xl font-bold"
                                        >角色</span
                                    >
                                </div>
                            </div>

                            <!-- 上传提示 -->
                            <div class="text-xs text-gray-500">
                                点击头像上传图片
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

                    <!-- 操作按钮区域 -->
                    <div class="flex gap-2 mb-6">
                        <button
                            @click="deleteCharacter"
                            class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-full flex items-center gap-2"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                            </svg>
                            删除角色
                        </button>
                        <button
                            @click="importCharacter"
                            class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-full flex items-center gap-2"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"/>
                            </svg>
                            导入角色
                        </button>
                    </div>

                    <!-- 其他角色信息表单 -->
                    <div class="space-y-4">
                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    角色描述
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.description || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.description"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="5"
                                placeholder="角色的物理外观、身份和基本设定"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    性格特点
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.personality || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.personality"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="6"
                                placeholder="描述角色的性格特征"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    场景设定
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.scenario || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.scenario"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="描述角色所处的场景和环境"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    开场白
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.first_mes || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.first_mes"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="角色的第一句话或开场问候"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    对话示例
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.mes_example || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.mes_example"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="6"
                                placeholder="示例对话格式，展示角色的说话风格"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    创作者笔记
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.creator_notes || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.creator_notes"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="创作时的备注和说明"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    系统提示词
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.system_prompt || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.system_prompt"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="AI系统使用的提示词"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    历史后指令
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.post_history_instructions || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="
                                    characterData.post_history_instructions
                                "
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="对话历史后的处理指令"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    备用问候语
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.alternate_greetings || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.alternate_greetings"
                                @blur="autoSave"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="备用开场白，用换行分隔多个问候语"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label class="block text-sm font-semibold text-gray-700">
                                    标签
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.tags || 0 }} tokens
                                </span>
                            </div>
                            <input
                                v-model="characterData.tags"
                                @blur="autoSave"
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
                                @blur="autoSave"
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
                                @blur="autoSave"
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
                :character-data="characterData"
                @toggle="toggleAIPanel"
            />

            <!-- 显示/隐藏面板按钮 -->
            <div
                v-if="!aiPanelVisible"
                class="card rounded-xl bg-white p-4 shadow-2xl flex items-center justify-center cursor-pointer hover:bg-gray-50 transition-colors"
                @click="toggleAIPanel"
            >
                <div class="text-center text-gray-500">
                    <svg
                        class="w-6 h-6 mx-auto mb-2"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M15 19l-7-7 7-7"
                        />
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
