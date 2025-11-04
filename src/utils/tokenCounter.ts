import {
  encode,
} from 'gpt-tokenizer'

/**
 * 计算文本的token数量
 * @param text 要计算的文本
 * @param model 模型名称（当前使用默认编码器）
 * @returns token数量
 */
export function calculateTokens(text: string): number {
    if (!text || text.trim() === '') {
        return 0
    }

    try {
        const tokens = encode(text)
        return tokens.length
    } catch (error) {
        console.warn('Token计算失败，使用字符数估算:', error)
        // 如果无法获取精确编码器，使用字符数的粗略估算
        // 英文大约4个字符=1个token，中文大约1.5个字符=1个token
        const chineseChars = (text.match(/[\u4e00-\u9fff]/g) || []).length
        const otherChars = text.length - chineseChars
        return Math.ceil(chineseChars / 1.5 + otherChars / 4)
    }
}

/**
 * 为不同字段类型获取推荐的模型编码器
 */
export function getModelForField(): string {
    // 使用默认的gpt-4o编码器
    return 'gpt-4o'
}

/**
 * 创建token计数器缓存，避免重复计算
 */
class TokenCounter {
    private cache = new Map<string, number>()

    countTokens(text: string): number {
        if (!text) return 0

        const cacheKey = `default:${text}`

        if (this.cache.has(cacheKey)) {
            return this.cache.get(cacheKey)!
        }

        const count = calculateTokens(text)
        this.cache.set(cacheKey, count)

        // 限制缓存大小
        if (this.cache.size > 1000) {
            const firstKey = this.cache.keys().next().value
            if (firstKey) {
                this.cache.delete(firstKey)
            }
        }

        return count
    }

    clearCache() {
        this.cache.clear()
    }
}

export const tokenCounter = new TokenCounter()