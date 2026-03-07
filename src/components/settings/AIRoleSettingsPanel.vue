<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAiStore } from '@/stores/ai'
import { AIConfigService, type AIRole } from '@/services/aiConfig'
import { useModal } from '@/composables/useModal'
import { useNotification } from '@/composables/useNotification'

const aiStore = useAiStore()
const { aiRoles, defaultRole } = storeToRefs(aiStore)
const { showAlertModal } = useModal()
const { showErrorToast, showInfoToast, showSuccessToast, showWarningToast } = useNotification()

const selectedRoleId = ref('')
const draftRole = ref<AIRole | null>(null)
const savedRole = ref<AIRole | null>(null)
const isCreatingRole = ref(false)
const savingRole = ref(false)

function createEmptyRoleDraft(): AIRole {
  return {
    name: '新 AI 角色',
    description: '',
    system_prompt: '',
    temperature: 0.7,
    max_tokens: 2000,
    tools_enabled: true,
    context_role_template: '角色卡编写助手',
    context_task_template: '帮助用户分析、创作和完善角色设定，结合角色卡与世界书提供建议。',
    context_instructions_template:
      '保持角色设定一致性，优先响应用户当前需求。必要时再调用工具修改角色卡或世界书。',
  }
}

function cloneRole(role: AIRole | null): AIRole | null {
  return role ? JSON.parse(JSON.stringify(role)) : null
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}

const selectedRoleEntry = computed(() => {
  return aiRoles.value.find((roleEntry) => roleEntry.id === selectedRoleId.value) ?? null
})

const roleDirty = computed(() => {
  return JSON.stringify(draftRole.value) !== JSON.stringify(savedRole.value)
})

const canSaveRole = computed(() => {
  return !!draftRole.value?.name.trim() && (isCreatingRole.value || roleDirty.value)
})

const canDeleteRole = computed(() => {
  return !!selectedRoleId.value && selectedRoleId.value !== defaultRole.value && !isCreatingRole.value
})

const canSetDefaultRole = computed(() => {
  return !!selectedRoleId.value && selectedRoleId.value !== defaultRole.value && !isCreatingRole.value
})

function syncEditorFromSelected(roleId?: string) {
  const nextRoleId = roleId ?? selectedRoleId.value
  const nextRole = aiRoles.value.find((roleEntry) => roleEntry.id === nextRoleId)?.role ?? null

  if (!nextRole) {
    draftRole.value = null
    savedRole.value = null
    return
  }

  selectedRoleId.value = nextRoleId
  draftRole.value = cloneRole(nextRole)
  savedRole.value = cloneRole(nextRole)
  isCreatingRole.value = false
}

async function loadRoles(forceRefresh = false, preferredRoleId?: string) {
  await aiStore.loadAIRoles(forceRefresh)

  const candidateRoleId =
    preferredRoleId ??
    (selectedRoleId.value && aiRoles.value.some((roleEntry) => roleEntry.id === selectedRoleId.value)
      ? selectedRoleId.value
      : defaultRole.value || aiRoles.value[0]?.id || '')

  if (candidateRoleId) {
    syncEditorFromSelected(candidateRoleId)
  } else {
    draftRole.value = null
    savedRole.value = null
    selectedRoleId.value = ''
  }
}

async function confirmDiscardRoleChanges() {
  if (!roleDirty.value && !isCreatingRole.value) {
    return true
  }

  return await showAlertModal('当前有未保存的角色修改，是否放弃这些修改？', undefined, {
    title: '放弃角色修改',
    type: 'warning',
    confirmText: '放弃修改',
    cancelText: '继续编辑',
  })
}

async function handleSelectRole(roleId: string) {
  if (roleId === selectedRoleId.value && !isCreatingRole.value) {
    return
  }

  const canSwitch = await confirmDiscardRoleChanges()
  if (!canSwitch) {
    return
  }

  syncEditorFromSelected(roleId)
}

async function handleCreateRole() {
  const canSwitch = await confirmDiscardRoleChanges()
  if (!canSwitch) {
    return
  }

  selectedRoleId.value = ''
  draftRole.value = createEmptyRoleDraft()
  savedRole.value = null
  isCreatingRole.value = true
}

function patchDraftField<K extends keyof AIRole>(field: K, value: AIRole[K]) {
  if (!draftRole.value) {
    return
  }

  draftRole.value = {
    ...draftRole.value,
    [field]: value,
  }
}

async function handleSaveRole() {
  if (!draftRole.value) {
    return
  }

  savingRole.value = true
  try {
    const roleToSave = cloneRole(draftRole.value)
    if (!roleToSave) {
      return
    }

    if (isCreatingRole.value) {
      const newRoleId = await AIConfigService.createRole(roleToSave)
      await loadRoles(true, newRoleId)
      showSuccessToast(`已创建角色「${roleToSave.name}」`, '创建成功')
      return
    }

    await AIConfigService.updateRole(selectedRoleId.value, roleToSave)
    await loadRoles(true, selectedRoleId.value)
    showSuccessToast(`已保存角色「${roleToSave.name}」`, '保存成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '保存失败')
  } finally {
    savingRole.value = false
  }
}

function handleDiscardChanges() {
  if (isCreatingRole.value) {
    const fallbackRoleId = defaultRole.value || aiRoles.value[0]?.id || ''
    if (fallbackRoleId) {
      syncEditorFromSelected(fallbackRoleId)
    } else {
      draftRole.value = null
      savedRole.value = null
      isCreatingRole.value = false
    }
    showInfoToast('已取消新角色创建', '已撤销')
    return
  }

  draftRole.value = cloneRole(savedRole.value)
  showInfoToast('已还原未保存的角色修改', '已撤销')
}

async function handleDeleteRole() {
  if (!selectedRoleEntry.value) {
    return
  }

  if (selectedRoleId.value === defaultRole.value) {
    showWarningToast('默认角色不能直接删除，请先切换默认角色。', '无法删除')
    return
  }

  const confirmed = await showAlertModal(
    `确定删除角色「${selectedRoleEntry.value.role.name}」吗？该操作不可撤销。`,
    undefined,
    {
      title: '删除 AI 角色',
      type: 'danger',
      confirmText: '删除角色',
      cancelText: '取消',
    },
  )

  if (!confirmed) {
    return
  }

  try {
    const deletedRoleId = selectedRoleId.value
    await AIConfigService.deleteRole(deletedRoleId)
    const nextRoleId = aiRoles.value.find((roleEntry) => roleEntry.id !== deletedRoleId)?.id
    await loadRoles(true, nextRoleId)
    showSuccessToast('角色已删除', '删除成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '删除失败')
  }
}

async function handleSetDefaultRole() {
  if (!selectedRoleId.value) {
    return
  }

  try {
    await AIConfigService.setDefaultRole(selectedRoleId.value)
    await loadRoles(true, selectedRoleId.value)
    showSuccessToast('默认角色已更新', '设置成功')
  } catch (error) {
    showErrorToast(getErrorMessage(error), '设置失败')
  }
}

onMounted(() => {
  void loadRoles()
})
</script>

<template>
  <section class="grid h-[calc(100vh-220px)] min-h-[560px] grid-cols-1 gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
    <aside class="flex min-h-0 flex-col rounded-3xl border border-gray-200 bg-white p-4 shadow-sm">
      <div class="flex items-center justify-between gap-3 border-b border-gray-100 pb-4">
        <div>
          <h2 class="text-lg font-semibold text-gray-900">AI 角色</h2>
          <p class="mt-1 text-sm text-gray-500">管理聊天主 system prompt 与上下文模板。</p>
        </div>

        <button
          type="button"
          class="inline-flex shrink-0 items-center whitespace-nowrap rounded-full bg-gray-900 px-4 py-2.5 text-sm font-medium leading-none text-white transition hover:bg-black"
          @click="handleCreateRole"
        >
          新建角色
        </button>
      </div>

      <div class="thin-scrollbar mt-4 flex-1 space-y-3 overflow-y-auto pr-1">
        <button
          v-for="roleEntry in aiRoles"
          :key="roleEntry.id"
          type="button"
          class="w-full rounded-2xl border p-4 text-left transition"
          :class="roleEntry.id === selectedRoleId ? 'border-blue-300 bg-blue-50 shadow-sm' : 'border-gray-200 bg-white hover:border-gray-300 hover:bg-gray-50'"
          @click="handleSelectRole(roleEntry.id)"
        >
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-sm font-semibold text-gray-900">{{ roleEntry.role.name }}</div>
              <p class="mt-1 line-clamp-2 text-xs text-gray-500">{{ roleEntry.role.description || '暂无描述' }}</p>
            </div>

            <div class="flex shrink-0 flex-wrap items-center justify-end gap-2">
              <span
                v-if="roleEntry.id === defaultRole"
                class="inline-flex items-center whitespace-nowrap rounded-full bg-blue-100 px-2.5 py-1 text-[11px] font-medium leading-none text-blue-700"
              >
                默认
              </span>
              <span
                :class="roleEntry.role.tools_enabled ? 'bg-emerald-100 text-emerald-700' : 'bg-gray-100 text-gray-600'"
                class="inline-flex items-center whitespace-nowrap rounded-full px-2.5 py-1 text-[11px] font-medium leading-none"
              >
                {{ roleEntry.role.tools_enabled ? '工具开启' : '工具关闭' }}
              </span>
            </div>
          </div>
        </button>
      </div>
    </aside>

    <div class="min-h-0 rounded-3xl border border-gray-200 bg-white p-5 shadow-sm">
      <div v-if="draftRole" class="flex h-full min-h-0 flex-col">
        <div class="flex flex-wrap items-start justify-between gap-3 border-b border-gray-100 pb-4">
          <div>
            <h2 class="text-xl font-semibold text-gray-900">
              {{ isCreatingRole ? '创建 AI 角色' : draftRole.name || '编辑 AI 角色' }}
            </h2>
            <p class="mt-1 text-sm text-gray-500">
              主系统提示词会作为第一层 system prompt；上下文模板会作为第二层结构化辅助上下文。
            </p>
          </div>

          <div class="flex flex-wrap gap-2">
            <button
              v-if="canSetDefaultRole"
              type="button"
              class="rounded-full border border-blue-200 bg-blue-50 px-4 py-2 text-sm font-medium text-blue-700 transition hover:bg-blue-100"
              @click="handleSetDefaultRole"
            >
              设为默认
            </button>
            <button
              v-if="canDeleteRole"
              type="button"
              class="rounded-full border border-red-200 bg-red-50 px-4 py-2 text-sm font-medium text-red-700 transition hover:bg-red-100"
              @click="handleDeleteRole"
            >
              删除角色
            </button>
          </div>
        </div>

        <div class="thin-scrollbar mt-5 flex-1 space-y-5 overflow-y-auto pr-1">
          <div class="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_240px]">
            <label class="block">
              <span class="text-sm font-medium text-gray-700">显示名</span>
              <input
                :value="draftRole.name"
                type="text"
                class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
                @input="patchDraftField('name', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <label class="block">
              <span class="text-sm font-medium text-gray-700">内部 ID</span>
              <input
                :value="selectedRoleId || '保存后自动生成'"
                type="text"
                readonly
                class="mt-2 w-full rounded-2xl border border-gray-200 bg-gray-50 px-4 py-3 text-sm text-gray-500 outline-none"
              />
            </label>
          </div>

          <label class="block">
            <span class="text-sm font-medium text-gray-700">描述</span>
            <textarea
              :value="draftRole.description"
              rows="3"
              class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
              @input="patchDraftField('description', ($event.target as HTMLTextAreaElement).value)"
            />
          </label>

          <div class="rounded-3xl border border-gray-100 bg-gray-50 p-4">
            <h3 class="text-sm font-semibold text-gray-900">聊天行为参数</h3>
            <div class="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-[160px_160px_minmax(0,1fr)]">
              <label class="block">
                <span class="text-sm font-medium text-gray-700">Temperature</span>
                <input
                  :value="draftRole.temperature"
                  type="number"
                  min="0"
                  max="2"
                  step="0.1"
                  class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
                  @input="patchDraftField('temperature', Number(($event.target as HTMLInputElement).value) || 0)"
                />
              </label>

              <label class="block">
                <span class="text-sm font-medium text-gray-700">Max Tokens</span>
                <input
                  :value="draftRole.max_tokens"
                  type="number"
                  min="1"
                  step="1"
                  class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
                  @input="patchDraftField('max_tokens', Number(($event.target as HTMLInputElement).value) || 1)"
                />
              </label>

              <label class="flex items-center justify-between rounded-2xl border border-gray-200 bg-white px-4 py-3">
                <div>
                  <span class="text-sm font-medium text-gray-700">工具自动调用</span>
                  <p class="mt-1 text-xs text-gray-500">关闭后不会声明工具，也不会进入工具调用链。</p>
                </div>
                <input
                  :checked="draftRole.tools_enabled"
                  type="checkbox"
                  class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  @change="patchDraftField('tools_enabled', ($event.target as HTMLInputElement).checked)"
                />
              </label>
            </div>
          </div>

          <label class="block">
            <span class="text-sm font-medium text-gray-700">主系统提示词</span>
            <textarea
              :value="draftRole.system_prompt"
              rows="7"
              class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
              @input="patchDraftField('system_prompt', ($event.target as HTMLTextAreaElement).value)"
            />
          </label>

          <div class="grid grid-cols-1 gap-5 xl:grid-cols-3">
            <label class="block">
              <span class="text-sm font-medium text-gray-700">上下文角色模板</span>
              <textarea
                :value="draftRole.context_role_template"
                rows="8"
                class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
                @input="patchDraftField('context_role_template', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>

            <label class="block">
              <span class="text-sm font-medium text-gray-700">上下文任务模板</span>
              <textarea
                :value="draftRole.context_task_template"
                rows="8"
                class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
                @input="patchDraftField('context_task_template', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>

            <label class="block">
              <span class="text-sm font-medium text-gray-700">上下文指令模板</span>
              <textarea
                :value="draftRole.context_instructions_template"
                rows="8"
                class="mt-2 w-full rounded-2xl border border-gray-200 px-4 py-3 text-sm text-gray-900 outline-none transition focus:border-blue-400 focus:ring-2 focus:ring-blue-100"
                @input="patchDraftField('context_instructions_template', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
          </div>
        </div>

        <div class="mt-5 flex flex-col gap-3 border-t border-gray-100 pt-5 sm:flex-row sm:items-center sm:justify-between">
          <p class="text-sm text-gray-500">
            {{ roleDirty || isCreatingRole ? '当前角色有未保存修改，保存后会立即同步到聊天面板。' : '当前角色配置已保存。' }}
          </p>

          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              class="rounded-full border border-gray-200 px-4 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="(!roleDirty && !isCreatingRole) || savingRole"
              @click="handleDiscardChanges"
            >
              放弃修改
            </button>
            <button
              type="button"
              class="rounded-full bg-gray-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-black disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="!canSaveRole || savingRole"
              @click="handleSaveRole"
            >
              {{ savingRole ? '保存中...' : isCreatingRole ? '创建角色' : '保存角色' }}
            </button>
          </div>
        </div>
      </div>

      <div v-else class="flex h-full min-h-[520px] flex-col items-center justify-center rounded-3xl border border-dashed border-gray-200 bg-gray-50 px-6 text-center">
        <h2 class="text-xl font-semibold text-gray-900">选择一个 AI 角色</h2>
        <p class="mt-2 max-w-md text-sm text-gray-500">
          从左侧选择已有角色开始编辑，或者新建一个角色来管理主 system prompt 与上下文模板。
        </p>
        <button
          type="button"
          class="mt-6 rounded-full bg-blue-600 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-blue-700"
          @click="handleCreateRole"
        >
          创建第一个角色
        </button>
      </div>
    </div>
  </section>
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
