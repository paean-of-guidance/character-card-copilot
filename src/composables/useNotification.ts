import { inject } from 'vue'

// Toast方法类型定义
export interface NotificationMethods {
  showSuccessToast: (message: string, title?: string) => void
  showWarningToast: (message: string, title?: string) => void
  showInfoToast: (message: string, title?: string) => void
  showErrorToast: (message: string, title?: string) => void
}

// 全局Toast方法Symbol
export const NotificationSymbol = Symbol('notification')

/**
 * 使用Toast通知的Composable Hook
 * @returns Toast通知方法对象
 */
export function useNotification(): NotificationMethods {
  const notification = inject<NotificationMethods>(NotificationSymbol)

  if (!notification) {
    throw new Error('useNotification must be used within a component that provides notification')
  }

  return notification
}

/**
 * 检查Toast通知是否可用
 * @returns 是否可用
 */
export function useNotificationAvailable(): boolean {
  try {
    useNotification()
    return true
  } catch {
    return false
  }
}