import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ChatMessage } from '@/types/api'
import { devLog } from '@/utils/logger'

/**
 * 聊天状态管理 Store
 * 用于在组件卸载/重载时保持聊天历史
 */
export const useChatStore = defineStore('chat', () => {
  // 当前活跃角色的聊天历史（按角色 UUID 存储）
  const chatHistories = ref<Map<string, ChatMessage[]>>(new Map())

  // 当前活跃的角色 UUID
  const activeCharacterUUID = ref<string | null>(null)

  // 后端会话是否活跃
  const isBackendSessionActive = ref(false)

  /**
   * 设置当前活跃角色
   */
  function setActiveCharacter(uuid: string) {
    activeCharacterUUID.value = uuid
    isBackendSessionActive.value = true
  }

  /**
   * 获取指定角色的聊天历史
   */
  function getChatHistory(uuid: string): ChatMessage[] {
    return chatHistories.value.get(uuid) || []
  }

  /**
   * 设置指定角色的聊天历史
   */
  function setChatHistory(uuid: string, messages: ChatMessage[]) {
    chatHistories.value.set(uuid, messages)
    devLog(`💾 Store: 保存 ${uuid} 的 ${messages.length} 条聊天历史`)
  }

  /**
   * 添加消息到指定角色的聊天历史
   */
  function addMessage(uuid: string, message: ChatMessage) {
    const history = getChatHistory(uuid)
    history.push(message)
    setChatHistory(uuid, history)
  }

  /**
   * 清空指定角色的聊天历史
   */
  function clearChatHistory(uuid: string) {
    chatHistories.value.set(uuid, [])
    devLog(`🗑️ Store: 清空 ${uuid} 的聊天历史`)
  }

  /**
   * 获取当前活跃角色的聊天历史
   */
  function getCurrentChatHistory(): ChatMessage[] {
    if (!activeCharacterUUID.value) return []
    return getChatHistory(activeCharacterUUID.value)
  }

  /**
   * 设置当前活跃角色的聊天历史
   */
  function setCurrentChatHistory(messages: ChatMessage[]) {
    if (!activeCharacterUUID.value) return
    setChatHistory(activeCharacterUUID.value, messages)
  }

  /**
   * 清理所有聊天历史（用于退出登录等场景）
   */
  function clearAll() {
    chatHistories.value.clear()
    activeCharacterUUID.value = null
    isBackendSessionActive.value = false
  }

  return {
    // 状态
    chatHistories,
    activeCharacterUUID,
    isBackendSessionActive,

    // 方法
    setActiveCharacter,
    getChatHistory,
    setChatHistory,
    addMessage,
    clearChatHistory,
    getCurrentChatHistory,
    setCurrentChatHistory,
    clearAll,
  }
})
