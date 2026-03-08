<template>
    <div class="space-y-3">
        <!-- 搜索框和操作按钮 -->
        <div class="flex gap-2 rounded-2xl border border-white/60 bg-white/70 p-3 shadow-[0_4px_12px_rgba(148,163,184,0.08)] backdrop-blur">
            <!-- 搜索输入框 -->
            <div class="flex-1 relative">
                <input
                    v-model="searchText"
                    type="text"
                    class="w-full rounded-xl border border-slate-200/80 bg-white/90 px-4 py-2.5 pl-10 text-sm text-slate-800 placeholder-slate-300 focus:outline-none focus:border-blue-300 focus:ring-2 focus:ring-blue-500/10 transition-all"
                    placeholder="搜索条目（名称、关键词、内容、备注）"
                    @input="handleSearch"
                />
                <svg
                    class="w-4 h-4 text-slate-300 absolute left-3.5 top-1/2 transform -translate-y-1/2"
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
            <div class="relative flex">
                <button
                    class="rounded-xl border border-slate-200/80 bg-white/90 px-3 py-2 text-sm text-slate-600 hover:bg-slate-50 transition-colors flex items-center gap-1.5"
                    @click="showFilters = !showFilters"
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
                            d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                        />
                    </svg>
                    筛选
                </button>

                <!-- 筛选下拉菜单 -->
                <div
                    v-if="showFilters"
                    class="absolute right-0 top-full mt-2 w-64 rounded-2xl border border-white/70 bg-white/95 p-4 shadow-[0_12px_32px_rgba(148,163,184,0.20)] backdrop-blur-xl space-y-3 z-10"
                >
                    <div>
                        <label class="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2 block">显示状态</label>
                        <div class="space-y-2">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    v-model="filterEnabled"
                                    type="checkbox"
                                    class="w-4 h-4 text-blue-600 border-slate-300 rounded focus:ring-blue-500"
                                />
                                <span class="text-sm text-slate-600">显示启用的条目</span>
                            </label>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    v-model="filterDisabled"
                                    type="checkbox"
                                    class="w-4 h-4 text-blue-600 border-slate-300 rounded focus:ring-blue-500"
                                />
                                <span class="text-sm text-slate-600">显示禁用的条目</span>
                            </label>
                        </div>
                    </div>

                    <div class="border-t border-slate-100 pt-3">
                        <label class="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2 block">排序方式</label>
                        <select
                            v-model="sortBy"
                            class="w-full rounded-xl border border-slate-200/80 bg-white/90 px-3 py-2 text-sm text-slate-700 focus:outline-none focus:ring-2 focus:ring-blue-500/10"
                            @change="handleSortChange"
                        >
                            <option value="insertion_order">插入顺序</option>
                            <option value="priority">优先级</option>
                            <option value="name">名称</option>
                        </select>
                    </div>

                    <div>
                        <label class="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2 block">排序方向</label>
                        <select
                            v-model="sortOrder"
                            class="w-full rounded-xl border border-slate-200/80 bg-white/90 px-3 py-2 text-sm text-slate-700 focus:outline-none focus:ring-2 focus:ring-blue-500/10"
                            @change="handleSortChange"
                        >
                            <option value="asc">升序</option>
                            <option value="desc">降序</option>
                        </select>
                    </div>

                    <div class="border-t border-slate-100 pt-3">
                        <button
                            class="w-full rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium py-2 px-3 text-sm transition-colors"
                            @click="resetFilters"
                        >
                            重置筛选
                        </button>
                    </div>
                </div>
            </div>

            <!-- 新建条目按钮 -->
            <button
                class="glass-btn glass-btn--primary"
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
                新建
            </button>
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
