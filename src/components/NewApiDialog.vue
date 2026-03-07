<script setup lang="ts">
import { ref } from 'vue';
import { useApiStore } from '@/stores/api';
import type { ApiConfig } from '@/types/api';

const emit = defineEmits<{
  created: [api: ApiConfig];
  cancel: [];
}>();

const apiStore = useApiStore();
const profileName = ref('');
const loading = ref(false);
const error = ref('');

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
      endpoint: '',
      key: '',
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
  <div class="dialog-overlay">
    <div class="dialog">
      <div class="dialog-header">
        <h2 class="text-xl font-semibold text-gray-800">新建 API 配置</h2>
        <p class="mt-1 text-sm text-gray-500">先创建名称，随后在详情面板中补全端点、密钥与模型。</p>
      </div>

      <div class="dialog-body">
        <div class="form-group">
          <label for="profile-name" class="form-label">
            API配置名称 <span class="text-red-500">*</span>
          </label>
          <input
            id="profile-name"
            v-model="profileName"
            type="text"
            class="form-input"
            placeholder="请输入API配置名称"
            :disabled="loading"
            @input="handleInput"
            @keydown.enter="handleCreate"
          />
          <div v-if="error" class="form-error">
            {{ error }}
          </div>
          <div class="form-hint">
            配置名称用于识别不同的API设置，建议使用有意义的名称。
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button
          class="rounded-full border border-gray-200 px-4 py-2 font-medium text-gray-700 transition hover:bg-gray-50"
          :disabled="loading"
          @click="handleCancel"
        >
          取消
        </button>
        <button
          class="rounded-full bg-blue-600 px-4 py-2 font-medium text-white transition hover:bg-blue-700"
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
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow: hidden;
}

.dialog-header {
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.dialog-body {
  padding: 1.5rem;
  max-height: 60vh;
  overflow-y: auto;
}

.dialog-footer {
  padding: 1.5rem;
  border-top: 1px solid #e5e7eb;
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-label {
  display: block;
  font-weight: 500;
  color: #374151;
  margin-bottom: 0.5rem;
}

.form-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 0.875rem;
  transition: border-color 0.2s;
}

.form-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-input:disabled {
  background-color: #f3f4f6;
  cursor: not-allowed;
}

.form-error {
  color: #ef4444;
  font-size: 0.875rem;
  margin-top: 0.25rem;
}

.form-hint {
  color: #6b7280;
  font-size: 0.75rem;
  margin-top: 0.5rem;
}

.btn {
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #f3f4f6;
  color: #374151;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #e5e7eb;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2563eb;
}
</style>
