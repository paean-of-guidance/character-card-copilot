import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { ChatMessage } from '@/types/api'
import type { CommandMetadata, CommandResult } from '@/types/commands'
import { invoke } from '@tauri-apps/api/core'
import { backendCommandService } from '@/services/backendCommandService'

export const useAiStore = defineStore('ai', () => {
  // ===== 状态 =====
  const isLoading = ref(false)
  const isBackendSessionActive = ref(false)
  const currentSessionUUID = ref<string>('')
  const lastTokenStats = ref<any>(null)

  // ===== 动作 =====

  /**
   * 加载角色会话
   * 封装 invoke('load_character_session')
   */
  async function loadCharacterSession(uuid: string) {
    try {
      isLoading.value = true
      await invoke('load_character_session', { uuid })
      // 会话状态通过事件监听器更新
    } catch (error) {
      console.error('加载角色会话失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 发送聊天消息
   * 封装 invoke('send_chat_message')
   */
  async function sendChatMessage(message: string) {
    try {
      isLoading.value = true
      await invoke('send_chat_message', { message })
      // 消息状态通过事件监听器更新
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
      const history = await invoke<ChatMessage[]>('load_chat_history', { characterId })
      return history
    } catch (error) {
      console.error('加载聊天历史失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
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
    backendCommandService.clearCache()
  }

  // ===== 会话状态管理 =====

  /**
   * 更新后端会话状态
   * 由事件监听器调用
   */
  function updateSessionState(uuid: string, active: boolean) {
    currentSessionUUID.value = uuid
    isBackendSessionActive.value = active
  }

  /**
   * 清除会话状态
   * 由事件监听器调用
   */
  function clearSessionState() {
    currentSessionUUID.value = ''
    isBackendSessionActive.value = false
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
  }

  return {
    // 状态
    isLoading: readonly(isLoading),
    isBackendSessionActive: readonly(isBackendSessionActive),
    currentSessionUUID: readonly(currentSessionUUID),
    lastTokenStats: readonly(lastTokenStats),

    // AI操作
    loadCharacterSession,
    sendChatMessage,
    editChatMessage,
    deleteChatMessage,
    regenerateLastMessage,
    continueChat,
    loadChatHistory,

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
