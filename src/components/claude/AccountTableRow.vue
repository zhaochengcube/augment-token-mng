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

    <!-- 服务名称 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <span class="font-medium text-text block truncate" v-tooltip="account.service_name">{{ account.service_name }}</span>
    </td>

    <!-- 网站地址 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <a
        v-if="account.website_url"
        :href="account.website_url"
        class="text-accent no-underline hover:underline truncate block"
        v-tooltip="account.website_url"
        @click.stop.prevent="openExternalUrl"
      >{{ displayUrl }}</a>
      <span v-else class="text-text-muted">-</span>
    </td>

    <!-- 到期时间 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <span :class="expiryStatusClass">{{ formattedExpiryDate }}</span>
      <span v-if="daysLeftText" class="text-[11px] opacity-80 ml-1">({{ daysLeftText }})</span>
    </td>

    <!-- 标签 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <!-- 添加标签按钮（无标签时显示） -->
      <span
        v-if="!account.tag"
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
        :style="{ '--tag-color': account.tag_color || DEFAULT_TAG_COLOR }"
        @click.stop="openTagEditor"
      >{{ account.tag }}</span>
    </td>

    <!-- 操作 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle whitespace-nowrap text-[13px] text-text">
      <div class="flex items-center justify-center gap-1.5">
        <button class="btn btn--ghost btn--icon-sm" @click="$emit('edit', account)" v-tooltip="$t('common.edit')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>
        <button class="btn btn--ghost btn--icon-sm text-danger" @click="$emit('delete', account)" v-tooltip="$t('common.delete')">
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
    :token="accountAsToken"
    :all-tokens="allAccountsAsTokens"
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

const DEFAULT_TAG_COLOR = '#f97316'

const props = defineProps({
  account: {
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
  allAccounts: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['edit', 'delete', 'select', 'account-updated'])

const showTagEditor = ref(false)

// 选择和点击
const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleRowClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  } else {
    emit('edit', props.account)
  }
}

// 标签相关
const accountAsToken = computed(() => ({
  tag_name: props.account.tag || '',
  tag_color: props.account.tag_color || ''
}))

const allAccountsAsTokens = computed(() =>
  props.allAccounts.map(acc => ({
    tag_name: acc.tag || '',
    tag_color: acc.tag_color || ''
  }))
)

const openTagEditor = () => {
  showTagEditor.value = true
}

const handleTagSave = ({ tagName, tagColor }) => {
  props.account.tag = tagName
  props.account.tag_color = tagColor
  props.account.updated_at = Math.floor(Date.now() / 1000)
  emit('account-updated', props.account)
  window.$notify?.success($t('messages.tagUpdated'))
}

const handleTagClear = () => {
  props.account.tag = ''
  props.account.tag_color = ''
  props.account.updated_at = Math.floor(Date.now() / 1000)
  emit('account-updated', props.account)
  window.$notify?.success($t('messages.tagCleared'))
}

// 打开外部链接
const openExternalUrl = async () => {
  const url = props.account.website_url
  if (!url) return
  try {
    await invoke('open_url', { url })
  } catch (error) {
    console.error('Failed to open URL:', error)
  }
}

// 简化显示的 URL（移除 https:// 等前缀）
const displayUrl = computed(() => {
  if (!props.account.website_url) return ''
  return props.account.website_url.replace(/^https?:\/\//i, '').replace(/\/$/, '')
})

// 格式化到期日期
const formattedExpiryDate = computed(() => {
  if (!props.account.expiry_date) return $t('subscriptions.noExpiry')
  const date = new Date(props.account.expiry_date * 1000)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
})

const daysLeft = computed(() => {
  if (!props.account.expiry_date) return null
  const now = Date.now()
  const expiry = props.account.expiry_date * 1000
  return Math.ceil((expiry - now) / (1000 * 60 * 60 * 24))
})

const daysLeftText = computed(() => {
  if (daysLeft.value === null) return ''
  if (daysLeft.value < 0) return $t('subscriptions.expired')
  if (daysLeft.value === 0) return $t('subscriptions.expirestoday')
  return $t('subscriptions.daysLeft', { days: daysLeft.value })
})

// 过期状态样式
const expiryStatusClass = computed(() => {
  if (!props.account.expiry_date) return 'text-text-muted'
  if (daysLeft.value === null) return 'text-text-muted'
  if (daysLeft.value < 0) return 'text-danger'
  if (daysLeft.value < 10) return 'text-danger'
  if (daysLeft.value < 20) return 'text-warning'
  return 'text-success'
})
</script>
