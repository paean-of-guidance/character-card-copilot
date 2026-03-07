import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
  CharacterBook,
  WorldBookEntry,
  CreateWorldBookEntryParams,
  UpdateWorldBookEntryParams,
  WorldBookFilterOptions,
} from '@/types/character';
import {
  getWorldBook,
  initializeWorldBook,
  createWorldBookEntry,
  updateWorldBookEntry,
  deleteWorldBookEntry,
  filterWorldBookEntries,
  batchUpdateEntryEnabled,
  batchDeleteEntries,
  reorderEntries,
} from '@/services/worldBookService';
import { devLog } from '@/utils/logger';

export const useWorldBookStore = defineStore('worldBook', () => {
  // 状态
  const currentCharacterUuid = ref<string | null>(null);
  const worldBook = ref<CharacterBook | null>(null);
  const selectedEntryId = ref<number | null>(null);
  const expandedEntryIds = ref<Set<number>>(new Set());
  const isCreatingNew = ref(false);
  const isLoading = ref(false);

  // 筛选选项
  const filterOptions = ref<WorldBookFilterOptions>({
    searchText: '',
    showEnabled: true,
    showDisabled: true,
    sortBy: 'insertion_order',
    sortOrder: 'asc',
  });

  // 计算属性：筛选后的条目列表
  const filteredEntries = computed<WorldBookEntry[]>(() => {
    if (!worldBook.value || !worldBook.value.entries) {
      return [];
    }

    return filterWorldBookEntries(worldBook.value.entries, filterOptions.value);
  });

  // 计算属性：选中的条目
  const selectedEntry = computed<WorldBookEntry | null>(() => {
    if (selectedEntryId.value === null || !worldBook.value) {
      return null;
    }

    return worldBook.value.entries.find(e => e.id === selectedEntryId.value) || null;
  });

  // 计算属性：统计信息
  const statistics = computed(() => {
    if (!worldBook.value || !worldBook.value.entries) {
      return {
        total: 0,
        enabled: 0,
        disabled: 0,
      };
    }

    const entries = worldBook.value.entries;
    return {
      total: entries.length,
      enabled: entries.filter(e => e.enabled).length,
      disabled: entries.filter(e => !e.enabled).length,
    };
  });

  /**
   * 加载世界书数据
   */
  async function loadWorldBook(characterUuid: string): Promise<void> {
    try {
      isLoading.value = true;
      currentCharacterUuid.value = characterUuid;

      let book = await getWorldBook(characterUuid);

      // 如果世界书不存在，初始化一个
      if (!book) {
        book = await initializeWorldBook(characterUuid);
      }

      worldBook.value = book;
    } catch (error) {
      console.error('加载世界书失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 创建新条目
   */
  async function createEntry(params: CreateWorldBookEntryParams): Promise<WorldBookEntry> {
    if (!currentCharacterUuid.value) {
      throw new Error('未选择角色');
    }

    try {
      isLoading.value = true;
      const newEntry = await createWorldBookEntry(currentCharacterUuid.value, params);

      // 重新加载世界书数据
      await loadWorldBook(currentCharacterUuid.value);

      // 选中新创建的条目
      selectedEntryId.value = newEntry.id || null;
      isCreatingNew.value = false;

      return newEntry;
    } catch (error) {
      console.error('创建条目失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 更新条目
   */
  async function updateEntry(
    entryId: number,
    updates: UpdateWorldBookEntryParams
  ): Promise<void> {
    if (!currentCharacterUuid.value) {
      throw new Error('未选择角色');
    }

    try {
      isLoading.value = true;
      await updateWorldBookEntry(currentCharacterUuid.value, entryId, updates);

      // 重新加载世界书数据
      await loadWorldBook(currentCharacterUuid.value);
    } catch (error) {
      console.error('更新条目失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 删除条目
   */
  async function deleteEntry(entryId: number): Promise<void> {
    if (!currentCharacterUuid.value) {
      throw new Error('未选择角色');
    }

    try {
      isLoading.value = true;
      await deleteWorldBookEntry(currentCharacterUuid.value, entryId);

      // 如果删除的是当前选中的条目，清除选中状态
      if (selectedEntryId.value === entryId) {
        selectedEntryId.value = null;
      }

      // 重新加载世界书数据
      await loadWorldBook(currentCharacterUuid.value);
    } catch (error) {
      console.error('删除条目失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 批量更新条目启用状态
   */
  async function batchUpdateEnabled(entryIds: number[], enabled: boolean): Promise<void> {
    if (!currentCharacterUuid.value) {
      throw new Error('未选择角色');
    }

    try {
      isLoading.value = true;
      await batchUpdateEntryEnabled(currentCharacterUuid.value, entryIds, enabled);

      // 重新加载世界书数据
      await loadWorldBook(currentCharacterUuid.value);
    } catch (error) {
      console.error('批量更新条目状态失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 批量删除条目
   */
  async function batchDelete(entryIds: number[]): Promise<void> {
    if (!currentCharacterUuid.value) {
      throw new Error('未选择角色');
    }

    try {
      isLoading.value = true;
      await batchDeleteEntries(currentCharacterUuid.value, entryIds);

      // 清除选中状态
      if (selectedEntryId.value && entryIds.includes(selectedEntryId.value)) {
        selectedEntryId.value = null;
      }

      // 重新加载世界书数据
      await loadWorldBook(currentCharacterUuid.value);
    } catch (error) {
      console.error('批量删除条目失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 重新排序条目
   */
  async function reorder(entryIds: number[]): Promise<void> {
    if (!currentCharacterUuid.value) {
      throw new Error('未选择角色');
    }

    try {
      isLoading.value = true;
      await reorderEntries(currentCharacterUuid.value, entryIds);

      // 重新加载世界书数据
      await loadWorldBook(currentCharacterUuid.value);
    } catch (error) {
      console.error('重新排序条目失败:', error);
      throw error;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 切换条目展开状态
   */
  function toggleEntryExpanded(entryId: number): void {
    if (expandedEntryIds.value.has(entryId)) {
      expandedEntryIds.value.delete(entryId);
    } else {
      expandedEntryIds.value.add(entryId);
    }
  }

  /**
   * 选中条目
   */
  function selectEntry(entryId: number | null): void {
    devLog('🎯 worldBookStore.selectEntry called with entryId:', entryId);
    selectedEntryId.value = entryId;
    isCreatingNew.value = false;

    // 立即查找并输出选中的条目
    const entry = worldBook.value?.entries.find(e => e.id === entryId);
    devLog('  - Found entry:', entry);
    devLog('  - selectedEntry computed will be:', selectedEntry.value);
  }

  /**
   * 开始创建新条目
   */
  function startCreatingNew(): void {
    isCreatingNew.value = true;
    selectedEntryId.value = null;
  }

  /**
   * 取消创建新条目
   */
  function cancelCreatingNew(): void {
    isCreatingNew.value = false;
  }

  /**
   * 更新筛选选项
   */
  function updateFilterOptions(options: Partial<WorldBookFilterOptions>): void {
    filterOptions.value = {
      ...filterOptions.value,
      ...options,
    };
  }

  /**
   * 重置筛选选项
   */
  function resetFilterOptions(): void {
    filterOptions.value = {
      searchText: '',
      showEnabled: true,
      showDisabled: true,
      sortBy: 'insertion_order',
      sortOrder: 'asc',
    };
  }

  /**
   * 清除状态
   */
  function clearState(): void {
    currentCharacterUuid.value = null;
    worldBook.value = null;
    selectedEntryId.value = null;
    expandedEntryIds.value.clear();
    isCreatingNew.value = false;
    resetFilterOptions();
  }

  return {
    // 状态
    currentCharacterUuid,
    worldBook,
    selectedEntryId,
    expandedEntryIds,
    isCreatingNew,
    isLoading,
    filterOptions,

    // 计算属性
    filteredEntries,
    selectedEntry,
    statistics,

    // 方法
    loadWorldBook,
    createEntry,
    updateEntry,
    deleteEntry,
    batchUpdateEnabled,
    batchDelete,
    reorder,
    toggleEntryExpanded,
    selectEntry,
    startCreatingNew,
    cancelCreatingNew,
    updateFilterOptions,
    resetFilterOptions,
    clearState,
  };
});
