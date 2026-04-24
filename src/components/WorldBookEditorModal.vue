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
        <div class="absolute inset-0 bg-black/65 backdrop-blur-md"></div>

        <!-- Modal内容 -->
        <div
          class="liquid-modal relative w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col"
          @click.stop
        >
          <!-- 头部 -->
          <div class="px-6 py-4 border-b border-white/8 flex items-center justify-between">
            <h3 class="text-lg font-semibold text-white/90">
              {{ isCreatingNew ? '新建世界书条目' : '编辑世界书条目' }}
            </h3>
            <button
              @click="handleClose"
              class="p-2 hover:bg-white/10 rounded-xl transition-colors"
            >
              <MdClose class="w-5 h-5 text-white/40" />
            </button>
          </div>

          <!-- 表单内容 -->
          <div class="flex-1 overflow-y-auto px-6 py-5 space-y-5">
            <!-- ═══ 核心字段 ═══ -->

            <!-- 条目名称 -->
            <div>
              <label class="text-sm font-semibold text-white/70 mb-2 block">
                条目名称
                <span class="text-xs font-normal text-white/35 ml-1">(可选)</span>
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
              <label class="text-sm font-semibold text-white/70 mb-2 block">
                关键词 <span class="text-red-400">*</span>
              </label>
              <div class="space-y-2">
                <div class="flex flex-wrap gap-1.5 mb-2">
                  <span
                    v-for="(key, index) in formData.keys"
                    :key="index"
                    class="inline-flex items-center gap-1 rounded-full bg-indigo-500/15 border border-indigo-400/25 px-2.5 py-0.5 text-xs font-medium text-indigo-300"
                  >
                    {{ key }}
                    <button type="button" class="text-indigo-400/60 hover:text-indigo-200" @click="removeKey(index)">
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
              <label class="text-sm font-semibold text-white/70 mb-2 block">
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
              <label class="text-sm font-semibold text-white/70 mb-2 block">
                备注
                <span class="text-xs font-normal text-white/35 ml-1">(可选)</span>
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
                <div class="w-9 h-5 bg-white/15 peer-focus:ring-2 peer-focus:ring-violet-500/25 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-violet-500"></div>
              </label>
              <span class="text-sm font-medium text-white/75">启用此条目</span>
            </div>

            <!-- ═══ 高级设置 ═══ -->
            <div class="pt-3 border-t border-white/8">
              <button
                type="button"
                class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-white/35 hover:text-white/60 transition-colors"
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
                  <label class="text-sm font-semibold text-white/70 mb-2 block">
                    次要关键词
                    <span class="text-xs font-normal text-white/35 ml-1">(选择性触发时使用)</span>
                  </label>
                  <div class="space-y-2">
                    <div class="flex flex-wrap gap-1.5 mb-2">
                      <span
                        v-for="(key, index) in formData.secondary_keys"
                        :key="index"
                        class="inline-flex items-center gap-1 rounded-full bg-violet-500/15 border border-violet-400/25 px-2.5 py-0.5 text-xs font-medium text-violet-300"
                      >
                        {{ key }}
                        <button type="button" class="text-violet-400/60 hover:text-violet-200" @click="removeSecondaryKey(index)">
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
                    <label class="text-sm font-semibold text-white/70 mb-2 block">优先级</label>
                    <input
                      v-model.number="formData.priority"
                      type="number"
                      min="0"
                      max="100"
                      class="modal-input"
                    />
                  </div>
                  <div>
                    <label class="text-sm font-semibold text-white/70 mb-2 block">插入位置</label>
                    <select v-model="formData.position" class="modal-input">
                      <option value="before_char" class="bg-slate-900">角色定义之前</option>
                      <option value="after_char" class="bg-slate-900">角色定义之后</option>
                    </select>
                  </div>
                </div>

                <!-- 常驻条目 -->
                <div class="flex items-center gap-3">
                  <label class="relative inline-flex items-center cursor-pointer">
                    <input v-model="formData.constant" type="checkbox" class="sr-only peer" />
                    <div class="w-9 h-5 bg-white/15 peer-focus:ring-2 peer-focus:ring-amber-500/25 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-amber-500"></div>
                  </label>
                  <span class="text-sm font-medium text-white/75">常驻条目 (总是生效)</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="px-6 py-4 border-t border-white/8 flex items-center justify-between">
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

.modal-input,
.modal-textarea {
  width: 100%;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.90);
  border-radius: 12px;
  font-size: 0.875rem;
  transition: all 0.2s ease;
  padding: 0.625rem 1rem;
}

.modal-input:focus,
.modal-textarea:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.09);
  border-color: rgba(139, 92, 246, 0.55);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.15), inset 0 1px 0 rgba(255, 255, 255, 0.10);
}

.modal-input::placeholder,
.modal-textarea::placeholder {
  color: rgba(255, 255, 255, 0.28);
}

.modal-textarea {
  padding: 0.75rem 1rem;
  line-height: 1.6;
  resize: none;
}
</style>