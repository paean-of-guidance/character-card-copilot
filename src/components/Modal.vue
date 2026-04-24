<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center"
        @click="handleBackdropClick"
      >
        <!-- 背景遮罩 -->
        <div class="absolute inset-0 bg-black/65 backdrop-blur-md"></div>

        <!-- Modal内容 -->
        <div
          class="liquid-modal relative mx-4 w-full max-w-md max-h-[90vh] overflow-hidden"
          @click.stop
        >
          <!-- 图标和标题区域 -->
          <div class="flex items-start gap-4 p-6 pb-0">
            <!-- 图标 -->
            <div
              :class="[
                'flex-shrink-0 w-12 h-12 rounded-2xl flex items-center justify-center',
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
              <h3 class="mb-2 text-lg font-semibold text-white/90">
                {{ currentOptions.title }}
              </h3>
              <p class="text-sm leading-relaxed text-white/60">
                {{ currentOptions.message }}
              </p>
            </div>
          </div>

          <!-- 按钮区域 -->
          <div class="flex justify-end gap-3 p-6 pt-4">
            <button
              @click="handleCancel"
              class="glass-btn glass-btn--neutral"
            >
              {{ currentOptions.cancelText }}
            </button>
            <button
              @click="handleConfirm"
              :class="['glass-btn', getConfirmButtonClass()]"
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

const visible = ref(false)

const currentOptions = ref<ModalOptions>({
  title: '',
  message: '',
  confirmText: '确认',
  cancelText: '取消',
  type: 'info'
})

watch(() => props.options, (newOptions) => {
  if (newOptions) {
    const config = getModalConfig(newOptions.type)
    currentOptions.value = { ...config, ...newOptions }
    visible.value = true
  } else {
    visible.value = false
  }
}, { immediate: true })

function getIconClass(): string {
  const type = currentOptions.value.type
  const classes = {
    danger: 'bg-red-500/20 text-red-300',
    warning: 'bg-yellow-500/20 text-yellow-300',
    info: 'bg-indigo-500/20 text-indigo-300'
  }
  return classes[type || 'info']
}

function getConfirmButtonClass(): string {
  const type = currentOptions.value.type
  const classes = {
    danger: 'glass-btn--danger',
    warning: 'glass-btn--primary',
    info: 'glass-btn--primary'
  }
  return classes[type || 'info']
}

async function handleConfirm() {
  try {
    await currentOptions.value.onConfirm?.()
    emit('confirm')
  } catch (error) {
    console.error('Modal confirm callback error:', error)
  }
  closeModal()
}

async function handleCancel() {
  try {
    await currentOptions.value.onCancel?.()
    emit('cancel')
  } catch (error) {
    console.error('Modal cancel callback error:', error)
  }
  closeModal()
}

function handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    handleCancel()
  }
}

function closeModal() {
  visible.value = false
  emit('close')
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && visible.value) {
    handleCancel()
  }
}

watch(visible, (newValue) => {
  if (newValue) {
    document.addEventListener('keydown', handleKeydown)
    document.body.style.overflow = 'hidden'
  } else {
    document.removeEventListener('keydown', handleKeydown)
    document.body.style.overflow = ''
  }
})
</script>

<style scoped>
.modal-enter-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.modal-leave-active {
  transition: all 0.2s ease-in;
}
.modal-enter-from {
  opacity: 0;
  transform: scale(0.88) translateY(12px);
}
.modal-leave-to {
  opacity: 0;
  transform: scale(0.92) translateY(8px);
}
</style>
