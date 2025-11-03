<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch } from "vue";
import {
    MdOutlineRefresh,
    MdOutlinePlayCircle,
    MdOutlineEdit,
    MdOutlineDelete,
} from "vue-icons-plus/md";
import { getAllApiConfigs } from "@/services/apiConfig";
import type { ApiConfig } from "@/types/api";
import { AIConfigService, type AIRole } from "@/services/aiConfig";
import { AIChatService, type ChatCompletionOptions } from "@/services/aiChat";
import { AIToolsService } from "@/services/aiTools";
import { ChatHistoryManager } from "@/services/chatHistory";
import type { ChatMessage } from "@/types/api";
import { listen } from '@tauri-apps/api/event';
import MarkdownRenderer from './MarkdownRenderer.vue';

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

// 聊天历史记录管理
let chatHistoryManager: ChatHistoryManager | null = null;

// 对话相关状态
const messages = ref<
    Array<{
        id: string;
        role: "user" | "assistant";
        content: string;
        timestamp: Date;
        isEditing?: boolean;
    }>
>([]);

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
        apiConfigs.value = configs.filter((config) => config.enabled);
        if (apiConfigs.value.length > 0 && !selectedApi.value) {
            selectedApi.value = apiConfigs.value[0].profile;
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

// 生成唯一ID
function generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// 发送消息
async function sendMessage() {
    if (!userInput.value.trim() || isLoading.value) return;

    const userMessage = userInput.value.trim();
    userInput.value = "";

    // 重置输入框高度
    if (textareaRef.value) {
        textareaRef.value.style.height = "40px";
    }
    inputRows.value = 1;

    // 添加用户消息
    const userMessageObj = {
        id: generateId(),
        role: "user" as const,
        content: userMessage,
        timestamp: new Date(),
    };
    messages.value.push(userMessageObj);

    // 保存用户消息到历史记录
    if (chatHistoryManager) {
        try {
            await chatHistoryManager.saveMessage({
                role: "user",
                content: userMessage,
                timestamp: userMessageObj.timestamp.getTime(),
            });
        } catch (error) {
            console.error("保存用户消息失败:", error);
        }
    }

    isLoading.value = true;

    try {
        // TODO: 实现AI调用逻辑
        // 这里将集成CharacterData作为上下文
        await simulateAIResponse();
    } catch (error) {
        console.error("发送消息失败:", error);
        const errorMessageObj = {
            id: generateId(),
            role: "assistant" as const,
            content: "抱歉，发生了错误，请稍后重试。",
            timestamp: new Date(),
            isEditing: false,
        };
        messages.value.push(errorMessageObj);

        // 保存错误消息到历史记录
        if (chatHistoryManager) {
            try {
                await chatHistoryManager.saveMessage({
                    role: "assistant",
                    content: errorMessageObj.content,
                    timestamp: errorMessageObj.timestamp.getTime(),
                });
            } catch (error) {
                console.error("保存错误消息失败:", error);
            }
        }
    } finally {
        isLoading.value = false;
    }
}

// 真实的AI响应
async function simulateAIResponse() {
    try {
        // 检查是否有可用的API配置
        if (!selectedApi.value) {
            throw new Error("请先选择API配置");
        }

        if (!currentRoleConfig.value) {
            throw new Error("请先选择AI角色");
        }

        // 获取API配置
        const apiConfigs = await getAllApiConfigs();
        const apiConfig = apiConfigs.find(
            (config) => config.profile === selectedApi.value,
        );

        if (!apiConfig) {
            throw new Error("API配置不存在");
        }

        // 验证API配置
        const validationErrors = AIChatService.validateApiConfig(apiConfig);
        if (validationErrors.length > 0) {
            throw new Error(`API配置验证失败: ${validationErrors.join(", ")}`);
        }

        // 构建聊天消息
        const conversationHistory = messages.value
            .slice(-10) // 只保留最近10条消息作为上下文
            .filter((msg) => msg.role !== "assistant" || msg.content.trim())
            .map((msg) => ({
                role: msg.role as "user" | "assistant",
                content: msg.content,
            }));

        const systemPrompt = currentRoleConfig.value.system_prompt;
        const currentMessage = userInput.value;

        const chatMessages: ChatMessage[] = AIChatService.buildMessages(
            systemPrompt,
            conversationHistory,
            currentMessage,
            props.characterData,
        );

        // 获取工具（临时强制启用工具进行测试）
        const tools = await convertToolsToChatTools(); // currentRoleConfig.value.tools_enabled
            // ? await convertToolsToChatTools()
            // : undefined;

        // 构建聊天完成选项
        const options: ChatCompletionOptions = {
            model: apiConfig.model,
            messages: chatMessages,
            temperature: currentRoleConfig.value.temperature,
            max_tokens: currentRoleConfig.value.max_tokens,
            tools,
            tool_choice: tools ? "auto" : "none",
        };

        console.log("发送聊天请求:", {
            api: apiConfig.profile,
            model: apiConfig.model,
            messageCount: chatMessages.length,
            toolsEnabled: currentRoleConfig.value.tools_enabled,
            toolCount: tools?.length || 0,
            forceEnabledTools: true, // 临时强制启用
        });

        // 调用AI服务
        const response = await AIChatService.createChatCompletion(
            apiConfig,
            options,
        );

        if (response.choices.length === 0) {
            throw new Error("AI未返回响应");
        }

        const aiMessage = response.choices[0].message.content;

        const aiMessageObj = {
            id: generateId(),
            role: "assistant" as const,
            content: aiMessage,
            timestamp: new Date(),
        };
        messages.value.push(aiMessageObj);

        // 保存AI消息到历史记录
        if (chatHistoryManager) {
            try {
                await chatHistoryManager.saveMessage({
                    role: "assistant",
                    content: aiMessage,
                    timestamp: aiMessageObj.timestamp.getTime(),
                });
            } catch (error) {
                console.error("保存AI消息失败:", error);
            }
        }

        // 处理工具调用（如果有）
        if (response.choices[0].message.tool_calls) {
            await handleToolCalls(response.choices[0].message.tool_calls);
        }
    } catch (error) {
        console.error("AI调用失败:", error);

        messages.value.push({
            id: generateId(),
            role: "assistant",
            content: `抱歉，AI调用失败：${error instanceof Error ? error.message : "未知错误"}`,
            timestamp: new Date(),
            isEditing: false,
        });
    }
}

// 将AI工具转换为聊天工具格式
async function convertToolsToChatTools() {
    try {
        // 获取可用的AI工具
        const tools = await AIToolsService.getAvailableTools();

        // 转换为OpenAI格式
        const convertedTools = tools.map((tool) => ({
            type: "function" as const,
            function: {
                name: tool.name,
                description: tool.description,
                parameters: {
                    type: "object" as const,
                    properties: tool.parameters.reduce(
                        (acc, param) => {
                            acc[param.name] = {
                                type: param.parameter_type,
                                description: param.description,
                                ...(param.schema
                                    ? { schema: param.schema }
                                    : {}),
                            };
                            return acc;
                        },
                        {} as Record<string, any>,
                    ),
                    required: tool.parameters
                        .filter((p) => p.required)
                        .map((p) => p.name),
                },
            },
        }));

                return convertedTools;

    } catch (error) {
        console.error("转换工具失败:", error);
        return undefined;
    }
}

// 处理键盘事件
function handleKeydown(event: KeyboardEvent) {
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

// 处理AI工具调用
async function handleToolCalls(toolCalls: any[]) {
    for (const toolCall of toolCalls) {
        if (toolCall.type === 'function') {
            const functionName = toolCall.function.name;
            let functionArgs;

            try {
                functionArgs = JSON.parse(toolCall.function.arguments);
            } catch (error) {
                console.error("解析工具调用参数失败:", error);
                continue;
            }

            try {
                // 执行工具调用
                const result = await AIToolsService.executeToolCall({
                    tool_name: functionName,
                    parameters: functionArgs,
                    character_uuid: getCurrentCharacterId() || undefined,
                    context: props.characterData,
                });

                console.log("工具执行结果:", result);
                console.log("工具执行详情:", JSON.stringify(result, null, 2));

                // 将工具执行结果作为消息添加到对话中
                const toolResultMessage = {
                    id: generateId(),
                    role: "assistant" as const,
                    content: `工具执行结果：${result.success ?
                        `成功更新了${result.data?.update_count || 0}个字段：${result.data?.updated_fields?.map((f: any) => f.description).join('、') || '未知字段'}` :
                        `失败：${result.error || '未知错误'}`}`,
                    timestamp: new Date(),
                    isEditing: false,
                };

                messages.value.push(toolResultMessage);

                // 保存工具结果到聊天历史
                if (chatHistoryManager) {
                    try {
                        await chatHistoryManager.saveMessage({
                            role: "assistant",
                            content: toolResultMessage.content,
                            timestamp: toolResultMessage.timestamp.getTime(),
                        });
                    } catch (error) {
                        console.error("保存工具结果失败:", error);
                    }
                }

                // 如果工具执行成功，可能需要刷新角色数据
                if (result.success && props.characterData) {
                    // 可以通过事件通知父组件刷新数据
                    // 这里先简单处理，实际可以通过emit通知父组件
                    console.log("角色数据已更新，建议刷新界面");
                }

            } catch (error) {
                console.error("工具执行失败:", error);

                const errorMessage = {
                    id: generateId(),
                    role: "assistant" as const,
                    content: `工具执行失败：${error instanceof Error ? error.message : "未知错误"}`,
                    timestamp: new Date(),
                    isEditing: false,
                };

                messages.value.push(errorMessage);

                if (chatHistoryManager) {
                    try {
                        await chatHistoryManager.saveMessage({
                            role: "assistant",
                            content: errorMessage.content,
                            timestamp: errorMessage.timestamp.getTime(),
                        });
                    } catch (saveError) {
                        console.error("保存工具错误消息失败:", saveError);
                    }
                }
            }
        }
    }
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
        chatHistoryManager = null;
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

        // 创建新的聊天历史管理器
        chatHistoryManager = new ChatHistoryManager(characterId);

        // 加载历史记录
        const history = await chatHistoryManager.loadHistory();

        // 转换为前端消息格式
        if (history.length > 0) {
            messages.value = history.map((msg, index) => ({
                id: `${msg.timestamp || index}_${characterId}`,
                role: msg.role === "assistant" ? "assistant" : "user",
                content: msg.content,
                timestamp: new Date(msg.timestamp || Date.now()),
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
        if (newName !== oldName) {
            console.log(`角色切换: ${oldName} -> ${newName}`);
            await initializeChatHistory();
        }
    },
    { immediate: true },
);

// 监听消息变化，自动滚动到底部
watch(
    () => messages.value.length,
    () => {
        nextTick(() => {
            if (chatMessagesRef.value) {
                chatMessagesRef.value.scrollTop = chatMessagesRef.value.scrollHeight;
            }
        });
    }
);


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
    if (index >= 0 && index < messages.value.length && chatHistoryManager) {
        try {
            const originalContent = messages.value[index].content;
            const newContent = editingContent.value.trim();

            if (!newContent) {
                // 如果内容为空，删除消息
                await deleteMessage(index);
                return;
            }

            if (newContent !== originalContent) {
                // 更新前端消息
                messages.value[index].content = newContent;
                messages.value[index].isEditing = false;

                // 更新历史记录
                await chatHistoryManager.updateMessage(index, {
                    role:
                        messages.value[index].role === "assistant"
                            ? "assistant"
                            : "user",
                    content: newContent,
                    timestamp: messages.value[index].timestamp.getTime(),
                });
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
    if (index >= 0 && index < messages.value.length && chatHistoryManager) {
        try {
            // 删除前端消息
            messages.value.splice(index, 1);

            // 删除历史记录
            await chatHistoryManager.deleteMessage(index);
        } catch (error) {
            console.error("删除消息失败:", error);
        }
    }
}

// 触发AI回复
async function triggerAIReply(userMessage: string) {
    isLoading.value = true;

    try {
        // 检查是否有可用的API配置
        if (!selectedApi.value) {
            throw new Error("请先选择API配置");
        }

        if (!currentRoleConfig.value) {
            throw new Error("请先选择AI角色");
        }

        // 获取API配置
        const apiConfigs = await getAllApiConfigs();
        const apiConfig = apiConfigs.find(
            (config) => config.profile === selectedApi.value,
        );

        if (!apiConfig) {
            throw new Error("API配置不存在");
        }

        // 验证API配置
        const validationErrors = AIChatService.validateApiConfig(apiConfig);
        if (validationErrors.length > 0) {
            throw new Error(`API配置验证失败: ${validationErrors.join(", ")}`);
        }

        // 构建聊天消息（不包含用户消息，因为我们使用单独触发的消息）
        const conversationHistory = messages.value
            .slice(-10)
            .filter((msg) => msg.role !== "assistant" || msg.content.trim())
            .map((msg) => ({
                role: msg.role as "user" | "assistant",
                content: msg.content,
            }));

        const systemPrompt = currentRoleConfig.value.system_prompt;
        const chatMessages: ChatMessage[] = AIChatService.buildMessages(
            systemPrompt,
            conversationHistory,
            userMessage,
            props.characterData,
        );

        // 获取工具（临时强制启用工具进行测试）
        const tools = await convertToolsToChatTools(); // currentRoleConfig.value.tools_enabled
            // ? await convertToolsToChatTools()
            // : undefined;

        // 构建聊天完成选项
        const options: ChatCompletionOptions = {
            model: apiConfig.model,
            messages: chatMessages,
            temperature: currentRoleConfig.value.temperature,
            max_tokens: currentRoleConfig.value.max_tokens,
            tools,
            tool_choice: tools ? "auto" : "none",
        };

        // 调用AI服务
        const response = await AIChatService.createChatCompletion(
            apiConfig,
            options,
        );

        if (response.choices.length === 0) {
            throw new Error("AI未返回响应");
        }

        const aiMessage = response.choices[0].message.content;

        const aiMessageObj = {
            id: generateId(),
            role: "assistant" as const,
            content: aiMessage,
            timestamp: new Date(),
        };
        messages.value.push(aiMessageObj);

        // 保存AI消息到历史记录
        if (chatHistoryManager) {
            try {
                await chatHistoryManager.saveMessage({
                    role: "assistant",
                    content: aiMessage,
                    timestamp: aiMessageObj.timestamp.getTime(),
                });
            } catch (error) {
                console.error("保存AI消息失败:", error);
            }
        }
    } catch (error) {
        console.error("触发AI回复失败:", error);

        const errorMessageObj = {
            id: generateId(),
            role: "assistant" as const,
            content: `抱歉，AI调用失败：${error instanceof Error ? error.message : "未知错误"}`,
            timestamp: new Date(),
            isEditing: false,
        };
        messages.value.push(errorMessageObj);

        // 保存错误消息到历史记录
        if (chatHistoryManager) {
            try {
                await chatHistoryManager.saveMessage({
                    role: "assistant",
                    content: errorMessageObj.content,
                    timestamp: errorMessageObj.timestamp.getTime(),
                });
            } catch (error) {
                console.error("保存错误消息失败:", error);
            }
        }
    } finally {
        isLoading.value = false;
    }
}

// 重新生成响应
async function regenerateResponse() {
    if (messages.value.length === 0) return;

    // 找到倒数第二条消息（最后一条是AI回复）
    const lastMessage = messages.value[messages.value.length - 1];
    const secondLastMessage = messages.value[messages.value.length - 2];

    if (lastMessage.role === "assistant") {
        // 删除最后一条AI回复
        await deleteMessage(messages.value.length - 1);

        if (secondLastMessage && secondLastMessage.role === "user") {
            // 重新触发AI回复
            await triggerAIReply(secondLastMessage.content);
        }
    }
}

onMounted(async () => {
    loadApiConfigs();
    loadAIRoles();

    // 监听工具执行事件，用于调试
    await listen('tool-executed', (event) => {
        console.log('🔧 工具执行成功:', event.payload);
        const payload = event.payload as any;
        if (payload) {
            console.log(`工具名称: ${payload.tool_name}`);
            console.log(`角色UUID: ${payload.character_uuid}`);
            console.log(`更新字段数: ${payload.update_count}`);
            if (payload.updated_fields && Array.isArray(payload.updated_fields)) {
                console.log('更新详情:');
                payload.updated_fields.forEach((field: any) => {
                    console.log(`  - ${field.field}: ${field.description}`);
                });
            }
        }
    });
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
                        v-for="(message, index) in messages"
                        :key="message.id"
                        class="flex"
                        :class="
                            message.role === 'user'
                                ? 'justify-end'
                                : 'justify-start'
                        "
                    >
                        <div
                            class="max-w-[80%] px-4 py-2 rounded-lg group relative"
                            :class="
                                message.role === 'user'
                                    ? 'bg-blue-500 text-white rounded-br-sm'
                                    : 'bg-white border border-gray-200 text-gray-800 rounded-bl-sm'
                            "
                        >
                            <MarkdownRenderer
                                v-if="message.role === 'assistant'"
                                :content="message.content"
                                class="text-sm"
                            />
                            <div
                                v-else
                                class="text-sm whitespace-pre-wrap"
                            >
                                {{ message.content }}
                            </div>
                            <div
                                class="text-xs mt-1 opacity-70"
                                :class="
                                    message.role === 'user'
                                        ? 'text-blue-100'
                                        : 'text-gray-500'
                                "
                            >
                                {{ formatTime(message.timestamp) }}
                            </div>

                            <!-- 消息操作按钮 -->
                            <div
                                v-if="!isLoading"
                                class="absolute -bottom-6 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1"
                                :class="
                                    message.role === 'user'
                                        ? 'left-0'
                                        : 'right-0'
                                "
                            >
                                <!-- 用户消息：触发AI回复按钮 -->
                                <button
                                    v-if="
                                        message.role === 'user' &&
                                        index === messages.length - 1
                                    "
                                    @click="triggerAIReply(message.content)"
                                    class="p-1 bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
                                    title="触发AI回复"
                                >
                                    <MdOutlinePlayCircle
                                        class="w-4 h-4 text-gray-600"
                                    />
                                </button>

                                <!-- AI消息：重新生成按钮 -->
                                <button
                                    v-if="
                                        message.role === 'assistant' &&
                                        index === messages.length - 1
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
                                    @click="editMessage(index)"
                                    class="p-1 bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
                                    title="编辑消息"
                                >
                                    <MdOutlineEdit
                                        class="w-4 h-4 text-gray-600"
                                    />
                                </button>

                                <!-- 删除按钮 -->
                                <button
                                    @click="deleteMessage(index)"
                                    class="p-1 bg-gray-100 hover:bg-red-100 rounded-full transition-colors"
                                    title="删除消息"
                                >
                                    <MdOutlineDelete
                                        class="w-4 h-4 text-gray-600 hover:text-red-600"
                                    />
                                </button>
                            </div>

                            <!-- 编辑模式的输入框 -->
                            <div v-if="message.isEditing" class="mt-2">
                                <textarea
                                    v-model="editingContent"
                                    @keydown="handleEditKeydown(index, $event)"
                                    @blur="saveEdit(index)"
                                    class="w-full p-2 border border-gray-300 rounded text-sm resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    rows="3"
                                    placeholder="编辑消息内容..."
                                ></textarea>
                                <div class="flex gap-2 mt-2">
                                    <button
                                        @click="saveEdit(index)"
                                        class="text-xs bg-blue-500 text-white px-3 py-1 rounded hover:bg-blue-600 transition-colors"
                                    >
                                        保存
                                    </button>
                                    <button
                                        @click="cancelEdit(index)"
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
            <div class="border-t border-gray-200 pt-4">
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
