import { computed, readonly, ref } from 'vue'
import { defineStore } from 'pinia'
import type {
  ApiConfig,
  ApiTestResult,
  CreateApiRequest,
  ModelInfo,
  UpdateApiRequest,
} from '@/types/api'
import * as apiConfigService from '@/services/apiConfig'

const CACHE_DURATION = 5 * 60 * 1000

function cloneApi(config: ApiConfig): ApiConfig {
  return { ...config }
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}

export const useApiStore = defineStore('api', () => {
  const apis = ref<ApiConfig[]>([])
  const loading = ref(false)
  const lastFetch = ref(0)

  const selectedProfile = ref('')
  const draft = ref<ApiConfig | null>(null)
  const saving = ref(false)
  const saveError = ref('')
  const testing = ref(false)
  const lastTestResult = ref<ApiTestResult | null>(null)

  const defaultApi = computed(() => {
    return apis.value.find((api) => api.default) ?? null
  })

  const enabledApis = computed(() => {
    return apis.value.filter((api) => api.enabled)
  })

  const disabledApis = computed(() => {
    return apis.value.filter((api) => !api.enabled)
  })

  const isCacheValid = computed(() => {
    return Date.now() - lastFetch.value < CACHE_DURATION
  })

  const selectedApi = computed(() => {
    if (!selectedProfile.value) {
      return null
    }

    return apis.value.find((api) => api.profile === selectedProfile.value) ?? null
  })

  const dirty = computed(() => {
    if (!selectedApi.value || !draft.value) {
      return false
    }

    return JSON.stringify(selectedApi.value) !== JSON.stringify(draft.value)
  })

  function clearDraftState() {
    draft.value = null
    saveError.value = ''
    lastTestResult.value = null
  }

  function syncDraftFromSelection() {
    if (!selectedApi.value) {
      clearDraftState()
      return
    }

    draft.value = cloneApi(selectedApi.value)
    saveError.value = ''
    lastTestResult.value = null
  }

  function selectApi(profile: string | null) {
    selectedProfile.value = profile ?? ''
    syncDraftFromSelection()
  }

  function clearSelection() {
    selectedProfile.value = ''
    clearDraftState()
  }

  function patchDraft(patch: Partial<ApiConfig>) {
    if (!draft.value) {
      return
    }

    draft.value = {
      ...draft.value,
      ...patch,
    }

    saveError.value = ''
    lastTestResult.value = null
  }

  function discardDraft() {
    syncDraftFromSelection()
  }

  async function loadAllApis(force = false) {
    if (!force && isCacheValid.value && apis.value.length > 0) {
      return apis.value
    }

    loading.value = true
    try {
      apis.value = await apiConfigService.getAllApiConfigs()
      lastFetch.value = Date.now()

      if (selectedProfile.value) {
        const exists = apis.value.some((api) => api.profile === selectedProfile.value)
        if (!exists) {
          clearSelection()
        } else if (!dirty.value) {
          syncDraftFromSelection()
        }
      }

      return apis.value
    } catch (error) {
      console.error('加载API配置失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  async function refreshApis() {
    return await loadAllApis(true)
  }

  async function getApiByProfile(profile: string): Promise<ApiConfig | undefined> {
    const cached = apis.value.find((api) => api.profile === profile)
    if (cached) {
      return cached
    }

    const remote = await apiConfigService.getApiConfigByProfile(profile)
    if (!remote) {
      return undefined
    }

    apis.value.push(remote)
    return remote
  }

  async function getDefaultApi(): Promise<ApiConfig | undefined> {
    if (defaultApi.value) {
      return defaultApi.value
    }

    const remote = await apiConfigService.getDefaultApiConfig()
    if (!remote) {
      return undefined
    }

    await refreshApis()
    return apis.value.find((api) => api.profile === remote.profile)
  }

  async function createApi(config: CreateApiRequest) {
    const created = await apiConfigService.createApiConfig(config)
    await refreshApis()
    selectApi(created.profile)
    return created
  }

  async function updateApi(config: UpdateApiRequest) {
    await apiConfigService.updateApiConfig(config)
    await refreshApis()

    if (selectedProfile.value === config.original_profile || selectedProfile.value === config.profile) {
      selectApi(config.profile)
    }
  }

  async function saveDraft() {
    if (!draft.value || !selectedApi.value) {
      throw new Error('当前没有可保存的API配置')
    }

    saving.value = true
    saveError.value = ''

    try {
      const request: UpdateApiRequest = {
        profile: draft.value.profile,
        original_profile: selectedApi.value.profile,
        provider: draft.value.provider,
        base_url: draft.value.base_url,
        api_key: draft.value.api_key,
        model: draft.value.model,
        max_tokens: draft.value.max_tokens,
        context_window: draft.value.context_window,
        default: draft.value.default,
        enabled: draft.value.enabled,
      }

      await updateApi(request)
      syncDraftFromSelection()
      return selectedApi.value
    } catch (error) {
      saveError.value = getErrorMessage(error)
      throw error
    } finally {
      saving.value = false
    }
  }

  async function deleteApi(profile: string) {
    await apiConfigService.deleteApiConfig(profile)
    await refreshApis()

    if (selectedProfile.value === profile) {
      clearSelection()
    }
  }

  async function copyApi(profile: string) {
    const originalApi = await getApiByProfile(profile)
    if (!originalApi) {
      throw new Error(`API配置 ${profile} 不存在`)
    }

    const created = await apiConfigService.copyApiConfig(originalApi)
    await refreshApis()
    selectApi(created.profile)
    return created
  }

  async function setDefaultApi(profile: string) {
    await apiConfigService.setDefaultApiConfig(profile)
    await refreshApis()

    if (selectedProfile.value === profile) {
      syncDraftFromSelection()
    }
  }

  async function toggleApi(profile: string, enabled: boolean) {
    await apiConfigService.toggleApiConfig(profile, enabled)
    await refreshApis()

    if (selectedProfile.value === profile) {
      syncDraftFromSelection()
    }
  }

  async function testDraft() {
    if (!draft.value) {
      throw new Error('当前没有可测试的API配置')
    }

    testing.value = true
    saveError.value = ''

    try {
      lastTestResult.value = await apiConfigService.testApiConnection(draft.value)
      return lastTestResult.value
    } finally {
      testing.value = false
    }
  }

  async function testConnection(config: ApiConfig): Promise<ApiTestResult> {
    return await apiConfigService.testApiConnection(config)
  }

  async function fetchModels(config: ApiConfig): Promise<ModelInfo[]> {
    return await apiConfigService.fetchModels(config)
  }

  function clearCache() {
    lastFetch.value = 0
  }

  function reset() {
    apis.value = []
    loading.value = false
    lastFetch.value = 0
    selectedProfile.value = ''
    clearDraftState()
    saving.value = false
    testing.value = false
  }

  return {
    apis: readonly(apis),
    loading: readonly(loading),
    lastFetch: readonly(lastFetch),
    selectedProfile: readonly(selectedProfile),
    selectedApi,
    draft: readonly(draft),
    saving: readonly(saving),
    saveError: readonly(saveError),
    testing: readonly(testing),
    lastTestResult: readonly(lastTestResult),
    dirty,
    defaultApi,
    enabledApis,
    disabledApis,
    isCacheValid,
    loadAllApis,
    refreshApis,
    getApiByProfile,
    getDefaultApi,
    createApi,
    updateApi,
    saveDraft,
    deleteApi,
    copyApi,
    setDefaultApi,
    toggleApi,
    selectApi,
    clearSelection,
    patchDraft,
    discardDraft,
    testDraft,
    testConnection,
    fetchModels,
    clearCache,
    reset,
  }
})
