<template>
    <div class="space-y-3">
        <!-- 搜索框和操作按钮 -->
        <div class="flex gap-2">
            <!-- 搜索输入框 -->
            <div class="flex-1 relative">
                <input
                    v-model="searchText"
                    type="text"
                    class="w-full border border-gray-200 rounded-lg px-4 py-3 pl-10 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    placeholder="搜索条目（支持搜索名称、关键词、内容、备注）"
                    @input="handleSearch"
                />
                <svg
                    class="w-5 h-5 text-gray-400 absolute left-3 top-1/2 transform -translate-y-1/2"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    />
                </svg>
            </div>

            <!-- 筛选按钮 -->
            <div class="relative flex py-1">
                <button
                    class="bg-gray-200 hover:bg-gray-300 text-gray-700 text-sm font-medium py-2 px-3 rounded-lg flex items-center gap-1.5"
                    @click="showFilters = !showFilters"
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
                            d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                        />
                    </svg>
                    筛选
                </button>

                <!-- 筛选下拉菜单 -->
                <div
                    v-if="showFilters"
                    class="absolute right-0 mt-2 w-64 bg-white rounded-lg shadow-xl border border-gray-200 p-4 space-y-3 z-10"
                >
                    <div>
                        <label
                            class="text-sm font-semibold text-gray-700 mb-2 block"
                            >显示状态</label
                        >
                        <div class="space-y-2">
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    v-model="filterEnabled"
                                    type="checkbox"
                                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                                />
                                <span class="text-sm text-gray-700"
                                    >显示启用的条目</span
                                >
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    v-model="filterDisabled"
                                    type="checkbox"
                                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                                />
                                <span class="text-sm text-gray-700"
                                    >显示禁用的条目</span
                                >
                            </label>
                        </div>
                    </div>

                    <div class="border-t border-gray-200 pt-3">
                        <label
                            class="text-sm font-semibold text-gray-700 mb-2 block"
                            >排序方式</label
                        >
                        <select
                            v-model="sortBy"
                            class="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            @change="handleSortChange"
                        >
                            <option value="insertion_order">插入顺序</option>
                            <option value="priority">优先级</option>
                            <option value="name">名称</option>
                        </select>
                    </div>

                    <div>
                        <label
                            class="text-sm font-semibold text-gray-700 mb-2 block"
                            >排序方向</label
                        >
                        <select
                            v-model="sortOrder"
                            class="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            @change="handleSortChange"
                        >
                            <option value="asc">升序</option>
                            <option value="desc">降序</option>
                        </select>
                    </div>

                    <div class="border-t border-gray-200 pt-3">
                        <button
                            class="w-full bg-gray-200 hover:bg-gray-300 text-gray-700 font-medium py-1.5 px-3 rounded-lg text-sm"
                            @click="resetFilters"
                        >
                            重置筛选
                        </button>
                    </div>
                </div>
            </div>

            <!-- 新建条目按钮 -->
            <button
                class="bg-blue-500 hover:bg-blue-700 text-white text-sm font-medium my-1 px-3 rounded-full flex items-center gap-1.5"
                @click="$emit('create')"
            >
                <svg
                    class="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 4v16m8-8H4"
                    />
                </svg>
                新建条目
            </button>
        </div>

        <!-- 统计信息 -->
        <div class="bg-gray-100 rounded-lg p-4">
            <div class="flex items-center justify-between text-sm">
                <div class="flex items-center gap-4">
                    <span class="text-gray-600">
                        共
                        <span class="font-bold text-gray-900">{{
                            statistics.total
                        }}</span>
                        个条目
                    </span>
                    <span class="text-green-600">
                        启用:
                        <span class="font-bold">{{ statistics.enabled }}</span>
                    </span>
                    <span class="text-gray-500">
                        禁用:
                        <span class="font-bold">{{ statistics.disabled }}</span>
                    </span>
                </div>
                <div
                    v-if="filteredCount !== statistics.total"
                    class="text-gray-600"
                >
                    筛选结果:
                    <span class="font-bold text-blue-600">{{
                        filteredCount
                    }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";

interface Props {
    statistics: {
        total: number;
        enabled: number;
        disabled: number;
    };
    filteredCount: number;
}

interface Emits {
    (e: "search", searchText: string): void;
    (
        e: "filter",
        options: {
            showEnabled: boolean;
            showDisabled: boolean;
            sortBy: "insertion_order" | "priority" | "name";
            sortOrder: "asc" | "desc";
        },
    ): void;
    (e: "create"): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

// 状态
const searchText = ref("");
const showFilters = ref(false);
const filterEnabled = ref(true);
const filterDisabled = ref(true);
const sortBy = ref<"insertion_order" | "priority" | "name">("insertion_order");
const sortOrder = ref<"asc" | "desc">("asc");

// 搜索防抖定时器
let searchDebounceTimer: number | null = null;

function handleSearch(): void {
    if (searchDebounceTimer) {
        clearTimeout(searchDebounceTimer);
    }

    searchDebounceTimer = window.setTimeout(() => {
        emit("search", searchText.value);
    }, 300);
}

function handleSortChange(): void {
    emitFilter();
}

function emitFilter(): void {
    emit("filter", {
        showEnabled: filterEnabled.value,
        showDisabled: filterDisabled.value,
        sortBy: sortBy.value,
        sortOrder: sortOrder.value,
    });
}

function resetFilters(): void {
    searchText.value = "";
    filterEnabled.value = true;
    filterDisabled.value = true;
    sortBy.value = "insertion_order";
    sortOrder.value = "asc";

    emit("search", "");
    emitFilter();
}

// 监听筛选选项变化
watch([filterEnabled, filterDisabled], () => {
    emitFilter();
});

// 点击外部关闭筛选面板
document.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (!target.closest(".relative")) {
        showFilters.value = false;
    }
});
</script>
