<script setup lang="ts">
import { ref, watch } from 'vue';
import { MdClose } from 'vue-icons-plus/md';
import type { CreateWorldBookEntryParams, UpdateWorldBookEntryParams } from '@/types/character';

const props = defineProps<{
  visible: boolean;
  entry: any;
  isCreatingNew: boolean;
}>();

const emit = defineEmits<{
  close: [];
  save: [data: CreateWorldBookEntryParams | UpdateWorldBookEntryParams];
  delete: [];
}>();

// 表单数据
const formData = ref({
  name: '',
  content: '',
  comment: '',
  keys: [] as string[],
  secondary_keys: [] as string[],
  constant: false,
  enabled: true,
  position: 'before_char' as 'before_char' | 'after_char',
  priority: 0,
  insertion_order: 0,
});

// 新关键词输入
const newKey = ref('');
const newSecondaryKey = ref('');

// 高级设置显示
const showAdvanced = ref(false);

// 初始化表单数据
watch(
  () => props.entry,
  (newEntry) => {
    if (newEntry) {
      formData.value = {
        name: newEntry.name || '',
        content: newEntry.content || '',
        comment: newEntry.comment || '',
        keys: newEntry.keys || [],
        secondary_keys: newEntry.secondary_keys || [],
        constant: newEntry.constant || false,
        enabled: newEntry.enabled !== false,
        position: newEntry.position || 'before_char',
        priority: newEntry.priority || 0,
        insertion_order: newEntry.insertion_order || 0,
      };
      showAdvanced.value = false;
    }
  },
  { immediate: true }
);

// 添加关键词
function addKey() {
  if (newKey.value.trim() && !formData.value.keys.includes(newKey.value.trim())) {
    formData.value.keys.push(newKey.value.trim());
    newKey.value = '';
  }
}

// 删除关键词
function removeKey(index: number) {
  formData.value.keys.splice(index, 1);
}

// 添加次要关键词
function addSecondaryKey() {
  if (newSecondaryKey.value.trim() && !formData.value.secondary_keys.includes(newSecondaryKey.value.trim())) {
    formData.value.secondary_keys.push(newSecondaryKey.value.trim());
    newSecondaryKey.value = '';
  }
}

// 删除次要关键词
function removeSecondaryKey(index: number) {
  formData.value.secondary_keys.splice(index, 1);
}

// 保存
function handleSave() {
  emit('save', { ...formData.value });
}

// 关闭
function handleClose() {
  emit('close');
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click="handleClose"
      >
        <!-- 背景遮罩 -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>

        <!-- Modal内容 -->
        <div
          class="relative bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col"
          @click.stop
        >
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
            <h3 class="text-xl font-semibold text-gray-900">
              {{ isCreatingNew ? '新建世界书条目' : '编辑世界书条目' }}
            </h3>
            <button
              @click="handleClose"
              class="p-2 hover:bg-gray-100 rounded-full transition-colors"
            >
              <MdClose class="w-6 h-6 text-gray-500" />
            </button>
          </div>

          <!-- 表单内容 -->
          <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
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
                    class="bg-blue-500 hover:bg-blue-700 text-white text-sm font-medium py-1.5 px-4 rounded-full"
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
                        placeholder="输入次要关键词"
                        @keypress.enter.prevent="addSecondaryKey"
                      />
                      <button
                        type="button"
                        class="bg-purple-500 hover:bg-purple-700 text-white text-sm font-medium py-1.5 px-4 rounded-full"
                        @click="addSecondaryKey"
                      >
                        添加
                      </button>
                    </div>
                  </div>
                </div>

                <!-- 插入位置 -->
                <div>
                  <label class="text-sm font-semibold text-gray-700 mb-2 block">
                    插入位置
                  </label>
                  <select
                    v-model="formData.position"
                    class="w-full border border-gray-200 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="before_char">角色定义之前</option>
                    <option value="after_char">角色定义之后</option>
                  </select>
                </div>

                <!-- 常驻条目 -->
                <div class="flex items-center gap-2">
                  <input
                    v-model="formData.constant"
                    type="checkbox"
                    id="constant"
                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                  />
                  <label for="constant" class="text-sm font-semibold text-gray-700">
                    常驻条目 (总是生效)
                  </label>
                </div>

                <!-- 启用状态 -->
                <div class="flex items-center gap-2">
                  <input
                    v-model="formData.enabled"
                    type="checkbox"
                    id="enabled"
                    class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                  />
                  <label for="enabled" class="text-sm font-semibold text-gray-700">
                    启用此条目
                  </label>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="px-6 py-4 border-t border-gray-200 flex items-center justify-between">
            <button
              v-if="!isCreatingNew"
              @click="$emit('delete')"
              class="px-4 py-2 text-sm font-medium text-red-600 hover:bg-red-50 rounded-lg transition-colors"
            >
              删除条目
            </button>
            <div class="flex gap-3 ml-auto">
              <button
                @click="handleClose"
                class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
              >
                取消
              </button>
              <button
                @click="handleSave"
                class="px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-700 rounded-lg transition-colors"
              >
                {{ isCreatingNew ? '创建' : '保存' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Modal 动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

.modal-enter-from .bg-white,
.modal-leave-to .bg-white {
  transform: scale(0.95) translateY(-20px);
}

.modal-enter-to .bg-white,
.modal-leave-from .bg-white {
  transform: scale(1) translateY(0);
}
</style>
