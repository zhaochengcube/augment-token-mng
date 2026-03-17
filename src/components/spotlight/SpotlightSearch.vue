<template>
  <div class="flex h-screen flex-col overflow-hidden bg-surface text-text" @keydown="handleKeydown">
    <!-- 搜索栏 -->
    <div :class="['flex shrink-0 items-center gap-2.5 px-4 py-3.5', query.trim() ? 'border-b border-border' : '']">
      <svg class="shrink-0 text-text-muted" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        ref="searchInputRef"
        v-model="query"
        :placeholder="$t('spotlight.placeholder')"
        class="flex-1 border-none bg-transparent text-[16px] text-text outline-none placeholder:text-text-muted"
        type="text"
        autocomplete="off"
        spellcheck="false"
      />
      <div v-if="query" class="flex shrink-0 cursor-pointer items-center rounded p-0.5 text-text-muted hover:bg-hover hover:text-text" @click="clearQuery">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </div>
      <kbd class="shrink-0 rounded border border-border bg-muted px-1.5 py-0.5 font-sans text-[11px] text-text-muted">ESC</kbd>
    </div>

    <!-- 结果列表 -->
    <div v-if="query.trim() && filteredBookmarks.length > 0" class="flex-1 overflow-y-auto p-1.5" ref="resultsRef">
      <div
        v-for="(bookmark, index) in filteredBookmarks"
        :key="bookmark.id"
        :class="['flex cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-2 transition-colors duration-75', index === activeIndex ? 'bg-hover' : '']"
        :ref="el => { if (index === activeIndex) activeItemRef = el }"
        @click="openBookmark(bookmark, $event)"
        @mouseenter="activeIndex = index"
      >
        <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-white" :style="{ background: bookmark.tag_color || DEFAULT_TAG_COLOR }">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" opacity="0.9">
            <path d="M17 3H7c-1.1 0-2 .9-2 2v16l7-3 7 3V5c0-1.1-.9-2-2-2z"/>
          </svg>
        </div>
        <div class="min-w-0 flex-1">
          <div class="truncate text-[13px] font-medium text-text" v-html="highlightText(bookmark.name || bookmark.url)"></div>
          <div v-if="bookmark.url" class="mt-px truncate text-[11px] text-text-muted" v-html="highlightText(bookmark.url)"></div>
        </div>
        <div v-if="bookmark.tag" class="shrink-0 rounded-full px-2 py-0.5 text-[11px] font-medium" :style="{ background: `color-mix(in srgb, ${bookmark.tag_color || DEFAULT_TAG_COLOR} 15%, transparent)`, color: bookmark.tag_color || DEFAULT_TAG_COLOR }">
          {{ bookmark.tag }}
        </div>
      </div>
    </div>

    <!-- 空状态：有搜索词无结果 -->
    <div v-else-if="query.trim() && filteredBookmarks.length === 0 && !isLoading" class="flex flex-1 flex-col items-center justify-center gap-2 py-10 text-[13px] text-text-muted">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <span>{{ $t('spotlight.noResults') }}</span>
    </div>

    <!-- 底部状态栏 -->
    <div v-if="query.trim()" class="flex shrink-0 items-center justify-between border-t border-border px-4 py-2 text-[11px] text-text-muted">
      <span>{{ filteredBookmarks.length }} / {{ bookmarks.length }}</span>
      <div class="flex items-center gap-2.5">
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-border bg-muted px-1 py-px font-sans text-[10px]">↑↓</kbd>
          {{ $t('spotlight.navigate') }}
        </span>
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-border bg-muted px-1 py-px font-sans text-[10px]">↵</kbd>
          {{ $t('spotlight.open') }}
        </span>
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-border bg-muted px-1 py-px font-sans text-[10px]">Ctrl↵</kbd>
          {{ $t('spotlight.openInternal') }}
        </span>
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-border bg-muted px-1 py-px font-sans text-[10px]">⇧Esc</kbd>
          {{ $t('spotlight.clearInput') }}
        </span>
        <span v-if="currentShortcutDisplay" class="ml-1 flex items-center gap-1 border-l border-border pl-2.5">
          <kbd class="rounded border border-border bg-muted px-1 py-px font-sans text-[10px]">{{ currentShortcutDisplay }}</kbd>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { useI18n } from 'vue-i18n'
import { match as pinyinMatch } from 'pinyin-pro'

const { t: $t, locale } = useI18n()

const DEFAULT_TAG_COLOR = '#f97316'
const WINDOW_WIDTH = 680
const SEARCH_BAR_HEIGHT = 56
const MAX_WINDOW_HEIGHT = 480

const query = ref('')
const bookmarks = ref([])
const activeIndex = ref(0)
const isLoading = ref(false)
const searchInputRef = ref(null)
const resultsRef = ref(null)
const activeItemRef = ref(null)

// 读取当前全局快捷键用于 footer 展示
const currentShortcutDisplay = computed(() => {
  const saved = localStorage.getItem('atm-spotlight-shortcut')
  return saved || ''
})

// 文本匹配检查（直接 + 拼音）
const textMatch = (target, q) => {
  if (!target) return false
  if (target.toLowerCase().includes(q)) return true
  try {
    const result = pinyinMatch(target, q)
    return result !== null && result !== undefined
  } catch {
    return false
  }
}

// 相关性评分
const scoreBookmark = (b, q) => {
  let score = 0
  const name = (b.name || '').toLowerCase()
  const url = (b.url || '').toLowerCase()
  const tag = (b.tag || '').toLowerCase()
  const desc = (b.description || '').toLowerCase()

  // 名称精确匹配（最高优先级）
  if (name === q) score += 1000
  // 名称以查询开头
  else if (name.startsWith(q)) score += 800
  // 名称包含查询
  else if (name.includes(q)) score += 600

  // 标签包含
  if (tag.includes(q)) score += 400

  // URL 包含
  if (url.includes(q)) score += 200

  // 描述包含
  if (desc.includes(q)) score += 100

  // 拼音匹配（最低优先级，仅在直接文本未匹配时补充）
  if (score === 0) {
    try {
      if (pinyinMatch(b.name, q)) score += 50
      if (pinyinMatch(b.tag, q)) score += 30
      if (pinyinMatch(b.url, q)) score += 10
      if (pinyinMatch(b.description, q)) score += 5
    } catch {}
  }

  return score
}

// 搜索过滤 + 相关性排序
const filteredBookmarks = computed(() => {
  const q = query.value.toLowerCase().trim()
  if (!q) return []
  return bookmarks.value
    .map(b => ({ bookmark: b, score: scoreBookmark(b, q) }))
    .filter(item => item.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, 50)
    .map(item => item.bookmark)
})

// 搜索关键词高亮
const highlightText = (text) => {
  if (!text) return ''
  const q = query.value.trim()
  if (!q) return escapeHtml(text)

  // 先尝试直接文本匹配高亮
  const escaped = escapeRegExp(q)
  const regex = new RegExp(`(${escaped})`, 'gi')
  if (regex.test(text)) {
    return escapeHtml(text).replace(
      new RegExp(`(${escapeRegExp(escapeHtml(q))})`, 'gi'),
      '<span class="text-accent font-semibold">$1</span>'
    )
  }

  // 拼音匹配高亮
  try {
    const indices = pinyinMatch(text, q)
    if (indices && indices.length > 0) {
      const chars = [...text]
      const indexSet = new Set(indices)
      return chars.map((ch, i) =>
        indexSet.has(i)
          ? `<span class="text-accent font-semibold">${escapeHtml(ch)}</span>`
          : escapeHtml(ch)
      ).join('')
    }
  } catch {}

  return escapeHtml(text)
}

const escapeHtml = (str) => {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

const escapeRegExp = (str) => {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

// 动态调整窗口高度
const resizeWindow = async () => {
  try {
    const appWindow = getCurrentWindow()
    const height = query.value.trim() ? MAX_WINDOW_HEIGHT : SEARCH_BAR_HEIGHT
    await appWindow.setSize(new LogicalSize(WINDOW_WIDTH, height))
  } catch {}
}

watch(query, () => {
  activeIndex.value = 0
  nextTick(resizeWindow)
})

watch(activeIndex, async () => {
  await nextTick()
  if (activeItemRef.value && activeItemRef.value.scrollIntoView) {
    activeItemRef.value.scrollIntoView({ block: 'nearest' })
  }
})

const hideSpotlight = async () => {
  try {
    await invoke('toggle_spotlight')
  } catch {
    // fallback silently
  }
}

const loadBookmarks = async () => {
  isLoading.value = true
  try {
    const result = await invoke('bookmark_load_local')
    bookmarks.value = result.bookmarks || []
  } catch (e) {
    console.error('Failed to load bookmarks:', e)
  } finally {
    isLoading.value = false
  }
}

// 打开书签 —— 支持外部和内部浏览器
const openBookmark = async (bookmark, event) => {
  if (!bookmark.url) return
  const useInternal = event?.ctrlKey || event?.metaKey
  try {
    if (useInternal) {
      await invoke('open_internal_browser', { url: bookmark.url, title: bookmark.name || bookmark.url })
    } else {
      await invoke('open_url', { url: bookmark.url })
    }
  } catch (e) {
    console.error('Failed to open URL:', e)
  }
  hideSpotlight()
}

const clearQuery = () => {
  query.value = ''
  searchInputRef.value?.focus()
}

const handleKeydown = (e) => {
  const list = filteredBookmarks.value
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      if (list.length > 0) {
        activeIndex.value = (activeIndex.value + 1) % list.length
      }
      break
    case 'ArrowUp':
      e.preventDefault()
      if (list.length > 0) {
        activeIndex.value = (activeIndex.value - 1 + list.length) % list.length
      }
      break
    case 'Enter':
      e.preventDefault()
      if (list[activeIndex.value]) {
        openBookmark(list[activeIndex.value], e)
      }
      break
    case 'Escape':
      e.preventDefault()
      if (e.shiftKey && query.value) {
        clearQuery()
      } else {
        hideSpotlight()
      }
      break
  }
}

let unlistenFocus = null

// 同步主题和语言：从 localStorage 读取并应用到当前窗口
const syncPreferences = () => {
  try {
    const theme = localStorage.getItem('atm-theme')
    if (theme === 'dark' || theme === 'light') {
      document.documentElement.dataset.theme = theme
      document.documentElement.style.colorScheme = theme
    }
  } catch {}
  try {
    const lang = localStorage.getItem('preferred-language')
    if (lang && (lang === 'zh-CN' || lang === 'en-US')) {
      locale.value = lang
    }
  } catch {}
}

onMounted(async () => {
  syncPreferences()
  resizeWindow()
  await loadBookmarks()
  searchInputRef.value?.focus()

  // 监听后端发来的聚焦事件（窗口重新显示时）
  unlistenFocus = await listen('spotlight-focus', () => {
    syncPreferences()
    query.value = ''
    activeIndex.value = 0
    resizeWindow()
    loadBookmarks()
    nextTick(() => searchInputRef.value?.focus())
  })
})

onUnmounted(() => {
  if (unlistenFocus) unlistenFocus()
})
</script>
