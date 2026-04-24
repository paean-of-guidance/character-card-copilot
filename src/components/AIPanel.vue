<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from "vue";
import { storeToRefs } from "pinia";
import type { ChatMessage } from "@/types/api";
import CommandPalette from "./CommandPalette.vue";
import Modal from "./Modal.vue";
import ToolExecutionCard from "./ToolExecutionCard.vue";
import ChatInput from "./ai/ChatInput.vue";
import AIPanelHeader from "./ai/AIPanelHeader.vue";
import AIPanelStatusChips from "./ai/AIPanelStatusChips.vue";
import AIPanelEmptyState from "./ai/AIPanelEmptyState.vue";
import MessageBubble from "./ai/MessageBubble.vue";
import type { CommandMetadata } from "@/types/commands";
import type { ModalOptions } from "@/utils/notification";
import { useChatStore } from "@/stores/chat";
import { useAiStore } from "@/stores/ai";
import { useApiStore } from "@/stores/api";
import {
    useAiEventListeners,
    type DisplayMessage,
} from "@/composables/ai/useAiEventListeners";
import { useMessageGrouping } from "@/composables/ai/useMessageGrouping";
import { useNotification } from "@/composables/useNotification";
import { devLog, devWarn } from "@/utils/logger";
const { showErrorToast } = useNotification();

// 组件props
const props = defineProps<{
    visible?: boolean;
    panelType?: "ai" | "chat" | "tools";
    characterData?: any; // CharacterData JSON对象
}>();

const emits = defineEmits<{
    toggle: [];
}>();

// 默认可见
const isVisible = ref(props.visible !== false);

// 使用 Pinia Store 管理聊天状态
const chatStore = useChatStore();
const aiStore = useAiStore();
const apiStore = useApiStore();
const { enabledApis, defaultApi, selectedProfile } = storeToRefs(apiStore);
const {
    aiRoles,
    currentRoleConfig,
    isLoading,
    isStopping,
    lastTokenStats,
    selectedRole: selectedRoleState,
    currentSessionUUID,
    isBackendSessionActive,
} = storeToRefs(aiStore);

// 对话相关状态 - 保持为 ref，但同步到 store
const messages = ref<DisplayMessage[]>([]);

// 后端事件相关状态
const contextBuiltInfo = ref<any>(null);
const isLoadingFromBackend = ref(false);

// 使用 AI 事件监听器 composable
const { setupListeners, cleanup: cleanupEventListeners } = useAiEventListeners(
    messages,
    contextBuiltInfo,
    isLoadingFromBackend,
);

// 输入内容（用于命令面板搜索）
const userInput = ref("");
const apiConfigs = computed(() => enabledApis.value);
const selectedApi = computed({
    get: () => selectedProfile.value,
    set: (value: string) => {
        apiStore.selectApi(value || null);
    },
});
const selectedRole = computed({
    get: () => selectedRoleState.value,
    set: (value: string) => {
        void aiStore.selectRole(value);
    },
});

// 聊天容器和输入框引用
const chatMessagesRef = ref<HTMLElement>();
const chatInputRef = ref<InstanceType<typeof ChatInput>>();

// 编辑相关状态
const editingContent = ref("");

// 命令面板相关状态
const showCommandPalette = ref(false);
const commandPaletteRef = ref<InstanceType<typeof CommandPalette>>();
const availableCommands = ref<CommandMetadata[]>([]);
const filteredCommands = ref<CommandMetadata[]>([]);
const commandSearchQuery = ref("");
const modalOptions = ref<ModalOptions | null>(null);
const pendingCommand = ref<CommandMetadata | null>(null);
const COMMAND_SEARCH_DEBOUNCE_MS = 120;
const COMMAND_REFRESH_DEBOUNCE_MS = 150;
let commandSearchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let commandRefreshDebounceTimer: ReturnType<typeof setTimeout> | null = null;

// 使用消息分组 composable
const groupedMessages = useMessageGrouping(messages);
const hasStreamingAssistant = computed(() =>
    messages.value.some((message) => message.role === 'assistant' && message.isStreaming),
);
const lastRenderableMessageId = computed(() => {
    for (let index = messages.value.length - 1; index >= 0; index -= 1) {
        const message = messages.value[index];
        if (message.role === "user" || message.role === "assistant") {
            return message.id;
        }
    }

    return "";
});
const currentRoleName = computed(() => currentRoleConfig.value?.name || selectedRole.value || '未选择');
const contextUsageLabel = computed(() => {
    const budgetUtilization = Number(lastTokenStats.value?.budget_utilization);

    if (!Number.isFinite(budgetUtilization) || budgetUtilization <= 0) {
        return '未计算';
    }

    const rounded = budgetUtilization >= 10 ? Math.round(budgetUtilization) : Number(budgetUtilization.toFixed(1));
    return `${rounded}% 已使用`;
});

// 切换显示/隐藏
function togglePanel() {
    isVisible.value = !isVisible.value;
    emits("toggle");
}

// 监听visible属性变化
const visible = computed(() => {
    return props.visible !== false && isVisible.value;
});

const commandAvailabilitySignature = computed(() => {
    return `${currentSessionUUID.value}|${isBackendSessionActive.value}|${messages.value
        .map((message) => message.role)
        .join(",")}|${messages.value.length}`;
});

function hasCurrentCharacterSession(characterId: string | null): boolean {
    return (
        !!characterId &&
        isBackendSessionActive.value &&
        currentSessionUUID.value === characterId
    );
}

// 加载API配置
async function loadApiConfigs() {
    try {
        await apiStore.loadAllApis();

        const hasSelectedEnabledApi = apiConfigs.value.some(
            (config) => config.profile === selectedApi.value,
        );

        if (!hasSelectedEnabledApi) {
            const preferredConfig =
                (defaultApi.value?.enabled ? defaultApi.value : null) ??
                apiConfigs.value[0];

            if (preferredConfig) {
                apiStore.selectApi(preferredConfig.profile);
            }
        }
    } catch (error) {
        console.error("加载API配置失败:", error);
    }
}

// 加载AI角色配置
async function loadAIRoles() {
    try {
        await aiStore.loadAIRoles();
    } catch (error) {
        console.error("加载AI角色配置失败:", error);
    }
}

// 发送消息（从 ChatInput 组件接收）
async function handleSendMessage(message: string) {
    if (isLoading.value) return;

    const characterId = getCurrentCharacterId();

    // 检查是否有活跃的后端会话
    if (!hasCurrentCharacterSession(characterId)) {
        if (!characterId) {
            console.error("无法获取角色ID，无法发送消息");
            return;
        }

        devLog("触发后端角色会话加载...");
        isLoadingFromBackend.value = true;
        try {
            await aiStore.ensureCharacterSession(characterId);
        } catch (error) {
            console.error("加载角色会话失败:", error);
            isLoadingFromBackend.value = false;
            return;
        }
    }

    isLoadingFromBackend.value = false;

    try {
        await aiStore.sendChatMessage(message);
    } catch (error) {
        showErrorToast(`${error}`, "发送消息失败");
        console.error("发送消息失败:", error);
    }
}

async function handleStopResponse() {
    if (!isLoading.value || isStopping.value) {
        return;
    }

    try {
        await aiStore.interruptResponse();
    } catch (error) {
        showErrorToast(`${error}`, "停止生成失败");
        console.error("停止生成失败:", error);
    }
}

// 处理来自 ChatInput 的键盘事件（命令面板导航）
function handleInputKeydown(event: KeyboardEvent) {
    if (commandPaletteRef.value) {
        commandPaletteRef.value.handleKeydown(event);
    }
}

function handleGlobalKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !isLoading.value) {
        return;
    }

    event.preventDefault();
    void handleStopResponse();
}

// 处理输入变化（用于命令面板搜索）
function handleInputChange(value: string) {
    userInput.value = value;
}

// 获取当前角色ID
function getCurrentCharacterId(): string | null {
    // 从当前URL路径获取UUID
    const pathParts = window.location.pathname.split("/");
    const editorIndex = pathParts.indexOf("editor");
    if (editorIndex !== -1 && pathParts[editorIndex + 1]) {
        return pathParts[editorIndex + 1];
    }
    return null;
}

// 初始化聊天历史记录
async function initializeChatHistory() {
    if (!props.characterData?.name) {
        // 如果没有角色数据，清空消息
        messages.value = [];
        return;
    }

    try {
        // 先清空当前消息，避免显示旧角色的消息
        messages.value = [];

        const characterId = getCurrentCharacterId();

        if (!characterId) {
            devWarn("无法获取角色UUID");
            return;
        }

        // 通过aiStore加载历史记录
        const history = await aiStore.loadChatHistory(characterId);

        // 转换为前端消息格式（保留所有 role 类型）
        if (history.length > 0) {
            messages.value = history.map((msg, index) => ({
                id: `${msg.timestamp || index}_${characterId}`,
                role: msg.role, // 保留原始 role：user/assistant/tool
                content: msg.content,
                timestamp: new Date(msg.timestamp || Date.now()),
                // 保留工具调用相关字段
                tool_calls: msg.tool_calls,
                tool_call_id: msg.tool_call_id,
                name: msg.name,
            }));

            devLog(
                `为角色 ${props.characterData.name} (ID: ${characterId}) 加载了 ${messages.value.length} 条聊天历史记录`,
            );

            // 自动滚动到底部显示最新消息 - 通过watch处理
        } else {
            devLog(`角色 ${props.characterData.name} 暂无聊天历史记录`);
        }
    } catch (error) {
        console.error("初始化聊天历史记录失败:", error);
        messages.value = [];
    }
}

async function ensureInitialCharacterSessionLoaded() {
    const characterId = getCurrentCharacterId();

    if (!characterId) {
        devWarn("无法获取角色UUID，跳过初始化角色会话");
        return;
    }

    isLoadingFromBackend.value = true;

    try {
        await aiStore.ensureCharacterSession(characterId);
    } catch (error) {
        console.error("初始化角色会话失败，回退到直接加载聊天历史:", error);
        isLoadingFromBackend.value = false;
        await initializeChatHistory();
        return;
    }

    isLoadingFromBackend.value = false;
}

// 监听角色数据变化
watch(
    () => props.characterData?.name,
    async (newName, oldName) => {
        if (newName && !oldName) {
            devLog(`角色初始化: ${newName}`);
            await ensureInitialCharacterSessionLoaded();
            return;
        }

        // 只在真正切换角色时才重新加载
        if (newName && oldName && newName !== oldName) {
            devLog(`角色切换: ${oldName} -> ${newName}`);

            const characterId = getCurrentCharacterId();

            // 如果已有后端会话，切换角色时重新加载目标角色会话
            if (isBackendSessionActive.value) {
                if (characterId) {
                    isLoadingFromBackend.value = true;
                    try {
                        await aiStore.loadCharacterSession(characterId);
                    } catch (error) {
                        console.error("重新加载角色会话失败:", error);
                        isLoadingFromBackend.value = false;
                    }
                }
            } else {
                await initializeChatHistory();
            }
        }
    },
);

// 监听消息变化，自动滚动到底部
watch(
    () => messages.value.length,
    () => {
        nextTick(() => {
            if (chatMessagesRef.value) {
                chatMessagesRef.value.scrollTop =
                    chatMessagesRef.value.scrollHeight;
            }
        });
    },
);

// 开始编辑消息（从 MessageBubble 触发）
function handleStartEdit(messageId: string) {
    const index = messages.value.findIndex((m) => m.id === messageId);
    if (index >= 0 && index < messages.value.length) {
        editingContent.value = messages.value[index].content;
        messages.value[index].isEditing = true;
    }
}

// 取消编辑（从 MessageBubble 触发）
function handleCancelEdit(messageId: string) {
    const index = messages.value.findIndex((m) => m.id === messageId);
    if (index >= 0 && index < messages.value.length) {
        messages.value[index].isEditing = false;
    }
    editingContent.value = "";
}

function handleToggleReasoning(messageId: string) {
    const message = messages.value.find((item) => item.id === messageId);
    if (!message) {
        return;
    }

    message.reasoningExpanded = !message.reasoningExpanded;
}

// 保存编辑（从 MessageBubble 触发）
async function handleSaveEdit(messageId: string, newContent: string) {
    const index = messages.value.findIndex((m) => m.id === messageId);
    if (index >= 0 && index < messages.value.length) {
        try {
            const originalContent = messages.value[index].content;

            if (!newContent) {
                // 如果内容为空，删除消息
                await deleteMessage(index);
                return;
            }

            if (newContent !== originalContent) {
                // 调用后端编辑消息
                await aiStore.editChatMessage(index, newContent);

                // 更新前端消息
                messages.value[index].content = newContent;
                messages.value[index].isEditing = false;

                devLog(`✅ 已编辑消息 [${index}]`);
            } else {
                // 内容没有变化，直接取消编辑状态
                messages.value[index].isEditing = false;
            }

            editingContent.value = "";
        } catch (error) {
            console.error("保存编辑失败:", error);
        }
    }
}

// 删除消息（从 MessageBubble 触发）
async function handleDeleteMessage(messageId: string) {
    const index = messages.value.findIndex((m) => m.id === messageId);
    await deleteMessage(index);
}

// 删除消息
async function deleteMessage(index: number) {
    if (index < 0 || index >= messages.value.length) {
        return;
    }

    try {
        const msg = messages.value[index];

        // 检测是否需要删除完整的工具调用链
        let deleteStartIndex = index;
        let deleteEndIndex = index;

        // 情况1: 删除的是普通 assistant（可能是工具调用后的最终回复）
        if (
            msg.role === "assistant" &&
            (!msg.tool_calls || msg.tool_calls.length === 0)
        ) {
            // 向前查找：是否有 tool 消息
            let hasToolMessages = false;
            let toolStartIndex = index - 1;

            // 跳过前面的 tool 消息
            while (
                toolStartIndex >= 0 &&
                messages.value[toolStartIndex].role === "tool"
            ) {
                hasToolMessages = true;
                toolStartIndex--;
            }

            // 如果找到了 tool 消息，再检查前面是否有带 tool_calls 的 assistant
            if (hasToolMessages && toolStartIndex >= 0) {
                const prevMsg = messages.value[toolStartIndex];
                if (
                    prevMsg.role === "assistant" &&
                    prevMsg.tool_calls &&
                    prevMsg.tool_calls.length > 0
                ) {
                    // 找到完整的工具调用链，删除整个链条
                    deleteStartIndex = toolStartIndex;
                    devLog(
                        `🔗 检测到工具调用链: [${deleteStartIndex}] 到 [${deleteEndIndex}]`,
                    );
                }
            }
        }

        // 情况2: 删除的是带 tool_calls 的 assistant（工具调用起点）
        if (
            msg.role === "assistant" &&
            msg.tool_calls &&
            msg.tool_calls.length > 0
        ) {
            // 向后查找所有关联的 tool 消息
            let j = index + 1;
            while (
                j < messages.value.length &&
                messages.value[j].role === "tool"
            ) {
                j++;
            }

            // 检查 tool 消息后面是否还有 assistant 回复（工具调用的最终回复）
            if (
                j < messages.value.length &&
                messages.value[j].role === "assistant"
            ) {
                deleteEndIndex = j;
                devLog(
                    `🔗 检测到工具调用链: [${deleteStartIndex}] 到 [${deleteEndIndex}]`,
                );
            } else {
                deleteEndIndex = j - 1;
            }
        }

        // 计算要删除的消息数量
        const deleteCount = deleteEndIndex - deleteStartIndex + 1;

        devLog(
            `🗑️ 删除消息: 从 [${deleteStartIndex}] 到 [${deleteEndIndex}]，共 ${deleteCount} 条`,
        );

        // 依次调用后端删除（从后往前删，避免索引变化）
        for (let i = deleteEndIndex; i >= deleteStartIndex; i--) {
            await aiStore.deleteChatMessage(i);
        }

        // 前端也删除（后端会通过事件同步，但为了即时响应先删除）
        messages.value.splice(deleteStartIndex, deleteCount);

        devLog(`✅ 已删除 ${deleteCount} 条消息`);
    } catch (error) {
        console.error("删除消息失败:", error);
    }
}

// 重新生成响应
async function regenerateResponse() {
    if (messages.value.length === 0) return;

    // 检查最后一条消息是否是AI回复
    const lastMessage = messages.value[messages.value.length - 1];

    if (lastMessage.role === "assistant") {
        try {
            // 先删除前端的最后一条AI消息（后端也会删除）
            messages.value.pop();

            // 调用后端重新生成命令（会自动删除后端历史并重新生成）
            await aiStore.regenerateLastMessage();

            devLog("✅ 重新生成完成");
        } catch (error) {
            console.error("重新生成失败:", error);
        }
    } else {
        devWarn("最后一条消息不是AI回复，无法重新生成");
    }
}

// 继续生成回复（当最后一条是用户消息时）
async function continueFromUserMessage() {
    const lastMessage = messages.value[messages.value.length - 1];
    if (!lastMessage || lastMessage.role !== "user") {
        const message = "最后一条消息不是用户消息，无法继续对话";
        devWarn(message);
        showErrorToast(message, "继续失败");
        return;
    }

    try {
        devLog("🔄 触发AI生成回复...");

        // 调用新的 continueChat API（专门用于基于最后一条用户消息生成AI回复）
        await aiStore.continueChat();

        devLog("✅ AI回复生成完成");
    } catch (error) {
        showErrorToast(`${error}`, "生成AI回复失败");
        console.error("生成AI回复失败:", error);
    }
}

// ==================== 命令面板相关函数 ====================

/**
 * 初始化命令系统
 */
async function initializeCommands() {
    // 从后端获取所有可用命令
    await refreshAvailableCommands();
}

/**
 * 更新可用命令列表
 */
async function refreshAvailableCommands(forceRefresh = false) {
    try {
        availableCommands.value = await aiStore.getAvailableCommands(forceRefresh);
        await updateFilteredCommands();
    } catch (error) {
        console.error("更新命令列表失败:", error);
    }
}

function clearCommandSearchDebounce() {
    if (commandSearchDebounceTimer) {
        clearTimeout(commandSearchDebounceTimer);
        commandSearchDebounceTimer = null;
    }
}

function clearCommandRefreshDebounce() {
    if (commandRefreshDebounceTimer) {
        clearTimeout(commandRefreshDebounceTimer);
        commandRefreshDebounceTimer = null;
    }
}

function scheduleCommandSearch() {
    clearCommandSearchDebounce();
    commandSearchDebounceTimer = setTimeout(() => {
        void updateFilteredCommands();
    }, COMMAND_SEARCH_DEBOUNCE_MS);
}

function scheduleCommandAvailabilityRefresh(forceRefresh = true) {
    clearCommandRefreshDebounce();
    commandRefreshDebounceTimer = setTimeout(() => {
        if (!showCommandPalette.value) {
            return;
        }

        void refreshAvailableCommands(forceRefresh);
    }, COMMAND_REFRESH_DEBOUNCE_MS);
}

/**
 * 更新过滤后的命令列表
 */
async function updateFilteredCommands() {
    try {
        filteredCommands.value = await aiStore.searchCommands(
            commandSearchQuery.value,
        );
    } catch (error) {
        console.error("搜索命令失败:", error);
    }
}

/**
 * 打开命令面板
 */
async function openCommandPalette() {
    // 设置用户输入为"/"
    if (chatInputRef.value) {
        chatInputRef.value.setValue("/");
    }
    commandSearchQuery.value = "";

    // 显示命令面板
    showCommandPalette.value = true;

    // 更新可用命令
    await refreshAvailableCommands(true);
}

/**
 * 关闭命令面板
 */
function closeCommandPalette() {
    showCommandPalette.value = false;
    commandSearchQuery.value = "";
    clearCommandSearchDebounce();
    clearCommandRefreshDebounce();

    // 清空输入框
    if (chatInputRef.value) {
        chatInputRef.value.clear();
    }
}

/**
 * 处理命令选择
 */
async function handleCommandSelect(command: CommandMetadata) {
    // 如果命令需要确认，显示确认对话框
    if (command.requires_confirmation) {
        pendingCommand.value = command;
        modalOptions.value = {
            title: "确认操作",
            message:
                command.confirmation_message ||
                `确定要执行 ${command.name} 吗？`,
            type: "danger",
            confirmText: "确认",
            cancelText: "取消",
            onConfirm: async () => {
                await confirmCommand();
            },
            onCancel: () => {
                cancelCommand();
            },
        };
        return;
    }

    // 直接执行命令
    await executeCommand(command);
}

/**
 * 执行命令
 */
async function executeCommand(command: CommandMetadata) {
    try {
        // 调用后端执行命令
        const result = await aiStore.executeCommand(
            command.id,
            userInput.value,
        );

        // 关闭命令面板
        closeCommandPalette();

        // 命令执行成功
        if (result.success) {
            devLog(`命令 ${command.name} 执行成功:`, result.message);
            // 可以在这里显示通知（使用右上角通知组件）
            // TODO: 集成通知系统
        } else {
            console.error(`命令 ${command.name} 执行失败:`, result.error);
            // 可以在这里显示错误通知
            // TODO: 集成通知系统
        }
    } catch (error) {
        console.error("命令执行失败:", error);
    }
}

/**
 * 确认执行命令
 */
async function confirmCommand() {
    const command = pendingCommand.value;
    if (command) {
        // 执行命令
        await executeCommand(command);
    }

    // 清理状态
    pendingCommand.value = null;
    modalOptions.value = null;
}

/**
 * 取消命令执行
 */
function cancelCommand() {
    // 清理状态
    pendingCommand.value = null;
    modalOptions.value = null;

    // 关闭命令面板
    closeCommandPalette();
}

/**
 * 监听用户输入变化，更新命令搜索
 */
watch(userInput, (newValue) => {
    if (showCommandPalette.value) {
        // 如果输入框为空或者输入了斜杠+空格，关闭命令面板
        // 注意：不要在 newValue === "/" 时关闭，因为这是刚打开命令面板的状态
        if (newValue === "" || /^\/\s/.test(newValue)) {
            closeCommandPalette();
            return;
        }

        // 提取搜索关键字（去除开头的"/"）
        commandSearchQuery.value = newValue.replace(/^\//, "");
        scheduleCommandSearch();
    }
});

watch(commandAvailabilitySignature, (newValue, oldValue) => {
    if (!oldValue || newValue === oldValue) {
        return;
    }

    aiStore.clearCommandCache();
    scheduleCommandAvailabilityRefresh();
});

onMounted(async () => {
    window.addEventListener("keydown", handleGlobalKeydown);
    await setupListeners();

    await Promise.all([loadApiConfigs(), loadAIRoles()]);

    // 初始化命令系统
    await initializeCommands();

    // 先从 store 恢复聊天历史（如果有）
    const characterId = getCurrentCharacterId();
    if (characterId) {
        const storedHistory = chatStore.getChatHistory(characterId);
        if (storedHistory.length > 0) {
            devLog(`📦 从 Store 恢复 ${storedHistory.length} 条聊天历史`);
            messages.value = storedHistory.map((msg, index) => ({
                id: `${msg.timestamp || index}_${characterId}`,
                role: msg.role, // 保留原始 role：user/assistant/tool
                content: msg.content,
                timestamp: new Date(
                    (msg.timestamp || Date.now() / 1000) * 1000,
                ),
                // 保留工具调用相关字段
                tool_calls: msg.tool_calls,
                tool_call_id: msg.tool_call_id,
                name: msg.name,
            }));
        }
    }

    // 事件监听器初始化完成后，检查是否需要重新加载会话
    if (props.characterData?.name && characterId) {
        const storedHistory = chatStore.getChatHistory(characterId);

        if (chatStore.isBackendSessionActive && storedHistory.length > 0) {
            devLog(`🔄 组件重新挂载，后端会话已存在，跳过重复加载`);
            aiStore.updateSessionState(characterId, true);
            // 不重新加载，使用 store 中的数据即可
        } else if (storedHistory.length === 0) {
            await ensureInitialCharacterSessionLoaded();
        }
    }

    // 注：tool-executed 事件监听器已在上方注册（Line 477），
    // 负责创建 tool 消息并添加到 messages 数组
});

// 组件卸载时清理事件监听器并保存状态到 store
onUnmounted(() => {
    window.removeEventListener("keydown", handleGlobalKeydown);
    clearCommandSearchDebounce();
    clearCommandRefreshDebounce();

    // 保存当前聊天历史到 store
    const characterId = getCurrentCharacterId();
    if (characterId && messages.value.length > 0) {
        const chatMessages: ChatMessage[] = messages.value.map((msg) => ({
            role: msg.role,
            content: msg.content,
            timestamp: Math.floor(msg.timestamp.getTime() / 1000),
            name: undefined,
            tool_calls: undefined,
            tool_call_id: undefined,
        }));
        chatStore.setChatHistory(characterId, chatMessages);
        devLog(`💾 组件卸载，保存 ${chatMessages.length} 条消息到 Store`);
    }

    cleanupEventListeners();
});
</script>

<template>
    <div
        v-if="visible"
        class="card flex w-[min(50%,42rem)] min-w-[24rem] max-w-[42rem] flex-shrink-0 rounded-[28px] border border-white/70 bg-white/78 p-4 shadow-[0_22px_55px_rgba(148,163,184,0.22)] backdrop-blur-xl lg:p-5"
    >
        <div class="flex h-full min-h-0 w-full min-w-0 flex-col gap-4">
            <AIPanelHeader @toggle="togglePanel" />

            <!-- 对话消息区域 -->
            <div
                ref="chatMessagesRef"
                class="flex-1 min-w-0 overflow-y-auto rounded-[24px] border border-white/70 bg-[linear-gradient(180deg,_rgba(248,250,252,0.95)_0%,_rgba(241,245,249,0.92)_100%)] p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.8)]"
            >
                <AIPanelEmptyState v-if="messages.length === 0" />

                <div v-else class="space-y-4">
                    <div
                        v-for="group in groupedMessages"
                        :key="
                            group.type === 'normal'
                                ? group.message.id
                                : `tool-${group.toolCallId}-${group.timestamp.getTime()}`
                        "
                        class="flex"
                        :class="
                            group.type === 'normal' &&
                            group.message.role === 'user'
                                ? 'justify-end'
                                : 'justify-start'
                        "
                    >
                        <!-- 工具执行卡片 -->
                        <ToolExecutionCard
                            v-if="group.type === 'tool-meta'"
                            :tool-call="group.toolCall"
                            :tool-result="group.toolResult"
                            :timestamp="group.timestamp"
                        />

                        <!-- 普通消息 -->
                        <MessageBubble
                            v-else-if="group.type === 'normal'"
                            :message-id="group.message.id"
                            :role="group.message.role as 'user' | 'assistant'"
                            :content="group.message.content"
                            :reasoning-content="group.message.reasoningContent"
                            :reasoning-expanded="group.message.reasoningExpanded || false"
                            :reasoning-loading="group.message.isReasoningStreaming || false"
                            :timestamp="group.message.timestamp"
                            :is-editing="group.message.isEditing"
                            :loading="group.message.isStreaming || false"
                            :is-last-message="group.message.id === lastRenderableMessageId"
                            @continue="continueFromUserMessage"
                            @regenerate="regenerateResponse"
                            @start-edit="handleStartEdit(group.message.id)"
                            @save-edit="
                                handleSaveEdit(group.message.id, $event)
                            "
                            @cancel-edit="handleCancelEdit(group.message.id)"
                            @delete="handleDeleteMessage(group.message.id)"
                            @toggle-reasoning="handleToggleReasoning(group.message.id)"
                        />
                    </div>

                    <!-- 加载中指示器 -->
                    <div v-if="isLoading && !hasStreamingAssistant" class="flex justify-start">
                        <div
                            class="rounded-2xl border border-slate-200 bg-white/90 px-4 py-2 shadow-sm"
                        >
                            <div class="flex items-center gap-2">
                                <div
                                    class="h-2 w-2 rounded-full bg-slate-400 animate-bounce"
                                ></div>
                                <div
                                    class="h-2 w-2 rounded-full bg-slate-400 animate-bounce"
                                    style="animation-delay: 0.1s"
                                ></div>
                                <div
                                    class="h-2 w-2 rounded-full bg-slate-400 animate-bounce"
                                    style="animation-delay: 0.2s"
                                ></div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 用户输入区域 -->
            <div class="relative">
                <!-- 命令面板 -->
                <CommandPalette
                    ref="commandPaletteRef"
                    :visible="showCommandPalette"
                    :commands="filteredCommands"
                    :searchQuery="commandSearchQuery"
                    @select="handleCommandSelect"
                    @close="closeCommandPalette"
                />

                <ChatInput
                    ref="chatInputRef"
                    :disabled="isLoading"
                    :loading="isLoading"
                    :can-stop="isLoading"
                    :stopping="isStopping"
                    :command-palette-open="showCommandPalette"
                    @send="handleSendMessage"
                    @stop="handleStopResponse"
                    @open-command-palette="openCommandPalette"
                    @keydown="handleInputKeydown"
                    @input="handleInputChange"
                />
            </div>

            <AIPanelStatusChips
                :selected-role="selectedRole"
                :selected-api="selectedApi"
                :current-role-name="currentRoleName"
                :ai-roles="aiRoles"
                :api-configs="apiConfigs"
                :context-usage-label="contextUsageLabel"
                @update:selected-role="selectedRole = $event"
                @update:selected-api="selectedApi = $event"
            />
        </div>

        <!-- 命令确认对话框 -->
        <Modal :options="modalOptions" @close="modalOptions = null" />
    </div>
</template>

<style scoped>
/* 面板动画 */
.card {
    animation: slideInRight 0.3s ease-out;
}

@keyframes slideInRight {
    from {
        opacity: 0;
        transform: translateX(20px);
    }
    to {
        opacity: 1;
        transform: translateX(0);
    }
}

/* 消息区域滚动条样式 */
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

/* 加载动画 */
@keyframes bounce {
    0%,
    80%,
    100% {
        transform: scale(0);
    }
    40% {
        transform: scale(1);
    }
}

.animate-bounce {
    animation: bounce 1.4s infinite ease-in-out both;
}

/* 旋转动画 */
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.animate-spin {
    animation: spin 1s linear infinite;
}

</style>
