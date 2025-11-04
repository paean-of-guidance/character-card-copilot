<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center"
        @click="handleBackdropClick"
      >
        <!-- 背景遮罩 -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm"></div>

        <!-- Modal内容 -->
        <div
          class="relative bg-white rounded-xl shadow-2xl max-w-md w-full mx-4 max-h-[90vh] overflow-hidden"
          @click.stop
        >
          <!-- 图标和标题区域 -->
          <div class="flex items-start gap-4 p-6 pb-0">
            <!-- 图标 -->
            <div
              :class="[
                'flex-shrink-0 w-12 h-12 rounded-full flex items-center justify-center',
                getIconClass()
              ]"
            >
              <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                <path v-if="currentOptions.type === 'danger'" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"/>
                <path v-else-if="currentOptions.type === 'warning'" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"/>
                <path v-else d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 011 1v4a1 1 0 11-2 0V7a1 1 0 011-1zM8 10a1 1 0 000 2h4a1 1 0 100-2H8z"/>
              </svg>
            </div>

            <!-- 标题和内容 -->
            <div class="flex-1">
              <h3 class="text-lg font-semibold text-gray-900 mb-2">
                {{ currentOptions.title }}
              </h3>
              <p class="text-gray-600 text-sm leading-relaxed">
                {{ currentOptions.message }}
              </p>
            </div>
          </div>

          <!-- 按钮区域 -->
          <div class="flex gap-3 justify-end p-6 pt-4">
            <button
              @click="handleCancel"
              class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
            >
              {{ currentOptions.cancelText }}
            </button>
            <button
              @click="handleConfirm"
              :class="[
                'px-4 py-2 text-sm font-medium rounded-lg transition-colors',
                getConfirmButtonClass()
              ]"
            >
              {{ currentOptions.confirmText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { ModalOptions } from '@/utils/notification'
import { getModalConfig } from '@/utils/notification'

interface Props {
  options: ModalOptions | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  confirm: []
  cancel: []
  close: []
}>()

// Modal显示状态
const visible = ref(false)

// 当前配置
const currentOptions = ref<ModalOptions>({
  title: '',
  message: '',
  confirmText: '确认',
  cancelText: '取消',
  type: 'info'
})

// 监听props变化
watch(() => props.options, (newOptions) => {
  if (newOptions) {
    const config = getModalConfig(newOptions.type)
    currentOptions.value = {
      ...config,
      ...newOptions
    }
    visible.value = true
  } else {
    visible.value = false
  }
}, { immediate: true })

// 获取图标样式
function getIconClass(): string {
  const type = currentOptions.value.type
  const classes = {
    danger: 'bg-red-100 text-red-600',
    warning: 'bg-yellow-100 text-yellow-600',
    info: 'bg-blue-100 text-blue-600'
  }
  return classes[type || 'info']
}

// 获取确认按钮样式
function getConfirmButtonClass(): string {
  const type = currentOptions.value.type
  const classes = {
    danger: 'bg-red-500 hover:bg-red-600 text-white',
    warning: 'bg-yellow-500 hover:bg-yellow-600 text-white',
    info: 'bg-blue-500 hover:bg-blue-600 text-white'
  }
  return classes[type || 'info']
}

// 处理确认按钮点击
async function handleConfirm() {
  try {
    await currentOptions.value.onConfirm?.()
    emit('confirm')
  } catch (error) {
    console.error('Modal confirm callback error:', error)
  }
  closeModal()
}

// 处理取消按钮点击
async function handleCancel() {
  try {
    await currentOptions.value.onCancel?.()
    emit('cancel')
  } catch (error) {
    console.error('Modal cancel callback error:', error)
  }
  closeModal()
}

// 处理背景点击（通常是关闭Modal）
function handleBackdropClick(event: MouseEvent) {
  // 只有点击背景遮罩时才关闭，点击内容区域不关闭
  if (event.target === event.currentTarget) {
    handleCancel()
  }
}

// 关闭Modal
function closeModal() {
  visible.value = false
  emit('close')
}

// ESC键关闭Modal
function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && visible.value) {
    handleCancel()
  }
}

// 监听键盘事件
watch(visible, (newValue) => {
  if (newValue) {
    document.addEventListener('keydown', handleKeydown)
    // 禁止背景滚动
    document.body.style.overflow = 'hidden'
  } else {
    document.removeEventListener('keydown', handleKeydown)
    // 恢复背景滚动
    document.body.style.overflow = ''
  }
})
</script>

<style scoped>
/* Modal进入和离开动画 */
.modal-enter-active {
  transition: all 0.3s ease-out;
}

.modal-leave-active {
  transition: all 0.3s ease-in;
}

.modal-enter-from {
  opacity: 0;
  transform: scale(0.9);
}

.modal-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

/* 背景遮罩动画 */
.modal-enter-from .absolute,
.modal-leave-to .absolute {
  opacity: 0;
}
</style>