import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { CharacterData, TavernCardV2 } from '@/types/character'
import * as characterStorage from '@/services/characterStorage'
import { CharacterStateService } from '@/services/characterState'

const ALTERNATE_GREETING_MARKER = '<START_ALT>'

function parseAlternateGreetingSegments(value: string) {
  return value
    .split(ALTERNATE_GREETING_MARKER)
    .map(segment => segment.trim())
    .filter(segment => segment.length > 0)
}

export const useCharacterStore = defineStore('character', () => {
  // ===== 状态 =====
  const characters = ref<CharacterData[]>([])
  const currentCharacterUUID = ref<string>('')
  const loading = ref(false)
  const lastFetch = ref(0)
  const CACHE_DURATION = 5 * 60 * 1000 // 5分钟缓存

  // ===== 计算属性 =====
  const currentCharacter = computed(() => {
    return characters.value.find(c => c.uuid === currentCharacterUUID.value) || null
  })

  const charactersCount = computed(() => {
    return characters.value.length
  })

  const isCacheValid = computed(() => {
    return Date.now() - lastFetch.value < CACHE_DURATION
  })

  const charactersWithWorldBook = computed(() => {
    return characters.value.filter(c => c.card.data.character_book?.entries?.length)
  })

  // ===== 动作 =====

  /**
   * 加载所有角色
   */
  async function loadAllCharacters(force = false) {
    // 如果缓存有效且不是强制刷新，则跳过
    if (!force && isCacheValid.value && characters.value.length > 0) {
      return
    }

    loading.value = true
    try {
      const data = await characterStorage.getAllCharacters()
      characters.value = data
      lastFetch.value = Date.now()
    } catch (error) {
      console.error('加载角色列表失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  /**
   * 刷新角色列表（强制重新加载）
   */
  async function refreshCharacters() {
    return loadAllCharacters(true)
  }

  /**
   * 根据UUID获取角色
   */
  async function getCharacterByUUID(uuid: string): Promise<CharacterData | undefined> {
    // 先尝试从缓存中获取
    let character = characters.value.find(c => c.uuid === uuid)
    if (!character) {
      // 缓存中没有，从后端获取
      const foundCharacter = await characterStorage.getCharacterByUUID(uuid)
      if (foundCharacter) {
        characters.value.push(foundCharacter)
        character = foundCharacter
      }
    }
    return character
  }

  /**
   * 创建新角色
   */
  async function createCharacter(name: string) {
    const newCharacter = await characterStorage.createCharacter(name)
    characters.value.push(newCharacter)
    lastFetch.value = Date.now()
    return newCharacter
  }

  /**
   * 更新角色卡
   */
  async function updateCharacter(uuid: string, card: TavernCardV2) {
    await characterStorage.updateCharacter(uuid, card)

    // 更新缓存中的数据
    const index = characters.value.findIndex(c => c.uuid === uuid)
    if (index >= 0) {
      characters.value[index].card = card
      characters.value[index].meta.updatedAt = new Date().toISOString()
    }
    lastFetch.value = Date.now()
  }

  /**
   * 更新角色单个字段
   */
  async function updateCharacterField(uuid: string, fieldName: string, fieldValue: string) {
    await characterStorage.updateCharacterField(uuid, fieldName, fieldValue)

    // 更新缓存中的数据
    const character = characters.value.find(c => c.uuid === uuid)
    if (character) {
      // 更新对应的字段
      const fieldMap: Record<string, string> = {
        'name': 'name',
        'description': 'description',
        'personality': 'personality',
        'scenario': 'scenario',
        'first_mes': 'first_mes',
        'mes_example': 'mes_example',
        'creator_notes': 'creator_notes',
        'system_prompt': 'system_prompt',
        'post_history_instructions': 'post_history_instructions',
        'alternate_greetings': 'alternate_greetings',
        'tags': 'tags',
        'creator': 'creator',
        'character_version': 'character_version',
      }

      const targetField = fieldMap[fieldName]
      if (targetField) {
        if (fieldName === 'alternate_greetings') {
          const values = parseAlternateGreetingSegments(fieldValue)
          ;(character.card.data as any)[targetField] = values
        } else if (fieldName === 'tags') {
          // 处理数组类型字段
          const values = fieldValue.split('\n').map(v => v.trim()).filter(v => v.length > 0)
          ;(character.card.data as any)[targetField] = values
        } else {
          ;(character.card.data as any)[targetField] = fieldValue
        }
        character.meta.updatedAt = new Date().toISOString()
      }
    }
    lastFetch.value = Date.now()
  }

  /**
   * 删除角色
   */
  async function deleteCharacter(uuid: string) {
    await characterStorage.deleteCharacter(uuid)
    characters.value = characters.value.filter(c => c.uuid !== uuid)

    // 如果删除的是当前编辑的角色，清除当前角色状态
    if (currentCharacterUUID.value === uuid) {
      currentCharacterUUID.value = ''
      await clearCurrentCharacter()
    }

    lastFetch.value = Date.now()
  }

  /**
   * 上传背景图片
   */
  async function uploadBackgroundImage(uuid: string, file: File) {
    const backgroundPath = await characterStorage.uploadBackgroundImage(uuid, file)

    // 更新缓存中的数据
    const character = characters.value.find(c => c.uuid === uuid)
    if (character) {
      character.backgroundPath = backgroundPath
    }

    return backgroundPath
  }

  /**
   * 更新角色背景图片路径
   */
  async function updateCharacterBackgroundPath(uuid: string, backgroundPath: string) {
    await characterStorage.updateCharacterBackgroundPath(uuid, backgroundPath)

    // 更新缓存中的数据
    const character = characters.value.find(c => c.uuid === uuid)
    if (character) {
      character.backgroundPath = backgroundPath
    }
  }

  /**
   * 导出角色卡
   */
  async function exportCharacterCard(uuid: string, outputPath: string) {
    return await characterStorage.exportCharacterCard(uuid, outputPath)
  }

  /**
   * 导入角色卡
   */
  async function importCharacterCard(filePath: string) {
    const character = await characterStorage.importCharacterCard(filePath)
    characters.value.push(character)
    lastFetch.value = Date.now()
    return character
  }

  /**
   * 从字节数据导入角色卡
   */
  async function importCharacterCardFromBytes(fileData: Uint8Array, fileName: string) {
    const character = await characterStorage.importCharacterCardFromBytes(fileData, fileName)
    characters.value.push(character)
    lastFetch.value = Date.now()
    return character
  }

  // ===== 当前角色状态管理 =====

  /**
   * 设置当前编辑角色
   */
  async function setCurrentCharacter(uuid: string) {
    currentCharacterUUID.value = uuid
    await CharacterStateService.setActiveCharacter(uuid)
  }

  /**
   * 获取当前编辑角色UUID
   */
  async function getCurrentCharacterUUID(): Promise<string | null> {
    return await CharacterStateService.getActiveCharacter()
  }

  /**
   * 清除当前编辑角色
   */
  async function clearCurrentCharacter() {
    currentCharacterUUID.value = ''
    await CharacterStateService.clearActiveCharacter()
  }

  /**
   * 检查是否有活跃角色
   */
  async function hasActiveCharacter(): Promise<boolean> {
    return await CharacterStateService.hasActiveCharacter()
  }

  /**
   * 清除缓存
   */
  function clearCache() {
    lastFetch.value = 0
  }

  /**
   * 重置状态
   */
  function reset() {
    characters.value = []
    currentCharacterUUID.value = ''
    loading.value = false
    lastFetch.value = 0
  }

  /**
   * 从后端事件更新角色数据（不触发重新加载，只更新缓存）
   */
  function updateCharacterFromBackend(uuid: string, characterData: CharacterData) {
    // 安全检查：确保数据有效
    if (!characterData || !characterData.uuid) {
      console.warn('updateCharacterFromBackend: 接收到无效的角色数据', { uuid, characterData })
      return
    }

    const index = characters.value.findIndex(c => c && c.uuid === uuid)
    if (index >= 0) {
      // 更新现有缓存
      characters.value[index] = characterData
    } else {
      // 添加到缓存
      characters.value.push(characterData)
    }
    lastFetch.value = Date.now()
  }

  return {
    // 状态
    characters,
    currentCharacterUUID: readonly(currentCharacterUUID),
    loading: readonly(loading),
    lastFetch: readonly(lastFetch),

    // 计算属性
    currentCharacter,
    charactersCount,
    isCacheValid,
    charactersWithWorldBook,

    // 动作
    loadAllCharacters,
    refreshCharacters,
    getCharacterByUUID,
    createCharacter,
    updateCharacter,
    updateCharacterField,
    deleteCharacter,
    uploadBackgroundImage,
    updateCharacterBackgroundPath,
    exportCharacterCard,
    importCharacterCard,
    importCharacterCardFromBytes,

    // 当前角色状态管理
    setCurrentCharacter,
    getCurrentCharacterUUID,
    clearCurrentCharacter,
    hasActiveCharacter,

    // 缓存管理
    clearCache,
    reset,

    // 后端事件更新
    updateCharacterFromBackend,
  }
})
