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
  if (!props.api.base_url) {
    return '未设置端点'
  }

  try {
    return new URL(props.api.base_url).host
  } catch {
    return props.api.base_url
  }
})

const providerLabel = computed(() => {
  switch (props.api.provider) {
    case 'claude':
      return 'Claude'
    case 'gemini_v1_beta':
      return 'Gemini v1beta'
    default:
      return 'OpenAI Responses'
  }
})

function handleSelect() {
  emit('select', props.api)
}

function handleCopy() {
  emit('copy', props.api)
}

function handleDelete() {
  emit('delete', props.api.profile)
}
</script>

<template>
  <button
    type="button"
    class="w-full rounded-2xl border px-4 py-3 text-left transition-all"
    :class="{
      'border-blue-500 bg-blue-50 shadow-sm': selected,
      'border-gray-200 bg-white hover:border-gray-300 hover:bg-gray-50': !selected,
      'opacity-70': !api.enabled,
    }"
    @click="handleSelect"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2 min-w-0">
          <component
            :is="api.enabled ? MdOutlineCheckCircle : MdOutlineRadioButtonUnchecked"
            class="h-4 w-4 shrink-0"
            :class="api.enabled ? 'text-green-500' : 'text-gray-400'"
          />
          <h3 class="truncate text-sm font-semibold text-gray-900">
            {{ api.profile }}
          </h3>
          <span
            v-if="api.default"
            class="inline-flex shrink-0 items-center whitespace-nowrap rounded-full bg-blue-100 px-2.5 py-1 text-[11px] font-medium leading-none text-blue-700"
          >
            默认
          </span>
        </div>

        <div class="mt-2 flex flex-wrap items-center gap-2 text-xs text-gray-500">
          <span class="inline-flex shrink-0 items-center whitespace-nowrap rounded-full bg-gray-100 px-2.5 py-1 leading-none">
            {{ api.enabled ? '已启用' : '未启用' }}
          </span>
          <span class="inline-flex shrink-0 items-center whitespace-nowrap rounded-full bg-gray-100 px-2.5 py-1 leading-none">
            {{ providerLabel }}
          </span>
          <span class="inline-flex max-w-full items-center truncate rounded-full bg-gray-100 px-2.5 py-1 leading-none">
            {{ api.model || '未设置模型' }}
          </span>
        </div>

        <p class="mt-2 truncate text-xs text-gray-500">
          {{ endpointLabel }}
        </p>
      </div>

      <div class="flex shrink-0 items-center gap-1">
        <button
          type="button"
          class="rounded-full p-2 text-gray-500 transition-colors hover:bg-gray-200 hover:text-gray-700"
          title="复制配置"
          @click.stop="handleCopy"
        >
          <MdContentCopy class="h-4 w-4" />
        </button>
        <button
          type="button"
          class="rounded-full p-2 text-red-500 transition-colors hover:bg-red-50 hover:text-red-600"
          title="删除配置"
          @click.stop="handleDelete"
        >
          <MdDelete class="h-4 w-4" />
        </button>
      </div>
    </div>
  </button>
</template>
