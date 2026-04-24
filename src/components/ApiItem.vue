<script setup lang="ts">
import { computed } from 'vue'
import { MdContentCopy, MdDelete, MdOutlineCheckCircle, MdOutlineRadioButtonUnchecked } from 'vue-icons-plus/md'
import type { ApiConfig } from '@/types/api'

const props = defineProps<{
  api: ApiConfig
  selected?: boolean
}>()

const emit = defineEmits<{
  select: [api: ApiConfig]
  copy: [api: ApiConfig]
  delete: [profile: string]
}>()

const endpointLabel = computed(() => {
  if (!props.api.base_url) return '未设置端点'
  try {
    return new URL(props.api.base_url).host
  } catch {
    return props.api.base_url
  }
})

const providerLabel = computed(() => {
  switch (props.api.provider) {
    case 'claude': return 'Claude'
    case 'gemini_v1_beta': return 'Gemini v1beta'
    default: return 'OpenAI Responses'
  }
})

function handleSelect() { emit('select', props.api) }
function handleCopy() { emit('copy', props.api) }
function handleDelete() { emit('delete', props.api.profile) }
</script>

<template>
  <button
    type="button"
    class="w-full rounded-2xl border px-4 py-3 text-left transition-all"
    :class="{
      'border-violet-400/35 bg-violet-500/15 shadow-[0_0_0_1px_rgba(167,139,250,0.15)]': selected,
      'border-white/10 bg-white/5 hover:border-white/18 hover:bg-white/8': !selected,
      'opacity-55': !api.enabled,
    }"
    @click="handleSelect"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 items-center gap-2">
          <component
            :is="api.enabled ? MdOutlineCheckCircle : MdOutlineRadioButtonUnchecked"
            class="h-4 w-4 shrink-0"
            :class="api.enabled ? 'text-emerald-400' : 'text-white/30'"
          />
          <h3 class="truncate text-sm font-semibold text-white/85">{{ api.profile }}</h3>
          <span v-if="api.default" class="liquid-badge liquid-badge--primary shrink-0 whitespace-nowrap">默认</span>
        </div>

        <div class="mt-2 flex flex-wrap items-center gap-1.5 text-xs text-white/40">
          <span class="liquid-badge">{{ api.enabled ? '已启用' : '未启用' }}</span>
          <span class="liquid-badge">{{ providerLabel }}</span>
          <span class="liquid-badge max-w-[10rem] truncate">{{ api.model || '未设置模型' }}</span>
        </div>

        <p class="mt-2 truncate text-xs text-white/30">{{ endpointLabel }}</p>
      </div>

      <div class="flex shrink-0 items-center gap-1">
        <button type="button" class="rounded-full p-2 text-white/35 transition-colors hover:bg-white/10 hover:text-white/65" title="复制配置" @click.stop="handleCopy">
          <MdContentCopy class="h-4 w-4" />
        </button>
        <button type="button" class="rounded-full p-2 text-red-400/50 transition-colors hover:bg-red-500/15 hover:text-red-300" title="删除配置" @click.stop="handleDelete">
          <MdDelete class="h-4 w-4" />
        </button>
      </div>
    </div>
  </button>
</template>
