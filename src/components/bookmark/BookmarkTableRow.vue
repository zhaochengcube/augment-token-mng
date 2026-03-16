<template>
  <tr
    :class="[
      'group transition-colors duration-200',
      'hover:bg-accent/6',
      isSelected ? 'bg-accent/10' : ''
    ]"
    @click="handleRowClick"
  >
    <!-- 多选框 -->
    <td class="w-11 text-center py-3.5 border-b border-border/50 align-middle whitespace-nowrap text-[13px] text-text relative first-cell">
      <div class="inline-flex items-center justify-center h-5 cursor-pointer align-middle leading-none" @click.stop="toggleSelection">
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
      </div>
    </td>

    <!-- 名称 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <span
        class="font-medium text-text block truncate"
        v-tooltip="bookmark.name"
      >{{ bookmark.name }}</span>
    </td>

    <!-- 网站地址 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <a
        v-if="bookmark.url"
        :href="normalizedUrl"
        class="text-accent no-underline hover:underline truncate block"
        v-tooltip="bookmark.url"
        @click.stop.prevent="openExternalUrl"
      >{{ displayUrl }}</a>
      <span v-else class="text-text-muted">-</span>
    </td>

    <!-- 描述 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <span
        v-if="bookmark.description"
        class="text-text-secondary truncate block cursor-pointer hover:text-text"
        v-tooltip="bookmark.description"
        @click.stop="copyDescription"
      >
        {{ bookmark.description }}
      </span>
      <span v-else class="text-text-muted">-</span>
    </td>

    <!-- 标签 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <!-- 添加标签按钮（无标签时显示） -->
      <span
        v-if="!bookmark.tag"
        class="inline-flex items-center gap-0.5 px-1.5 py-0.5 border border-dashed border-border rounded text-text-muted text-xs cursor-pointer hover:border-accent hover:text-accent transition-colors"
        @click.stop="openTagEditor"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </span>
      <!-- 标签（有标签时显示，可点击编辑） -->
      <span
        v-else
        class="badge editable badge--sm"
        :style="{ '--tag-color': bookmark.tag_color }"
        @click.stop="openTagEditor"
      >{{ bookmark.tag }}</span>
    </td>

    <!-- 操作 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle whitespace-nowrap text-[13px] text-text">
      <div class="flex items-center justify-center gap-1.5">
        <button class="btn btn--ghost btn--icon-sm" @click.stop="copyUrl" v-tooltip="$t('common.copyLink')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
          </svg>
        </button>
        <button class="btn btn--ghost btn--icon-sm text-danger" @click.stop="$emit('delete', bookmark)" v-tooltip="$t('common.delete')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </div>
    </td>
  </tr>

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

const toggleSelection = () => {
  emit('select', props.bookmark.id)
}

const handleRowClick = () => {
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

const handleTagClear = () => {
  props.bookmark.tag = ''
  props.bookmark.tag_color = ''
  props.bookmark.updated_at = Math.floor(Date.now() / 1000)
  emit('bookmark-updated', props.bookmark)
  window.$notify?.success($t('messages.tagCleared'))
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

// 标准化 URL
const normalizedUrl = computed(() => {
  const url = props.bookmark.url
  if (!url) return ''
  if (/^https?:\/\//i.test(url)) return url
  return `https://${url}`
})

// 简化显示的 URL
const displayUrl = computed(() => {
  if (!props.bookmark.url) return ''
  return props.bookmark.url.replace(/^https?:\/\//i, '').replace(/\/$/, '')
})
</script>
