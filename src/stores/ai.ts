import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { ChatMessage } from '@/types/api'
import type { CommandMetadata, CommandResult } from '@/types/commands'
import { invoke } from '@tauri-apps/api/core'
import { backendCommandService } from '@/services/backendCommandService'
import { AIConfigService, type AIRole, type AIRoleEntry } from '@/services/aiConfig'

const SESSION_WAIT_INTERVAL_MS = 50
const SESSION_WAIT_TIMEOUT_MS = 2000
const AI_RESPONSE_INTERRUPTED_ERROR = 'AI 响应已中断'

export const useAiStore = defineStore('ai', () => {
  const isLoading = ref(false)
  const isStopping = ref(false)
  const isBackendSessionActive = ref(false)
  const currentSessionUUID = ref<string>('')
  const lastTokenStats = ref<any>(null)

  const selectedRole = ref('')
  const aiRoles = ref<AIRoleEntry[]>([])
  const currentRoleConfig = ref<AIRole | null>(null)
  const defaultRole = ref('')
  const rolesLoaded = ref(false)

  function invalidateCommandAvailability() {
    backendCommandService.clearCache()
  }

  function syncCurrentRoleConfig(roleId: string) {
    currentRoleConfig.value = aiRoles.value.find((roleEntry) => roleEntry.id === roleId)?.role ?? null
  }

  function getEffectiveRoleId() {
    return selectedRole.value || defaultRole.value || aiRoles.value[0]?.id || ''
  }

  function isInterruptedError(error: unknown) {
    if (error instanceof Error) {
      return error.message.includes(AI_RESPONSE_INTERRUPTED_ERROR)
    }

    return String(error).includes(AI_RESPONSE_INTERRUPTED_ERROR)
  }

  async function loadCharacterSession(uuid: string) {
    try {
      isLoading.value = true
      await invoke('load_character_session', { uuid })
      invalidateCommandAvailability()
    } catch (error) {
      console.error('加载角色会话失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function ensureCharacterSession(uuid: string, timeoutMs = SESSION_WAIT_TIMEOUT_MS) {
    if (isBackendSessionActive.value && currentSessionUUID.value === uuid) {
      return
    }

    await loadCharacterSession(uuid)

    const startedAt = Date.now()
    while (Date.now() - startedAt < timeoutMs) {
      if (isBackendSessionActive.value && currentSessionUUID.value === uuid) {
        return
      }

      await new Promise((resolve) => setTimeout(resolve, SESSION_WAIT_INTERVAL_MS))
    }

    throw new Error('后端会话加载超时')
  }

  async function sendChatMessage(message: string) {
    try {
      isLoading.value = true
      isStopping.value = false
      const roleId = getEffectiveRoleId() || null
      await invoke('send_chat_message', { message, roleId })
      invalidateCommandAvailability()
    } catch (error) {
      if (isInterruptedError(error)) {
        return
      }
      console.error('发送消息失败:', error)
      throw error
    } finally {
      isLoading.value = false
      isStopping.value = false
    }
  }

  async function editChatMessage(index: number, newContent: string) {
    try {
      isLoading.value = true
      await invoke('edit_chat_message', { index, newContent })
      invalidateCommandAvailability()
    } catch (error) {
      console.error('编辑消息失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function deleteChatMessage(index: number) {
    try {
      isLoading.value = true
      await invoke('delete_chat_message', { index })
      invalidateCommandAvailability()
    } catch (error) {
      console.error('删除消息失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function regenerateLastMessage() {
    try {
      isLoading.value = true
      isStopping.value = false
      const roleId = getEffectiveRoleId() || null
      await invoke('regenerate_last_message', { roleId })
      invalidateCommandAvailability()
    } catch (error) {
      if (isInterruptedError(error)) {
        return
      }
      console.error('重新生成失败:', error)
      throw error
    } finally {
      isLoading.value = false
      isStopping.value = false
    }
  }

  async function continueChat() {
    try {
      isLoading.value = true
      isStopping.value = false
      const roleId = getEffectiveRoleId() || null
      await invoke('continue_chat', { roleId })
      invalidateCommandAvailability()
    } catch (error) {
      if (isInterruptedError(error)) {
        return
      }
      console.error('继续对话失败:', error)
      throw error
    } finally {
      isLoading.value = false
      isStopping.value = false
    }
  }

  async function interruptResponse(): Promise<boolean> {
    if (!currentSessionUUID.value) {
      return false
    }

    try {
      isStopping.value = true
      const interrupted = await invoke<boolean>('interrupt_ai_response', {
        uuid: currentSessionUUID.value,
      })

      if (interrupted) {
        isLoading.value = false
        invalidateCommandAvailability()
      }

      return interrupted
    } catch (error) {
      console.error('中断AI响应失败:', error)
      throw error
    } finally {
      isStopping.value = false
    }
  }

  async function loadChatHistory(characterId: string): Promise<ChatMessage[]> {
    try {
      isLoading.value = true
      return await invoke<ChatMessage[]>('load_chat_history', { characterId })
    } catch (error) {
      console.error('加载聊天历史失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  async function loadAIRoles(forceRefresh = false): Promise<AIRoleEntry[]> {
    if (!forceRefresh && rolesLoaded.value && aiRoles.value.length > 0) {
      return aiRoles.value
    }

    try {
      const config = await AIConfigService.getConfig()
      defaultRole.value = config.default_role
      aiRoles.value = await AIConfigService.getAllRoles()
      rolesLoaded.value = true

      const roleExists = aiRoles.value.some((roleEntry) => roleEntry.id === selectedRole.value)

      if (!selectedRole.value || !roleExists) {
        selectedRole.value = config.default_role || aiRoles.value[0]?.id || ''
      }

      syncCurrentRoleConfig(selectedRole.value)
      return aiRoles.value
    } catch (error) {
      console.error('加载AI角色配置失败:', error)
      throw error
    }
  }

  async function selectRole(roleId: string) {
    if (!rolesLoaded.value) {
      await loadAIRoles()
    }

    selectedRole.value = roleId
    syncCurrentRoleConfig(roleId)
  }

  async function getAvailableCommands(forceRefresh = false): Promise<CommandMetadata[]> {
    return await backendCommandService.getCommands(forceRefresh)
  }

  async function searchCommands(query: string): Promise<CommandMetadata[]> {
    return await backendCommandService.searchCommands(query)
  }

  async function executeCommand(commandId: string, userInput?: string): Promise<CommandResult> {
    return await backendCommandService.executeCommand(commandId, userInput)
  }

  function clearCommandCache(): void {
    invalidateCommandAvailability()
  }

  function updateSessionState(uuid: string, active: boolean) {
    currentSessionUUID.value = uuid
    isBackendSessionActive.value = active
    invalidateCommandAvailability()
  }

  function clearSessionState() {
    currentSessionUUID.value = ''
    isBackendSessionActive.value = false
    invalidateCommandAvailability()
  }

  function updateTokenStats(stats: any) {
    lastTokenStats.value = stats
  }

  function clearTokenStats() {
    lastTokenStats.value = null
  }

  function reset() {
    isLoading.value = false
    isStopping.value = false
    isBackendSessionActive.value = false
    currentSessionUUID.value = ''
    lastTokenStats.value = null
    selectedRole.value = ''
    aiRoles.value = []
    currentRoleConfig.value = null
    defaultRole.value = ''
    rolesLoaded.value = false
    invalidateCommandAvailability()
  }

  return {
    isLoading: readonly(isLoading),
    isStopping: readonly(isStopping),
    isBackendSessionActive: readonly(isBackendSessionActive),
    currentSessionUUID: readonly(currentSessionUUID),
    lastTokenStats: readonly(lastTokenStats),
    selectedRole: readonly(selectedRole),
    aiRoles: readonly(aiRoles),
    currentRoleConfig: readonly(currentRoleConfig),
    defaultRole: readonly(defaultRole),

    loadCharacterSession,
    ensureCharacterSession,
    sendChatMessage,
    editChatMessage,
    deleteChatMessage,
    regenerateLastMessage,
    continueChat,
    interruptResponse,
    loadChatHistory,
    loadAIRoles,
    selectRole,

    getAvailableCommands,
    searchCommands,
    executeCommand,
    clearCommandCache,

    updateSessionState,
    clearSessionState,
    updateTokenStats,
    clearTokenStats,

    reset,
  }
})
