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

function patchDraftField(field: keyof ApiConfig, value: string | boolean | number) {
  apiStore.patchDraft({ [field]: value } as Partial<ApiConfig>)
}

function handleModelMetaUpdate(modelMeta: { max_tokens?: number; context_window?: number } | null) {
  if (!modelMeta) {
    return
  }

  const patch: Partial<ApiConfig> = {}

  if (typeof modelMeta.max_tokens === 'number') {
    patch.max_tokens = modelMeta.max_tokens
  }

  if (typeof modelMeta.context_window === 'number') {
    patch.context_window = modelMeta.context_window
  }

  if (Object.keys(patch).length > 0) {
    apiStore.patchDraft(patch)
  }
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
  <div class="h-full min-h-0 w-full overflow-y-auto px-3 py-3 lg:px-4">
    <div class="mx-auto flex h-full min-h-0 max-w-7xl flex-col gap-3">
      <!-- Tab 切换 -->
      <div class="liquid-panel flex flex-wrap items-center gap-2 p-2">
        <button
          type="button"
          class="rounded-full px-4 py-2 text-sm font-medium transition"
          :class="activeTab === 'api' ? 'bg-white/15 text-white/95 border border-white/20' : 'text-white/50 hover:bg-white/8 hover:text-white/75'"
          @click="activeTab = 'api'"
        >
          API 配置
        </button>
        <button
          type="button"
          class="rounded-full px-4 py-2 text-sm font-medium transition"
          :class="activeTab === 'ai-role' ? 'bg-white/15 text-white/95 border border-white/20' : 'text-white/50 hover:bg-white/8 hover:text-white/75'"
          @click="activeTab = 'ai-role'"
        >
          AI 角色
        </button>
      </div>

      <div
        v-if="activeTab === 'api'"
        class="grid min-h-0 flex-1 grid-cols-1 gap-3 xl:grid-cols-[320px_minmax(0,1fr)]"
      >
        <!-- 左侧 API 列表 -->
        <section class="liquid-panel flex min-h-0 flex-col overflow-hidden p-3">
          <div class="flex shrink-0 items-start justify-between gap-3">
            <div>
              <h2 class="text-lg font-semibold text-white/90">API 配置</h2>
              <p class="mt-1 text-sm text-white/45">管理端点、模型、凭证与默认配置。</p>
            </div>
            <span class="liquid-badge">{{ apis.length }} 个配置</span>
          </div>

          <div class="mt-3 shrink-0">
            <input
              v-model="searchQuery"
              type="text"
              class="liquid-input"
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

        <!-- 右侧详情 -->
        <section class="liquid-panel-elevated min-h-0 overflow-y-auto p-4">
          <div v-if="hasSelection && draft && selectedApi" class="flex min-h-full flex-col">
            <!-- 配置头部 -->
            <div class="flex flex-col gap-4 border-b border-white/8 pb-5 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <div class="flex flex-wrap items-center gap-2">
                  <h2 class="text-2xl font-semibold text-white/90">{{ selectedApi.profile }}</h2>
                  <span class="liquid-badge" :class="selectedApi.enabled ? 'liquid-badge--success' : ''">
                    {{ selectedApi.enabled ? '已启用' : '未启用' }}
                  </span>
                  <span v-if="selectedApi.default" class="liquid-badge liquid-badge--primary">默认配置</span>
                  <span v-if="dirty" class="liquid-badge liquid-badge--warning">有未保存修改</span>
                </div>
                <p class="mt-2 text-sm text-white/40">当前端点：{{ endpointSummary }}</p>
              </div>

              <div class="flex flex-wrap items-center gap-2">
                <button type="button" class="glass-btn glass-btn--neutral" @click="handleCopyConfig(selectedApi)">
                  <MdContentCopy class="h-4 w-4" /> 复制
                </button>
                <button type="button" class="glass-btn glass-btn--danger" @click="handleDeleteConfig(selectedApi.profile)">
                  <MdDelete class="h-4 w-4" /> 删除
                </button>
              </div>
            </div>

            <!-- 配置表单 -->
            <div class="mt-5 grid flex-1 grid-cols-1 gap-4 2xl:grid-cols-2">
              <!-- 基本信息 -->
              <div class="settings-card">
                <h3 class="settings-card-title">基本信息</h3>
                <div class="mt-4 space-y-4">
                  <label class="block text-sm">
                    <span class="settings-label">配置名称</span>
                    <input :value="draft.profile" type="text" class="liquid-input mt-2" placeholder="例如 OpenAI 主账号"
                      @input="patchDraftField('profile', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="block text-sm">
                    <span class="settings-label">Provider</span>
                    <select :value="draft.provider" class="liquid-input mt-2"
                      @change="handleProviderChange(($event.target as HTMLSelectElement).value as ApiProvider)">
                      <option v-for="option in providerOptions" :key="option.value" :value="option.value" class="bg-slate-900">
                        {{ option.label }}
                      </option>
                    </select>
                  </label>
                  <label class="block text-sm">
                    <span class="settings-label">API 端点</span>
                    <input :value="draft.base_url" type="text" class="liquid-input mt-2" placeholder="https://api.example.com/v1"
                      @input="patchDraftField('base_url', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <div class="rounded-xl border border-white/8 bg-white/4 px-4 py-3 text-xs text-white/35">
                    Base URL 将按 provider 分流；OpenAI Responses、Claude、Gemini 不再共用同一套端点约定。
                  </div>
                </div>
              </div>

              <!-- 凭证与模型 -->
              <div class="settings-card">
                <h3 class="settings-card-title">凭证与模型</h3>
                <div class="mt-4 space-y-4">
                  <label class="block text-sm">
                    <span class="settings-label">API 密钥</span>
                    <input :value="draft.api_key" type="password" class="liquid-input mt-2" placeholder="sk-..."
                      @input="patchDraftField('api_key', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <div class="text-sm">
                    <span class="settings-label">模型</span>
                    <ModelSelect :api-config="draft" :model-value="draft.model" class="mt-2"
                      @update:modelValue="patchDraftField('model', $event)"
                      @update:modelMeta="handleModelMetaUpdate" />
                  </div>
                  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <label class="block text-sm">
                      <span class="settings-label">Context Window</span>
                      <input :value="draft.context_window" type="number" min="1" step="1" class="liquid-input mt-2"
                        @input="patchDraftField('context_window', Number(($event.target as HTMLInputElement).value) || 65534)" />
                      <p class="mt-2 text-xs text-white/30">默认 65534，可手动覆盖。</p>
                    </label>
                    <label class="block text-sm">
                      <span class="settings-label">Max Tokens</span>
                      <input :value="draft.max_tokens" type="number" min="1" step="1" class="liquid-input mt-2"
                        @input="patchDraftField('max_tokens', Number(($event.target as HTMLInputElement).value) || 8192)" />
                      <p class="mt-2 text-xs text-white/30">默认 8192，可手动覆盖。</p>
                    </label>
                  </div>
                </div>
              </div>

              <!-- 连通性 -->
              <div class="settings-card">
                <h3 class="settings-card-title">连通性</h3>
                <div class="mt-4 space-y-3">
                  <div v-if="saveError" class="rounded-xl border border-red-400/25 bg-red-500/12 px-4 py-3 text-sm text-red-300">
                    {{ saveError }}
                  </div>
                  <div v-if="lastTestResult" class="rounded-xl border px-4 py-3 text-sm"
                    :class="lastTestResult.success ? 'border-emerald-400/25 bg-emerald-500/12 text-emerald-300' : 'border-yellow-400/25 bg-yellow-500/12 text-yellow-300'"
                  >
                    <div>{{ lastTestResult.message }}</div>
                    <div v-if="lastTestResult.error" class="mt-2 whitespace-pre-wrap break-all text-xs opacity-75">
                      {{ lastTestResult.error }}
                    </div>
                  </div>
                  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <button type="button" class="glass-btn glass-btn--primary justify-center py-3 disabled:opacity-40 disabled:cursor-not-allowed"
                      :disabled="testing || !hasDraftCredentials" @click="handleTestConnection">
                      <MdWifiTethering class="h-4 w-4" />
                      {{ testing ? '测试中...' : '测试连接' }}
                    </button>
                    <button type="button"
                      class="glass-btn justify-center py-3 disabled:opacity-40 disabled:cursor-not-allowed"
                      :class="selectedApi.enabled ? 'glass-btn--neutral' : 'glass-btn--primary'"
                      :disabled="!canEnableDraft" @click="handleToggleEnabled">
                      {{ selectedApi.enabled ? '禁用配置' : '启用配置' }}
                    </button>
                  </div>
                  <button v-if="canSetDefault" type="button" class="glass-btn glass-btn--primary w-full justify-center py-3" @click="handleSetDefault">
                    设为默认配置
                  </button>
                </div>
              </div>

              <!-- 危险操作 -->
              <div class="settings-card settings-card--danger">
                <h3 class="text-sm font-semibold text-red-300">危险操作提示</h3>
                <p class="mt-3 text-sm text-red-400/75">
                  删除会永久移除该配置；禁用配置会同时取消默认状态。若刚修改过端点、密钥或模型，请重新测试连接后再启用。
                </p>
              </div>
            </div>

            <!-- 底部保存栏 -->
            <div class="mt-5 flex flex-col gap-3 border-t border-white/8 pt-5 sm:flex-row sm:items-center sm:justify-between">
              <p class="text-sm text-white/35">
                {{ dirty ? '有未保存修改，建议先保存再切换或设默认。' : '当前草稿与已保存配置保持一致。' }}
              </p>
              <div class="flex flex-wrap items-center gap-2">
                <button type="button" class="glass-btn glass-btn--neutral disabled:opacity-40 disabled:cursor-not-allowed"
                  :disabled="!dirty || saving" @click="handleDiscardDraft">
                  放弃修改
                </button>
                <button type="button" class="glass-btn glass-btn--primary disabled:opacity-40 disabled:cursor-not-allowed"
                  :disabled="!dirty || saving" @click="handleSaveDraft">
                  <MdSave class="h-4 w-4" />
                  {{ saving ? '保存中...' : '保存配置' }}
                </button>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-else class="flex min-h-[520px] flex-col items-center justify-center rounded-3xl border border-dashed border-white/12 bg-white/3 px-6 text-center">
            <h2 class="text-xl font-semibold text-white/75">选择一个 API 配置</h2>
            <p class="mt-2 max-w-md text-sm text-white/40">
              从左侧选择已有配置开始编辑，或者先创建一个新的 API 配置。
            </p>
            <button type="button" class="glass-btn glass-btn--primary mt-6" @click="handleOpenCreateDialog">
              创建第一个配置
            </button>
          </div>
        </section>
      </div>

      <AIRoleSettingsPanel v-else />
    </div>

    <NewApiDialog v-if="showNewApiDialog" @created="handleApiCreated" @cancel="showNewApiDialog = false" />
  </div>
</template>

<style scoped>
.settings-card {
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.10);
  background: rgba(255, 255, 255, 0.05);
  padding: 1rem;
  backdrop-filter: blur(16px);
}
.settings-card--danger {
  border-color: rgba(239, 68, 68, 0.20);
  background: rgba(239, 68, 68, 0.08);
}
.settings-card-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.80);
}
.settings-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.55);
}
</style>
