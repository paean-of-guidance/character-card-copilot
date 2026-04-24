<template>
  <div class="liquid-card mb-2 transition-all duration-200 hover:border-violet-400/30">
    <!-- 条目头部（可点击展开/收起） -->
    <div
      class="flex items-center justify-between px-4 py-3 cursor-pointer"
      @click="toggleExpanded"
    >
      <div class="flex items-center gap-3 flex-1 min-w-0">
        <!-- 展开/收起图标 -->
        <svg
          class="w-4 h-4 text-white/35 transition-transform shrink-0"
          :class="{ 'rotate-90': isExpanded }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>

        <!-- 条目信息 -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="text-sm font-semibold text-white/85 truncate">
              {{ entry.comment || entry.name || `条目 #${entry.id}` }}
            </span>

            <!-- 状态标记 -->
            <span
              class="inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-semibold"
              :class="entry.enabled
                ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-400/25'
                : 'bg-white/8 text-white/40 border border-white/12'"
            >
              {{ entry.enabled ? '启用' : '禁用' }}
            </span>

            <!-- 优先级标签 -->
            <span
              v-if="entry.priority !== undefined && entry.priority !== 10"
              class="inline-flex items-center rounded-full bg-indigo-500/15 border border-indigo-400/25 px-2 py-0.5 text-[10px] font-semibold text-indigo-300"
            >
              P{{ entry.priority }}
            </span>

            <!-- 常驻标签 -->
            <span
              v-if="entry.constant"
              class="inline-flex items-center rounded-full bg-amber-500/15 border border-amber-400/25 px-2 py-0.5 text-[10px] font-semibold text-amber-300"
            >
              常驻
            </span>
          </div>

          <!-- 关键词摘要 -->
          <div class="mt-0.5 text-xs text-white/35">
            <span v-for="(key, index) in entry.keys.slice(0, 3)" :key="index">
              <span class="inline-flex items-center rounded bg-white/8 px-1.5 py-0.5 text-[10px] text-white/50 mr-1">{{ key }}</span>
            </span>
            <span v-if="entry.keys.length > 3" class="text-[10px] text-white/35">+{{ entry.keys.length - 3 }}</span>
          </div>
        </div>
      </div>

      <!-- 操作按钮组 -->
      <div class="flex items-center gap-1 shrink-0 ml-2" @click.stop>
        <button
          class="p-1.5 text-white/35 hover:text-indigo-300 hover:bg-indigo-500/15 rounded-lg transition-colors"
          title="编辑"
          @click="handleEdit"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        </button>

        <button
          class="p-1.5 text-white/35 hover:text-red-300 hover:bg-red-500/15 rounded-lg transition-colors"
          title="删除"
          @click="handleDelete"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 详细内容（展开时显示） -->
    <div
      v-if="isExpanded"
      class="px-4 pb-4 space-y-3 border-t border-white/8 pt-3"
    >
      <!-- 完整关键词列表 -->
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider text-white/35">关键词</label>
        <div class="mt-1.5 flex flex-wrap gap-1.5">
          <span
            v-for="(key, index) in entry.keys"
            :key="index"
            class="inline-flex items-center rounded-full bg-indigo-500/15 border border-indigo-400/25 px-2.5 py-0.5 text-xs font-medium text-indigo-300"
          >
            {{ key }}
          </span>
        </div>
      </div>

      <!-- 次要关键词 -->
      <div v-if="entry.secondary_keys && entry.secondary_keys.length > 0">
        <label class="text-xs font-semibold uppercase tracking-wider text-white/35">次要关键词</label>
        <div class="mt-1.5 flex flex-wrap gap-1.5">
          <span
            v-for="(key, index) in entry.secondary_keys"
            :key="index"
            class="inline-flex items-center rounded-full bg-violet-500/15 border border-violet-400/25 px-2.5 py-0.5 text-xs font-medium text-violet-300"
          >
            {{ key }}
          </span>
        </div>
      </div>

      <!-- 内容 -->
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider text-white/35">内容</label>
        <p class="mt-1 text-sm text-white/70 leading-relaxed whitespace-pre-wrap rounded-xl bg-white/5 border border-white/8 p-3">{{ entry.content }}</p>
      </div>

      <!-- 备注 -->
      <div v-if="entry.comment">
        <label class="text-xs font-semibold uppercase tracking-wider text-white/35">备注</label>
        <p class="mt-1 text-sm text-white/45 italic">{{ entry.comment }}</p>
      </div>

      <!-- 高级设置 -->
      <div class="grid grid-cols-2 gap-3 pt-2 border-t border-white/8">
        <div>
          <label class="text-[10px] font-medium uppercase tracking-wider text-white/30">插入位置</label>
          <p class="text-xs text-white/60 mt-0.5">
            {{ entry.position === 'before_char' ? '角色定义之前' : '角色定义之后' }}
          </p>
        </div>
        <div>
          <label class="text-[10px] font-medium uppercase tracking-wider text-white/30">插入顺序</label>
          <p class="text-xs text-white/60 mt-0.5">{{ entry.insertion_order }}</p>
        </div>
        <div>
          <label class="text-[10px] font-medium uppercase tracking-wider text-white/30">插入深度</label>
          <p class="text-xs text-white/60 mt-0.5">{{ extensionsDepth }}</p>
        </div>
        <div>
          <label class="text-[10px] font-medium uppercase tracking-wider text-white/30">概率</label>
          <p class="text-xs text-white/60 mt-0.5">{{ extensionsProbability }}%</p>
        </div>
        <div>
          <label class="text-[10px] font-medium uppercase tracking-wider text-white/30">大小写敏感</label>
          <p class="text-xs text-white/60 mt-0.5">{{ entry.case_sensitive ? '是' : '否' }}</p>
        </div>
        <div>
          <label class="text-[10px] font-medium uppercase tracking-wider text-white/30">选择性触发</label>
          <p class="text-xs text-white/60 mt-0.5">{{ entry.selective ? '是' : '否' }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { WorldBookEntry } from '@/types/character';
import { devLog } from '@/utils/logger';

interface Props {
  entry: WorldBookEntry;
  isExpanded: boolean;
}

interface Emits {
  (e: 'toggle'): void;
  (e: 'edit', entryId: number | undefined): void;
  (e: 'delete', entryId: number | undefined): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 从 extensions 读取插入深度
const extensionsDepth = computed(() => {
  const ext = props.entry.extensions as any;
  return ext?.depth ?? 5;
});

// 从 extensions 读取概率
const extensionsProbability = computed(() => {
  const ext = props.entry.extensions as any;
  return ext?.probability ?? 100;
});

function toggleExpanded(): void {
  emit('toggle');
}

function handleEdit(): void {
  devLog('🔘 WorldBookEntry edit button clicked');
  emit('edit', props.entry.id);
}

function handleDelete(): void {
  devLog('🗑️ WorldBookEntry delete button clicked');
  emit('delete', props.entry.id);
}
</script>
