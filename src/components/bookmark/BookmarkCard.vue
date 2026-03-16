<template>
  <div
    class="bg-surface border border-border rounded-lg p-3 cursor-pointer relative group transition-all duration-150 hover:bg-hover hover:border-border-strong"
    :class="{
      'border-accent bg-accent/5': isSelected
    }"
    @click="handleCardClick"
  >
    <!-- 头部：选择框 + 标题 -->
    <div class="flex items-center gap-2 mb-3 pr-16">
      <!-- 选择框（悬浮或选择模式时显示） -->
      <div
        class="selection-checkbox"
        :class="{ 'visible': selectionMode || isSelected }"
        @click.stop="toggleSelection"
      >
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
      </div>

      <!-- 标题 -->
      <h3 class="text-[15px] font-semibold text-text m-0 flex-1 truncate">{{ bookmark.name }}</h3>
    </div>

    <!-- 操作按钮（悬停显示） -->
    <div class="absolute top-2 right-2 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
      <button
        class="w-7 h-7 border-none rounded bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-accent hover:text-white"
        @click.stop="copyUrl"
        :title="$t('common.copyLink')"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
        </svg>
      </button>
      <button
        class="w-7 h-7 border-none rounded bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-danger hover:text-white"
        @click.stop="$emit('delete', bookmark)"
        :title="$t('common.delete')"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
      </button>
    </div>

    <!-- 属性列表 -->
    <div class="flex flex-col gap-1.5">
      <!-- 网站地址 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
          </svg>
          <span>{{ $t('bookmarks.fields.url') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text truncate">
          <a
            v-if="bookmark.url"
            :href="normalizedUrl"
            class="text-accent no-underline hover:underline cursor-pointer"
            @click.stop.prevent="openExternalUrl"
          >{{ displayUrl }}</a>
          <span v-else class="text-text-muted">—</span>
        </div>
      </div>

      <!-- 标签 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.63 5.84C17.27 5.33 16.67 5 16 5L5 5.01C3.9 5.01 3 5.9 3 7v10c0 1.1.9 1.99 2 1.99L16 19c.67 0 1.27-.33 1.63-.84L22 12l-4.37-6.16z"/>
          </svg>
          <span>{{ $t('bookmarks.fields.tag') }}</span>
        </div>
        <div class="flex-1 text-[13px]">
          <!-- 添加标签按钮（无标签时显示） -->
          <span
            v-if="!bookmark.tag"
            class="inline-flex items-center gap-0.5 px-1.5 py-0.5 border border-dashed border-border rounded text-text-muted text-xs cursor-pointer hover:border-accent hover:text-accent transition-colors"
            @click.stop="openTagEditor"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
            </svg>
            {{ $t('tokenList.clickToAddTag') }}
          </span>
          <!-- 标签（有标签时显示，可点击编辑） -->
          <span
            v-else
            class="badge editable badge--sm"
            :style="{ '--tag-color': bookmark.tag_color }"
            @click.stop="openTagEditor"
          >{{ bookmark.tag }}</span>
        </div>
      </div>

      <!-- 描述 (如果有) -->
      <div v-if="bookmark.description" class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 18h12v-2H3v2zM3 6v2h18V6H3zm0 7h18v-2H3v2z"/>
          </svg>
          <span>{{ $t('bookmarks.fields.description') }}</span>
        </div>
        <div
          class="flex-1 text-xs text-text-secondary truncate cursor-pointer hover:text-text"
          v-tooltip="bookmark.description"
          @click.stop="copyDescription"
        >{{ bookmark.description }}</div>
      </div>
    </div>
  </div>

  <!-- 标签编辑模态框 -->
  <TagEditorModal
    v-model:visible="showTagEditor"
    :token="bookmarkAsToken"
    :all-tokens="allBookmarksAsTokens"
    @save="handleTagSave"
    @clear="handleTagClear"
  />
</template>

<script setup>
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import TagEditorModal from '../token/TagEditorModal.vue'

const { t: $t } = useI18n()

const props = defineProps({
  bookmark: {
    type: Object,
    required: true
  },
  isSelected: {
    type: Boolean,
    default: false
  },
  selectionMode: {
    type: Boolean,
    default: false
  },
  allBookmarks: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['edit', 'delete', 'select', 'bookmark-updated'])

const showTagEditor = ref(false)

// 选择和点击
const toggleSelection = () => {
  emit('select', props.bookmark.id)
}

const handleCardClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  } else {
    emit('edit', props.bookmark)
  }
}

// 标签相关
const bookmarkAsToken = computed(() => ({
  tag_name: props.bookmark.tag || '',
  tag_color: props.bookmark.tag_color || ''
}))

const allBookmarksAsTokens = computed(() =>
  props.allBookmarks.map(b => ({
    tag_name: b.tag || '',
    tag_color: b.tag_color || ''
  }))
)

const openTagEditor = () => {
  showTagEditor.value = true
}

const handleTagSave = ({ tagName, tagColor }) => {
  props.bookmark.tag = tagName
  props.bookmark.tag_color = tagColor
  props.bookmark.updated_at = Math.floor(Date.now() / 1000)
  emit('bookmark-updated', props.bookmark)
  window.$notify?.success($t('messages.tagUpdated'))
}

const handleTagClear = () => {
  props.bookmark.tag = ''
  props.bookmark.tag_color = ''
  props.bookmark.updated_at = Math.floor(Date.now() / 1000)
  emit('bookmark-updated', props.bookmark)
  window.$notify?.success($t('messages.tagCleared'))
}

const copyUrl = async () => {
  if (!props.bookmark.url) {
    window.$notify?.warning($t('bookmarks.messages.noUrl'))
    return
  }
  try {
    await navigator.clipboard.writeText(props.bookmark.url)
    window.$notify?.success($t('common.copySuccess'))
  } catch (error) {
    window.$notify?.error($t('common.copyFailed'))
  }
}

const copyDescription = async () => {
  if (!props.bookmark.description) return
  try {
    await navigator.clipboard.writeText(props.bookmark.description)
    window.$notify?.success($t('common.copySuccess'))
  } catch (error) {
    window.$notify?.error($t('common.copyFailed'))
  }
}

// 打开外部链接
const openExternalUrl = async () => {
  const url = normalizedUrl.value
  if (!url) return
  try {
    await invoke('open_url', { url })
  } catch (error) {
    console.error('Failed to open URL:', error)
  }
}

// 标准化 URL（确保有协议前缀）
const normalizedUrl = computed(() => {
  const url = props.bookmark.url
  if (!url) return ''
  if (/^https?:\/\//i.test(url)) return url
  return `https://${url}`
})

// 简化显示的 URL（移除 https:// 等前缀）
const displayUrl = computed(() => {
  if (!props.bookmark.url) return ''
  return props.bookmark.url.replace(/^https?:\/\//i, '').replace(/\/$/, '')
})
</script>
