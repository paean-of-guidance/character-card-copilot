<template>
    <!-- 整体气泡容器 -->
    <div
        class="max-w-[80%] px-4 py-3 rounded-lg bg-yellow-50 border border-yellow-200 text-gray-800 rounded-bl-sm space-y-2"
    >
        <!-- 工具调用部分 -->
        <div
            v-for="(toolCall, index) in toolCalls"
            :key="toolCall.id"
            class="rounded-md overflow-hidden"
        >
            <!-- 头部 -->
            <div
                class="flex items-center gap-2 cursor-pointer hover:bg-yellow-100 rounded-md p-2 transition-colors duration-200 select-none focus-visible:ring-2 focus-visible:ring-yellow-400 focus-visible:outline-none"
                role="button"
                :aria-expanded="expandedArgs[index]"
                aria-label="展开或折叠工具调用参数"
                tabindex="0"
                @click="toggleArgs(index)"
                @keydown.enter="toggleArgs(index)"
                @keydown.space.prevent="toggleArgs(index)"
            >
                <MdBuild
                    class="w-5 h-5 text-yellow-600 flex-shrink-0"
                    aria-hidden="true"
                />
                <span
                    class="font-medium text-sm text-gray-800 flex-1 truncate"
                    >{{ toolCall.function.name }}</span
                >
                <MdExpandMore
                    :class="[
                        'w-5 h-5 text-gray-500 flex-shrink-0 transition-transform duration-300',
                        expandedArgs[index] && 'rotate-180',
                    ]"
                    aria-hidden="true"
                />
            </div>

            <!-- 参数展示区域 -->
            <div
                v-show="expandedArgs[index]"
                class="mt-2 bg-yellow-100 border-t border-yellow-200 rounded-md p-3 max-h-48 overflow-auto transition-all duration-300 ease-in-out"
                role="region"
                aria-label="工具调用参数"
            >
                <pre
                    class="font-mono text-xs text-gray-700 whitespace-pre-wrap break-words"
                    v-html="
                        colorizeJSON(formatJSON(toolCall.function.arguments))
                    "
                ></pre>
            </div>
        </div>

        <!-- 工具执行结果部分 -->
        <div
            v-for="(result, index) in toolResults"
            :key="result.tool_call_id || index"
            class="rounded-md overflow-hidden"
        >
            <!-- 头部 -->
            <div
                :class="[
                    'flex items-center gap-2 cursor-pointer rounded-md p-2 transition-colors duration-200 select-none focus-visible:ring-2 focus-visible:outline-none',
                    isResultSuccess(result)
                        ? 'hover:bg-green-50 focus-visible:ring-green-400'
                        : 'hover:bg-red-50 focus-visible:ring-red-400',
                ]"
                role="button"
                :aria-expanded="expandedResults[index]"
                :aria-label="`展开或折叠工具执行${isResultSuccess(result) ? '成功' : '失败'}结果`"
                tabindex="0"
                @click="toggleResult(index)"
                @keydown.enter="toggleResult(index)"
                @keydown.space.prevent="toggleResult(index)"
            >
                <MdCheckCircle
                    v-if="isResultSuccess(result)"
                    class="w-5 h-5 text-green-600 flex-shrink-0"
                    aria-hidden="true"
                />
                <MdError
                    v-else
                    class="w-5 h-5 text-red-600 flex-shrink-0"
                    aria-hidden="true"
                />
                <span class="font-medium text-sm text-gray-800 flex-1">
                    {{
                        isResultSuccess(result)
                            ? "工具执行成功"
                            : "工具执行失败"
                    }}
                </span>
                <MdExpandMore
                    :class="[
                        'w-5 h-5 text-gray-500 flex-shrink-0 transition-transform duration-300',
                        expandedResults[index] && 'rotate-180',
                    ]"
                    aria-hidden="true"
                />
            </div>

            <!-- 结果展示区域 -->
            <div
                v-show="expandedResults[index]"
                :class="[
                    'mt-2 border-t rounded-md p-3 max-h-48 overflow-auto transition-all duration-300 ease-in-out',
                    isResultSuccess(result)
                        ? 'bg-green-50 border-green-200'
                        : 'bg-red-50 border-red-200',
                ]"
                role="region"
                aria-label="工具执行结果"
            >
                <pre
                    class="text-sm text-gray-700 whitespace-pre-wrap break-words"
                    v-html="colorizeJSON(formatJSON(result.content))"
                ></pre>
            </div>
        </div>

        <!-- 时间戳 -->
        <div v-if="timestamp" class="text-xs mt-2 opacity-70 text-gray-500">
            {{ formatTime(timestamp) }}
        </div>
    </div>
</template>

<script setup lang="ts">
/**
 * ToolExecutionCard 组件
 *
 * 用于显示 AI 工具调用流程的完整卡片，包括：
 * - 工具调用信息（工具名称、参数）
 * - 工具执行结果（成功/失败状态、返回数据）
 *
 * 设计特点：
 * - 黄色主题视觉区分（与普通消息气泡区分）
 * - 可折叠展开的参数和结果区域
 * - 自动 JSON 语法高亮（无需外部依赖）
 * - 根据执行结果显示成功（绿色）或失败（红色）样式
 *
 * @example
 * <ToolExecutionCard
 *   :tool-calls="[{ id: 'call_123', type: 'function', function: { name: 'search', arguments: '{}' } }]"
 *   :tool-results="[{ role: 'tool', content: '{"success": true}', tool_call_id: 'call_123' }]"
 *   :timestamp="new Date()"
 * />
 */
import { ref } from "vue";
import {
    MdBuild,
    MdExpandMore,
    MdCheckCircle,
    MdError,
} from "vue-icons-plus/md";
import type { ToolCall, ChatMessage } from "@/types/api";

/**
 * 工具结果类型
 *
 * 扩展自 ChatMessage，但允许 timestamp 为 Date 或 number 类型
 * 以兼容前端（Date 对象）和后端（Unix 时间戳）的不同格式
 */
interface ToolResult extends Omit<ChatMessage, "timestamp"> {
    /** 消息时间戳，支持 Date 对象或毫秒级 Unix 时间戳 */
    timestamp?: Date | number;
}

/**
 * 组件 Props 定义
 */
interface Props {
    /** 工具调用列表（来自 assistant 消息的 tool_calls 字段） */
    toolCalls: ToolCall[];
    /** 工具执行结果列表（role 为 'tool' 的消息） */
    toolResults: ToolResult[];
    /** 工具调用发起时间 */
    timestamp?: Date;
}

withDefaults(defineProps<Props>(), {
    toolCalls: () => [],
    toolResults: () => [],
});

// ==================== 状态管理 ====================

/** 工具参数区域的折叠/展开状态（索引 -> 是否展开） */
const expandedArgs = ref<Record<number, boolean>>({});

/** 工具结果区域的折叠/展开状态（索引 -> 是否展开） */
const expandedResults = ref<Record<number, boolean>>({});

// ==================== 交互逻辑 ====================

/**
 * 切换指定工具调用的参数展开状态
 * @param index 工具调用在列表中的索引
 */
function toggleArgs(index: number) {
    expandedArgs.value[index] = !expandedArgs.value[index];
}

/**
 * 切换指定工具结果的展开状态
 * @param index 工具结果在列表中的索引
 */
function toggleResult(index: number) {
    expandedResults.value[index] = !expandedResults.value[index];
}

// ==================== 工具函数 ====================

/**
 * 判断工具执行结果是否成功
 *
 * 使用启发式规则：检查内容中是否包含错误关键词
 * - 包含 error/failed/exception -> 失败（红色样式）
 * - 不包含任何错误关键词 -> 成功（绿色样式）
 *
 * @param result 工具结果消息
 * @returns true 表示成功，false 表示失败
 */
function isResultSuccess(result: ToolResult): boolean {
    const content = result.content.toLowerCase();
    return (
        !content.includes("error") &&
        !content.includes("failed") &&
        !content.includes("exception")
    );
}

/**
 * 格式化 JSON 字符串
 *
 * 尝试解析并美化 JSON，解析失败时返回原始字符串
 *
 * @param jsonString 待格式化的 JSON 字符串
 * @returns 格式化后的 JSON（2 空格缩进）或原始字符串
 */
function formatJSON(jsonString: string): string {
    try {
        const parsed = JSON.parse(jsonString);
        return JSON.stringify(parsed, null, 2);
    } catch {
        return jsonString; // 解析失败时返回原始字符串
    }
}

/**
 * 对 JSON 字符串进行语法高亮
 *
 * 使用正则表达式替换实现轻量级语法高亮，无需外部依赖库
 * 支持的着色元素：
 * - 键名（蓝色）
 * - 字符串值（绿色）
 * - 数字（橙色）
 * - 关键字 true/false/null（紫色）
 * - 括号和逗号（灰色）
 *
 * @param json 待着色的 JSON 字符串
 * @returns 包含 HTML span 标签的着色 JSON 字符串
 */
function colorizeJSON(json: string): string {
    return (
        json
            // 1. 键名着色 "key":
            .replace(/"([^"]+)"(\s*:)/g, '<span class="json-key">"$1"</span>$2')
            // 2. 字符串值着色 "value"
            .replace(
                /:\s*"([^"]*)"/g,
                ': <span class="json-string">"$1"</span>',
            )
            // 3. 数字着色
            .replace(
                /:\s*(-?\d+\.?\d*)/g,
                ': <span class="json-number">$1</span>',
            )
            // 4. 关键字着色 true/false/null
            .replace(
                /:\s*(true|false|null)/g,
                ': <span class="json-keyword">$1</span>',
            )
            // 5. 括号着色
            .replace(/([{}[\],])/g, '<span class="json-bracket">$1</span>')
    );
}

/**
 * 格式化时间戳为本地化时间字符串
 *
 * @param date 日期对象
 * @returns 格式化的时间字符串（例如：14:23:56）
 */
function formatTime(date: Date): string {
    return date.toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    });
}
</script>

<style scoped>
/* JSON 着色样式 */
:deep(.json-key) {
    color: #2563eb; /* blue-600 */
    font-weight: 500;
}

:deep(.json-string) {
    color: #16a34a; /* green-600 */
}

:deep(.json-number) {
    color: #ea580c; /* orange-600 */
}

:deep(.json-keyword) {
    color: #9333ea; /* purple-600 */
    font-weight: 500;
}

:deep(.json-bracket) {
    color: #4b5563; /* gray-600 */
}
</style>
