<template>
  <Teleport to="body">
    <div class="fixed top-4 right-4 z-50 pointer-events-none">
      <TransitionGroup
        name="toast"
        tag="div"
        class="flex flex-col gap-2"
      >
        <div
          v-for="toast in visibleToasts"
          :key="toast.id"
          :class="[
            'pointer-events-auto',
            'max-w-sm w-full',
            'rounded-lg shadow-lg',
            'flex items-start gap-3 p-4',
            'transition-all duration-300 ease-in-out',
            getToastConfig(toast.type).bgColor,
            getToastConfig(toast.type).textColor
          ]"
          role="alert"
          @mouseenter="pauseTimer(toast.id)"
          @mouseleave="resumeTimer(toast.id)"
        >
          <!-- 图标 -->
          <div class="flex-shrink-0 text-xl">
            {{ getToastConfig(toast.type).icon }}
          </div>

          <!-- 内容 -->
          <div class="flex-1 min-w-0">
            <p v-if="toast.title" class="font-semibold text-sm mb-1">
              {{ toast.title }}
            </p>
            <p class="text-sm">
              {{ toast.message }}
            </p>
          </div>

          <!-- 关闭按钮 -->
          <button
            @click="removeToast(toast.id)"
            class="flex-shrink-0 ml-2 text-white/80 hover:text-white transition-colors"
            :aria-label="'关闭通知'"
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import type { Toast } from '@/utils/notification'
import { getToastConfig } from '@/utils/notification'

interface Props {
  toasts: Toast[]
}

const props = defineProps<Props>()

// 控制最大显示数量
const MAX_VISIBLE_TOASTS = 3

// 可见的toast列表
const visibleToasts = ref<Toast[]>([])

// 暂停的定时器
const pausedTimers = new Set<string>()

// 过滤并限制可见toast数量
const filteredToasts = computed(() => {
  return props.toasts.slice(-MAX_VISIBLE_TOASTS)
})

// 移除toast
function removeToast(id: string) {
  const index = visibleToasts.value.findIndex(toast => toast.id === id)
  if (index > -1) {
    visibleToasts.value.splice(index, 1)
  }

  // 清理定时器
  pausedTimers.delete(id)
}

// 暂停定时器
function pauseTimer(id: string) {
  pausedTimers.add(id)
}

// 恢复定时器
function resumeTimer(id: string) {
  pausedTimers.delete(id)
}

// 创建自动移除的定时器
function createAutoRemoveTimer(toast: Toast) {
  const duration = toast.duration || getToastConfig(toast.type).duration

  return setTimeout(() => {
    if (!pausedTimers.has(toast.id)) {
      removeToast(toast.id)
    } else {
      // 如果被暂停，等待恢复后再次检查
      const checkInterval = setInterval(() => {
        if (!pausedTimers.has(toast.id)) {
          removeToast(toast.id)
          clearInterval(checkInterval)
        }
      }, 100)

      // 设置超时防止无限循环
      setTimeout(() => {
        clearInterval(checkInterval)
        removeToast(toast.id)
      }, 10000) // 最多等待10秒
    }
  }, duration)
}

// 监听props.toasts变化
function updateVisibleToasts() {
  const newToasts = filteredToasts.value

  // 添加新的toast
  newToasts.forEach(toast => {
    if (!visibleToasts.value.find(t => t.id === toast.id)) {
      visibleToasts.value.push(toast)
      createAutoRemoveTimer(toast)
    }
  })

  // 移除不存在的toast
  visibleToasts.value = visibleToasts.value.filter(toast =>
    newToasts.find(t => t.id === toast.id)
  )
}

// 监听toast列表变化
let timer: NodeJS.Timeout
watch(() => props.toasts, () => {
  // 使用防抖避免频繁更新
  clearTimeout(timer)
  timer = setTimeout(updateVisibleToasts, 50)
}, { deep: true, immediate: true })

// 组件卸载时清理所有定时器
onUnmounted(() => {
  visibleToasts.value.forEach(toast => {
    pausedTimers.delete(toast.id)
  })
  clearTimeout(timer)
})

// 导出方法供父组件使用
defineExpose({
  removeToast,
  pauseTimer,
  resumeTimer
})
</script>

<style scoped>
/* Toast进入和离开动画 */
.toast-enter-active {
  transition: all 0.3s ease-out;
}

.toast-leave-active {
  transition: all 0.3s ease-in;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

.toast-move {
  transition: transform 0.3s ease;
}

/* 确保toast堆叠时的动画效果 */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}
</style>