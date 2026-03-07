<script setup lang="ts">
import type { AIRoleEntry } from '@/services/aiConfig';
import type { ApiConfig } from '@/types/api';

interface Props {
    selectedRole: string;
    selectedApi: string;
    currentRoleName: string;
    apiConfigs: ReadonlyArray<ApiConfig>;
    aiRoles: ReadonlyArray<AIRoleEntry>;
    contextUsageLabel: string;
}

defineProps<Props>();

const emit = defineEmits<{
    'update:selectedRole': [value: string];
    'update:selectedApi': [value: string];
}>();

function handleRoleChange(event: Event) {
    emit('update:selectedRole', (event.target as HTMLSelectElement).value);
}

function handleApiChange(event: Event) {
    emit('update:selectedApi', (event.target as HTMLSelectElement).value);
}
</script>

<template>
    <div class="flex flex-wrap items-center gap-2 px-1">
        <label class="chip-select chip-select-role">
            <span class="chip-label">角色</span>
            <select :value="selectedRole" class="chip-native-select" :disabled="aiRoles.length === 0" @change="handleRoleChange">
                <option value="" disabled>选择AI角色</option>
                <option v-for="role in aiRoles" :key="role.id" :value="role.id">
                    {{ role.role.name }}
                </option>
            </select>
            <span class="chip-value">{{ currentRoleName || '未选择' }}</span>
        </label>

        <label class="chip-select chip-select-api">
            <span class="chip-label">API</span>
            <select :value="selectedApi" class="chip-native-select" :disabled="apiConfigs.length === 0" @change="handleApiChange">
                <option value="" disabled>选择API配置</option>
                <option v-for="config in apiConfigs" :key="config.profile" :value="config.profile">
                    {{ config.profile }}
                </option>
            </select>
            <span class="chip-value">{{ selectedApi || '未配置' }}</span>
        </label>

        <div class="info-chip">
            <span class="chip-label">上下文</span>
            <span class="chip-value">{{ contextUsageLabel }}</span>
        </div>
    </div>
</template>

<style scoped>
.chip-select,
.info-chip {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 2.2rem;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.7);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.72) 0%, rgba(255, 255, 255, 0.5) 100%);
    box-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.8),
        0 8px 18px rgba(148, 163, 184, 0.1);
    padding: 0.35rem 0.8rem;
    backdrop-filter: blur(16px);
}

.chip-select-role {
    min-width: 11rem;
}

.chip-select-api {
    min-width: 10rem;
}

.chip-label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #64748b;
}

.chip-value {
    min-width: 0;
    max-width: 9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    font-weight: 600;
    color: #0f172a;
}

.chip-native-select {
    position: absolute;
    inset: 0;
    cursor: pointer;
    opacity: 0;
}

.chip-native-select:disabled {
    cursor: not-allowed;
}
</style>
