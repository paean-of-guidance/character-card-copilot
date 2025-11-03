<template>
    <div
        class="markdown-content prose prose-sm max-w-none"
        v-html="renderedContent"
    ></div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'

interface Props {
    content: string
}

const props = defineProps<Props>()

// 配置 markdown-it 实例
const md = new MarkdownIt({
    html: true,        // 允许 HTML 标签
    linkify: true,     // 自动链接识别
    typographer: true, // 启用印刷美化
    highlight: function (str: string, lang: string): string {
        if (lang && hljs.getLanguage(lang)) {
            try {
                return '<pre class="hljs"><code>' +
                       hljs.highlight(str, { language: lang, ignoreIllegals: true }).value +
                       '</code></pre>'
            } catch (__) {}
        }
        return '<pre class="hljs"><code>' + (md as any).utils.escapeHtml(str) + '</code></pre>'
    }
})

// 渲染后的内容
const renderedContent = computed(() => {
    if (!props.content) return ''
    return md.render(props.content)
})
</script>

<style scoped>
/* 导入 GitHub 风格的代码高亮样式 */
@import 'highlight.js/styles/github.css';

/* 自定义 Markdown 样式 */
.markdown-content {
    line-height: 1.6;
    color: #333;
}

.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4),
.markdown-content :deep(h5),
.markdown-content :deep(h6) {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    line-height: 1.25;
}

.markdown-content :deep(h1) { font-size: 1.5em; }
.markdown-content :deep(h2) { font-size: 1.3em; }
.markdown-content :deep(h3) { font-size: 1.1em; }

.markdown-content :deep(p) {
    margin-bottom: 0.8em;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
    margin-bottom: 0.8em;
    padding-left: 1.5em;
}

.markdown-content :deep(li) {
    margin-bottom: 0.3em;
}

.markdown-content :deep(code) {
    background-color: #f1f5f9;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: 'Courier New', monospace;
    font-size: 0.9em;
}

.markdown-content :deep(pre) {
    background-color: #f6f8fa;
    border: 1px solid #d1d9e0;
    border-radius: 6px;
    padding: 1em;
    overflow-x: auto;
    margin-bottom: 1em;
}

.markdown-content :deep(pre code) {
    background-color: transparent;
    padding: 0;
    border-radius: 0;
    font-size: 0.9em;
    line-height: 1.45;
}

.markdown-content :deep(blockquote) {
    border-left: 4px solid #d1d9e0;
    padding-left: 1em;
    margin: 1em 0;
    color: #656d76;
    font-style: italic;
}

.markdown-content :deep(a) {
    color: #0969da;
    text-decoration: none;
}

.markdown-content :deep(a:hover) {
    text-decoration: underline;
}

.markdown-content :deep(table) {
    border-collapse: collapse;
    margin-bottom: 1em;
    width: 100%;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
    border: 1px solid #d1d9e0;
    padding: 0.5em;
    text-align: left;
}

.markdown-content :deep(th) {
    background-color: #f6f8fa;
    font-weight: 600;
}

/* GitHub 风格的代码高亮覆盖 */
.markdown-content :deep(.hljs) {
    background: #f6f8fa !important;
    color: #24292f;
}

.markdown-content :deep(.hljs-comment),
.markdown-content :deep(.hljs-quote) {
    color: #6e7781;
    font-style: italic;
}

.markdown-content :deep(.hljs-keyword),
.markdown-content :deep(.hljs-selector-tag),
.markdown-content :deep(.hljs-subst) {
    color: #cf222e;
}

.markdown-content :deep(.hljs-number),
.markdown-content :deep(.hljs-literal),
.markdown-content :deep(.hljs-variable),
.markdown-content :deep(.hljs-template-variable),
.markdown-content :deep(.hljs-tag .hljs-attr) {
    color: #0a3069;
}

.markdown-content :deep(.hljs-string),
.markdown-content :deep(.hljs-doctag) {
    color: #116329;
}

.markdown-content :deep(.hljs-title),
.markdown-content :deep(.hljs-section),
.markdown-content :deep(.hljs-selector-id) {
    color: #8250df;
    font-weight: bold;
}

.markdown-content :deep(.hljs-subst) {
    font-weight: normal;
}

.markdown-content :deep(.hljs-type),
.markdown-content :deep(.hljs-class .hljs-title),
.markdown-content :deep(.hljs-tag),
.markdown-content :deep(.hljs-regexp),
.markdown-content :deep(.hljs-link) {
    color: #1f2328;
}

.markdown-content :deep(.hljs-symbol),
.markdown-content :deep(.hljs-bullet),
.markdown-content :deep(.hljs-built_in),
.markdown-content :deep(.hljs-builtin-name) {
    color: #0969da;
}

.markdown-content :deep(.hljs-meta),
.markdown-content :deep(.hljs-deletion) {
    color: #8250df;
}

.markdown-content :deep(.hljs-addition) {
    color: #116329;
    background: #dcffe4;
}

.markdown-content :deep(.hljs-emphasis) {
    font-style: italic;
}

.markdown-content :deep(.hljs-strong) {
    font-weight: bold;
}
</style>