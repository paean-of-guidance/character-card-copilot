<script setup lang="ts">
import { ref } from 'vue';
import { useApiStore } from '@/stores/api';
import type { ApiConfig, ApiProvider } from '@/types/api';

const emit = defineEmits<{
  created: [api: ApiConfig];
  cancel: [];
}>();

const apiStore = useApiStore();
const profileName = ref('');
const provider = ref<ApiProvider>('open_ai_compatible');
const loading = ref(false);
const error = ref('');

const providerBaseUrls: Record<ApiProvider, string> = {
  open_ai_compatible: 'https://api.openai.com/v1',
  open_ai_responses: 'https://api.openai.com/v1',
  claude: 'https://api.anthropic.com',
  gemini_v1_beta: 'https://generativelanguage.googleapis.com/v1beta',
}

function validateProfileName(): string | null {
  const trimmed = profileName.value.trim();

  if (!trimmed) {
    return 'API配置名称不能为空';
  }

  if (trimmed.length < 2) {
    return 'API配置名称至少需要2个字符';
  }

  if (trimmed.length > 50) {
    return 'API配置名称不能超过50个字符';
  }

  return null;
}

async function handleCreate() {
  const validationError = validateProfileName();
  if (validationError) {
    error.value = validationError;
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    const newApi = await apiStore.createApi({
      profile: profileName.value.trim(),
      provider: provider.value,
      base_url: providerBaseUrls[provider.value],
      api_key: '',
      model: '',
      default: false,
      enabled: false,
    });

    emit('created', newApi);
  } catch (err) {
    error.value = err instanceof Error ? err.message : '创建API配置失败，请重试';
    console.error('创建API配置失败:', err);
  } finally {
    loading.value = false;
  }
}

function handleCancel() {
  if (!loading.value) {
    emit('cancel');
  }
}

function handleInput() {
  error.value = '';
}
</script>

<template>
  <div class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/65 backdrop-blur-md">
    <div class="liquid-modal w-full max-w-md overflow-hidden">
      <div class="px-6 py-4 border-b border-white/8">
        <h2 class="text-xl font-semibold text-white/90">新建 API 配置</h2>
        <p class="mt-1 text-sm text-white/45">先选择 provider 和名称，随后在详情面板中补全 Base URL、密钥与模型。</p>
      </div>

      <div class="px-6 py-5 space-y-5">
        <div class="space-y-2">
          <label for="provider" class="block text-sm font-medium text-white/70">
            Provider <span class="text-red-400">*</span>
          </label>
          <select
            id="provider"
            v-model="provider"
            class="dialog-input"
            :disabled="loading"
          >
            <option value="open_ai_compatible" class="bg-slate-900">OpenAI Compatible</option>
            <option value="open_ai_responses" class="bg-slate-900">OpenAI Responses</option>
            <option value="claude" class="bg-slate-900">Claude</option>
            <option value="gemini_v1_beta" class="bg-slate-900">Gemini v1beta</option>
          </select>
        </div>

        <div class="space-y-2">
          <label for="profile-name" class="block text-sm font-medium text-white/70">
            API配置名称 <span class="text-red-400">*</span>
          </label>
          <input
            id="profile-name"
            v-model="profileName"
            type="text"
            class="dialog-input"
            placeholder="请输入API配置名称"
            :disabled="loading"
            @input="handleInput"
            @keydown.enter="handleCreate"
          />
          <div v-if="error" class="text-sm text-red-300 mt-1">
            {{ error }}
          </div>
          <div class="text-xs text-white/35 mt-1">
            配置名称用于识别不同的API设置，建议使用有意义的名称。
          </div>
        </div>
      </div>

      <div class="px-6 py-4 border-t border-white/8 flex justify-end gap-3">
        <button
          class="glass-btn glass-btn--neutral"
          :disabled="loading"
          @click="handleCancel"
        >
          取消
        </button>
        <button
          class="glass-btn glass-btn--primary"
          :disabled="loading || !profileName.trim()"
          @click="handleCreate"
        >
          {{ loading ? '创建中...' : '确定' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-input {
  width: 100%;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.90);
  border-radius: 12px;
  font-size: 0.875rem;
  padding: 0.625rem 1rem;
  transition: all 0.2s ease;
}

.dialog-input:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.09);
  border-color: rgba(139, 92, 246, 0.55);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.15);
}

.dialog-input::placeholder {
  color: rgba(255, 255, 255, 0.28);
}

.dialog-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
