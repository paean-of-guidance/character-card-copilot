<script setup lang="ts">
import { onMounted, ref, watch, nextTick, onUnmounted, computed, onBeforeUnmount } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useAppStore } from "@/stores/app";
import { useCharacterStore } from "@/stores/character";
import { useAiStore } from "@/stores/ai";
import {
    deleteCharacter as deleteCharacterByUUID,
    exportCharacterCard,
} from "@/services/characterStorage";
import AIPanel from "@/components/AIPanel.vue";
import WorldBookEditor from "@/components/WorldBookEditor.vue";
import CharacterEditorPanel from "@/components/editor/CharacterEditorPanel.vue";
import EditorModeSwitcher from "@/components/editor/EditorModeSwitcher.vue";
import AIPanelToggleCard from "@/components/editor/AIPanelToggleCard.vue";
import {
    uploadBackgroundImage,
    updateCharacterBackgroundPath,
} from "@/services/characterStorage";
import { save } from "@tauri-apps/plugin-dialog";
import { CharacterStateService } from "@/services/characterState";
import { listen } from "@tauri-apps/api/event";
import { readFile } from "@tauri-apps/plugin-fs";
import { tokenCounter } from "@/utils/tokenCounter";
import { devLog } from "@/utils/logger";
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
const aiStore = useAiStore();
const route = useRoute();
const router = useRouter();
const { showSuccessToast, showErrorToast, showWarningToast } =
    useNotification();
const { showAlertModal } = useModal();
const isLoading = ref(false);
const characterUUID = ref<string>("");
const aiPanelVisible = ref(true);
const backgroundPath = ref<string>("");
const thumbnailPath = ref<string>("");
const isUploading = ref(false);
const ALTERNATE_GREETING_MARKER = "<START_ALT>";

const avatarSrc = computed(() => {
    return imageSrc.value;
});

const imageSrc = ref("");
let revokeUrl: string | null = null;

function revokeImageUrl() {
    if (revokeUrl) {
        URL.revokeObjectURL(revokeUrl);
        revokeUrl = null;
    }
}

async function loadAvatar(path: string) {
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
        console.error("读取头像失败", error);
        revokeImageUrl();
        imageSrc.value = "";
    }
}

watch(
    () => [thumbnailPath.value, backgroundPath.value],
    ([thumb, bg]) => {
        loadAvatar(thumb || bg || "");
    },
    { immediate: true },
);

onBeforeUnmount(() => {
    revokeImageUrl();
});

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
    devLog("Editor: 初始化后端事件监听器...");

    // 角色加载事件
    const unlistenCharacterLoaded = await listen<CharacterLoadedPayload>(
        "character-loaded",
        async (event) => {
            devLog("Editor: 🎭 角色加载事件:", event.payload);
            const payload = event.payload;

            // ✅ 更新 Store 缓存（不会闪烁）
            characterStore.updateCharacterFromBackend(payload.uuid, payload.character_data);

            // 如果是当前编辑的角色，更新本地数据
            if (payload.uuid === characterUUID.value) {
                devLog("Editor: 更新角色数据到编辑器");
                await updateEditorFromCharacterData(payload.character_data);
            }
        },
    );

    // 角色更新事件
    const unlistenCharacterUpdated = await listen<CharacterUpdatedPayload>(
        "character-updated",
        async (event) => {
            devLog("Editor: 🔄 角色更新事件:", event.payload);
            const payload = event.payload;

            // ✅ 更新 Store 缓存（工具调用修改后会触发此事件）
            characterStore.updateCharacterFromBackend(payload.uuid, payload.character_data);

            // 如果是当前编辑的角色，更新本地数据
            if (payload.uuid === characterUUID.value) {
                devLog("Editor: 角色数据已更新，同步到编辑器");
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
            devLog("Editor: 🚪 会话卸载事件:", event.payload);
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

    devLog("Editor: ✅ 后端事件监听器初始化完成");
}

/**
 * 清理所有事件监听器
 */
function cleanupEventListeners() {
    devLog("Editor: 清理事件监听器...");
    eventUnlisteners.value.forEach((unlisten) => {
        try {
            unlisten();
        } catch (error) {
            console.error("Editor: 清理事件监听器失败:", error);
        }
    });
    eventUnlisteners.value = [];
    devLog("Editor: ✅ 事件监听器清理完成");
}

/**
 * 从CharacterData更新编辑器表单数据
 */
function parseAlternateGreetingSegments(
    source: string | string[] | undefined | null,
) {
    if (Array.isArray(source)) {
        return source
            .map((segment) => segment.trim())
            .filter((segment) => segment.length > 0);
    }

    return (source || "")
        .split(ALTERNATE_GREETING_MARKER)
        .map((segment) => segment.trim())
        .filter((segment) => segment.length > 0);
}

function formatAlternateGreetingsForInput(values?: string[]) {
    const segments = parseAlternateGreetingSegments(values ?? []);
    if (!segments.length) {
        return "";
    }
    return segments
        .map((segment) => `${ALTERNATE_GREETING_MARKER}\n${segment}`)
        .join("\n");
}

function serializeAlternateGreetingsValue(value: string | string[]) {
    const segments = parseAlternateGreetingSegments(value);
    if (!segments.length) {
        return "";
    }
    return segments
        .map((segment) => `${ALTERNATE_GREETING_MARKER}\n${segment}`)
        .join("\n");
}

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
            alternate_greetings: formatAlternateGreetingsForInput(
                cardData.alternate_greetings,
            ),
            tags: cardData.tags ? [...cardData.tags] : [],
            creator: cardData.creator || "",
            character_version: cardData.character_version || "",
        };

        // 更新背景路径
        backgroundPath.value = incomingCharacterData.backgroundPath || "";
        thumbnailPath.value = incomingCharacterData.thumbnailPath || "";

        devLog("Editor: 角色数据已同步到编辑器");
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
            backgroundPath.value = uploadedPath.backgroundPath;
            thumbnailPath.value = uploadedPath.thumbnailPath;

            // 更新角色的background_path字段
            await updateCharacterBackgroundPath(
                characterUUID.value,
                uploadedPath.backgroundPath,
            );
            devLog("头像上传成功:", uploadedPath);
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
    tags: [] as string[],
    creator: "",
    character_version: "",
});

// 加载角色数据
async function loadCharacterData(uuid: string) {
    if (!uuid) return;

    isLoading.value = true;
    try {
        // ✅ 使用 Store 加载（带缓存）
        let character = await characterStore.getCharacterByUUID(uuid);
        if (!character) {
            await characterStore.refreshCharacters();
            character = await characterStore.getCharacterByUUID(uuid);
        }
        if (character) {
            characterUUID.value = uuid;
            backgroundPath.value = character.backgroundPath || "";
            thumbnailPath.value = character.thumbnailPath || "";

            // 🔥 新增：触发后端会话加载，让AI可以看到角色数据
            devLog("Editor: 触发后端会话加载...", uuid);
            try {
                await aiStore.loadCharacterSession(uuid);
                devLog("Editor: 后端会话加载成功");
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
                alternate_greetings: formatAlternateGreetingsForInput(
                    character.card.data.alternate_greetings,
                ),
                tags: [...character.card.data.tags],
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
    const normalizeValue = (value: string | string[]) => {
        if (fieldName === "alternate_greetings") {
            return serializeAlternateGreetingsValue(value);
        }
        if (fieldName === "tags") {
            if (Array.isArray(value)) {
                return value.join(",");
            }
            return value || "";
        }
        if (Array.isArray(value)) {
            return value.join("\n");
        }
        return value || "";
    };

    const oldStr = normalizeValue(oldValue);
    const newStr = normalizeValue(newValue);

    // 只有值真正改变时才更新
    if (oldStr !== newStr) {
        try {
            await characterStore.updateCharacterField(
                characterUUID.value,
                fieldName,
                newStr,
            );
            devLog(`字段 ${fieldName} 已保存`);
        } catch (error) {
            console.error(`更新字段 ${fieldName} 失败:`, error);
            showErrorToast(`保存 ${fieldName} 失败`, "保存错误");
        }
    } else {
        devLog(`字段 ${fieldName} 值未变化，跳过保存`);
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
        () => characterData.value.tags.join(","),
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
        updateTokenCount("tags", characterData.value.tags.join(", "));
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
                devLog("角色删除成功");
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
    <div class="h-[calc(100vh-5rem)] w-full px-2 py-2 bg-[linear-gradient(180deg,_#f8fafc_0%,_#eef2ff_100%)]">
        <div class="flex h-full w-full gap-3">
            <!-- 左侧：角色信息显示 -->
            <div
                ref="editorContainerRef"
                class="rounded-[24px] border border-white/70 bg-white/75 p-4 overflow-y-auto shadow-[0_16px_40px_rgba(148,163,184,0.14)] backdrop-blur-xl"
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
                    <div class="mb-5">
                        <div class="flex items-center gap-4 mb-4">
                            <!-- 角色卡预览 -->
                            <div
                                class="w-20 h-20 rounded-2xl flex items-center justify-center shadow-[0_8px_20px_rgba(148,163,184,0.18)] overflow-hidden cursor-pointer hover:opacity-80 transition-opacity relative border border-white/60"
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
                                    v-if="avatarSrc"
                                    :src="avatarSrc"
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
                                    class="w-full rounded-xl border border-slate-200/80 bg-white/90 px-4 py-2 text-lg font-medium text-slate-800 placeholder-slate-300 focus:outline-none focus:border-blue-300 focus:ring-2 focus:ring-blue-500/10"
                                    placeholder="请输入角色名称"
                                />
                            </div>
                        </div>
                    </div>

                    <!-- 操作按钮区域 -->
                    <div class="flex gap-3 mb-5 items-center">
                        <button class="glass-btn glass-btn--danger" @click="deleteCharacter">
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                            删除角色
                        </button>
                        <button class="glass-btn glass-btn--primary" @click="exportCharacter">
                            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 11l3 3m0 0l3-3m-3 3V8" />
                            </svg>
                            导出角色
                        </button>
                        <EditorModeSwitcher
                            :mode="editorMode"
                            @toggle-mode="toggleEditorMode"
                        />
                    </div>

                    <!-- 角色编辑表单 -->
                    <div v-if="editorMode === 'character'">
                        <CharacterEditorPanel
                            :character-data="characterData"
                            :full-character-data="fullCharacterData"
                            :token-counts="tokenCounts"
                            @update-field="updateField"
                        />
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
            <AIPanelToggleCard
                v-if="!aiPanelVisible"
                @toggle="toggleAIPanel"
            />
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
