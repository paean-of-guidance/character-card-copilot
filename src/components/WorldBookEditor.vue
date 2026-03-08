<template>
  <div class="h-full">
    <!-- 世界书编辑器 -->
    <div class="h-full flex flex-col rounded-[20px] border border-white/70 bg-white/85 p-4 shadow-[0_8px_24px_rgba(148,163,184,0.12)] backdrop-blur overflow-hidden">
      <!-- 标题区 -->
      <div class="mb-4">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-lg font-semibold text-slate-900">世界书编辑器</h2>
            <p class="text-xs text-slate-400 mt-0.5">管理角色的知识库条目</p>
          </div>
          <!-- 统计 chips -->
          <div class="flex items-center gap-2">
            <span class="inline-flex items-center rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-medium text-slate-600">
              {{ worldBookStore.statistics.total }} 条目
            </span>
            <span class="inline-flex items-center rounded-full bg-green-50 px-2.5 py-0.5 text-xs font-medium text-green-700">
              {{ worldBookStore.statistics.enabled }} 启用
            </span>
            <span
              v-if="worldBookStore.filteredEntries.length !== worldBookStore.statistics.total"
              class="inline-flex items-center rounded-full bg-blue-50 px-2.5 py-0.5 text-xs font-medium text-blue-700"
            >
              {{ worldBookStore.filteredEntries.length }} 筛选
            </span>
          </div>
        </div>
      </div>

      <!-- 搜索和筛选组件 -->
      <WorldBookSearch
        :statistics="worldBookStore.statistics"
        :filtered-count="worldBookStore.filteredEntries.length"
        @search="handleSearch"
        @filter="handleFilter"
        @create="handleCreateNew"
      />

      <!-- 条目列表 -->
      <div class="flex-1 overflow-y-auto mt-4">
        <div v-if="worldBookStore.isLoading" class="flex items-center justify-center h-32">
          <div class="text-slate-400 text-sm">加载中...</div>
        </div>

        <!-- 空态 -->
        <div v-else-if="worldBookStore.filteredEntries.length === 0" class="flex flex-col items-center justify-center py-16 text-center">
          <div class="w-16 h-16 rounded-2xl bg-slate-100 flex items-center justify-center mb-4">
            <svg class="w-8 h-8 text-slate-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          </div>
          <p class="text-sm font-medium text-slate-500 mb-1">暂无条目</p>
          <p class="text-xs text-slate-400 mb-4">创建世界书条目来丰富角色的知识库</p>
          <button
            class="glass-btn glass-btn--primary"
            @click="handleCreateNew"
          >
            创建第一个条目
          </button>
        </div>

        <div v-else class="space-y-2">
          <WorldBookEntry
            v-for="entry in worldBookStore.filteredEntries"
            :key="entry.id"
            :entry="entry"
            :is-expanded="worldBookStore.expandedEntryIds.has(entry.id || 0)"
            @toggle="worldBookStore.toggleEntryExpanded(entry.id || 0)"
            @edit="handleEdit"
            @delete="handleDelete"
          />
        </div>
      </div>
    </div>

    <!-- 编辑模态框 -->
    <WorldBookEditorModal
      :visible="showEditor"
      :entry="worldBookStore.selectedEntry"
      :is-creating-new="worldBookStore.isCreatingNew"
      @close="handleCancel"
      @save="handleSave"
      @delete="handleDeleteFromEditor"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { useWorldBookStore } from '@/stores/worldBook';
import { useNotification } from '@/composables/useNotification';
import { useModal } from '@/composables/useModal';
import WorldBookSearch from './WorldBookSearch.vue';
import WorldBookEntry from './WorldBookEntry.vue';
import WorldBookEditorModal from './WorldBookEditorModal.vue';
import type { CreateWorldBookEntryParams, UpdateWorldBookEntryParams } from '@/types/character';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { devLog } from '@/utils/logger';

interface Props {
  characterUuid: string;
}

const props = defineProps<Props>();

// Store
const worldBookStore = useWorldBookStore();

// Notification & Modal
const { showSuccessToast, showErrorToast } = useNotification();
const { showAlertModal } = useModal();

// 计算属性
const showEditor = computed(() => {
  return worldBookStore.selectedEntryId !== null || worldBookStore.isCreatingNew;
});

// 事件监听器清理函数
let unlistenWorldBookCreated: UnlistenFn | null = null;
let unlistenWorldBookDeleted: UnlistenFn | null = null;
let unlistenToolExecuted: UnlistenFn | null = null;

// 生命周期
onMounted(async () => {
  await worldBookStore.loadWorldBook(props.characterUuid);

  unlistenWorldBookCreated = await listen('world-book-entry-created', async (event) => {
    devLog('📚 收到世界书条目创建事件:', event.payload);
    const payload = event.payload as { character_uuid: string; entry_id: number; entry_name?: string; keys: string[] };
    if (payload.character_uuid === props.characterUuid) {
      devLog('✅ 刷新世界书数据...');
      await worldBookStore.loadWorldBook(props.characterUuid);
    }
  });

  unlistenWorldBookDeleted = await listen('world-book-entry-deleted', async (event) => {
    devLog('🗑️ 收到世界书条目删除事件:', event.payload);
    const payload = event.payload as { character_uuid: string; entry_id: number; entry_name?: string; keys: string[] };
    if (payload.character_uuid === props.characterUuid) {
      devLog('✅ 刷新世界书数据...');
      await worldBookStore.loadWorldBook(props.characterUuid);
    }
  });

  unlistenToolExecuted = await listen('tool-executed', (event) => {
    devLog('🔧 收到工具执行事件:', event.payload);
    const payload = event.payload as { tool_name: string; character_uuid?: string };
    if (payload.tool_name === 'create_world_book_entry' &&
        payload.character_uuid === props.characterUuid) {
      devLog('✅ 工具执行成功，数据已刷新');
    }
  });
});

onUnmounted(() => {
  if (unlistenWorldBookCreated) unlistenWorldBookCreated();
  if (unlistenWorldBookDeleted) unlistenWorldBookDeleted();
  if (unlistenToolExecuted) unlistenToolExecuted();
});

// 事件处理
function handleSearch(searchText: string): void {
  worldBookStore.updateFilterOptions({ searchText });
}

function handleFilter(options: {
  showEnabled: boolean;
  showDisabled: boolean;
  sortBy: 'insertion_order' | 'priority' | 'name';
  sortOrder: 'asc' | 'desc';
}): void {
  worldBookStore.updateFilterOptions(options);
}

function handleCreateNew(): void {
  worldBookStore.startCreatingNew();
}

function handleEdit(entryId: number | undefined): void {
  devLog('🖊️ WorldBookEditor.handleEdit called with entryId:', entryId);
  if (entryId !== undefined) {
    worldBookStore.selectEntry(entryId);
  }
}

async function handleDelete(entryId: number | undefined): Promise<void> {
  if (entryId === undefined) return;

  const confirmed = await showAlertModal(
    '确定要删除这个条目吗？此操作不可撤销。',
    undefined,
    { title: '删除确认', type: 'danger', confirmText: '确认删除', cancelText: '取消' }
  );

  if (confirmed) {
    try {
      await worldBookStore.deleteEntry(entryId);
      showSuccessToast('条目已删除', '删除成功');
    } catch (error) {
      showErrorToast('删除失败: ' + error, '删除失败');
    }
  }
}

async function handleSave(data: CreateWorldBookEntryParams | UpdateWorldBookEntryParams): Promise<void> {
  try {
    if (worldBookStore.isCreatingNew) {
      await worldBookStore.createEntry(data as CreateWorldBookEntryParams);
      showSuccessToast('条目创建成功！', '创建成功');
    } else if (worldBookStore.selectedEntryId !== null) {
      await worldBookStore.updateEntry(worldBookStore.selectedEntryId, data);
      showSuccessToast('条目更新成功！', '更新成功');
    }
  } catch (error) {
    showErrorToast('保存失败: ' + error, '保存失败');
  }
}

function handleCancel(): void {
  worldBookStore.selectEntry(null);
  worldBookStore.cancelCreatingNew();
}

async function handleDeleteFromEditor(): Promise<void> {
  if (worldBookStore.selectedEntryId === null) return;

  const confirmed = await showAlertModal(
    '确定要删除这个条目吗？此操作不可撤销。',
    undefined,
    { title: '删除确认', type: 'danger', confirmText: '确认删除', cancelText: '取消' }
  );

  if (confirmed) {
    try {
      await worldBookStore.deleteEntry(worldBookStore.selectedEntryId);
      showSuccessToast('条目删除成功！', '删除成功');
    } catch (error) {
      showErrorToast('删除失败: ' + error, '删除失败');
    }
  }
}
</script>
