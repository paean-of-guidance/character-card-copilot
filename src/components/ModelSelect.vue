<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { MdRefresh } from 'vue-icons-plus/md'
import type { ApiConfig, ModelInfo } from '@/types/api'
import { useApiStore } from '@/stores/api'

const props = defineProps<{
  apiConfig: ApiConfig
  modelValue?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'update:modelMeta': [value: Pick<ModelInfo, 'max_tokens' | 'context_window'> | null]
}>()

const apiStore = useApiStore()

const rootRef = ref<HTMLElement | null>(null)
const models = ref<ModelInfo[]>([])
const loading = ref(false)
const error = ref('')
const isOpen = ref(false)
const searchQuery = ref('')

const selectedModel = computed({
  get: () => props.modelValue ?? '',
  set: (value: string) => emit('update:modelValue', value),
})

function emitModelMeta(model: ModelInfo | null) {
  if (!model) {
    emit('update:modelMeta', null)
    return
  }

  emit('update:modelMeta', {
    max_tokens: model.max_tokens,
    context_window: model.context_window,
  })
}

const displayModels = computed(() => {
  let modelList = models.value

  if (props.modelValue && !models.value.some((model) => model.id === props.modelValue)) {
    modelList = [{ id: props.modelValue, object: 'model' }, ...models.value]
  }

  if (!searchQuery.value.trim()) {
    return modelList
  }

  const query = searchQuery.value.trim().toLowerCase()
  return modelList.filter((model) => {
    const owner = typeof model.owned_by === 'string' ? model.owned_by : ''
    return model.id.toLowerCase().includes(query) || owner.toLowerCase().includes(query)
  })
})

async function loadModels() {
  if (!props.apiConfig.base_url || !props.apiConfig.api_key) {
    error.value = '请先填写 Base URL 和 API 密钥'
    return
  }

  loading.value = true
  error.value = ''

  try {
    models.value = await apiStore.fetchModels(props.apiConfig)
    emitModelMeta(models.value.find((model) => model.id === selectedModel.value) ?? null)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '获取模型失败'
    console.error('获取模型失败:', err)
  } finally {
    loading.value = false
  }
}

function openDropdown() {
  isOpen.value = true
  if (models.value.length === 0) {
    void loadModels()
  }
}

function handleInput(event: Event) {
  const value = (event.target as HTMLInputElement).value
  searchQuery.value = value
  selectedModel.value = value
  emitModelMeta(models.value.find((model) => model.id === value) ?? null)
  openDropdown()
}

function selectModel(model: ModelInfo) {
  selectedModel.value = model.id
  emitModelMeta(model)
  searchQuery.value = ''
  isOpen.value = false
}

function handleClickOutside(event: MouseEvent) {
  if (!rootRef.value) {
    return
  }

  const target = event.target as Node | null
  if (target && !rootRef.value.contains(target)) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div ref="rootRef" class="relative w-full">
    <div class="relative flex items-center">
      <input
        v-model="selectedModel"
        type="text"
        class="liquid-input w-full px-4 py-3 pr-12 text-sm"
        placeholder="选择或输入模型名称"
        @input="handleInput"
        @focus="openDropdown"
      />

      <button
        type="button"
        class="absolute right-3 rounded-full p-1 text-white/40 transition-colors hover:bg-white/10 hover:text-white/70 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="loading || !apiConfig.base_url || !apiConfig.api_key"
        title="刷新模型列表"
        @click="loadModels"
      >
        <MdRefresh class="h-4 w-4" :class="{ 'animate-spin': loading }" />
      </button>
    </div>

    <div
      v-if="isOpen"
      class="liquid-panel absolute left-0 right-0 top-[calc(100%+8px)] z-20 overflow-hidden"
    >
      <div v-if="loading" class="px-4 py-3 text-sm text-white/50">
        正在获取模型列表...
      </div>
      <div v-else-if="error" class="px-4 py-3 text-sm text-red-300">
        {{ error }}
      </div>
      <div v-else-if="displayModels.length === 0" class="px-4 py-3 text-sm text-white/50">
        {{ searchQuery.trim() ? '没有匹配的模型' : '暂无可用模型' }}
      </div>
      <div v-else class="max-h-64 overflow-y-auto py-2">
        <button
          v-for="model in displayModels"
          :key="model.id"
          type="button"
          class="flex w-full items-center justify-between gap-3 px-4 py-2 text-left text-sm transition-colors hover:bg-white/8"
          :class="selectedModel === model.id ? 'bg-violet-500/15 text-violet-300' : 'text-white/75'"
          @click="selectModel(model)"
        >
          <span class="truncate font-medium">{{ model.id }}</span>
          <span v-if="model.owned_by" class="truncate text-xs text-white/35">{{ model.owned_by }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
