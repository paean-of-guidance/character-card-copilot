<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { MdAdd } from 'vue-icons-plus/md'
import type { ApiConfig } from '@/types/api'
import { useApiStore } from '@/stores/api'
import ApiItem from './ApiItem.vue'

const props = withDefaults(
  defineProps<{
    selectedProfile?: string
    searchQuery?: string
  }>(),
  {
    selectedProfile: '',
    searchQuery: '',
  },
)

const emit = defineEmits<{
  select: [api: ApiConfig]
  copy: [api: ApiConfig]
  delete: [profile: string]
  create: []
}>()

const apiStore = useApiStore()
const { apis, loading } = storeToRefs(apiStore)

const filteredApis = computed(() => {
  const query = props.searchQuery.trim().toLowerCase()
  if (!query) {
    return apis.value
  }

  return apis.value.filter((api) => {
    return [api.profile, api.base_url, api.model, api.provider]
      .filter(Boolean)
      .some((value) => value.toLowerCase().includes(query))
  })
})

onMounted(async () => {
  await apiStore.loadAllApis()
})
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-2.5">
    <div v-if="loading" class="flex-1 rounded-xl border border-dashed border-gray-200 bg-gray-50 px-3 py-6 text-center text-sm text-gray-500">
      正在加载 API 配置...
    </div>

    <template v-else>
      <div class="thin-scrollbar min-h-0 flex-1 overflow-y-auto pr-0.5">
        <div
          v-if="filteredApis.length === 0"
          class="rounded-xl border border-dashed border-gray-200 bg-gray-50 px-3 py-6 text-center"
        >
          <p class="text-sm font-medium text-gray-700">
            {{ apis.length === 0 ? '还没有 API 配置' : '没有匹配的配置' }}
          </p>
          <p class="mt-1 text-xs text-gray-500">
            {{ apis.length === 0 ? '先创建一个配置，再测试和启用。' : '试试更换搜索关键词。' }}
          </p>
        </div>

        <div v-else class="space-y-2.5 pb-1">
          <ApiItem
            v-for="api in filteredApis"
            :key="api.profile"
            :api="api"
            :selected="api.profile === selectedProfile"
            @select="emit('select', $event)"
            @copy="emit('copy', $event)"
            @delete="emit('delete', $event)"
          />
        </div>
      </div>

        <button
        type="button"
        class="shrink-0 flex w-full items-center justify-center gap-2 rounded-xl border border-dashed border-blue-300 bg-blue-50 px-3 py-3 text-sm font-medium text-blue-700 transition-colors hover:border-blue-400 hover:bg-blue-100"
        @click="emit('create')"
      >
        <MdAdd class="h-4 w-4" />
        新建 API 配置
      </button>
    </template>
  </div>
</template>

<style scoped>
.thin-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: #cbd5e1 transparent;
}

.thin-scrollbar::-webkit-scrollbar {
  width: 5px;
}

.thin-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.thin-scrollbar::-webkit-scrollbar-thumb {
  border-radius: 9999px;
  background-color: #cbd5e1;
}

.thin-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: #94a3b8;
}
</style>
