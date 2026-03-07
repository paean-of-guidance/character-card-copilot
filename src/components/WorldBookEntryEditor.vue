<template>
  <div class="h-full flex flex-col bg-white rounded-xl shadow-2xl">
    <!-- 编辑器头部 -->
    <div class="px-3 py-2 border-b border-gray-200">
      <h3 class="text-lg font-semibold text-gray-900">
        {{ isCreatingNew ? '新建条目' : '编辑条目' }}
      </h3>
    </div>

    <!-- 编辑表单 -->
    <div class="flex-1 overflow-y-auto px-3 py-2 space-y-3">
      <!-- 条目名称 -->
      <div>
        <label class="text-sm font-semibold text-gray-700 mb-2 block">
          条目名称
          <span class="text-xs font-normal text-gray-500 ml-1">(可选)</span>
        </label>
        <input
          v-model="formData.name"
          type="text"
          class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="条目的简短名称"
        />
      </div>

      <!-- 关键词 -->
      <div>
        <label class="text-sm font-semibold text-gray-700 mb-2 block">
          关键词 <span class="text-red-500">*</span>
        </label>
        <div class="space-y-2">
          <div class="flex flex-wrap gap-2 mb-2">
            <span
              v-for="(key, index) in formData.keys"
              :key="index"
              class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full text-sm flex items-center gap-1"
            >
              {{ key }}
              <button
                type="button"
                class="text-blue-600 hover:text-blue-800"
                @click="removeKey(index)"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </span>
          </div>
          <div class="flex gap-2">
            <input
              v-model="newKey"
              type="text"
              class="flex-1 border border-gray-200 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="输入关键词后按回车添加"
              @keypress.enter.prevent="addKey"
            />
            <button
              type="button"
              class="bg-blue-500 hover:bg-blue-700 text-white text-sm font-medium py-1.5 px-3 rounded-full"
              @click="addKey"
            >
              添加
            </button>
          </div>
        </div>
      </div>

      <!-- 内容 -->
      <div>
        <label class="text-sm font-semibold text-gray-700 mb-2 block">
          内容 <span class="text-red-500">*</span>
        </label>
        <textarea
          v-model="formData.content"
          rows="6"
          class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="当关键词被触发时插入的内容"
        ></textarea>
      </div>

      <!-- 备注 -->
      <div>
        <label class="text-sm font-semibold text-gray-700 mb-2 block">
          备注
          <span class="text-xs font-normal text-gray-500 ml-1">(可选)</span>
        </label>
        <textarea
          v-model="formData.comment"
          rows="2"
          class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="关于这个条目的备注，不会影响提示词"
        ></textarea>
      </div>

      <!-- 高级设置 -->
      <div class="pt-2 border-t border-gray-200">
        <button
          type="button"
          class="flex items-center gap-2 text-sm font-semibold text-gray-700 mb-3"
          @click="showAdvanced = !showAdvanced"
        >
          <svg
            class="w-4 h-4 transition-transform"
            :class="{ 'rotate-90': showAdvanced }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
          高级设置
        </button>

        <div v-if="showAdvanced" class="space-y-4">
          <!-- 次要关键词 -->
          <div>
            <label class="text-sm font-semibold text-gray-700 mb-2 block">
              次要关键词
              <span class="text-xs font-normal text-gray-500 ml-1">(选择性触发时使用)</span>
            </label>
            <div class="space-y-2">
              <div class="flex flex-wrap gap-2 mb-2">
                <span
                  v-for="(key, index) in formData.secondary_keys"
                  :key="index"
                  class="px-3 py-1 bg-purple-100 text-purple-800 rounded-full text-sm flex items-center gap-1"
                >
                  {{ key }}
                  <button
                    type="button"
                    class="text-purple-600 hover:text-purple-800"
                    @click="removeSecondaryKey(index)"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </span>
              </div>
              <div class="flex gap-2">
                <input
                  v-model="newSecondaryKey"
                  type="text"
                  class="flex-1 border border-gray-200 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="输入次要关键词后按回车添加"
                  @keypress.enter.prevent="addSecondaryKey"
                />
                <button
                  type="button"
                  class="bg-purple-500 hover:bg-purple-700 text-white text-sm font-medium py-1.5 px-3 rounded-full"
                  @click="addSecondaryKey"
                >
                  添加
                </button>
              </div>
            </div>
          </div>

          <!-- 优先级 -->
          <div>
            <label class="text-sm font-semibold text-gray-700 mb-2 block">
              优先级
              <span class="text-xs font-normal text-gray-500 ml-1">(Token预算不足时，低优先级条目会被丢弃)</span>
            </label>
            <input
              v-model.number="formData.priority"
              type="number"
              min="0"
              max="100"
              class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <!-- 插入深度 -->
          <div>
            <label class="text-sm font-semibold text-gray-700 mb-2 block">
              插入深度
              <span class="text-xs font-normal text-gray-500 ml-1">(扫描上下文的深度)</span>
            </label>
            <input
              v-model.number="extensionsDepth"
              type="number"
              min="0"
              max="100"
              class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <!-- 概率 -->
          <div>
            <label class="text-sm font-semibold text-gray-700 mb-2 block">
              概率
              <span class="text-xs font-normal text-gray-500 ml-1">(触发概率 0-100%)</span>
            </label>
            <input
              v-model.number="extensionsProbability"
              type="number"
              min="0"
              max="100"
              class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <!-- 插入位置 -->
          <div>
            <label class="text-sm font-semibold text-gray-700 mb-2 block">插入位置</label>
            <select
              v-model="formData.position"
              class="w-full border border-gray-200 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="before_char">角色定义之前</option>
              <option value="after_char">角色定义之后</option>
            </select>
          </div>

          <!-- 开关选项 -->
          <div class="space-y-3">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="formData.enabled"
                type="checkbox"
                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <span class="text-sm font-medium text-gray-700">启用此条目</span>
            </label>

            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="formData.case_sensitive"
                type="checkbox"
                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <span class="text-sm font-medium text-gray-700">大小写敏感</span>
            </label>

            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="formData.selective"
                type="checkbox"
                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <span class="text-sm font-medium text-gray-700">选择性触发（需要同时匹配主要和次要关键词）</span>
            </label>

            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="formData.constant"
                type="checkbox"
                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
              />
              <span class="text-sm font-medium text-gray-700">常驻条目（总是插入，不受关键词限制）</span>
            </label>
          </div>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="px-3 py-2 border-t border-gray-200 flex gap-2">
      <button
        class="flex-1 bg-blue-500 hover:bg-blue-700 text-white text-sm font-medium py-1.5 px-3 rounded-full"
        @click="handleSave"
      >
        {{ isCreatingNew ? '创建' : '保存' }}
      </button>
      <button
        class="flex-1 bg-gray-200 hover:bg-gray-300 text-gray-700 text-sm font-medium py-1.5 px-3 rounded-full"
        @click="handleCancel"
      >
        取消
      </button>
      <button
        v-if="!isCreatingNew"
        class="bg-red-500 hover:bg-red-700 text-white text-sm font-medium py-1.5 px-3 rounded-full"
        @click="handleDelete"
      >
        删除
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useNotification } from '@/composables/useNotification';
import { useModal } from '@/composables/useModal';
import type { WorldBookEntry, CreateWorldBookEntryParams, UpdateWorldBookEntryParams } from '@/types/character';
import { devLog } from '@/utils/logger';

interface Props {
  entry?: WorldBookEntry | null;
  isCreatingNew?: boolean;
}

interface Emits {
  (e: 'save', data: CreateWorldBookEntryParams | UpdateWorldBookEntryParams): void;
  (e: 'cancel'): void;
  (e: 'delete'): void;
}

const props = withDefaults(defineProps<Props>(), {
  entry: null,
  isCreatingNew: false,
});

const emit = defineEmits<Emits>();

// Notification & Modal
const { showWarningToast } = useNotification();
const { showAlertModal } = useModal();

// 表单数据的默认值
const getDefaultFormData = () => ({
  name: '',
  keys: [],
  content: '',
  comment: '',
  enabled: true,
  priority: 10,
  position: 'before_char' as 'before_char' | 'after_char',
  case_sensitive: false,
  selective: false,
  secondary_keys: [],
  constant: false,
});

// 表单数据
const formData = ref<CreateWorldBookEntryParams & UpdateWorldBookEntryParams>(getDefaultFormData());

// UI状态
const showAdvanced = ref(false);
const newKey = ref('');
const newSecondaryKey = ref('');

// Extensions 字段
const extensionsDepth = ref(5);
const extensionsProbability = ref(100);

// 监听entry和isCreatingNew变化，更新表单数据
watch([() => props.entry, () => props.isCreatingNew], ([entry, creating]) => {
  devLog('🔍 WorldBookEntryEditor watch triggered:');
  devLog('  - entry:', entry);
  devLog('  - isCreatingNew:', creating);
  devLog('  - props.entry:', props.entry);
  devLog('  - props.isCreatingNew:', props.isCreatingNew);

  if (creating) {
    devLog('✏️ 创建新条目模式');
    // 创建新条目：重置表单为默认值
    formData.value = getDefaultFormData();
    extensionsDepth.value = 5;
    extensionsProbability.value = 100;
  } else if (entry) {
    devLog('📝 编辑现有条目模式');
    devLog('  - entry data:', JSON.stringify(entry, null, 2));
    // 编辑现有条目：保留所有原始字段，包括 extensions、id、insertion_order 等
    formData.value = {
      ...entry,
      name: entry.name || '',
      keys: [...entry.keys],
      content: entry.content,
      comment: entry.comment || '',
      enabled: entry.enabled,
      priority: entry.priority || 10,
      position: entry.position || 'before_char',
      case_sensitive: entry.case_sensitive || false,
      selective: entry.selective || false,
      secondary_keys: entry.secondary_keys ? [...entry.secondary_keys] : [],
      constant: entry.constant || false,
    };

    // 读取 extensions 字段
    const ext = entry.extensions as any;
    extensionsDepth.value = ext?.depth ?? 5;
    extensionsProbability.value = ext?.probability ?? 100;

    devLog('  - formData after assignment:', JSON.stringify(formData.value, null, 2));
    devLog('  - extensionsDepth:', extensionsDepth.value);
    devLog('  - extensionsProbability:', extensionsProbability.value);
  } else {
    devLog('⚠️ Neither creating nor editing - no action taken');
  }
}, { immediate: true });

function addKey(): void {
  if (newKey.value.trim()) {
    formData.value.keys.push(newKey.value.trim());
    newKey.value = '';
  }
}

function removeKey(index: number): void {
  formData.value.keys.splice(index, 1);
}

function addSecondaryKey(): void {
  if (newSecondaryKey.value.trim()) {
    if (!formData.value.secondary_keys) {
      formData.value.secondary_keys = [];
    }
    formData.value.secondary_keys.push(newSecondaryKey.value.trim());
    newSecondaryKey.value = '';
  }
}

function removeSecondaryKey(index: number): void {
  formData.value.secondary_keys?.splice(index, 1);
}

function handleSave(): void {
  // 验证
  if (formData.value.keys.length === 0) {
    showWarningToast('请至少添加一个关键词', '验证失败');
    return;
  }

  if (!formData.value.content.trim()) {
    showWarningToast('请输入内容', '验证失败');
    return;
  }

  // 更新 extensions 字段
  const dataToSave = {
    ...formData.value,
    extensions: {
      ...formData.value.extensions,
      depth: extensionsDepth.value,
      probability: extensionsProbability.value,
    },
  };

  devLog('💾 Saving entry with extensions:', dataToSave.extensions);
  emit('save', dataToSave);
}

function handleCancel(): void {
  emit('cancel');
}

async function handleDelete(): Promise<void> {
  const confirmed = await showAlertModal(
    '确定要删除这个条目吗？此操作不可撤销。',
    undefined,
    {
      title: '删除确认',
      type: 'danger',
      confirmText: '确认删除',
      cancelText: '取消'
    }
  );

  if (confirmed) {
    emit('delete');
  }
}
</script>
