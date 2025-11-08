<script setup lang="ts">
import { onMounted, ref, watch, nextTick, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { useCharacterStore } from "@/stores/character";
import {
    updateCharacterField,
    deleteCharacter as deleteCharacterByUUID,
    exportCharacterCard,
} from "@/services/characterStorage";
import AIPanel from "@/components/AIPanel.vue";
import WorldBookEditor from "@/components/WorldBookEditor.vue";
import {
    uploadBackgroundImage,
    updateCharacterBackgroundPath,
} from "@/services/characterStorage";
import { save } from "@tauri-apps/plugin-dialog";
import { CharacterStateService } from "@/services/characterState";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { tokenCounter } from "@/utils/tokenCounter";
import { useNotification } from "@/composables/useNotification";
import { useModal } from "@/composables/useModal";
import type {
    CharacterLoadedPayload,
    CharacterUpdatedPayload,
    SessionUnloadedPayload,
    ErrorPayload,
} from "@/types/events";

const appStore = useAppStore();
const characterStore = useCharacterStore();
const route = useRoute();
const router = useRouter();
const { showSuccessToast, showErrorToast, showWarningToast } =
    useNotification();
const { showAlertModal } = useModal();
const isLoading = ref(false);
const characterUUID = ref<string>("");
const aiPanelVisible = ref(true);
const backgroundPath = ref<string>("");
const isUploading = ref(false);

// 编辑器模式：'character' 或 'worldBook'
const editorMode = ref<"character" | "worldBook">("character");

// 编辑器容器引用
const editorContainerRef = ref<HTMLElement>();

// Token计数数据
const tokenCounts = ref<Record<string, number>>({});

// 后端事件监听相关状态
const eventUnlisteners = ref<(() => void)[]>([]);

// 切换AI面板显示状态
function toggleAIPanel() {
    aiPanelVisible.value = !aiPanelVisible.value;
}

// 切换编辑器模式
function toggleEditorMode() {
    const newMode =
        editorMode.value === "character" ? "worldBook" : "character";
    editorMode.value = newMode;

    // 世界书模式下自动隐藏AI面板，获得更多空间
    // if (newMode === "worldBook") {
    //     aiPanelVisible.value = false;
    // }
}

// ==================== 后端事件监听 ====================

/**
 * 初始化后端事件监听器
 */
async function initializeBackendEventListeners() {
    console.log("Editor: 初始化后端事件监听器...");

    // 角色加载事件
    const unlistenCharacterLoaded = await listen<CharacterLoadedPayload>(
        "character-loaded",
        async (event) => {
            console.log("Editor: 🎭 角色加载事件:", event.payload);
            const payload = event.payload;

            // ✅ 更新 Store 缓存（不会闪烁）
            characterStore.updateCharacterFromBackend(payload.uuid, payload.character_data);

            // 如果是当前编辑的角色，更新本地数据
            if (payload.uuid === characterUUID.value) {
                console.log("Editor: 更新角色数据到编辑器");
                await updateEditorFromCharacterData(payload.character_data);
            }
        },
    );

    // 角色更新事件
    const unlistenCharacterUpdated = await listen<CharacterUpdatedPayload>(
        "character-updated",
        async (event) => {
            console.log("Editor: 🔄 角色更新事件:", event.payload);
            const payload = event.payload;

            // ✅ 更新 Store 缓存（工具调用修改后会触发此事件）
            characterStore.updateCharacterFromBackend(payload.uuid, payload.character_data);

            // 如果是当前编辑的角色，更新本地数据
            if (payload.uuid === characterUUID.value) {
                console.log("Editor: 角色数据已更新，同步到编辑器");
                await updateEditorFromCharacterData(payload.character_data);

                // 显示更新通知
                switch (payload.update_type) {
                    case "BasicInfo":
                        showSuccessToast("角色基本信息已更新", "数据同步");
                        break;
                    case "Worldbook":
                        showSuccessToast("世界书已更新", "数据同步");
                        break;
                    case "Tags":
                        showSuccessToast("角色标签已更新", "数据同步");
                        break;
                    case "FullData":
                        showSuccessToast("角色数据已更新", "数据同步");
                        break;
                    default:
                        if (
                            typeof payload.update_type === "object" &&
                            "Fields" in payload.update_type
                        ) {
                            showSuccessToast("角色字段已更新", "数据同步");
                        }
                }
            }
        },
    );

    // 会话卸载事件
    const unlistenSessionUnloaded = await listen<SessionUnloadedPayload>(
        "session-unloaded",
        (event) => {
            console.log("Editor: 🚪 会话卸载事件:", event.payload);
            const payload = event.payload;

            // 如果是当前编辑角色的会话被卸载，显示提示
            if (payload.uuid === characterUUID.value) {
                showWarningToast("角色会话已结束", "会话管理");
            }
        },
    );

    // 错误事件
    const unlistenError = await listen<ErrorPayload>("error", (event) => {
        console.error("Editor: ❌ 错误事件:", event.payload);
        const payload = event.payload;

        // 如果是当前编辑角色相关的错误，显示错误提示
        if (payload.uuid === characterUUID.value) {
            showErrorToast(
                `系统错误: ${payload.error_message}`,
                payload.error_code,
            );
        }
    });

    // 保存所有清理函数
    eventUnlisteners.value.push(
        unlistenCharacterLoaded,
        unlistenCharacterUpdated,
        unlistenSessionUnloaded,
        unlistenError,
    );

    console.log("Editor: ✅ 后端事件监听器初始化完成");
}

/**
 * 清理所有事件监听器
 */
function cleanupEventListeners() {
    console.log("Editor: 清理事件监听器...");
    eventUnlisteners.value.forEach((unlisten) => {
        try {
            unlisten();
        } catch (error) {
            console.error("Editor: 清理事件监听器失败:", error);
        }
    });
    eventUnlisteners.value = [];
    console.log("Editor: ✅ 事件监听器清理完成");
}

/**
 * 从CharacterData更新编辑器表单数据
 */
async function updateEditorFromCharacterData(incomingCharacterData: any) {
    try {
        // 保存完整的角色对象
        fullCharacterData.value = incomingCharacterData;

        // 更新表单数据
        const cardData = incomingCharacterData.card.data;
        characterData.value = {
            name: cardData.name || "",
            description: cardData.description || "",
            personality: cardData.personality || "",
            scenario: cardData.scenario || "",
            first_mes: cardData.first_mes || "",
            mes_example: cardData.mes_example || "",
            creator_notes: cardData.creator_notes || "",
            system_prompt: cardData.system_prompt || "",
            post_history_instructions: cardData.post_history_instructions || "",
            alternate_greetings: cardData.alternate_greetings?.join("\n") || "",
            tags: cardData.tags?.join(", ") || "",
            creator: cardData.creator || "",
            character_version: cardData.character_version || "",
        };

        // 更新背景路径
        backgroundPath.value = incomingCharacterData.backgroundPath || "";

        console.log("Editor: 角色数据已同步到编辑器");
    } catch (error) {
        console.error("Editor: 更新编辑器数据失败:", error);
        showErrorToast("同步角色数据失败", "数据同步错误");
    }
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
        } catch (error) {
            console.error("头像上传失败:", error);
            showErrorToast("头像上传失败，请重试", "上传失败");
        } finally {
            isUploading.value = false;
        }
    };

    input.click();
}

// 完整的角色数据对象（用于传递给 AI）
const fullCharacterData = ref<any>(null);

// 角色表单数据（用于编辑）
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
        // ✅ 使用 Store 加载（带缓存）
        const character = await characterStore.getCharacterByUUID(uuid);
        if (character) {
            characterUUID.value = uuid;
            backgroundPath.value = character.backgroundPath || "";

            // 🔥 新增：触发后端会话加载，让AI可以看到角色数据
            console.log("Editor: 触发后端会话加载...", uuid);
            try {
                await invoke("load_character_session", { uuid });
                console.log("Editor: 后端会话加载成功");
            } catch (error) {
                console.error("Editor: 后端会话加载失败:", error);
            }

            // 保存完整的 character 对象（用于传递给 AI）
            fullCharacterData.value = character;

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

// 更新单个字段（安全保存，保留世界书等数据）
async function updateField(
    fieldName: string,
    oldValue: string | string[],
    newValue: string | string[],
) {
    if (!characterUUID.value) return;

    // 转换字符串数组为字符串进行比较
    const oldStr = Array.isArray(oldValue) ? oldValue.join("\n") : oldValue || "";
    const newStr = Array.isArray(newValue)
        ? newValue.join("\n")
        : newValue || "";

    // 只有值真正改变时才更新
    if (oldStr !== newStr) {
        try {
            await updateCharacterField(characterUUID.value, fieldName, newStr);
            console.log(`字段 ${fieldName} 已保存`);
        } catch (error) {
            console.error(`更新字段 ${fieldName} 失败:`, error);
            showErrorToast(`保存 ${fieldName} 失败`, "保存错误");
        }
    } else {
        console.log(`字段 ${fieldName} 值未变化，跳过保存`);
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

    // 初始化后端事件监听器
    await initializeBackendEventListeners();

    // 检查路由参数
    const uuid = route.params.uuid as string;
    if (uuid) {
        await loadCharacterData(uuid);
        // 设置当前活跃角色
        await CharacterStateService.setActiveCharacter(uuid);
    }

    // ✅ 已移除旧的事件监听器，使用 initializeBackendEventListeners 中的标准监听器
});

// 组件卸载时清理事件监听器
onUnmounted(() => {
    cleanupEventListeners();
});

// 计算tokens的函数
function updateTokenCount(fieldName: string, text: string) {
    const count = tokenCounter.countTokens(text);
    tokenCounts.value[fieldName] = count;
}

// 监听字段变化更新token计数
watch(
    [
        () => characterData.value.description,
        () => characterData.value.personality,
        () => characterData.value.scenario,
        () => characterData.value.first_mes,
        () => characterData.value.mes_example,
        () => characterData.value.creator_notes,
        () => characterData.value.system_prompt,
        () => characterData.value.post_history_instructions,
        () => characterData.value.alternate_greetings,
        () => characterData.value.tags,
    ],
    () => {
        updateTokenCount("description", characterData.value.description);
        updateTokenCount("personality", characterData.value.personality);
        updateTokenCount("scenario", characterData.value.scenario);
        updateTokenCount("first_mes", characterData.value.first_mes);
        updateTokenCount("mes_example", characterData.value.mes_example);
        updateTokenCount("creator_notes", characterData.value.creator_notes);
        updateTokenCount("system_prompt", characterData.value.system_prompt);
        updateTokenCount(
            "post_history_instructions",
            characterData.value.post_history_instructions,
        );
        updateTokenCount(
            "alternate_greetings",
            characterData.value.alternate_greetings,
        );
        updateTokenCount("tags", characterData.value.tags);
    },
    { immediate: true },
);

// 删除角色功能
async function deleteCharacter() {
    if (!characterUUID.value) return;

    try {
        const confirmed = await showAlertModal(
            `确定要删除"${characterData.value.name || "这个角色"}"吗？此操作不可撤销。`,
            async () => {
                // 调用删除角色的API
                await deleteCharacterByUUID(characterUUID.value);
                console.log("角色删除成功");
            },
            {
                title: "删除确认",
                type: "danger",
                confirmText: "确认删除",
                cancelText: "取消",
            },
        );

        if (confirmed) {
            showSuccessToast("角色删除成功", "操作完成");
            // 等待Toast显示一下再跳转
            setTimeout(() => {
                router.push("/");
            }, 500);
        }
    } catch (error) {
        console.error("删除角色失败:", error);
        showErrorToast("删除角色失败，请重试", "删除失败");
    }
}

// 导出角色功能
async function exportCharacter() {
    if (!characterUUID.value) return;

    try {
        isLoading.value = true;

        // 使用角色名称作为文件名，如果没有图片导出 JSON，有图片导出 PNG
        const hasImage = !!backgroundPath.value;
        const fileName = characterData.value.name || "未命名角色";
        const extension = hasImage ? "png" : "json";

        // 打开保存对话框
        const filePath = await save({
            defaultPath: `${fileName}.${extension}`,
            filters: [
                {
                    name: hasImage ? "PNG 图片" : "JSON 文件",
                    extensions: [extension],
                },
            ],
        });

        if (!filePath) {
            // 用户取消了保存
            return;
        }

        // 调用导出API
        const fileType = await exportCharacterCard(
            characterUUID.value,
            filePath,
        );
        showSuccessToast(
            `角色已导出为 ${fileType.toUpperCase()} 格式`,
            "导出成功",
        );
    } catch (error) {
        console.error("导出角色失败:", error);
        showErrorToast("导出角色失败，请重试", "导出失败");
    } finally {
        isLoading.value = false;
    }
}

// 组件卸载时清除活跃角色状态
onUnmounted(async () => {
    await CharacterStateService.clearActiveCharacter();
});
</script>

<template>
    <div class="h-[calc(100vh-5rem)] bg-gray-50 w-full px-1 py-2">
        <div class="flex h-full w-full gap-2">
            <!-- 左侧：角色信息显示 -->
            <div
                ref="editorContainerRef"
                class="card rounded-xl bg-white p-3 overflow-y-auto shadow-2xl"
                :class="aiPanelVisible ? 'w-[70%]' : 'w-full'"
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
                                    @blur="
                                        updateField(
                                            'name',
                                            fullCharacterData?.card?.data?.name || '',
                                            characterData.name,
                                        )
                                    "
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
                            class="bg-red-500 hover:bg-red-700 text-white text-sm font-medium py-1.5 px-3 rounded-full flex items-center gap-1.5"
                        >
                            <svg
                                class="w-3.5 h-3.5"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                />
                            </svg>
                            删除角色
                        </button>
                        <button
                            @click="exportCharacter"
                            class="bg-blue-500 hover:bg-blue-700 text-white text-sm font-medium py-1.5 px-3 rounded-full flex items-center gap-1.5"
                        >
                            <svg
                                class="w-3.5 h-3.5"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 11l3 3m0 0l3-3m-3 3V8"
                                />
                            </svg>
                            导出角色
                        </button>
                        <button
                            @click="toggleEditorMode"
                            class="bg-purple-500 hover:bg-purple-700 text-white text-sm font-medium py-1.5 px-3 rounded-full flex items-center gap-1.5"
                        >
                            <svg
                                class="w-3.5 h-3.5"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                                />
                            </svg>
                            {{
                                editorMode === "character"
                                    ? "世界书编辑"
                                    : "角色编辑"
                            }}
                        </button>
                    </div>

                    <!-- 角色编辑表单 -->
                    <div v-if="editorMode === 'character'" class="space-y-4">
                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    角色描述
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.description || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.description"
                                @blur="
                                    updateField(
                                        'description',
                                        fullCharacterData?.card?.data?.description || '',
                                        characterData.description,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="5"
                                placeholder="角色的物理外观、身份和基本设定"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    性格特点
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.personality || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.personality"
                                @blur="
                                    updateField(
                                        'personality',
                                        fullCharacterData?.card?.data?.personality || '',
                                        characterData.personality,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="6"
                                placeholder="描述角色的性格特征"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    场景设定
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.scenario || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.scenario"
                                @blur="
                                    updateField(
                                        'scenario',
                                        fullCharacterData?.card?.data?.scenario || '',
                                        characterData.scenario,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="描述角色所处的场景和环境"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    开场白
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.first_mes || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.first_mes"
                                @blur="
                                    updateField(
                                        'first_mes',
                                        fullCharacterData?.card?.data?.first_mes || '',
                                        characterData.first_mes,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="角色的第一句话或开场问候"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    对话示例
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.mes_example || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.mes_example"
                                @blur="
                                    updateField(
                                        'mes_example',
                                        fullCharacterData?.card?.data?.mes_example || '',
                                        characterData.mes_example,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="6"
                                placeholder="示例对话格式，展示角色的说话风格"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    创作者笔记
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.creator_notes || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.creator_notes"
                                @blur="
                                    updateField(
                                        'creator_notes',
                                        fullCharacterData?.card?.data?.creator_notes || '',
                                        characterData.creator_notes,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="创作时的备注和说明"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    系统提示词
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.system_prompt || 0 }} tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.system_prompt"
                                @blur="
                                    updateField(
                                        'system_prompt',
                                        fullCharacterData?.card?.data?.system_prompt || '',
                                        characterData.system_prompt,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="4"
                                placeholder="AI系统使用的提示词"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    历史后指令
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{
                                        tokenCounts.post_history_instructions ||
                                        0
                                    }}
                                    tokens
                                </span>
                            </div>
                            <textarea
                                v-model="
                                    characterData.post_history_instructions
                                "
                                @blur="
                                    updateField(
                                        'post_history_instructions',
                                        fullCharacterData?.card?.data?.post_history_instructions || '',
                                        characterData.post_history_instructions,
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="对话历史后的处理指令"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    备用问候语
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.alternate_greetings || 0 }}
                                    tokens
                                </span>
                            </div>
                            <textarea
                                v-model="characterData.alternate_greetings"
                                @blur="
                                    updateField(
                                        'alternate_greetings',
                                        fullCharacterData?.card?.data?.alternate_greetings || [],
                                        characterData.alternate_greetings.split('\n'),
                                    )
                                "
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                                rows="3"
                                placeholder="备用开场白，用换行分隔多个问候语"
                            ></textarea>
                        </div>

                        <div>
                            <div class="flex items-center justify-between mb-2">
                                <label
                                    class="block text-sm font-semibold text-gray-700"
                                >
                                    标签
                                </label>
                                <span class="text-xs text-gray-500">
                                    {{ tokenCounts.tags || 0 }} tokens
                                </span>
                            </div>
                            <input
                                v-model="characterData.tags"
                                @blur="
                                    updateField(
                                        'tags',
                                        fullCharacterData?.card?.data?.tags || [],
                                        characterData.tags.split(',').map(t => t.trim()),
                                    )
                                "
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
                                @blur="
                                    updateField(
                                        'creator',
                                        fullCharacterData?.card?.data?.creator || '',
                                        characterData.creator,
                                    )
                                "
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
                                @blur="
                                    updateField(
                                        'character_version',
                                        fullCharacterData?.card?.data?.character_version || '',
                                        characterData.character_version,
                                    )
                                "
                                type="text"
                                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                                placeholder="角色卡版本号"
                            />
                        </div>
                    </div>

                    <!-- 世界书编辑器 -->
                    <div
                        v-else-if="editorMode === 'worldBook'"
                        class="flex-1 min-h-0"
                    >
                        <WorldBookEditor
                            v-if="characterUUID"
                            :character-uuid="characterUUID"
                        />
                    </div>
                </div>
            </div>

            <!-- 右侧：AI Panel -->
            <AIPanel
                v-if="aiPanelVisible"
                :visible="aiPanelVisible"
                panel-type="ai"
                :character-data="fullCharacterData"
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
