<template>
    <div
        ref="rootRef"
        class="markdown-content max-w-none"
        v-html="renderedContent"
    ></div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'

interface Props {
    content: string
}

const props = defineProps<Props>()

const rootRef = ref<HTMLElement | null>(null)

const COPY_ICON_SVG = `
<svg class="copy-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
</svg>`.trim()

const CHECK_ICON_SVG = `
<svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
    <polyline points="20 6 9 17 4 12"></polyline>
</svg>`.trim()

// 配置 markdown-it 实例
const md = new MarkdownIt({
    html: true,        // 允许 HTML 标签
    linkify: true,     // 自动链接识别
    typographer: true, // 启用印刷美化
})

// 自定义 fenced code block 渲染（接管整个 ```...``` 块的输出）
md.renderer.rules.fence = (tokens, idx) => {
    const token = tokens[idx]
    const info = token.info ? token.info.trim() : ''
    const lang = info.split(/\s+/)[0] || ''
    const source = token.content

    const highlighted =
        lang && hljs.getLanguage(lang)
            ? (() => {
                  try {
                      return hljs.highlight(source, { language: lang, ignoreIllegals: true }).value
                  } catch (__) {
                      return md.utils.escapeHtml(source)
                  }
              })()
            : md.utils.escapeHtml(source)

    const escapedLang = md.utils.escapeHtml(lang)
    const langAttr = escapedLang ? ` data-lang="${escapedLang}"` : ''

    return `<div class="code-block"${langAttr}><button class="code-copy-btn" type="button" aria-label="复制代码" title="复制代码">${COPY_ICON_SVG}${CHECK_ICON_SVG}<span class="copy-label">复制</span></button><pre class="hljs"><code>${highlighted}</code></pre></div>`
}

// 渲染后的内容
const renderedContent = computed(() => {
    if (!props.content) return ''
    return md.render(props.content)
})

async function handleRootClick(event: MouseEvent) {
    const target = event.target as HTMLElement | null
    if (!target) return

    const button = target.closest('.code-copy-btn') as HTMLButtonElement | null
    if (!button) return

    event.preventDefault()
    const codeEl = button.closest('.code-block')?.querySelector('pre code')
    if (!codeEl) return

    const text = codeEl.textContent || ''
    const label = button.querySelector('.copy-label') as HTMLElement | null

    try {
        await navigator.clipboard.writeText(text)
    } catch {
        // 兜底：使用临时 textarea
        const textarea = document.createElement('textarea')
        textarea.value = text
        textarea.style.position = 'fixed'
        textarea.style.opacity = '0'
        document.body.appendChild(textarea)
        textarea.select()
        try {
            document.execCommand('copy')
        } finally {
            document.body.removeChild(textarea)
        }
    }

    button.classList.add('copied')
    if (label) label.textContent = '已复制'

    window.setTimeout(() => {
        button.classList.remove('copied')
        if (label) label.textContent = '复制'
    }, 1500)
}

onMounted(() => {
    rootRef.value?.addEventListener('click', handleRootClick)
})

onUnmounted(() => {
    rootRef.value?.removeEventListener('click', handleRootClick)
})
</script>

<style scoped>
/* 导入 GitHub 暗色主题的代码高亮样式 */
@import 'highlight.js/styles/github-dark.css';

/* 自定义 Markdown 样式 - 暗色液态玻璃风格 */
.markdown-content {
    line-height: 1.6;
    color: rgba(255, 255, 255, 0.85);
    min-width: 0;
    font-size: 0.875rem;
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
    color: rgba(255, 255, 255, 0.95);
}

.markdown-content :deep(h1) { font-size: 1.5em; }
.markdown-content :deep(h2) { font-size: 1.3em; }
.markdown-content :deep(h3) { font-size: 1.1em; }

.markdown-content :deep(p) {
    margin-bottom: 0.8em;
}

.markdown-content :deep(strong) {
    color: rgba(255, 255, 255, 0.95);
    font-weight: 600;
}

.markdown-content :deep(em) {
    color: rgba(255, 255, 255, 0.90);
    font-style: italic;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
    margin-bottom: 0.8em;
    padding-left: 1.5em;
}

.markdown-content :deep(ul) {
    list-style-type: disc;
}

.markdown-content :deep(ol) {
    list-style-type: decimal;
}

.markdown-content :deep(li) {
    margin-bottom: 0.3em;
}

.markdown-content :deep(li::marker) {
    color: rgba(167, 139, 250, 0.6);
}

/* 行内代码 */
.markdown-content :deep(code) {
    background-color: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.10);
    color: rgba(244, 114, 182, 0.95);
    padding: 0.15em 0.4em;
    border-radius: 6px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.875em;
    overflow-wrap: anywhere;
    word-break: break-word;
}

/* 代码块容器（包含 pre + 复制按钮） */
.markdown-content :deep(.code-block) {
    position: relative;
    margin: 1em 0;
    background: rgba(13, 10, 32, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.10);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-radius: 12px;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
    overflow: hidden;
}

/* 代码块容器内的 pre：去掉自身背景和边框，padding 统一 */
.markdown-content :deep(.code-block > pre) {
    background: transparent;
    border: none;
    border-radius: 0;
    padding: 1rem 1.25rem;
    margin: 0;
    max-width: 100%;
    overflow-x: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    word-break: break-word;
    box-shadow: none;
}

/* 语言标签（通过 data-lang 注入） */
.markdown-content :deep(.code-block[data-lang])::before {
    content: attr(data-lang);
    position: absolute;
    top: 0.45rem;
    left: 0.95rem;
    font-size: 0.7rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    color: rgba(255, 255, 255, 0.30);
    letter-spacing: 0.04em;
    text-transform: lowercase;
    pointer-events: none;
    user-select: none;
    z-index: 1;
}

.markdown-content :deep(.code-block[data-lang] > pre) {
    padding-top: 1.85rem;
}

/* 复制按钮 */
.markdown-content :deep(.code-copy-btn) {
    position: absolute;
    top: 0.4rem;
    right: 0.4rem;
    z-index: 2;
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.55rem;
    border-radius: 8px;
    font-size: 0.72rem;
    font-weight: 500;
    line-height: 1;
    color: rgba(255, 255, 255, 0.65);
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.18s ease, background 0.18s ease, color 0.18s ease, border-color 0.18s ease;
}

.markdown-content :deep(.code-block:hover .code-copy-btn),
.markdown-content :deep(.code-copy-btn:focus-visible),
.markdown-content :deep(.code-copy-btn.copied) {
    opacity: 1;
}

.markdown-content :deep(.code-copy-btn:hover) {
    color: rgba(255, 255, 255, 0.95);
    background: rgba(255, 255, 255, 0.14);
    border-color: rgba(255, 255, 255, 0.22);
}

.markdown-content :deep(.code-copy-btn:active) {
    transform: scale(0.97);
}

.markdown-content :deep(.code-copy-btn .copy-icon),
.markdown-content :deep(.code-copy-btn .check-icon) {
    width: 0.85rem;
    height: 0.85rem;
    flex-shrink: 0;
}

.markdown-content :deep(.code-copy-btn .check-icon) {
    display: none;
    color: rgb(74, 222, 128);
}

.markdown-content :deep(.code-copy-btn.copied) {
    color: rgb(134, 239, 172);
    background: rgba(34, 197, 94, 0.14);
    border-color: rgba(74, 222, 128, 0.35);
}

.markdown-content :deep(.code-copy-btn.copied .copy-icon) {
    display: none;
}

.markdown-content :deep(.code-copy-btn.copied .check-icon) {
    display: inline-block;
}

.markdown-content :deep(pre code) {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.90);
    padding: 0;
    border-radius: 0;
    display: block;
    font-size: 0.875em;
    line-height: 1.6;
    white-space: inherit;
    overflow-wrap: inherit;
    word-break: inherit;
}

/* 引用块 */
.markdown-content :deep(blockquote) {
    border-left: 3px solid rgba(167, 139, 250, 0.55);
    padding: 0.4em 0 0.4em 1em;
    margin: 1em 0;
    color: rgba(255, 255, 255, 0.60);
    font-style: italic;
    background: rgba(139, 92, 246, 0.06);
    border-radius: 0 8px 8px 0;
}

.markdown-content :deep(blockquote p:last-child) {
    margin-bottom: 0;
}

/* 链接 */
.markdown-content :deep(a) {
    color: rgb(165, 180, 252);
    text-decoration: none;
    border-bottom: 1px solid rgba(165, 180, 252, 0.35);
    transition: border-color 0.15s ease, color 0.15s ease;
}

.markdown-content :deep(a:hover) {
    color: rgb(199, 210, 254);
    border-bottom-color: rgba(199, 210, 254, 0.75);
}

/* 水平分割线 */
.markdown-content :deep(hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.10);
    margin: 1.5em 0;
}

/* 表格 */
.markdown-content :deep(table) {
    border-collapse: collapse;
    margin: 1em 0;
    width: 100%;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.10);
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 0.5em 0.75em;
    text-align: left;
    color: rgba(255, 255, 255, 0.80);
}

.markdown-content :deep(th) {
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.90);
    font-weight: 600;
}

.markdown-content :deep(tr:hover td) {
    background: rgba(255, 255, 255, 0.03);
}

/* hljs 背景覆盖：github-dark 主题会给 .hljs 加深色背景，这里改为透明由外层容器提供背景 */
.markdown-content :deep(.code-block pre.hljs) {
    background: transparent;
}
.markdown-content :deep(code.hljs) {
    background: transparent;
    padding: 0;
}
</style>
