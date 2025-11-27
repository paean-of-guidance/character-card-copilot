<script setup lang="ts">
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
</script>

<template>
    <div class="space-y-4">
        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    角色描述
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.description || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.description"
                @blur="
                    onUpdateField(
                        'description',
                        props.fullCharacterData?.card?.data?.description || '',
                        props.characterData.description,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="5"
                placeholder="角色的物理外观、身份和基本设定"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    性格特点
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.personality || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.personality"
                @blur="
                    onUpdateField(
                        'personality',
                        props.fullCharacterData?.card?.data?.personality || '',
                        props.characterData.personality,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="6"
                placeholder="描述角色的性格特征"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    场景设定
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.scenario || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.scenario"
                @blur="
                    onUpdateField(
                        'scenario',
                        props.fullCharacterData?.card?.data?.scenario || '',
                        props.characterData.scenario,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="3"
                placeholder="描述角色所处的场景和环境"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    开场白
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.first_mes || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.first_mes"
                @blur="
                    onUpdateField(
                        'first_mes',
                        props.fullCharacterData?.card?.data?.first_mes || '',
                        props.characterData.first_mes,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="4"
                placeholder="角色的第一句话或开场问候"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    对话示例
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.mes_example || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.mes_example"
                @blur="
                    onUpdateField(
                        'mes_example',
                        props.fullCharacterData?.card?.data?.mes_example || '',
                        props.characterData.mes_example,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="6"
                placeholder="示例对话格式，展示角色的说话风格"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    创作者笔记
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.creator_notes || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.creator_notes"
                @blur="
                    onUpdateField(
                        'creator_notes',
                        props.fullCharacterData?.card?.data?.creator_notes || '',
                        props.characterData.creator_notes,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="4"
                placeholder="创作时的备注和说明"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    系统提示词
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.system_prompt || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.system_prompt"
                @blur="
                    onUpdateField(
                        'system_prompt',
                        props.fullCharacterData?.card?.data?.system_prompt || '',
                        props.characterData.system_prompt,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="4"
                placeholder="AI系统使用的提示词"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    历史后指令
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.post_history_instructions || 0 }}
                    tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.post_history_instructions"
                @blur="
                    onUpdateField(
                        'post_history_instructions',
                        props.fullCharacterData?.card?.data?.post_history_instructions || '',
                        props.characterData.post_history_instructions,
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="3"
                placeholder="对话历史后的处理指令"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    备用问候语
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.alternate_greetings || 0 }} tokens
                </span>
            </div>
            <textarea
                v-model="props.characterData.alternate_greetings"
                @blur="
                    onUpdateField(
                        'alternate_greetings',
                        props.fullCharacterData?.card?.data?.alternate_greetings || [],
                        splitAlternateGreetings(
                            props.characterData.alternate_greetings || '',
                        )
                    )
                "
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3 resize-none"
                rows="3"
                placeholder="备用开场白，使用 <START_ALT> 标记每段开头"
            ></textarea>
        </div>

        <div>
            <div class="flex items-center justify-between mb-2">
                <label class="block text-sm font-semibold text-gray-700">
                    标签
                </label>
                <span class="text-xs text-gray-500">
                    {{ props.tokenCounts.tags || 0 }} tokens
                </span>
            </div>
            <input
                v-model="props.characterData.tags"
                @blur="
                    onUpdateField(
                        'tags',
                        props.fullCharacterData?.card?.data?.tags || [],
                        props.characterData.tags
                            .split(',')
                            .map((t: string) => t.trim())
                    )
                "
                type="text"
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                placeholder="角色标签，用逗号分隔"
            />
        </div>

        <div>
            <label class="block text-sm font-semibold text-gray-700 mb-2"
                >创作者</label
            >
            <input
                v-model="props.characterData.creator"
                @blur="
                    onUpdateField(
                        'creator',
                        props.fullCharacterData?.card?.data?.creator || '',
                        props.characterData.creator,
                    )
                "
                type="text"
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                placeholder="创作者名称"
            />
        </div>

        <div>
            <label class="block text-sm font-semibold text-gray-700 mb-2"
                >角色版本</label
            >
            <input
                v-model="props.characterData.character_version"
                @blur="
                    onUpdateField(
                        'character_version',
                        props.fullCharacterData?.card?.data?.character_version || '',
                        props.characterData.character_version,
                    )
                "
                type="text"
                class="w-full bg-white border border-gray-200 rounded-lg px-4 py-3"
                placeholder="角色卡版本号"
            />
        </div>
    </div>
</template>
