<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from "vue";
import { getAllApiConfigs } from "@/services/apiConfig";
import type { ApiConfig, ChatMessage } from "@/types/api";
import { AIConfigService, type AIRole } from "@/services/aiConfig";
import CommandPalette from "./CommandPalette.vue";
import Modal from "./Modal.vue";
import ToolExecutionCard from "./ToolExecutionCard.vue";
import ChatInput from "./ai/ChatInput.vue";
import MessageBubble from "./ai/MessageBubble.vue";
import { backendCommandService } from "@/services/backendCommandService";
import type { CommandMetadata } from "@/types/commands";
import type { ModalOptions } from "@/utils/notification";
import { useChatStore } from "@/stores/chat";
import { useAiStore } from "@/stores/ai";
import { useAiEventListeners, type DisplayMessage } from "@/composables/ai/useAiEventListeners";
import { useMessageGrouping } from "@/composables/ai/useMessageGrouping";

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

// 对话相关状态 - 保持为 ref，但同步到 store
const messages = ref<DisplayMessage[]>([]);

// 后端事件相关状态
const contextBuiltInfo = ref<any>(null);
const isLoadingFromBackend = ref(false);

// 使用 AI 事件监听器 composable
const { setupListeners, cleanup: cleanupEventListeners } = useAiEventListeners(
    messages,
    contextBuiltInfo,
    isLoadingFromBackend
);

// 输入内容（用于命令面板搜索）
const userInput = ref("");

const selectedApi = ref("");
const apiConfigs = ref<ApiConfig[]>([]);

// AI角色相关状态
const selectedRole = ref("");
const aiRoles = ref<Array<{ name: string; role: AIRole }>>([]);
const currentRoleConfig = ref<AIRole | null>(null);
const defaultRole = ref("");

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

// 使用消息分组 composable
const groupedMessages = useMessageGrouping(messages);

// 切换显示/隐藏
function togglePanel() {
    isVisible.value = !isVisible.value;
    emits("toggle");
}

// 监听visible属性变化
const visible = computed(() => {
    return props.visible !== false && isVisible.value;
});

// 加载API配置
async function loadApiConfigs() {
    try {
        const configs = await getAllApiConfigs();
        // 过滤出已启用的配置
        const enabledConfigs = configs.filter((config) => config.enabled);

        // 将默认配置排在第一位
        apiConfigs.value = enabledConfigs.sort((a, b) => {
            if (a.default && !b.default) return -1;
            if (!a.default && b.default) return 1;
            return 0;
        });

        // 优先选择默认配置，如果没有默认配置则选择第一个
        if (apiConfigs.value.length > 0 && !selectedApi.value) {
            const defaultConfig = apiConfigs.value.find(
                (config) => config.default,
            );
            selectedApi.value = defaultConfig
                ? defaultConfig.profile
                : apiConfigs.value[0].profile;
        }
    } catch (error) {
        console.error("加载API配置失败:", error);
    }
}

// 加载AI角色配置
async function loadAIRoles() {
    try {
        const config = await AIConfigService.getConfig();
        defaultRole.value = config.default_role;

        aiRoles.value = await AIConfigService.getAllRoles();

        if (!selectedRole.value && config.default_role) {
            selectedRole.value = config.default_role;
        }
    } catch (error) {
        console.error("加载AI角色配置失败:", error);
    }
}

// 更新当前角色配置
async function updateCurrentRoleConfig() {
    if (!selectedRole.value) {
        currentRoleConfig.value = null;
        return;
    }

    try {
        const role = await AIConfigService.getRole(selectedRole.value);
        currentRoleConfig.value = role;
    } catch (error) {
        console.error("获取角色配置失败:", error);
    }
}

// 监听角色选择变化
watch(selectedRole, () => {
    updateCurrentRoleConfig();
});

// 发送消息（从 ChatInput 组件接收）
async function handleSendMessage(message: string) {
    if (aiStore.isLoading) return;

    // 检查是否有活跃的后端会话
    if (!aiStore.isBackendSessionActive) {
        const characterId = getCurrentCharacterId();
        if (!characterId) {
            console.error("无法获取角色ID，无法发送消息");
            return;
        }

        console.log("触发后端角色会话加载...");
        isLoadingFromBackend.value = true;
        try {
            await aiStore.loadCharacterSession(characterId);
            // 等待角色加载事件完成后再发送消息
            setTimeout(async () => {
                if (aiStore.isBackendSessionActive) {
                    await aiStore.sendChatMessage(message);
                } else {
                    console.error("后端会话加载失败");
                    isLoadingFromBackend.value = false;
                }
            }, 500);
        } catch (error) {
            console.error("加载角色会话失败:", error);
            isLoadingFromBackend.value = false;
        }
    } else {
        // 直接发送消息
        try {
            await aiStore.sendChatMessage(message);
        } catch (error) {
            console.error("发送消息失败:", error);
        }
    }
}

// 处理来自 ChatInput 的键盘事件（命令面板导航）
function handleInputKeydown(event: KeyboardEvent) {
    if (commandPaletteRef.value) {
        commandPaletteRef.value.handleKeydown(event);
    }
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
            console.warn("无法获取角色UUID");
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

            console.log(
                `为角色 ${props.characterData.name} (ID: ${characterId}) 加载了 ${messages.value.length} 条聊天历史记录`,
            );

            // 自动滚动到底部显示最新消息 - 通过watch处理
        } else {
            console.log(`角色 ${props.characterData.name} 暂无聊天历史记录`);
        }
    } catch (error) {
        console.error("初始化聊天历史记录失败:", error);
        messages.value = [];
    }
}


// 监听角色数据变化
watch(
    () => props.characterData?.name,
    async (newName, oldName) => {
        // 只在真正切换角色时才重新加载（跳过初始加载，由 onMounted 处理）
        if (newName && oldName && newName !== oldName) {
            console.log(`角色切换: ${oldName} -> ${newName}`);

            // 如果使用后端会话，重新加载会话
            if (aiStore.isBackendSessionActive) {
                const characterId = getCurrentCharacterId();
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

// 删除工具调用组（从 ToolExecutionCard 触发）
async function deleteToolExecutionGroup(groupIndex: number) {
    const group = groupedMessages.value[groupIndex];

    if (!group || group.type !== 'tool-execution') {
        console.error(`❌ 组 ${groupIndex} 不是有效的工具调用组`);
        return;
    }

    // 通过时间戳找到工具调用链的起始消息（带 tool_calls 的 assistant）
    // 注意：timestamp 是从带 tool_calls 的 assistant 消息继承的
    const targetTimestamp = group.timestamp;

    // 在原始消息数组中找到对应的 assistant 消息
    const startIndex = messages.value.findIndex(
        msg => msg.role === 'assistant' &&
               msg.tool_calls &&
               msg.tool_calls.length > 0 &&
               msg.timestamp.getTime() === targetTimestamp.getTime()
    );

    if (startIndex === -1) {
        console.error(`❌ 未找到工具调用组 ${groupIndex} 的起始消息`);
        return;
    }

    console.log(`🎯 删除工具调用组 [${groupIndex}]，起始消息索引: ${startIndex}`);
    await deleteMessage(startIndex);
}

// 开始编辑消息（从 MessageBubble 触发）
function handleStartEdit(messageId: string) {
    const index = messages.value.findIndex(m => m.id === messageId);
    if (index >= 0 && index < messages.value.length) {
        editingContent.value = messages.value[index].content;
        messages.value[index].isEditing = true;
    }
}

// 取消编辑（从 MessageBubble 触发）
function handleCancelEdit(messageId: string) {
    const index = messages.value.findIndex(m => m.id === messageId);
    if (index >= 0 && index < messages.value.length) {
        messages.value[index].isEditing = false;
    }
    editingContent.value = "";
}

// 保存编辑（从 MessageBubble 触发）
async function handleSaveEdit(messageId: string, newContent: string) {
    const index = messages.value.findIndex(m => m.id === messageId);
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

                console.log(`✅ 已编辑消息 [${index}]`);
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
    const index = messages.value.findIndex(m => m.id === messageId);
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
        if (msg.role === 'assistant' && (!msg.tool_calls || msg.tool_calls.length === 0)) {
            // 向前查找：是否有 tool 消息
            let hasToolMessages = false;
            let toolStartIndex = index - 1;

            // 跳过前面的 tool 消息
            while (toolStartIndex >= 0 && messages.value[toolStartIndex].role === 'tool') {
                hasToolMessages = true;
                toolStartIndex--;
            }

            // 如果找到了 tool 消息，再检查前面是否有带 tool_calls 的 assistant
            if (hasToolMessages && toolStartIndex >= 0) {
                const prevMsg = messages.value[toolStartIndex];
                if (prevMsg.role === 'assistant' && prevMsg.tool_calls && prevMsg.tool_calls.length > 0) {
                    // 找到完整的工具调用链，删除整个链条
                    deleteStartIndex = toolStartIndex;
                    console.log(`🔗 检测到工具调用链: [${deleteStartIndex}] 到 [${deleteEndIndex}]`);
                }
            }
        }

        // 情况2: 删除的是带 tool_calls 的 assistant（工具调用起点）
        if (msg.role === 'assistant' && msg.tool_calls && msg.tool_calls.length > 0) {
            // 向后查找所有关联的 tool 消息
            let j = index + 1;
            while (j < messages.value.length && messages.value[j].role === 'tool') {
                j++;
            }

            // 检查 tool 消息后面是否还有 assistant 回复（工具调用的最终回复）
            if (j < messages.value.length && messages.value[j].role === 'assistant') {
                deleteEndIndex = j;
                console.log(`🔗 检测到工具调用链: [${deleteStartIndex}] 到 [${deleteEndIndex}]`);
            } else {
                deleteEndIndex = j - 1;
            }
        }

        // 计算要删除的消息数量
        const deleteCount = deleteEndIndex - deleteStartIndex + 1;

        console.log(`🗑️ 删除消息: 从 [${deleteStartIndex}] 到 [${deleteEndIndex}]，共 ${deleteCount} 条`);

        // 依次调用后端删除（从后往前删，避免索引变化）
        for (let i = deleteEndIndex; i >= deleteStartIndex; i--) {
            await aiStore.deleteChatMessage(i);
        }

        // 前端也删除（后端会通过事件同步，但为了即时响应先删除）
        messages.value.splice(deleteStartIndex, deleteCount);

        console.log(`✅ 已删除 ${deleteCount} 条消息`);
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
            aiStore.isLoading = true;

            // 先删除前端的最后一条AI消息（后端也会删除）
            messages.value.pop();

            // 调用后端重新生成命令（会自动删除后端历史并重新生成）
            await aiStore.regenerateLastMessage();

            console.log("✅ 重新生成完成");
        } catch (error) {
            console.error("重新生成失败:", error);
            aiStore.isLoading = false;
        }
    } else {
        console.warn("最后一条消息不是AI回复，无法重新生成");
    }
}

// 继续生成回复（当最后一条是用户消息时）
async function continueFromUserMessage() {
    try {
        console.log("🔄 触发AI生成回复...");

        // 调用新的 continueChat API（专门用于基于最后一条用户消息生成AI回复）
        await aiStore.continueChat();

        console.log("✅ AI回复生成完成");
    } catch (error) {
        console.error("生成AI回复失败:", error);
    }
}

// ==================== 命令面板相关函数 ====================

/**
 * 初始化命令系统
 */
async function initializeCommands() {
    // 从后端获取所有可用命令
    await updateAvailableCommands();
}

/**
 * 更新可用命令列表
 */
async function updateAvailableCommands() {
    try {
        availableCommands.value = await backendCommandService.getCommands();
        await updateFilteredCommands();
    } catch (error) {
        console.error('更新命令列表失败:', error);
    }
}

/**
 * 更新过滤后的命令列表
 */
async function updateFilteredCommands() {
    try {
        filteredCommands.value = await backendCommandService.searchCommands(
            commandSearchQuery.value
        );
    } catch (error) {
        console.error('搜索命令失败:', error);
    }
}

/**
 * 打开命令面板
 */
function openCommandPalette() {
    // 设置用户输入为"/"
    if (chatInputRef.value) {
        chatInputRef.value.setValue("/");
    }
    commandSearchQuery.value = "";

    // 更新可用命令
    updateAvailableCommands();

    // 显示命令面板
    showCommandPalette.value = true;
}

/**
 * 关闭命令面板
 */
function closeCommandPalette() {
    showCommandPalette.value = false;
    commandSearchQuery.value = "";

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
            message: command.confirmation_message || `确定要执行 ${command.name} 吗？`,
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
        const result = await backendCommandService.executeCommand(
            command.id,
            userInput.value
        );

        // 关闭命令面板
        closeCommandPalette();

        // 命令执行成功
        if (result.success) {
            console.log(`命令 ${command.name} 执行成功:`, result.message);
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
        updateFilteredCommands();
    }
});

onMounted(async () => {
    loadApiConfigs();
    loadAIRoles();

    // 初始化命令系统
    initializeCommands();

    // 先从 store 恢复聊天历史（如果有）
    const characterId = getCurrentCharacterId();
    if (characterId) {
        const storedHistory = chatStore.getChatHistory(characterId);
        if (storedHistory.length > 0) {
            console.log(`📦 从 Store 恢复 ${storedHistory.length} 条聊天历史`);
            messages.value = storedHistory.map((msg, index) => ({
                id: `${msg.timestamp || index}_${characterId}`,
                role: msg.role, // 保留原始 role：user/assistant/tool
                content: msg.content,
                timestamp: new Date((msg.timestamp || Date.now() / 1000) * 1000),
                // 保留工具调用相关字段
                tool_calls: msg.tool_calls,
                tool_call_id: msg.tool_call_id,
                name: msg.name,
            }));
        }
    }

    // 初始化后端事件监听器（必须先完成，才能接收后续事件）
    await setupListeners();

    // 事件监听器初始化完成后，检查是否需要重新加载会话
    // 只在 store 中有数据但后端会话已失效时才重新加载
    if (props.characterData?.name && characterId) {
        const storedHistory = chatStore.getChatHistory(characterId);
        if (chatStore.isBackendSessionActive && storedHistory.length > 0) {
            console.log(`🔄 组件重新挂载，后端会话已存在，跳过重复加载`);
            aiStore.isBackendSessionActive = true;
            // 不重新加载，使用 store 中的数据即可
        }
    }

    // 注：tool-executed 事件监听器已在上方注册（Line 477），
    // 负责创建 tool 消息并添加到 messages 数组
});

// 组件卸载时清理事件监听器并保存状态到 store
onUnmounted(() => {
    // 保存当前聊天历史到 store
    const characterId = getCurrentCharacterId();
    if (characterId && messages.value.length > 0) {
        const chatMessages: ChatMessage[] = messages.value.map(msg => ({
            role: msg.role,
            content: msg.content,
            timestamp: Math.floor(msg.timestamp.getTime() / 1000),
            name: undefined,
            tool_calls: undefined,
            tool_call_id: undefined,
        }));
        chatStore.setChatHistory(characterId, chatMessages);
        console.log(`💾 组件卸载，保存 ${chatMessages.length} 条消息到 Store`);
    }

    cleanupEventListeners();
});
</script>

<template>
    <div v-if="visible" class="card rounded-xl w-1/2 bg-white p-6 shadow-2xl">
        <div class="h-full flex flex-col">
            <!-- 面板头部 -->
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-3">
                    <h2 class="text-sm font-semibold text-gray-900">
                        <span v-if="panelType === 'ai'">Copilot</span>
                        <span v-else-if="panelType === 'chat'">对话</span>
                        <span v-else-if="panelType === 'tools'">工具</span>
                        <span v-else>AI Panel</span>
                    </h2>

                    <!-- AI角色选择器 -->
                    <select
                        v-model="selectedRole"
                        class="text-sm border border-gray-300 rounded-lg px-2 py-1 bg-white"
                        :disabled="aiRoles.length === 0"
                    >
                        <option value="" disabled>选择AI角色</option>
                        <option
                            v-for="role in aiRoles"
                            :key="role.name"
                            :value="role.name"
                        >
                            {{ role.role.name }}
                            <span
                                v-if="role.name === defaultRole"
                                class="text-blue-500"
                                >(默认)</span
                            >
                        </option>
                    </select>

                    <!-- API配置选择器 -->
                    <select
                        v-model="selectedApi"
                        class="text-sm border border-gray-300 rounded-lg px-2 py-1 bg-white"
                        :disabled="apiConfigs.length === 0"
                    >
                        <option value="" disabled>选择API配置</option>
                        <option
                            v-for="config in apiConfigs"
                            :key="config.profile"
                            :value="config.profile"
                        >
                            {{ config.profile }} ({{ config.model }})
                        </option>
                    </select>
                </div>

                <button
                    @click="togglePanel"
                    class="text-gray-400 hover:text-gray-600 transition-colors"
                    title="隐藏面板"
                >
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>

            <!-- 对话消息区域 -->
            <div
                ref="chatMessagesRef"
                class="flex-1 overflow-y-auto mb-4 border border-gray-200 rounded-lg p-4 bg-gray-50"
            >
                <div
                    v-if="messages.length === 0"
                    class="flex items-center justify-center h-full text-gray-500"
                >
                    <div class="text-center">
                        <div class="text-4xl mb-2">💬</div>
                        <p class="text-sm">开始与AI助手对话</p>
                        <p class="text-xs text-gray-400 mt-1">
                            基于当前角色数据进行智能分析
                        </p>
                    </div>
                </div>

                <div v-else class="space-y-4">
                    <div
                        v-for="(group, groupIndex) in groupedMessages"
                        :key="group.type === 'normal' ? group.message.id : `tool-${groupIndex}`"
                        class="flex"
                        :class="
                            group.type === 'normal' && group.message.role === 'user'
                                ? 'justify-end'
                                : 'justify-start'
                        "
                    >
                        <!-- 工具执行卡片 -->
                        <ToolExecutionCard
                            v-if="group.type === 'tool-execution'"
                            :tool-calls="group.toolCalls"
                            :tool-results="group.toolResults"
                            :timestamp="group.timestamp"
                            @delete="deleteToolExecutionGroup(groupIndex)"
                        />

                        <!-- 普通消息 -->
                        <MessageBubble
                            v-else-if="group.type === 'normal'"
                            :message-id="group.message.id"
                            :role="group.message.role as 'user' | 'assistant'"
                            :content="group.message.content"
                            :timestamp="group.message.timestamp"
                            :is-editing="group.message.isEditing"
                            :loading="aiStore.isLoading"
                            :is-last-message="groupIndex === groupedMessages.length - 1"
                            @continue="continueFromUserMessage"
                            @regenerate="regenerateResponse"
                            @start-edit="handleStartEdit(group.message.id)"
                            @save-edit="handleSaveEdit(group.message.id, $event)"
                            @cancel-edit="handleCancelEdit(group.message.id)"
                            @delete="handleDeleteMessage(group.message.id)"
                        />
                    </div>

                    <!-- 加载中指示器 -->
                    <div v-if="aiStore.isLoading" class="flex justify-start">
                        <div
                            class="bg-white border border-gray-200 rounded-lg rounded-bl-sm px-4 py-2"
                        >
                            <div class="flex items-center gap-2">
                                <div
                                    class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                ></div>
                                <div
                                    class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                    style="animation-delay: 0.1s"
                                ></div>
                                <div
                                    class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                    style="animation-delay: 0.2s"
                                ></div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 用户输入区域 -->
            <div class="border-t border-gray-200 pt-4 relative">
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
                    :disabled="aiStore.isLoading"
                    :loading="aiStore.isLoading"
                    :command-palette-open="showCommandPalette"
                    @send="handleSendMessage"
                    @open-command-palette="openCommandPalette"
                    @keydown="handleInputKeydown"
                    @input="handleInputChange"
                />

                <!-- 状态提示 -->
                <div class="flex justify-between items-center mt-2">
                    <div class="text-xs text-gray-500 flex gap-4">
                        <span v-if="selectedRole">
                            角色: {{ currentRoleConfig?.name || selectedRole }}
                        </span>
                        <span v-else class="text-orange-500">请选择AI角色</span>
                        <span v-if="selectedApi">API: {{ selectedApi }}</span>
                        <span
                            v-else-if="apiConfigs.length === 0"
                            class="text-orange-500"
                            >请配置API</span
                        >
                    </div>
                    <div class="text-xs text-gray-400">
                        {{ characterData ? "已加载角色数据" : "无角色数据" }}
                    </div>
                </div>
            </div>
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

/* 选择器样式 */
select {
    transition:
        border-color 0.15s ease-in-out,
        box-shadow 0.15s ease-in-out;
}

select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
</style>
