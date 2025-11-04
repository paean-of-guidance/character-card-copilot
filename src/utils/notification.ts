// Toast通知类型定义
export type ToastType = 'success' | 'warning' | 'info' | 'error'

// Toast通知接口
export interface Toast {
  id: string
  type: ToastType
  title?: string
  message: string
  duration?: number
  timestamp: number
}

// Modal确认框接口
export interface ModalOptions {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'danger' | 'warning' | 'info'
  onConfirm?: () => void | Promise<void>
  onCancel?: () => void | Promise<void>
}

// 生成唯一ID
export function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// 获取Toast类型的默认配置
export function getToastConfig(type: ToastType): {
  duration: number
  icon: string
  bgColor: string
  textColor: string
} {
  const configs = {
    success: {
      duration: 3000,
      icon: '✓',
      bgColor: 'bg-green-500',
      textColor: 'text-white'
    },
    warning: {
      duration: 5000,
      icon: '⚠',
      bgColor: 'bg-yellow-500',
      textColor: 'text-white'
    },
    info: {
      duration: 3000,
      icon: 'ℹ',
      bgColor: 'bg-blue-500',
      textColor: 'text-white'
    },
    error: {
      duration: 5000,
      icon: '✕',
      bgColor: 'bg-red-500',
      textColor: 'text-white'
    }
  }

  return configs[type]
}

// 获取Modal类型的默认配置
export function getModalConfig(type?: 'danger' | 'warning' | 'info'): {
  title: string
  confirmText: string
  cancelText: string
  confirmBtnClass: string
} {
  const configs = {
    danger: {
      title: '危险操作确认',
      confirmText: '确认删除',
      cancelText: '取消',
      confirmBtnClass: 'bg-red-500 hover:bg-red-600 text-white'
    },
    warning: {
      title: '操作确认',
      confirmText: '确认',
      cancelText: '取消',
      confirmBtnClass: 'bg-yellow-500 hover:bg-yellow-600 text-white'
    },
    info: {
      title: '提示',
      confirmText: '知道了',
      cancelText: '取消',
      confirmBtnClass: 'bg-blue-500 hover:bg-blue-600 text-white'
    }
  }

  return configs[type || 'info']
}