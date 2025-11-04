import { inject } from 'vue'
import type { ModalOptions } from '@/utils/notification'

// Modal方法类型定义
export interface ModalMethods {
  showAlertModal: (
    message: string,
    onConfirm?: () => void | Promise<void>,
    options?: Partial<ModalOptions>
  ) => Promise<boolean>
}

// 全局Modal方法Symbol
export const ModalSymbol = Symbol('modal')

/**
 * 使用Modal确认框的Composable Hook
 * @returns Modal确认框方法对象
 */
export function useModal(): ModalMethods {
  const modal = inject<ModalMethods>(ModalSymbol)

  if (!modal) {
    throw new Error('useModal must be used within a component that provides modal')
  }

  return modal
}

/**
 * 显示确认删除的Modal
 * @param itemName 要删除的项目名称
 * @param onConfirm 确认删除后的回调
 * @returns Promise<boolean> 用户是否确认
 */
export function useDeleteModal(itemName: string, onConfirm?: () => void | Promise<void>) {
  const { showAlertModal } = useModal()

  return showAlertModal(
    `确定要删除"${itemName}"吗？此操作不可撤销。`,
    onConfirm,
    {
      title: '删除确认',
      type: 'danger',
      confirmText: '确认删除',
      cancelText: '取消'
    }
  )
}

/**
 * 检查Modal是否可用
 * @returns 是否可用
 */
export function useModalAvailable(): boolean {
  try {
    useModal()
    return true
  } catch {
    return false
  }
}