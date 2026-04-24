<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { MdOutlineDelete } from "vue-icons-plus/md";
import type { CommandMetadata } from "@/types/commands";

// 组件Props
const props = defineProps<{
    /** 是否显示命令面板 */
    visible: boolean;
    /** 可用的命令列表 */
    commands: CommandMetadata[];
    /** 当前搜索关键字 */
    searchQuery?: string;
}>();

// 组件Emits
const emits = defineEmits<{
    /** 选择命令 */
    select: [command: CommandMetadata];
    /** 关闭命令面板 */
    close: [];
    /** 更新选中索引 */
    "update:selectedIndex": [index: number];
}>();

// 状态管理
const selectedIndex = ref(0);
const commandListRef = ref<HTMLElement>();

// 计算属性：过滤后的命令列表
const filteredCommands = computed(() => {
    if (!props.searchQuery) {
        // 默认最多显示4条命令
        return props.commands.slice(0, 4);
    }

    // 根据搜索关键字过滤（由父组件CommandService处理）
    return props.commands;
});

// 计算属性：是否有可用命令
const hasCommands = computed(() => filteredCommands.value.length > 0);

// 监听命令列表变化，重置选中索引
watch(
    () => filteredCommands.value.length,
    (newLength) => {
        if (selectedIndex.value >= newLength) {
            selectedIndex.value = Math.max(0, newLength - 1);
        }
    },
);

// 监听visible变化，重置选中索引
watch(
    () => props.visible,
    (visible) => {
        if (visible) {
            selectedIndex.value = 0;
            // 自动滚动到顶部
            nextTick(() => {
                if (commandListRef.value) {
                    commandListRef.value.scrollTop = 0;
                }
            });
        }
    },
);

// 监听选中索引变化，自动滚动到可见区域
watch(selectedIndex, (newIndex) => {
    nextTick(() => {
        scrollToSelected(newIndex);
    });
});

/**
 * 向上导航
 */
function navigateUp() {
    if (selectedIndex.value > 0) {
        selectedIndex.value--;
    } else {
        // 循环到最后一项
        selectedIndex.value = filteredCommands.value.length - 1;
    }
}

/**
 * 向下导航
 */
function navigateDown() {
    if (selectedIndex.value < filteredCommands.value.length - 1) {
        selectedIndex.value++;
    } else {
        // 循环到第一项
        selectedIndex.value = 0;
    }
}

/**
 * 选择当前命令
 */
function selectCurrent() {
    if (hasCommands.value && selectedIndex.value >= 0) {
        const command = filteredCommands.value[selectedIndex.value];
        if (command) {
            emits("select", command);
        }
    }
}

/**
 * 关闭命令面板
 */
function close() {
    emits("close");
}

/**
 * 处理键盘事件
 */
function handleKeydown(event: KeyboardEvent) {
    switch (event.key) {
        case "ArrowUp":
            event.preventDefault();
            navigateUp();
            break;
        case "ArrowDown":
            event.preventDefault();
            navigateDown();
            break;
        case "Enter":
        case "Tab":
        case " ":
            event.preventDefault();
            selectCurrent();
            break;
        case "Escape":
            event.preventDefault();
            close();
            break;
    }
}

/**
 * 滚动到选中项
 */
function scrollToSelected(index: number) {
    if (!commandListRef.value) return;

    const container = commandListRef.value;
    const items = container.children;
    if (index < 0 || index >= items.length) return;

    const selectedItem = items[index] as HTMLElement;
    const containerRect = container.getBoundingClientRect();
    const itemRect = selectedItem.getBoundingClientRect();

    // 计算是否需要滚动
    const isAbove = itemRect.top < containerRect.top;
    const isBelow = itemRect.bottom > containerRect.bottom;

    if (isAbove) {
        // 滚动到顶部
        container.scrollTop = selectedItem.offsetTop;
    } else if (isBelow) {
        // 滚动到底部
        container.scrollTop =
            selectedItem.offsetTop -
            container.clientHeight +
            selectedItem.clientHeight;
    }
}

/**
 * 点击命令项
 */
function handleCommandClick(command: CommandMetadata, index: number) {
    selectedIndex.value = index;
    emits("select", command);
}

/**
 * 鼠标移入命令项
 */
function handleCommandMouseEnter(index: number) {
    selectedIndex.value = index;
}

/**
 * 获取命令图标组件
 */
function getCommandIcon(command: CommandMetadata) {
    // 目前只支持MdOutlineDelete图标
    // 未来可以扩展支持更多图标
    if (command.icon === "MdOutlineDelete") {
        return MdOutlineDelete;
    }
    return null;
}

// 暴露方法给父组件
defineExpose({
    handleKeydown,
    navigateUp,
    navigateDown,
    selectCurrent,
    close,
});
</script>

<template>
    <!-- 命令面板容器 -->
    <Transition name="command-palette" @before-enter="selectedIndex = 0">
        <div
            v-if="visible"
            class="liquid-modal absolute bottom-full left-0 right-0 mb-3 z-50 min-h-[120px] overflow-hidden"
            role="listbox"
            aria-label="命令面板"
            :aria-activedescendant="`command-${selectedIndex}`"
            aria-expanded="true"
        >
            <div ref="commandListRef" class="max-h-64 overflow-y-auto" role="presentation">
                <div
                    v-for="(command, index) in filteredCommands"
                    :key="command.id"
                    :id="`command-${index}`"
                    class="flex cursor-pointer items-center gap-3 border-l-2 px-4 py-3 transition-all duration-150 ease-in-out"
                    :class="{
                        'border-l-violet-400 bg-violet-500/15': index === selectedIndex,
                        'border-transparent hover:border-l-white/20 hover:bg-white/6': index !== selectedIndex,
                    }"
                    role="option"
                    :aria-selected="index === selectedIndex"
                    @click="handleCommandClick(command, index)"
                    @mouseenter="handleCommandMouseEnter(index)"
                >
                    <div v-if="command.icon" class="h-5 w-5 shrink-0 text-white/40">
                        <component :is="getCommandIcon(command)" />
                    </div>
                    <div class="min-w-0 flex-1">
                        <div class="flex items-baseline">
                            <span class="font-mono text-sm font-medium text-white/85">{{ command.name }}</span>
                            <span class="ml-2 text-xs text-white/40">{{ command.description }}</span>
                        </div>
                    </div>
                </div>

                <div v-if="!hasCommands" class="flex flex-col items-center justify-center py-8 text-white/40">
                    <div class="mb-2 text-3xl">{{ searchQuery ? "🔍" : "💬" }}</div>
                    <p class="text-sm font-medium">{{ searchQuery ? "未找到匹配的命令" : "暂无可用命令" }}</p>
                    <p class="mt-1 text-xs text-white/25">{{ searchQuery ? "尝试其他关键词" : "当前没有可执行的命令" }}</p>
                </div>
            </div>
        </div>
    </Transition>
</template>

<style scoped>
/* 面板动画 */
.command-palette-enter-active,
.command-palette-leave-active {
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.command-palette-enter-from {
    opacity: 0;
    transform: translateY(10px);
}

.command-palette-leave-to {
    opacity: 0;
    transform: translateY(10px);
}

/* 隐藏滚动条 */
.overflow-y-auto::-webkit-scrollbar {
    display: none;
}

.overflow-y-auto {
    scrollbar-width: none;
    -ms-overflow-style: none;
}
</style>
