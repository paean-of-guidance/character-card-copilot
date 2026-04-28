<script setup lang="ts">
import TagInput from "./TagInput.vue";
import { parseAlternateGreetingSegments } from "@/utils/characterFieldFormatters";

const props = defineProps<{
    characterData: any;
    fullCharacterData: any;
    tokenCounts: Record<string, number>;
}>();

const emit = defineEmits<{
    (e: "update-field", field: string, oldValue: any, newValue: any): void;
}>();

function onUpdateField(field: string, oldValue: any, newValue: any) {
    emit("update-field", field, oldValue, newValue);
}

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
        <div class="flex items-center justify-between rounded-2xl border border-white/10 bg-white/6 px-5 py-3 backdrop-blur-sm">
            <div class="flex items-center gap-3">
                <span class="text-sm font-semibold text-white/80">
                    {{ props.characterData.name || '未命名角色' }}
                </span>
                <span class="liquid-badge">{{ totalTokens() }} tokens</span>
            </div>
        </div>

        <!-- ═══ 核心设定 ═══ -->
        <div class="editor-section">
            <div class="section-title">核心设定</div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">角色描述</label>
                    <span class="field-token">{{ props.tokenCounts.description || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.description"
                    @blur="onUpdateField('description', props.fullCharacterData?.card?.data?.description || '', props.characterData.description)"
                    class="liquid-textarea"
                    rows="6"
                    placeholder="角色的物理外观、身份和基本设定"
                ></textarea>
            </div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">性格特点</label>
                    <span class="field-token">{{ props.tokenCounts.personality || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.personality"
                    @blur="onUpdateField('personality', props.fullCharacterData?.card?.data?.personality || '', props.characterData.personality)"
                    class="liquid-textarea"
                    rows="8"
                    placeholder="描述角色的性格特征"
                ></textarea>
            </div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">场景设定</label>
                    <span class="field-token">{{ props.tokenCounts.scenario || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.scenario"
                    @blur="onUpdateField('scenario', props.fullCharacterData?.card?.data?.scenario || '', props.characterData.scenario)"
                    class="liquid-textarea"
                    rows="5"
                    placeholder="描述角色所处的场景和环境"
                ></textarea>
            </div>
        </div>

        <!-- ═══ 对话内容 ═══ -->
        <div class="editor-section">
            <div class="section-title">对话内容</div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">开场白</label>
                    <span class="field-token">{{ props.tokenCounts.first_mes || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.first_mes"
                    @blur="onUpdateField('first_mes', props.fullCharacterData?.card?.data?.first_mes || '', props.characterData.first_mes)"
                    class="liquid-textarea"
                    rows="5"
                    placeholder="角色的第一句话或开场问候"
                ></textarea>
            </div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">备用问候语</label>
                    <span class="field-token">{{ props.tokenCounts.alternate_greetings || 0 }} tokens</span>
                </div>
                <p class="mb-2 text-xs text-white/35">
                    使用 &lt;START_ALT&gt; 标记每段备用开场白的开头，可定义多段。
                </p>
                        <textarea
                        v-model="props.characterData.alternate_greetings"
                        @blur="onUpdateField('alternate_greetings', props.fullCharacterData?.card?.data?.alternate_greetings || [], parseAlternateGreetingSegments(props.characterData.alternate_greetings || ''))"
                    class="liquid-textarea"
                    rows="10"
                    placeholder="备用开场白，使用 <START_ALT> 标记每段开头"
                ></textarea>
            </div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">对话示例</label>
                    <span class="field-token">{{ props.tokenCounts.mes_example || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.mes_example"
                    @blur="onUpdateField('mes_example', props.fullCharacterData?.card?.data?.mes_example || '', props.characterData.mes_example)"
                    class="liquid-textarea"
                    rows="8"
                    placeholder="示例对话格式，展示角色的说话风格"
                ></textarea>
            </div>
        </div>

        <!-- ═══ 元信息 ═══ -->
        <div class="editor-section">
            <div class="section-title">元信息</div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">创作者笔记</label>
                    <span class="field-token">{{ props.tokenCounts.creator_notes || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.creator_notes"
                    @blur="onUpdateField('creator_notes', props.fullCharacterData?.card?.data?.creator_notes || '', props.characterData.creator_notes)"
                    class="liquid-textarea"
                    rows="5"
                    placeholder="创作时的备注和说明"
                ></textarea>
            </div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">系统提示词</label>
                    <span class="field-token">{{ props.tokenCounts.system_prompt || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.system_prompt"
                    @blur="onUpdateField('system_prompt', props.fullCharacterData?.card?.data?.system_prompt || '', props.characterData.system_prompt)"
                    class="liquid-textarea"
                    rows="5"
                    placeholder="AI系统使用的提示词"
                ></textarea>
            </div>

            <div class="field-group">
                <div class="field-header">
                    <label class="field-label">历史后指令</label>
                    <span class="field-token">{{ props.tokenCounts.post_history_instructions || 0 }} tokens</span>
                </div>
                <textarea
                    v-model="props.characterData.post_history_instructions"
                    @blur="onUpdateField('post_history_instructions', props.fullCharacterData?.card?.data?.post_history_instructions || '', props.characterData.post_history_instructions)"
                    class="liquid-textarea"
                    rows="4"
                    placeholder="对话历史后的处理指令"
                ></textarea>
            </div>

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
                        class="liquid-input"
                        placeholder="创作者名称"
                    />
                </div>
                <div class="field-group">
                    <label class="field-label mb-2 block">角色版本</label>
                    <input
                        v-model="props.characterData.character_version"
                        @blur="onUpdateField('character_version', props.fullCharacterData?.card?.data?.character_version || '', props.characterData.character_version)"
                        type="text"
                        class="liquid-input"
                        placeholder="角色卡版本号"
                    />
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.editor-section {
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.10);
    background: rgba(255, 255, 255, 0.05);
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
}

.section-title {
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.35);
    margin-bottom: 4px;
}

.field-group {
    /* semantic wrapper */
}

.field-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
}

.field-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.70);
}

.field-token {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.30);
    font-variant-numeric: tabular-nums;
}
</style>
