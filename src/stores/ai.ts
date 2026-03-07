import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { ChatMessage } from '@/types/api'
import type { CommandMetadata, CommandResult } from '@/types/commands'
import { invoke } from '@tauri-apps/api/core'
import { backendCommandService } from '@/services/backendCommandService'
import { AIConfigService, type AIRole } from '@/services/aiConfig'

type StoredRole = {
  name: string
  role: AIRole
}

const SESSION_WAIT_INTERVAL_MS = 50
const SESSION_WAIT_TIMEOUT_MS = 2000

export const useAiStore = defineStore('ai', () => {
  // ===== 状态 =====
  const isLoading = ref(false)
  const isBackendSessionActive = ref(false)
  const currentSessionUUID = ref<string>('')
  const lastTokenStats = ref<any>(null)

  const selectedRole = ref('')
  const aiRoles = ref<StoredRole[]>([])
  const currentRoleConfig = ref<AIRole | null>(null)
  const defaultRole = ref('')
  const rolesLoaded = ref(false)

  function invalidateCommandAvailability() {
    backendCommandService.clearCache()
  }

  function syncCurrentRoleConfig(roleName: string) {
    currentRoleConfig.value =
      aiRoles.value.find((roleEntry) => roleEntry.name === roleName)?.role ?? null
  }

  // ===== 动作 =====

  /**
   * 加载角色会话
   * 封装 invoke('load_character_session')
   */
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

  /**
   * 确保后端会话已就绪
   */
  async function ensureCharacterSession(
    uuid: string,
    timeoutMs = SESSION_WAIT_TIMEOUT_MS,
  ) {
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

  /**
   * 发送聊天消息
   * 封装 invoke('send_chat_message')
   */
  async function sendChatMessage(message: string) {
    try {
      isLoading.value = true
      await invoke('send_chat_message', { message })
      invalidateCommandAvailability()
    } catch (error) {
      console.error('发送消息失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 编辑聊天消息
   * 封装 invoke('edit_chat_message')
   */
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

  /**
   * 删除聊天消息
   * 封装 invoke('delete_chat_message')
   */
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

  /**
   * 重新生成最后一条消息
   * 封装 invoke('regenerate_last_message')
   */
  async function regenerateLastMessage() {
    try {
      isLoading.value = true
      await invoke('regenerate_last_message')
      invalidateCommandAvailability()
    } catch (error) {
      console.error('重新生成失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 继续对话（当最后一条是用户消息时生成AI回复）
   * 封装 invoke('continue_chat')
   */
  async function continueChat() {
    try {
      isLoading.value = true
      await invoke('continue_chat')
      invalidateCommandAvailability()
    } catch (error) {
      console.error('继续对话失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 加载聊天历史
   * 封装 invoke('load_chat_history')
   */
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

  // ===== AI 角色配置 =====

  async function loadAIRoles(forceRefresh = false): Promise<StoredRole[]> {
    if (!forceRefresh && rolesLoaded.value && aiRoles.value.length > 0) {
      return aiRoles.value
    }

    try {
      const config = await AIConfigService.getConfig()
      defaultRole.value = config.default_role
      aiRoles.value = await AIConfigService.getAllRoles()
      rolesLoaded.value = true

      const roleExists = aiRoles.value.some(
        (roleEntry) => roleEntry.name === selectedRole.value,
      )

      if (!selectedRole.value || !roleExists) {
        selectedRole.value = config.default_role || aiRoles.value[0]?.name || ''
      }

      syncCurrentRoleConfig(selectedRole.value)
      return aiRoles.value
    } catch (error) {
      console.error('加载AI角色配置失败:', error)
      throw error
    }
  }

  async function selectRole(roleName: string) {
    if (!rolesLoaded.value) {
      await loadAIRoles()
    }

    selectedRole.value = roleName
    syncCurrentRoleConfig(roleName)
  }

  // ===== 工具和命令管理 =====

  /**
   * 获取可用命令列表
   * 集成 backendCommandService
   */
  async function getAvailableCommands(forceRefresh = false): Promise<CommandMetadata[]> {
    return await backendCommandService.getCommands(forceRefresh)
  }

  /**
   * 搜索命令
   * 集成 backendCommandService
   */
  async function searchCommands(query: string): Promise<CommandMetadata[]> {
    return await backendCommandService.searchCommands(query)
  }

  /**
   * 执行命令
   * 集成 backendCommandService
   */
  async function executeCommand(commandId: string, userInput?: string): Promise<CommandResult> {
    return await backendCommandService.executeCommand(commandId, userInput)
  }

  /**
   * 清除命令缓存
   * 集成 backendCommandService
   */
  function clearCommandCache(): void {
    invalidateCommandAvailability()
  }

  // ===== 会话状态管理 =====

  /**
   * 更新后端会话状态
   * 由事件监听器调用
   */
  function updateSessionState(uuid: string, active: boolean) {
    currentSessionUUID.value = uuid
    isBackendSessionActive.value = active
    invalidateCommandAvailability()
  }

  /**
   * 清除会话状态
   * 由事件监听器调用
   */
  function clearSessionState() {
    currentSessionUUID.value = ''
    isBackendSessionActive.value = false
    invalidateCommandAvailability()
  }

  /**
   * 更新Token统计
   * 由事件监听器调用
   */
  function updateTokenStats(stats: any) {
    lastTokenStats.value = stats
  }

  /**
   * 清除Token统计
   */
  function clearTokenStats() {
    lastTokenStats.value = null
  }

  /**
   * 重置状态
   */
  function reset() {
    isLoading.value = false
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
    // 状态
    isLoading: readonly(isLoading),
    isBackendSessionActive: readonly(isBackendSessionActive),
    currentSessionUUID: readonly(currentSessionUUID),
    lastTokenStats: readonly(lastTokenStats),
    selectedRole: readonly(selectedRole),
    aiRoles: readonly(aiRoles),
    currentRoleConfig: readonly(currentRoleConfig),
    defaultRole: readonly(defaultRole),

    // AI操作
    loadCharacterSession,
    ensureCharacterSession,
    sendChatMessage,
    editChatMessage,
    deleteChatMessage,
    regenerateLastMessage,
    continueChat,
    loadChatHistory,
    loadAIRoles,
    selectRole,

    // 工具和命令管理
    getAvailableCommands,
    searchCommands,
    executeCommand,
    clearCommandCache,

    // 会话状态管理
    updateSessionState,
    clearSessionState,
    updateTokenStats,
    clearTokenStats,

    // 工具方法
    reset,
  }
})
