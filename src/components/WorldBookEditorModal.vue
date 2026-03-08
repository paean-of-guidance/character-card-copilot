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

const newKey = ref('');
const newSecondaryKey = ref('');
const showAdvanced = ref(false);

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

function addKey() {
  if (newKey.value.trim() && !formData.value.keys.includes(newKey.value.trim())) {
    formData.value.keys.push(newKey.value.trim());
    newKey.value = '';
  }
}

function removeKey(index: number) {
  formData.value.keys.splice(index, 1);
}

function addSecondaryKey() {
  if (newSecondaryKey.value.trim() && !formData.value.secondary_keys.includes(newSecondaryKey.value.trim())) {
    formData.value.secondary_keys.push(newSecondaryKey.value.trim());
    newSecondaryKey.value = '';
  }
}

function removeSecondaryKey(index: number) {
  formData.value.secondary_keys.splice(index, 1);
}

function handleSave() {
  emit('save', { ...formData.value });
}

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
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm"></div>

        <!-- Modal内容 -->
        <div
          class="relative rounded-[24px] border border-white/70 bg-white/95 shadow-[0_24px_60px_rgba(148,163,184,0.25)] backdrop-blur-xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col"
          @click.stop
        >
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-slate-100 flex items-center justify-between">
            <h3 class="text-lg font-semibold text-slate-900">
              {{ isCreatingNew ? '新建世界书条目' : '编辑世界书条目' }}
            </h3>
            <button
              @click="handleClose"
              class="p-2 hover:bg-slate-100 rounded-xl transition-colors"
            >
              <MdClose class="w-5 h-5 text-slate-400" />
            </button>
          </div>

          <!-- 表单内容 -->
          <div class="flex-1 overflow-y-auto px-6 py-5 space-y-5">
            <!-- ═══ 核心字段 ═══ -->

            <!-- 条目名称 -->
            <div>
              <label class="text-sm font-semibold text-slate-700 mb-2 block">
                条目名称
                <span class="text-xs font-normal text-slate-400 ml-1">(可选)</span>
              </label>
              <input
                v-model="formData.name"
                type="text"
                class="modal-input"
                placeholder="条目的简短名称"
              />
            </div>

            <!-- 关键词 -->
            <div>
              <label class="text-sm font-semibold text-slate-700 mb-2 block">
                关键词 <span class="text-red-400">*</span>
              </label>
              <div class="space-y-2">
                <div class="flex flex-wrap gap-1.5 mb-2">
                  <span
                    v-for="(key, index) in formData.keys"
                    :key="index"
                    class="inline-flex items-center gap-1 rounded-full bg-blue-50 px-2.5 py-0.5 text-xs font-medium text-blue-700"
                  >
                    {{ key }}
                    <button type="button" class="text-blue-400 hover:text-blue-700" @click="removeKey(index)">
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="newKey"
                    type="text"
                    class="modal-input flex-1"
                    placeholder="输入关键词后按回车添加"
                    @keypress.enter.prevent="addKey"
                  />
                  <button
                    type="button"
                    class="glass-btn glass-btn--primary"
                    @click="addKey"
                  >
                    添加
                  </button>
                </div>
              </div>
            </div>

            <!-- 内容 -->
            <div>
              <label class="text-sm font-semibold text-slate-700 mb-2 block">
                内容 <span class="text-red-400">*</span>
              </label>
              <textarea
                v-model="formData.content"
                rows="6"
                class="modal-textarea"
                placeholder="当关键词被触发时插入的内容"
              ></textarea>
            </div>

            <!-- 备注 -->
            <div>
              <label class="text-sm font-semibold text-slate-700 mb-2 block">
                备注
                <span class="text-xs font-normal text-slate-400 ml-1">(可选)</span>
              </label>
              <textarea
                v-model="formData.comment"
                rows="2"
                class="modal-textarea"
                placeholder="关于这个条目的备注，不会影响提示词"
              ></textarea>
            </div>

            <!-- 启用状态 (核心字段，不放高级里) -->
            <div class="flex items-center gap-3">
              <label class="relative inline-flex items-center cursor-pointer">
                <input v-model="formData.enabled" type="checkbox" class="sr-only peer" />
                <div class="w-9 h-5 bg-slate-200 peer-focus:ring-2 peer-focus:ring-blue-500/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-500"></div>
              </label>
              <span class="text-sm font-medium text-slate-700">启用此条目</span>
            </div>

            <!-- ═══ 高级设置 ═══ -->
            <div class="pt-3 border-t border-slate-100">
              <button
                type="button"
                class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-slate-400 hover:text-slate-600 transition-colors"
                @click="showAdvanced = !showAdvanced"
              >
                <svg
                  class="w-3.5 h-3.5 transition-transform"
                  :class="{ 'rotate-90': showAdvanced }"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
                高级设置
              </button>

              <div v-if="showAdvanced" class="mt-4 space-y-4">
                <!-- 次要关键词 -->
                <div>
                  <label class="text-sm font-semibold text-slate-700 mb-2 block">
                    次要关键词
                    <span class="text-xs font-normal text-slate-400 ml-1">(选择性触发时使用)</span>
                  </label>
                  <div class="space-y-2">
                    <div class="flex flex-wrap gap-1.5 mb-2">
                      <span
                        v-for="(key, index) in formData.secondary_keys"
                        :key="index"
                        class="inline-flex items-center gap-1 rounded-full bg-purple-50 px-2.5 py-0.5 text-xs font-medium text-purple-700"
                      >
                        {{ key }}
                        <button type="button" class="text-purple-400 hover:text-purple-700" @click="removeSecondaryKey(index)">
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </span>
                    </div>
                    <div class="flex gap-2">
                      <input
                        v-model="newSecondaryKey"
                        type="text"
                        class="modal-input flex-1"
                        placeholder="输入次要关键词"
                        @keypress.enter.prevent="addSecondaryKey"
                      />
                      <button
                        type="button"
                        class="glass-btn glass-btn--primary"
                        @click="addSecondaryKey"
                      >
                        添加
                      </button>
                    </div>
                  </div>
                </div>

                <!-- 优先级 & 插入位置 -->
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="text-sm font-semibold text-slate-700 mb-2 block">优先级</label>
                    <input
                      v-model.number="formData.priority"
                      type="number"
                      min="0"
                      max="100"
                      class="modal-input"
                    />
                  </div>
                  <div>
                    <label class="text-sm font-semibold text-slate-700 mb-2 block">插入位置</label>
                    <select v-model="formData.position" class="modal-input">
                      <option value="before_char">角色定义之前</option>
                      <option value="after_char">角色定义之后</option>
                    </select>
                  </div>
                </div>

                <!-- 常驻条目 -->
                <div class="flex items-center gap-3">
                  <label class="relative inline-flex items-center cursor-pointer">
                    <input v-model="formData.constant" type="checkbox" class="sr-only peer" />
                    <div class="w-9 h-5 bg-slate-200 peer-focus:ring-2 peer-focus:ring-blue-500/20 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-amber-500"></div>
                  </label>
                  <span class="text-sm font-medium text-slate-700">常驻条目 (总是生效)</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="px-6 py-4 border-t border-slate-100 flex items-center justify-between">
            <button
              v-if="!isCreatingNew"
              @click="$emit('delete')"
              class="glass-btn glass-btn--danger"
            >
              删除条目
            </button>
            <div class="flex gap-3 ml-auto">
              <button
                @click="handleClose"
                class="glass-btn glass-btn--neutral"
              >
                取消
              </button>
              <button
                @click="handleSave"
                class="glass-btn glass-btn--primary"
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
@reference "tailwindcss";

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

.modal-input {
  @apply w-full rounded-xl border border-slate-200/80 bg-white/90 px-4 py-2.5 text-sm text-slate-800 placeholder-slate-300 transition-all duration-200;
  @apply focus:outline-none focus:border-blue-300 focus:ring-2 focus:ring-blue-500/10;
}

.modal-textarea {
  @apply w-full rounded-xl border border-slate-200/80 bg-white/90 px-4 py-3 text-sm leading-relaxed text-slate-800 placeholder-slate-300 resize-none transition-all duration-200;
  @apply focus:outline-none focus:border-blue-300 focus:ring-2 focus:ring-blue-500/10;
}
</style>