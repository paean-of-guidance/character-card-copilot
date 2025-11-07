<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from "vue";
import {
    MdOutlineRefresh,
    MdOutlineEdit,
    MdOutlineDelete,
} from "vue-icons-plus/md";
import { getAllApiConfigs } from "@/services/apiConfig";
import type { ApiConfig, ChatMessage } from "@/types/api";
import { AIConfigService, type AIRole } from "@/services/aiConfig";
import { listen } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/core';
import MarkdownRenderer from "./MarkdownRenderer.vue";
import CommandPalette from "./CommandPalette.vue";
import Modal from "./Modal.vue";
import ToolExecutionCard from "./ToolExecutionCard.vue";
import { backendCommandService } from "@/services/backendCommandService";
import type { CommandMetadata } from "@/types/commands";
import type { ModalOptions } from "@/utils/notification";
import { useChatStore } from "@/stores/chat";
import type {
  CharacterLoadedPayload,
  ChatHistoryLoadedPayload,
  MessageSentPayload,
  MessageReceivedPayload,
  ContextBuiltPayload,
  CharacterUpdatedPayload,
  ToolExecutedPayload,
  SessionUnloadedPayload,
  ErrorPayload,
  TokenStatsPayload,
  ProgressPayload
} from "@/types/events";

/**
 * 前端消息显示类型
 *
 * 扩展自后端的 ChatMessage 类型，添加前端特有的显示和交互字段
 *
 * 关键差异：
 * - timestamp: 后端使用 number (Unix 毫秒)，前端转换为 Date 对象方便显示
 * - id: 前端生成的唯一标识符，用于 v-for 的 key 绑定
 * - isEditing: 前端编辑状态标记
 *
 * 重要：保持 role 字段的完整性
 * - 必须保留所有可能的 role 值：'user' | 'assistant' | 'tool'
 * - 不能将 'tool' 消息转换为其他 role 类型
 * - 必须保留 tool_calls, tool_call_id, name 等可选字段
 */
interface DisplayMessage extends Omit<ChatMessage, 'timestamp'> {
    /** 前端生成的唯一 ID，用于列表渲染 key */
    id: string;
    /** 消息时间戳（Date 对象，方便前端格式化显示） */
    timestamp: Date;
    /** 消息是否处于编辑状态 */
    isEditing?: boolean;
}

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

// 对话相关状态 - 保持为 ref，但同步到 store
const messages = ref<DisplayMessage[]>([]);

const userInput = ref("");
const isLoading = ref(false);
const selectedApi = ref("");
const apiConfigs = ref<ApiConfig[]>([]);

// AI角色相关状态
const selectedRole = ref("");
const aiRoles = ref<Array<{ name: string; role: AIRole }>>([]);
const currentRoleConfig = ref<AIRole | null>(null);
const defaultRole = ref("");

// 输入框自适应高度
const textareaRef = ref<HTMLTextAreaElement>();
const inputRows = ref(1);

// 聊天容器引用
const chatMessagesRef = ref<HTMLElement>();

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

// 后端事件相关状态
const isBackendSessionActive = ref(false);
const currentSessionUUID = ref<string>("");
const contextBuiltInfo = ref<any>(null);
const lastTokenStats = ref<any>(null);
const isLoadingFromBackend = ref(false);

// 事件监听器清理函数列表
const eventUnlisteners = ref<(() => void)[]>([]);

/**
 * 分组消息类型
 *
 * 使用类型判别联合 (Discriminated Union) 区分不同类型的消息组：
 * - normal: 普通的用户或助手消息
 * - tool-execution: 工具调用流程组（包含调用请求和执行结果）
 */
type GroupedMessage =
    | { type: 'normal'; message: DisplayMessage }
    | { type: 'tool-execution'; toolCalls: import('@/types/api').ToolCall[]; toolResults: DisplayMessage[]; timestamp: Date };

/**
 * 消息分组计算属性
 *
 * 将原始消息列表转换为分组显示结构，主要功能：
 * 1. 合并工具调用流程：将 assistant 消息的 tool_calls 和后续的 tool 消息合并为一个卡片
 * 2. 保持普通消息不变：user 和不带 tool_calls 的 assistant 消息独立显示
 *
 * 处理逻辑示例：
 * ```
 * 原始消息序列：
 * [
 *   { role: 'user', content: '搜索XXX' },
 *   { role: 'assistant', content: '', tool_calls: [{id: 'call_1', ...}] },
 *   { role: 'tool', content: '{...}', tool_call_id: 'call_1' },
 *   { role: 'assistant', content: '根据搜索结果...' }
 * ]
 *
 * 分组后：
 * [
 *   { type: 'normal', message: {...} },                    // user 消息
 *   { type: 'tool-execution', toolCalls: [...], toolResults: [...] }, // 工具调用组
 *   { type: 'normal', message: {...} }                     // assistant 回复
 * ]
 * ```
 *
 * @returns 分组后的消息列表，用于渲染不同类型的消息卡片
 */
const groupedMessages = computed<GroupedMessage[]>(() => {
    const result: GroupedMessage[] = [];
    let i = 0;

    while (i < messages.value.length) {
        const msg = messages.value[i];

        // 检测工具调用起始点：带 tool_calls 的 assistant 消息
        if (msg.role === 'assistant' && msg.tool_calls && msg.tool_calls.length > 0) {
            const toolCalls = msg.tool_calls;
            const toolResults: DisplayMessage[] = [];
            let j = i + 1;

            // 收集紧随其后的所有 tool 消息（工具执行结果）
            while (j < messages.value.length && messages.value[j].role === 'tool') {
                toolResults.push(messages.value[j]);
                j++;
            }

            // 添加工具执行组（单个卡片显示）
            result.push({
                type: 'tool-execution',
                toolCalls,
                toolResults,
                timestamp: msg.timestamp
            });

            i = j; // 跳过已处理的 tool 消息
        } else if (msg.role !== 'tool') {
            // 普通消息（user 或不带 tool_calls 的 assistant）
            result.push({
                type: 'normal',
                message: msg
            });
            i++;
        } else {
            // 孤立的 tool 消息（没有对应的 tool_calls）
            // 理论上不应该发生，跳过以保证健壮性
            i++;
        }
    }

    return result;
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

// 自动调整输入框高度
function adjustTextareaHeight() {
    nextTick(() => {
        if (textareaRef.value) {
            const textarea = textareaRef.value;
            const lineHeight = 24; // 行高24px
            const maxRows = 5;
            const maxHeight = lineHeight * maxRows;

            // 先重置高度为最小高度
            textarea.style.height = "40px";

            // 获取实际需要的行数
            const lines = textarea.value.split("\n").length;

            // 只有当内容包含换行符或者内容长度超过一行时才调整高度
            if (lines > 1 || textarea.value.length > 60) {
                const scrollHeight = textarea.scrollHeight;
                const newHeight = Math.min(scrollHeight, maxHeight);
                textarea.style.height = newHeight + "px";
                inputRows.value = Math.min(lines, maxRows);
            } else {
                // 保持最小高度
                textarea.style.height = "40px";
                inputRows.value = 1;
            }
        }
    });
}

// 处理用户输入
function handleInput() {
    // 只有当输入内容包含换行符时才调整高度
    if (userInput.value.includes("\n") || userInput.value.length > 80) {
        adjustTextareaHeight();
    } else {
        // 如果没有换行符且长度较短，保持最小高度
        if (textareaRef.value) {
            textareaRef.value.style.height = "40px";
        }
        inputRows.value = 1;
    }
}

// 发送消息
async function sendMessage() {
    // 始终使用后端会话方式
    await sendMessageViaBackend();
}

// 处理键盘事件
function handleKeydown(event: KeyboardEvent) {
    // 如果命令面板打开，将键盘事件委托给命令面板处理
    if (showCommandPalette.value && commandPaletteRef.value) {
        // 命令面板处理以下按键：ArrowUp, ArrowDown, Enter, Tab, Space, Escape
        if (
            ["ArrowUp", "ArrowDown", "Enter", "Tab", " ", "Escape"].includes(
                event.key,
            )
        ) {
            commandPaletteRef.value.handleKeydown(event);
            return;
        }
    }

    // 检测"/"键触发命令面板
    // 当且仅当输入框完全为空时，按下"/"才触发命令面板
    if (event.key === "/" && userInput.value === "") {
        event.preventDefault();
        openCommandPalette();
        return;
    }

    // 普通发送消息逻辑（Shift+Enter换行，Enter发送）
    if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        sendMessage();
    }
}

// 格式化时间
function formatTime(date: Date) {
    return date.toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
    });
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

        // 直接调用后端加载历史记录
        const history = await invoke<ChatMessage[]>('load_chat_history', {
            characterId
        });

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

// ==================== 后端事件监听 ====================

/**
 * 初始化后端事件监听器
 */
async function initializeBackendEventListeners() {
    console.log("初始化后端事件监听器...");

    // 角色加载事件
    const unlistenCharacterLoaded = await listen<CharacterLoadedPayload>("character-loaded", (event) => {
        console.log("🎭 角色加载事件:", event.payload);
        const payload = event.payload;
        currentSessionUUID.value = payload.uuid;
        isBackendSessionActive.value = true;
        isLoadingFromBackend.value = false;

        // 可以在这里通知父组件角色数据已更新
        // emit('character-updated', payload.character_data);
    });

    // 聊天历史加载事件
    const unlistenChatHistoryLoaded = await listen<ChatHistoryLoadedPayload>("chat-history-loaded", (event) => {
        console.log("📚 聊天历史加载事件:", event.payload);
        const payload = event.payload;

        // 转换为前端消息格式（保留所有 role 类型）
        messages.value = payload.chat_history.map((msg, index) => ({
            id: `${msg.timestamp || index}_${payload.uuid}`,
            role: msg.role, // 保留原始 role：user/assistant/tool
            content: msg.content,
            timestamp: new Date((msg.timestamp || Date.now() / 1000) * 1000),
            // 保留工具调用相关字段
            tool_calls: msg.tool_calls,
            tool_call_id: msg.tool_call_id,
            name: msg.name,
        }));

        // 同步到 store
        chatStore.setChatHistory(payload.uuid, payload.chat_history);
        chatStore.setActiveCharacter(payload.uuid);

        console.log(`从后端加载了 ${messages.value.length} 条聊天历史记录`);
    });

    // 消息发送事件
    const unlistenMessageSent = await listen<MessageSentPayload>("message-sent", (event) => {
        console.log("📤 消息发送事件:", event.payload);
        const payload = event.payload;

        // 如果消息不在前端列表中，添加它
        const existingMessage = messages.value.find(msg =>
            msg.content === payload.message.content &&
            msg.role === "user"
        );

        if (!existingMessage) {
            const userMessageObj = {
                id: `${payload.message.timestamp}_sent_${payload.uuid}`,
                role: "user" as const,
                content: payload.message.content,
                timestamp: new Date(payload.message.timestamp || Date.now()),
            };
            messages.value.push(userMessageObj);
        }
    });

    // 消息接收事件
    const unlistenMessageReceived = await listen<MessageReceivedPayload>("message-received", (event) => {
        console.log("📥 消息接收事件:", event.payload);
        const payload = event.payload;

        // 如果有中间消息（工具调用流程），先插入它们
        if (payload.intermediate_messages && payload.intermediate_messages.length > 0) {
            console.log(`🔄 插入 ${payload.intermediate_messages.length} 条中间消息（tool 调用流程）`);

            const intermediateDisplayMessages = payload.intermediate_messages.map((msg, index) => ({
                id: `${msg.timestamp || Date.now()}_intermediate_${index}_${payload.uuid}`,
                role: msg.role,
                content: msg.content,
                timestamp: new Date(msg.timestamp || Date.now()),
                tool_calls: msg.tool_calls,
                tool_call_id: msg.tool_call_id,
                name: msg.name,
            }));

            messages.value.push(...intermediateDisplayMessages);
        }

        // 添加最终的 AI 回复消息
        const aiMessageObj: DisplayMessage = {
            id: `${payload.message.timestamp}_received_${payload.uuid}`,
            role: "assistant",
            content: payload.message.content,
            timestamp: new Date(payload.message.timestamp || Date.now()),
            // 保留工具调用字段（如果有）
            tool_calls: payload.message.tool_calls,
            tool_call_id: payload.message.tool_call_id,
            name: payload.message.name,
        };
        messages.value.push(aiMessageObj);

        // 设置加载完成
        isLoading.value = false;
    });

    // 上下文构建完成事件
    const unlistenContextBuilt = await listen<ContextBuiltPayload>("context-built", (event) => {
        console.log("🔧 上下文构建完成事件:", event.payload);
        const payload = event.payload;
        contextBuiltInfo.value = payload.context_result;
    });

    // 角色更新事件
    const unlistenCharacterUpdated = await listen<CharacterUpdatedPayload>("character-updated", (event) => {
        console.log("🔄 角色更新事件:", event.payload);

        // 可以在这里通知父组件角色数据已更新
        // emit('character-updated', event.payload.character_data);
    });

    /**
     * 工具执行事件监听器
     *
     * 用于调试和日志记录工具执行情况
     *
     * 注意：工具消息（role: "tool"）现在通过 message-received 事件的
     *      intermediate_messages 字段统一接收，无需在此创建消息
     *
     * 数据流：
     * Backend tool execution -> intermediate_messages -> message-received -> UI display
     */
    const unlistenToolExecuted = await listen<ToolExecutedPayload>("tool-executed", (event) => {
        const payload = event.payload;

        if (payload.success) {
            console.log("✅ 工具执行成功:", {
                工具名称: payload.tool_name,
                执行时间: `${payload.execution_time_ms}ms`,
                结果: payload.result
            });
        } else {
            console.error("❌ 工具执行失败:", {
                工具名称: payload.tool_name,
                错误: payload.error,
                执行时间: `${payload.execution_time_ms}ms`
            });
        }

        // 注：tool 消息会通过 message-received 事件的 intermediate_messages 字段接收
        // 无需在此手动创建，避免消息重复
    });

    // 会话卸载事件
    const unlistenSessionUnloaded = await listen<SessionUnloadedPayload>("session-unloaded", (event) => {
        console.log("🚪 会话卸载事件:", event.payload);
        const payload = event.payload;

        if (payload.uuid === currentSessionUUID.value) {
            isBackendSessionActive.value = false;
            currentSessionUUID.value = "";
            messages.value = [];
            contextBuiltInfo.value = null;
        }
    });

    // 错误事件
    const unlistenError = await listen<ErrorPayload>("error", (event) => {
        console.error("❌ 错误事件:", event.payload);
        const payload = event.payload;

        const errorMessageObj = {
            id: `error_${payload.timestamp}_${payload.uuid || 'unknown'}`,
            role: "assistant" as const,
            content: `⚠️ 系统错误 [${payload.error_code}]: ${payload.error_message}`,
            timestamp: new Date(payload.timestamp),
        };

        messages.value.push(errorMessageObj);
        isLoading.value = false;
    });

    // Token统计事件
    const unlistenTokenStats = await listen<TokenStatsPayload>("token-stats", (event) => {
        console.log("📊 Token统计事件:", event.payload);
        lastTokenStats.value = event.payload.token_usage;
    });

    // 进度事件
    const unlistenProgress = await listen<ProgressPayload>("progress", (event) => {
        console.log("📈 进度事件:", event.payload);
        const payload = event.payload;

        if (payload.operation === "ai_response") {
            isLoading.value = payload.progress < 1.0;
        }
    });

    // 保存所有清理函数
    eventUnlisteners.value.push(
        unlistenCharacterLoaded,
        unlistenChatHistoryLoaded,
        unlistenMessageSent,
        unlistenMessageReceived,
        unlistenContextBuilt,
        unlistenCharacterUpdated,
        unlistenToolExecuted,
        unlistenSessionUnloaded,
        unlistenError,
        unlistenTokenStats,
        unlistenProgress,
    );

    console.log("✅ 后端事件监听器初始化完成");
}

/**
 * 清理所有事件监听器
 */
function cleanupEventListeners() {
    console.log("清理事件监听器...");
    eventUnlisteners.value.forEach(unlisten => {
        try {
            unlisten();
        } catch (error) {
            console.error("清理事件监听器失败:", error);
        }
    });
    eventUnlisteners.value = [];
    console.log("✅ 事件监听器清理完成");
}

/**
 * 通过后端发送消息
 */
async function sendMessageViaBackend() {
    if (!userInput.value.trim() || isLoading.value) return;

    const message = userInput.value.trim();
    userInput.value = "";

    // 重置输入框高度
    if (textareaRef.value) {
        textareaRef.value.style.height = "40px";
    }
    inputRows.value = 1;

    // 检查是否有活跃的后端会话
    if (!isBackendSessionActive.value) {
        const characterId = getCurrentCharacterId();
        if (!characterId) {
            console.error("无法获取角色ID，无法发送消息");
            return;
        }

        console.log("触发后端角色会话加载...");
        isLoadingFromBackend.value = true;
        try {
            await invoke('load_character_session', { uuid: characterId });
            // 等待角色加载事件完成后再发送消息
            setTimeout(async () => {
                if (isBackendSessionActive.value) {
                    await invoke('send_chat_message', { message });
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
        isLoading.value = true;
        try {
            await invoke('send_chat_message', { message });
        } catch (error) {
            console.error("发送消息失败:", error);
            isLoading.value = false;
        }
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
            if (isBackendSessionActive.value) {
                const characterId = getCurrentCharacterId();
                if (characterId) {
                    isLoadingFromBackend.value = true;
                    try {
                        await invoke('load_character_session', { uuid: characterId });
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

// 获取消息在 messages 数组中的索引
function getMessageIndex(message: DisplayMessage): number {
    return messages.value.findIndex(m => m.id === message.id);
}

// 编辑消息
function editMessage(index: number) {
    if (index >= 0 && index < messages.value.length) {
        editingContent.value = messages.value[index].content;
        messages.value[index].isEditing = true;
    }
}

// 取消编辑
function cancelEdit(index: number) {
    if (index >= 0 && index < messages.value.length) {
        messages.value[index].isEditing = false;
    }
    editingContent.value = "";
}

// 保存编辑
async function saveEdit(index: number) {
    if (index >= 0 && index < messages.value.length) {
        try {
            const originalContent = messages.value[index].content;
            const newContent = editingContent.value.trim();

            if (!newContent) {
                // 如果内容为空，删除消息
                await deleteMessage(index);
                return;
            }

            if (newContent !== originalContent) {
                // 调用后端编辑消息
                await invoke('edit_chat_message', {
                    index,
                    newContent
                });

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

// 处理编辑时的键盘事件
function handleEditKeydown(index: number, event: KeyboardEvent) {
    if (event.key === "Enter" && event.ctrlKey) {
        // Ctrl+Enter 保存编辑
        event.preventDefault();
        saveEdit(index);
    } else if (event.key === "Escape") {
        // Escape 取消编辑
        event.preventDefault();
        cancelEdit(index);
    }
}

// 删除消息
async function deleteMessage(index: number) {
    if (index >= 0 && index < messages.value.length) {
        try {
            // 调用后端删除消息
            await invoke('delete_chat_message', { index });

            // 前端也删除（后端会通过事件同步，但为了即时响应先删除）
            messages.value.splice(index, 1);

            console.log(`✅ 已删除消息 [${index}]`);
        } catch (error) {
            console.error("删除消息失败:", error);
        }
    }
}

// 重新生成响应
async function regenerateResponse() {
    if (messages.value.length === 0) return;

    // 检查最后一条消息是否是AI回复
    const lastMessage = messages.value[messages.value.length - 1];

    if (lastMessage.role === "assistant") {
        try {
            isLoading.value = true;

            // 先删除前端的最后一条AI消息（后端也会删除）
            messages.value.pop();

            // 调用后端重新生成命令（会自动删除后端历史并重新生成）
            await invoke('regenerate_last_message');

            console.log("✅ 重新生成完成");
        } catch (error) {
            console.error("重新生成失败:", error);
            isLoading.value = false;
        }
    } else {
        console.warn("最后一条消息不是AI回复，无法重新生成");
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
    userInput.value = "/";
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

    // 清空输入框中的"/"或以"/"开头的命令
    if (userInput.value === "/" || userInput.value.startsWith("/")) {
        userInput.value = "";
    }

    // 重置输入框高度
    if (textareaRef.value) {
        textareaRef.value.style.height = "40px";
    }
    inputRows.value = 1;
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
    await initializeBackendEventListeners();

    // 事件监听器初始化完成后，检查是否需要重新加载会话
    // 只在 store 中有数据但后端会话已失效时才重新加载
    if (props.characterData?.name && characterId) {
        const storedHistory = chatStore.getChatHistory(characterId);
        if (chatStore.isBackendSessionActive && storedHistory.length > 0) {
            console.log(`🔄 组件重新挂载，后端会话已存在，跳过重复加载`);
            isBackendSessionActive.value = true;
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
                        />

                        <!-- 普通消息 -->
                        <div
                            v-else-if="group.type === 'normal'"
                            class="max-w-[80%] px-4 py-2 rounded-lg group relative"
                            :class="
                                group.message.role === 'user'
                                    ? 'bg-blue-500 text-white rounded-br-sm'
                                    : 'bg-white border border-gray-200 text-gray-800 rounded-bl-sm'
                            "
                        >
                            <MarkdownRenderer
                                v-if="group.message.role === 'assistant'"
                                :content="group.message.content"
                                class="text-sm"
                            />
                            <div v-else class="text-sm whitespace-pre-wrap">
                                {{ group.message.content }}
                            </div>
                            <div
                                class="text-xs mt-1 opacity-70"
                                :class="
                                    group.message.role === 'user'
                                        ? 'text-blue-100'
                                        : 'text-gray-500'
                                "
                            >
                                {{ formatTime(group.message.timestamp) }}
                            </div>

                            <!-- 消息操作按钮 -->
                            <div
                                v-if="!isLoading"
                                class="absolute -bottom-6 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1"
                                :class="
                                    group.message.role === 'user'
                                        ? 'left-0'
                                        : 'right-0'
                                "
                            >
                                <!-- AI消息：重新生成按钮 -->
                                <button
                                    v-if="
                                        group.message.role === 'assistant' &&
                                        groupIndex === groupedMessages.length - 1
                                    "
                                    @click="regenerateResponse()"
                                    class="p-1 bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
                                    title="重新生成"
                                >
                                    <MdOutlineRefresh
                                        class="w-4 h-4 text-gray-600"
                                    />
                                </button>

                                <!-- 编辑按钮 -->
                                <button
                                    @click="editMessage(getMessageIndex(group.message))"
                                    class="p-1 bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
                                    title="编辑消息"
                                >
                                    <MdOutlineEdit
                                        class="w-4 h-4 text-gray-600"
                                    />
                                </button>

                                <!-- 删除按钮 -->
                                <button
                                    @click="deleteMessage(getMessageIndex(group.message))"
                                    class="p-1 bg-gray-100 hover:bg-red-100 rounded-full transition-colors"
                                    title="删除消息"
                                >
                                    <MdOutlineDelete
                                        class="w-4 h-4 text-gray-600 hover:text-red-600"
                                    />
                                </button>
                            </div>

                            <!-- 编辑模式的输入框 -->
                            <div v-if="group.message.isEditing" class="mt-2">
                                <textarea
                                    v-model="editingContent"
                                    @keydown="handleEditKeydown(getMessageIndex(group.message), $event)"
                                    @blur="saveEdit(getMessageIndex(group.message))"
                                    class="w-full p-2 border border-gray-300 rounded text-sm resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    rows="3"
                                    placeholder="编辑消息内容..."
                                ></textarea>
                                <div class="flex gap-2 mt-2">
                                    <button
                                        @click="saveEdit(getMessageIndex(group.message))"
                                        class="text-xs bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 transition-colors"
                                    >
                                        保存
                                    </button>
                                    <button
                                        @click="cancelEdit(getMessageIndex(group.message))"
                                        class="text-xs bg-gray-300 text-gray-700 px-3 py-1 rounded hover:bg-gray-400 transition-colors"
                                    >
                                        取消
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- 加载中指示器 -->
                    <div v-if="isLoading" class="flex justify-start">
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

                <div class="flex gap-3">
                    <textarea
                        ref="textareaRef"
                        v-model="userInput"
                        @input="handleInput"
                        @keydown="handleKeydown"
                        :disabled="isLoading"
                        placeholder="输入消息... (Enter发送，Shift+Enter换行)"
                        class="flex-1 resize-none border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed overflow-hidden"
                        style="
                            height: 40px;
                            min-height: 40px;
                            max-height: 120px;
                            line-height: 24px;
                        "
                    ></textarea>

                    <button
                        @click="sendMessage"
                        :disabled="!userInput.trim() || isLoading"
                        class="bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 text-white px-4 py-2 rounded-lg transition-colors flex items-center justify-center self-end"
                        title="发送消息"
                        style="height: 40px"
                    >
                        <svg
                            v-if="!isLoading"
                            class="w-4 h-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                            />
                        </svg>
                        <div
                            v-else
                            class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"
                        ></div>
                    </button>
                </div>

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

/* 输入框样式 */
textarea {
    line-height: 1.5;
    font-family: inherit;
}

/* 输入框焦点样式 */
textarea:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
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
