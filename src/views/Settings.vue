<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { MdContentCopy, MdDelete, MdSave, MdWifiTethering } from 'vue-icons-plus/md'
import { useAppStore } from '@/stores/app'
import { useApiStore } from '@/stores/api'
import { useModal } from '@/composables/useModal'
import { useNotification } from '@/composables/useNotification'
import type { ApiConfig, ApiProvider } from '@/types/api'
import ApiList from '@/components/ApiList.vue'
import ModelSelect from '@/components/ModelSelect.vue'
import NewApiDialog from '@/components/NewApiDialog.vue'
import AIRoleSettingsPanel from '@/components/settings/AIRoleSettingsPanel.vue'

type SettingsTab = 'api' | 'ai-role'

const appStore = useAppStore()
const apiStore = useApiStore()
const { showAlertModal } = useModal()
const { showErrorToast, showInfoToast, showSuccessToast, showWarningToast } = useNotification()

const showNewApiDialog = ref(false)
const searchQuery = ref('')
const activeTab = ref<SettingsTab>('api')

const providerOptions: Array<{ value: ApiProvider; label: string; baseUrl: string }> = [
  { value: 'open_ai_compatible', label: 'OpenAI Compatible', baseUrl: 'https://api.openai.com/v1' },
  { value: 'open_ai_responses', label: 'OpenAI Responses', baseUrl: 'https://api.openai.com/v1' },
  { value: 'claude', label: 'Claude', baseUrl: 'https://api.anthropic.com' },
  { value: 'gemini_v1_beta', label: 'Gemini v1beta', baseUrl: 'https://generativelanguage.googleapis.com/v1beta' },
]

const { apis, selectedProfile, selectedApi, draft, dirty, saving, saveError, testing, lastTestResult } =
  storeToRefs(apiStore)

const hasSelection = computed(() => {
  return !!selectedApi.value && !!draft.value
})

const hasDraftCredentials = computed(() => {
  return !!draft.value?.base_url.trim() && !!draft.value?.api_key.trim()
})

const canEnableDraft = computed(() => {
  if (!draft.value || !selectedApi.value) {
    return false
  }

  if (selectedApi.value.enabled) {
    return true
  }

  return !!lastTestResult.value?.success
})

const canSetDefault = computed(() => {
  return !!selectedApi.value && selectedApi.value.enabled && !selectedApi.value.default
})

const endpointSummary = computed(() => {
  if (!draft.value?.base_url) {
    return '未设置端点'
  }

  try {
    return new URL(draft.value.base_url).host
  } catch {
    return draft.value.base_url
  }
})

function handleProviderChange(provider: ApiProvider) {
  patchDraftField('provider', provider)

  const option = providerOptions.find((item) => item.value === provider)
  const currentDraft = draft.value
  if (!currentDraft || !option) {
    return
  }

  if (!currentDraft.base_url || providerOptions.some((item) => item.baseUrl === currentDraft.base_url)) {
    patchDraftField('base_url', option.baseUrl)
  }
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}

async function initializePage() {
  appStore.setPageTitle('设置', true)
  await apiStore.loadAllApis()

  if (!selectedProfile.value && apis.value.length > 0) {
    apiStore.selectApi(apis.value[0].profile)
  }
}

onMounted(() => {
  void initializePage()
})

async function confirmDiscardChanges() {
  if (!dirty.value) {
    return true
  }

  return await showAlertModal('当前有未保存的修改，是否放弃这些修改？', undefined, {
    title: '放弃未保存修改',
    type: 'warning',
    confirmText: '放弃修改',
    cancelText: '继续编辑',
  })
}

async function handleSelectApi(api: ApiConfig) {
  if (api.profile === selectedProfile.value) {
    return
  }

  const canSwitch = await confirmDiscardChanges()
  if (!canSwitch) {
    return
  }

  apiStore.selectApi(api.profile)
}

function patchDraftField(field: keyof ApiConfig, value: string | boolean) {
  apiStore.patchDraft({ [field]: value } as Partial<ApiConfig>)
}

async function ensureDraftSaved() {
  if (!dirty.value) {
    return
  }

  await apiStore.saveDraft()
}

async function handleSaveDraft() {
  try {
    const savedApi = await apiStore.saveDraft()
    showSuccessToast(`已保存配置「${savedApi?.profile ?? ''}」`, '保存成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '保存失败')
  }
}

function handleDiscardDraft() {
  apiStore.discardDraft()
  showInfoToast('已还原未保存的修改', '已撤销')
}

async function handleTestConnection() {
  try {
    const result = await apiStore.testDraft()
    if (result.success) {
      showSuccessToast(result.message, '连接成功')
    } else {
      showWarningToast(result.error ? `${result.message}：${result.error}` : result.message, '连接未通过')
    }
  } catch (error) {
    showErrorToast(getErrorMessage(error), '测试失败')
  }
}

async function handleToggleEnabled() {
  if (!selectedApi.value) {
    return
  }

  try {
    if (selectedApi.value.enabled) {
      await ensureDraftSaved()
      await apiStore.toggleApi(selectedApi.value.profile, false)
      showInfoToast('已禁用该配置，如它曾是默认配置也会一并取消默认', '已禁用')
      return
    }

    if (!lastTestResult.value?.success) {
      showWarningToast('请先用当前草稿测试连接成功，再启用该配置', '无法启用')
      return
    }

    await ensureDraftSaved()
    await apiStore.toggleApi(selectedApi.value.profile, true)
    showSuccessToast('配置已启用', '操作成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '状态更新失败')
  }
}

async function handleSetDefault() {
  if (!selectedApi.value) {
    return
  }

  try {
    await ensureDraftSaved()
    await apiStore.setDefaultApi(selectedApi.value.profile)
    showSuccessToast('已设为默认配置', '操作成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '设置默认失败')
  }
}

async function handleCopyConfig(api: ApiConfig) {
  try {
    const copied = await apiStore.copyApi(api.profile)
    showSuccessToast(`已复制为「${copied.profile}」`, '复制成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '复制失败')
  }
}

async function handleDeleteConfig(profile: string) {
  const confirmed = await showAlertModal(`确定要删除配置「${profile}」吗？此操作不可撤销。`, undefined, {
    title: '删除 API 配置',
    type: 'danger',
    confirmText: '确认删除',
    cancelText: '取消',
  })

  if (!confirmed) {
    return
  }

  try {
    await apiStore.deleteApi(profile)
    if (!selectedApi.value && apis.value.length > 0) {
      apiStore.selectApi(apis.value[0].profile)
    }
    showSuccessToast(`已删除配置「${profile}」`, '删除成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '删除失败')
  }
}

async function handleOpenCreateDialog() {
  const canOpen = await confirmDiscardChanges()
  if (!canOpen) {
    return
  }

  showNewApiDialog.value = true
}

function handleApiCreated(api: ApiConfig) {
  showNewApiDialog.value = false
  apiStore.selectApi(api.profile)
  showSuccessToast(`已创建配置「${api.profile}」`, '创建成功')
}
</script>

<template>
  <div class="h-full min-h-0 w-full bg-gray-50 px-3 py-3 lg:px-4">
    <div class="mx-auto flex h-full min-h-0 max-w-7xl flex-col gap-3">
      <div class="flex flex-wrap items-center gap-2 rounded-[24px] border border-gray-200 bg-white p-2 shadow-sm">
        <button
          type="button"
          class="rounded-full px-4 py-2 text-sm font-medium transition"
          :class="activeTab === 'api' ? 'bg-gray-900 text-white' : 'text-gray-600 hover:bg-gray-100'"
          @click="activeTab = 'api'"
        >
          API 配置
        </button>
        <button
          type="button"
          class="rounded-full px-4 py-2 text-sm font-medium transition"
          :class="activeTab === 'ai-role' ? 'bg-gray-900 text-white' : 'text-gray-600 hover:bg-gray-100'"
          @click="activeTab = 'ai-role'"
        >
          AI 角色
        </button>
      </div>

      <div
        v-if="activeTab === 'api'"
        class="grid h-full min-h-0 grid-cols-1 gap-3 xl:grid-cols-[320px_minmax(0,1fr)]"
      >
      <section class="flex min-h-0 flex-col overflow-hidden rounded-[24px] border border-gray-200 bg-white p-3 shadow-sm">
        <div class="shrink-0 flex items-start justify-between gap-3">
          <div>
            <h2 class="text-lg font-semibold text-gray-900">API 配置</h2>
            <p class="mt-1 text-sm text-gray-500">管理端点、模型、凭证与默认配置。</p>
          </div>
          <span class="inline-flex shrink-0 items-center whitespace-nowrap rounded-full bg-gray-100 px-3 py-1 text-xs font-medium text-gray-600">
            {{ apis.length }} 个配置
          </span>
        </div>

        <div class="mt-3 shrink-0">
          <input
            v-model="searchQuery"
            type="text"
            class="w-full rounded-xl border border-gray-200 bg-gray-50 px-3.5 py-2.5 text-sm text-gray-900 transition focus:border-blue-500 focus:bg-white focus:outline-none focus:ring-4 focus:ring-blue-100"
            placeholder="搜索名称、端点或模型"
          />
        </div>

        <div class="mt-3 min-h-0 flex-1 overflow-hidden">
          <ApiList
            :selected-profile="selectedProfile"
            :search-query="searchQuery"
            @select="handleSelectApi"
            @copy="handleCopyConfig"
            @delete="handleDeleteConfig"
            @create="handleOpenCreateDialog"
          />
        </div>
      </section>

      <section class="thin-scrollbar min-h-0 overflow-y-auto rounded-[24px] border border-gray-200 bg-white p-4 shadow-sm">
        <div v-if="hasSelection && draft && selectedApi" class="flex min-h-full flex-col">
          <div class="flex flex-col gap-4 border-b border-gray-100 pb-5 lg:flex-row lg:items-start lg:justify-between">
            <div>
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="text-2xl font-semibold text-gray-900">{{ selectedApi.profile }}</h2>
                <span
                  class="rounded-full px-3 py-1 text-xs font-medium"
                  :class="selectedApi.enabled ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'"
                >
                  {{ selectedApi.enabled ? '已启用' : '未启用' }}
                </span>
                <span
                  v-if="selectedApi.default"
                  class="rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-700"
                >
                  默认配置
                </span>
                <span
                  v-if="dirty"
                  class="rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-700"
                >
                  有未保存修改
                </span>
              </div>
              <p class="mt-2 text-sm text-gray-500">
                当前端点：{{ endpointSummary }}
              </p>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-full border border-gray-200 px-4 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-50"
                @click="handleCopyConfig(selectedApi)"
              >
                <MdContentCopy class="h-4 w-4" />
                复制
              </button>
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-full border border-red-200 px-4 py-2 text-sm font-medium text-red-600 transition hover:bg-red-50"
                @click="handleDeleteConfig(selectedApi.profile)"
              >
                <MdDelete class="h-4 w-4" />
                删除
              </button>
            </div>
          </div>

          <div class="mt-5 grid flex-1 grid-cols-1 gap-4 2xl:grid-cols-2">
            <div class="rounded-2xl border border-gray-100 bg-gray-50 p-4">
              <h3 class="text-sm font-semibold text-gray-900">基本信息</h3>
              <div class="mt-4 space-y-4">
                <label class="block text-sm">
                  <span class="mb-2 block font-medium text-gray-700">配置名称</span>
                  <input
                    :value="draft.profile"
                    type="text"
                    class="w-full rounded-xl border border-gray-200 bg-white px-4 py-3 text-sm text-gray-900 transition focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                    placeholder="例如 OpenAI 主账号"
                    @input="patchDraftField('profile', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label class="block text-sm">
                  <span class="mb-2 block font-medium text-gray-700">Provider</span>
                  <select
                    :value="draft.provider"
                    class="w-full rounded-xl border border-gray-200 bg-white px-4 py-3 text-sm text-gray-900 transition focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                    @change="handleProviderChange(($event.target as HTMLSelectElement).value as ApiProvider)"
                  >
                    <option v-for="option in providerOptions" :key="option.value" :value="option.value">
                      {{ option.label }}
                    </option>
                  </select>
                </label>

                <label class="block text-sm">
                  <span class="mb-2 block font-medium text-gray-700">API 端点</span>
                  <input
                    :value="draft.base_url"
                    type="text"
                    class="w-full rounded-xl border border-gray-200 bg-white px-4 py-3 text-sm text-gray-900 transition focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                    placeholder="https://api.example.com/v1"
                    @input="patchDraftField('base_url', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <div class="rounded-2xl border border-gray-200 bg-white px-4 py-3 text-xs text-gray-500">
                  Base URL 将按 provider 分流；OpenAI Responses、Claude、Gemini 不再共用同一套端点约定。
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-gray-100 bg-gray-50 p-4">
              <h3 class="text-sm font-semibold text-gray-900">凭证与模型</h3>
              <div class="mt-4 space-y-4">
                <label class="block text-sm">
                  <span class="mb-2 block font-medium text-gray-700">API 密钥</span>
                  <input
                    :value="draft.api_key"
                    type="password"
                    class="w-full rounded-xl border border-gray-200 bg-white px-4 py-3 text-sm text-gray-900 transition focus:border-blue-500 focus:outline-none focus:ring-4 focus:ring-blue-100"
                    placeholder="sk-..."
                    @input="patchDraftField('api_key', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <div class="text-sm">
                  <span class="mb-2 block font-medium text-gray-700">模型</span>
                  <ModelSelect
                    :api-config="draft"
                    :model-value="draft.model"
                    @update:modelValue="patchDraftField('model', $event)"
                  />
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-gray-100 bg-gray-50 p-4">
              <h3 class="text-sm font-semibold text-gray-900">连通性</h3>
              <div class="mt-4 space-y-3">
                <div
                  v-if="saveError"
                  class="rounded-2xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700"
                >
                  {{ saveError }}
                </div>

                <div
                  v-if="lastTestResult"
                  class="rounded-2xl border px-4 py-3 text-sm"
                  :class="lastTestResult.success ? 'border-green-200 bg-green-50 text-green-700' : 'border-amber-200 bg-amber-50 text-amber-700'"
                >
                  <div>{{ lastTestResult.message }}</div>
                  <div v-if="lastTestResult.error" class="mt-2 whitespace-pre-wrap break-all text-xs opacity-80">
                    {{ lastTestResult.error }}
                  </div>
                </div>

                <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                  <button
                    type="button"
                    class="inline-flex items-center justify-center gap-2 rounded-2xl bg-blue-600 px-4 py-3 text-sm font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
                    :disabled="testing || !hasDraftCredentials"
                    @click="handleTestConnection"
                  >
                    <MdWifiTethering class="h-4 w-4" />
                    {{ testing ? '测试中...' : '测试连接' }}
                  </button>

                  <button
                    type="button"
                    class="rounded-2xl border px-4 py-3 text-sm font-medium transition disabled:cursor-not-allowed disabled:opacity-50"
                    :class="selectedApi.enabled ? 'border-gray-200 text-gray-700 hover:bg-gray-100' : 'border-green-200 text-green-700 hover:bg-green-50'"
                    :disabled="!canEnableDraft"
                    @click="handleToggleEnabled"
                  >
                    {{ selectedApi.enabled ? '禁用配置' : '启用配置' }}
                  </button>
                </div>

                <button
                  v-if="canSetDefault"
                  type="button"
                  class="w-full rounded-2xl border border-blue-200 bg-blue-50 px-4 py-3 text-sm font-medium text-blue-700 transition hover:bg-blue-100"
                  @click="handleSetDefault"
                >
                  设为默认配置
                </button>
              </div>
            </div>

            <div class="rounded-2xl border border-red-100 bg-red-50 p-4">
              <h3 class="text-sm font-semibold text-red-700">危险操作提示</h3>
              <p class="mt-3 text-sm text-red-600">
                删除会永久移除该配置；禁用配置会同时取消默认状态。若刚修改过端点、密钥或模型，请重新测试连接后再启用。
              </p>
            </div>
          </div>

          <div class="mt-5 flex flex-col gap-3 border-t border-gray-100 pt-5 sm:flex-row sm:items-center sm:justify-between">
            <p class="text-sm text-gray-500">
              {{ dirty ? '有未保存修改，建议先保存再切换或设默认。' : '当前草稿与已保存配置保持一致。' }}
            </p>

            <div class="flex flex-wrap items-center gap-2">
              <button
                type="button"
                class="rounded-full border border-gray-200 px-4 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
                :disabled="!dirty || saving"
                @click="handleDiscardDraft"
              >
                放弃修改
              </button>
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-full bg-gray-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-black disabled:cursor-not-allowed disabled:opacity-50"
                :disabled="!dirty || saving"
                @click="handleSaveDraft"
              >
                <MdSave class="h-4 w-4" />
                {{ saving ? '保存中...' : '保存配置' }}
              </button>
            </div>
          </div>
        </div>

        <div v-else class="flex min-h-full min-h-[520px] flex-col items-center justify-center rounded-3xl border border-dashed border-gray-200 bg-gray-50 px-6 text-center">
          <h2 class="text-xl font-semibold text-gray-900">选择一个 API 配置</h2>
          <p class="mt-2 max-w-md text-sm text-gray-500">
            从左侧选择已有配置开始编辑，或者先创建一个新的 API 配置。新的配置会以未启用状态创建，建议先填写信息并完成连接测试。
          </p>
          <button
            type="button"
            class="mt-6 rounded-full bg-blue-600 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-blue-700"
            @click="handleOpenCreateDialog"
          >
            创建第一个配置
          </button>
        </div>
      </section>

      </div>

      <AIRoleSettingsPanel v-else />
    </div>

    <NewApiDialog
      v-if="showNewApiDialog"
      @created="handleApiCreated"
      @cancel="showNewApiDialog = false"
    />
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
