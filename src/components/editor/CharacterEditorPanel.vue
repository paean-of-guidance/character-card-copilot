<script setup lang="ts">
import TagInput from "./TagInput.vue";

const props = defineProps<{
    characterData: any;
    fullCharacterData: any;
    tokenCounts: Record<string, number>;
}>();

const emit = defineEmits<{
    (e: "update-field", field: string, oldValue: any, newValue: any): void;
}>();

const ALTERNATE_GREETING_MARKER = "<START_ALT>";

function splitAlternateGreetings(value: string) {
    return value
        ? value
              .split(ALTERNATE_GREETING_MARKER)
              .map((segment) => segment.trim())
              .filter((segment) => segment.length > 0)
        : [];
}

function onUpdateField(field: string, oldValue: any, newValue: any) {
    emit("update-field", field, oldValue, newValue);
}

// 计算总 token 数
function totalTokens(): number {
    const tc = props.tokenCounts;
    return (
        (tc.description || 0) +
        (tc.personality || 0) +
        (tc.scenario || 0) +
        (tc.first_mes || 0) +
        (tc.mes_example || 0) +
        (tc.creator_notes || 0) +
        (tc.system_prompt || 0) +
        (tc.post_history_instructions || 0) +
        (tc.alternate_greetings || 0)
    );
}
</script>

<template>
    <div class="space-y-5">
        <!-- Summary Header -->
        <div
            class="flex items-center justify-between rounded-2xl border border-white/70 bg-white/80 px-5 py-3 shadow-[0_4px_16px_rgba(148,163,184,0.10)] backdrop-blur"
        >
            <div class="flex items-center gap-3">
                <span class="text-sm font-semibold text-slate-700">
                    {{ props.characterData.name || '未命名角色' }}
                </span>
                <span class="inline-flex items-center rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-medium text-slate-600">
                    {{ totalTokens() }} tokens
                </span>
            </div>
        </div>

        <!-- ═══ 核心设定 ═══ -->
        <div class="glass-section">
            <div class="section-title">核心设定</div>

            <!-- 角色描述 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">角色描述</label>
                    <span class="field-token">{{ props.tokenCounts.description || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.description"
                    @blur="onUpdateField('description', props.fullCharacterData?.card?.data?.description || '', props.characterData.description)"
                    class="field-textarea"
                    rows="6"
                    placeholder="角色的物理外观、身份和基本设定"
                ></textarea>
            </div>

            <!-- 性格特点 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">性格特点</label>
                    <span class="field-token">{{ props.tokenCounts.personality || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.personality"
                    @blur="onUpdateField('personality', props.fullCharacterData?.card?.data?.personality || '', props.characterData.personality)"
                    class="field-textarea"
                    rows="8"
                    placeholder="描述角色的性格特征"
                ></textarea>
            </div>

            <!-- 场景设定 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">场景设定</label>
                    <span class="field-token">{{ props.tokenCounts.scenario || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.scenario"
                    @blur="onUpdateField('scenario', props.fullCharacterData?.card?.data?.scenario || '', props.characterData.scenario)"
                    class="field-textarea"
                    rows="5"
                    placeholder="描述角色所处的场景和环境"
                ></textarea>
            </div>
        </div>

        <!-- ═══ 对话内容 ═══ -->
        <div class="glass-section">
            <div class="section-title">对话内容</div>

            <!-- 开场白 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">开场白</label>
                    <span class="field-token">{{ props.tokenCounts.first_mes || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.first_mes"
                    @blur="onUpdateField('first_mes', props.fullCharacterData?.card?.data?.first_mes || '', props.characterData.first_mes)"
                    class="field-textarea"
                    rows="5"
                    placeholder="角色的第一句话或开场问候"
                ></textarea>
            </div>

            <!-- 备用问候语 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">备用问候语</label>
                    <span class="field-token">{{ props.tokenCounts.alternate_greetings || 0 }} tokens</span>
                </div>
                <p class="mb-2 text-xs text-slate-400">
                    使用 &lt;START_ALT&gt; 标记每段备用开场白的开头，可定义多段。
                </p>
                <textarea
                    v-model="props.characterData.alternate_greetings"
                    @blur="onUpdateField('alternate_greetings', props.fullCharacterData?.card?.data?.alternate_greetings || [], splitAlternateGreetings(props.characterData.alternate_greetings || ''))"
                    class="field-textarea"
                    rows="10"
                    placeholder="备用开场白，使用 <START_ALT> 标记每段开头"
                ></textarea>
            </div>

            <!-- 对话示例 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">对话示例</label>
                    <span class="field-token">{{ props.tokenCounts.mes_example || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.mes_example"
                    @blur="onUpdateField('mes_example', props.fullCharacterData?.card?.data?.mes_example || '', props.characterData.mes_example)"
                    class="field-textarea"
                    rows="8"
                    placeholder="示例对话格式，展示角色的说话风格"
                ></textarea>
            </div>
        </div>

        <!-- ═══ 元信息 ═══ -->
        <div class="glass-section">
            <div class="section-title">元信息</div>

            <!-- 创作者笔记 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">创作者笔记</label>
                    <span class="field-token">{{ props.tokenCounts.creator_notes || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.creator_notes"
                    @blur="onUpdateField('creator_notes', props.fullCharacterData?.card?.data?.creator_notes || '', props.characterData.creator_notes)"
                    class="field-textarea"
                    rows="5"
                    placeholder="创作时的备注和说明"
                ></textarea>
            </div>

            <!-- 系统提示词 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">系统提示词</label>
                    <span class="field-token">{{ props.tokenCounts.system_prompt || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.system_prompt"
                    @blur="onUpdateField('system_prompt', props.fullCharacterData?.card?.data?.system_prompt || '', props.characterData.system_prompt)"
                    class="field-textarea"
                    rows="5"
                    placeholder="AI系统使用的提示词"
                ></textarea>
            </div>

            <!-- 历史后指令 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">历史后指令</label>
                    <span class="field-token">{{ props.tokenCounts.post_history_instructions || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.post_history_instructions"
                    @blur="onUpdateField('post_history_instructions', props.fullCharacterData?.card?.data?.post_history_instructions || '', props.characterData.post_history_instructions)"
                    class="field-textarea"
                    rows="4"
                    placeholder="对话历史后的处理指令"
                ></textarea>
            </div>

            <!-- 标签 -->
            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">标签</label>
                    <span class="field-token">{{ props.tokenCounts.tags || 0 }} tokens</span>
                </div>
                <TagInput
                    v-model="props.characterData.tags"
                    placeholder="输入标签后按回车添加"
                    @change="onUpdateField('tags', props.fullCharacterData?.card?.data?.tags || [], $event)"
                />
            </div>

            <!-- 创作者 & 版本 -->
            <div class="grid grid-cols-2 gap-4">
                <div class="field-group">
                    <label class="field-label mb-2 block">创作者</label>
                    <input
                        v-model="props.characterData.creator"
                        @blur="onUpdateField('creator', props.fullCharacterData?.card?.data?.creator || '', props.characterData.creator)"
                        type="text"
                        class="field-input"
                        placeholder="创作者名称"
                    />
                </div>
                <div class="field-group">
                    <label class="field-label mb-2 block">角色版本</label>
                    <input
                        v-model="props.characterData.character_version"
                        @blur="onUpdateField('character_version', props.fullCharacterData?.card?.data?.character_version || '', props.characterData.character_version)"
                        type="text"
                        class="field-input"
                        placeholder="角色卡版本号"
                    />
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
@reference "tailwindcss";

.glass-section {
    @apply rounded-[20px] border border-white/70 bg-white/85 p-5 shadow-[0_8px_24px_rgba(148,163,184,0.12)] backdrop-blur space-y-4;
}

.section-title {
    @apply text-xs font-semibold uppercase tracking-[0.16em] text-slate-400 mb-1;
}

.field-group {
    /* intentionally empty — just a semantic wrapper */
}

.field-header {
    @apply flex items-center justify-between mb-2;
}

.field-label {
    @apply text-sm font-semibold text-slate-700;
}

.field-token {
    @apply text-xs text-slate-400 tabular-nums;
}

.field-textarea {
    @apply w-full rounded-xl border border-slate-200/80 bg-white/90 px-4 py-3 text-sm leading-relaxed text-slate-800 placeholder-slate-300 resize-none transition-all duration-200;
    @apply focus:outline-none focus:border-blue-300 focus:ring-2 focus:ring-blue-500/10;
    min-height: 4.5rem;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: rgba(148, 163, 184, 0.82) transparent;
}

.field-textarea::-webkit-scrollbar {
    width: 8px;
}

.field-textarea::-webkit-scrollbar-track {
    background: transparent;
}

.field-textarea::-webkit-scrollbar-thumb {
    border-radius: 999px;
    border: 2px solid transparent;
    background: rgba(148, 163, 184, 0.78);
    background-clip: padding-box;
}

.field-textarea::-webkit-scrollbar-thumb:hover {
    background: rgba(100, 116, 139, 0.88);
    background-clip: padding-box;
}

.field-input {
    @apply w-full rounded-xl border border-slate-200/80 bg-white/90 px-4 py-2.5 text-sm text-slate-800 placeholder-slate-300 transition-all duration-200;
    @apply focus:outline-none focus:border-blue-300 focus:ring-2 focus:ring-blue-500/10;
}
</style>
