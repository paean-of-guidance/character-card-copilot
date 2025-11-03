<script setup lang="ts">
import { ref, computed } from 'vue';

// 组件props，可以支持不同类型的面板
const props = defineProps<{
  visible?: boolean;
  panelType?: 'ai' | 'chat' | 'tools';
}>();

const emits = defineEmits<{
  toggle: [];
}>();

// 默认可见
const isVisible = ref(props.visible !== false);

// 切换显示/隐藏
function togglePanel() {
  isVisible.value = !isVisible.value;
  emits('toggle');
}

// 监听visible属性变化
const visible = computed(() => {
  return props.visible !== false && isVisible.value;
});
</script>

<template>
  <div v-if="visible" class="card rounded-xl w-1/2 bg-white p-6 shadow-2xl">
    <div class="h-full flex flex-col">
      <!-- 面板头部 -->
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-xl font-semibold text-gray-900">
          <span v-if="panelType === 'ai'">AI 助手面板</span>
          <span v-else-if="panelType === 'chat'">对话面板</span>
          <span v-else-if="panelType === 'tools'">工具面板</span>
          <span v-else>AI Panel</span>
        </h2>
        <button
          @click="togglePanel"
          class="text-gray-400 hover:text-gray-600 transition-colors"
          title="隐藏面板"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 面板内容 -->
      <div class="flex-1 flex items-center justify-center">
        <div class="text-center text-gray-500">
          <div class="text-6xl mb-4">🤖</div>
          <h3 class="text-xl font-semibold mb-2">AI 助手面板</h3>
          <p class="text-sm">AI辅助功能正在开发中...</p>

          <!-- 开发中的功能提示 -->
          <div class="mt-8 p-4 bg-blue-50 rounded-lg border border-blue-200">
            <h4 class="text-sm font-medium text-blue-900 mb-2">即将推出</h4>
            <ul class="text-xs text-blue-700 space-y-1">
              <li>• 角色对话生成</li>
              <li>• 剧情建议</li>
              <li>• 角色设定优化</li>
              <li>• 创意灵感激发</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- 面板底部操作区 -->
      <div class="mt-6 pt-4 border-t border-gray-200">
        <div class="flex justify-between items-center">
          <div class="text-xs text-gray-500">
            v1.0.0 开发中
          </div>
          <button
            class="px-3 py-1 text-xs bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors"
            disabled
          >
            功能敬请期待
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 面板动画 */
.card {
  animation: slideInRight 0.3s ease-out;
}

@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>