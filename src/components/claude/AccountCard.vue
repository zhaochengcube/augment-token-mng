<template>
  <div
    class="bg-surface border border-border rounded-lg p-3 cursor-pointer relative group transition-all duration-150 hover:bg-hover hover:border-border-strong"
    :class="{
      'opacity-60 pointer-events-none': isDeleting,
      'border-accent bg-accent/5': isSelected
    }"
    @click="handleCardClick"
  >
    <!-- 右上角状态徽章 -->
    <div class="absolute right-3 top-3 z-10 flex items-center gap-1.5">
      <!-- 当前账号指示器 -->
      <span v-if="isCurrent" :class="['badge', statusBadgeClass]">
        <span class="status-dot" :class="statusDotClass"></span>
        {{ statusLabel }}
      </span>
    </div>
    <!-- 右上角悬浮按钮（z-index 更高，覆盖状态徽章） -->
    <div class="absolute right-3 top-3 z-20 flex items-center gap-1.5">
      <!-- 切换按钮（非当前账号，悬浮显示） -->
      <button
        v-if="!isCurrent"
        @click.stop="$emit('switch', account.id)"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors opacity-0 group-hover:opacity-100"
        :disabled="isSwitching"
        v-tooltip="$t('platform.claude.switch')"
      >
        <svg v-if="!isSwitching" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <span v-else class="w-3.5 h-3.5 border-2 border-accent border-t-transparent rounded-full animate-spin"></span>
      </button>
      <!-- 删除按钮（悬浮显示） -->
      <button
        @click.stop="$emit('delete', account)"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-danger hover:text-white transition-colors opacity-0 group-hover:opacity-100"
        v-tooltip="$t('common.delete')"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
      </button>
    </div>

    <!-- 头部：选择框 + 服务名称 -->
    <div class="flex items-center gap-2 mb-3 pr-8">
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

      <!-- 服务名称 -->
      <h3 class="text-[15px] font-semibold text-text m-0 flex-1 truncate">{{ account.service_name }}</h3>
    </div>

    <!-- 属性列表 -->
    <div class="flex flex-col gap-1.5">
      <!-- 网站地址 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.websiteUrl') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text truncate">
          <template v-if="account.website_url">
            <a
              :href="account.website_url"
              class="text-accent no-underline hover:underline cursor-pointer"
              @click.stop.prevent="openExternalUrl"
            >{{ displayUrl }}</a>
          </template>
          <span v-else class="text-text-muted">-</span>
        </div>
      </div>

      <!-- 到期时间 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM9 10H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.expiryDate') }}</span>
        </div>
        <div class="flex-1 text-[13px]">
          <span :class="expiryStatusClass">{{ formattedExpiryDate }}</span>
          <span v-if="daysLeftText" class="text-[11px] opacity-80 ml-1">({{ daysLeftText }})</span>
        </div>
      </div>

      <!-- 标签 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.63 5.84C17.27 5.33 16.67 5 16 5L5 5.01C3.9 5.01 3 5.9 3 7v10c0 1.1.9 1.99 2 1.99L16 19c.67 0 1.27-.33 1.63-.84L22 12l-4.37-6.16z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.tag') }}</span>
        </div>
        <div class="flex-1 text-[13px]">
          <!-- 添加标签按钮（无标签时显示） -->
          <span
            v-if="!account.tag"
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
            :style="{ '--tag-color': account.tag_color || DEFAULT_TAG_COLOR }"
            @click.stop="openTagEditor"
          >{{ account.tag }}</span>
        </div>
      </div>

      <!-- 备注 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 18h12v-2H3v2zm0-5h12v-2H3v2zm0-7h12v-2H3v2z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.notes') }}</span>
        </div>
        <div class="flex-1 text-xs text-text-secondary truncate">
          <template v-if="account.notes">
            <span
              class="cursor-pointer hover:text-text"
              v-tooltip="account.notes"
              @click.stop="copyNotes"
            >{{ account.notes }}</span>
          </template>
          <span v-else class="text-text-muted">-</span>
        </div>
      </div>
    </div>
  </div>

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
  isCurrent: {
    type: Boolean,
    default: false
  },
  isSwitching: {
    type: Boolean,
    default: false
  },
  isDeleting: {
    type: Boolean,
    default: false
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

const emit = defineEmits(['edit', 'delete', 'select', 'switch', 'account-updated'])

const showTagEditor = ref(false)

// 状态相关
const statusClass = computed(() => {
  return props.isCurrent ? 'current' : ''
})

const statusBadgeClass = computed(() => {
  switch (statusClass.value) {
    case 'current':
      return 'badge--success-tech'
    default:
      return ''
  }
})

const statusDotClass = computed(() => {
  switch (statusClass.value) {
    case 'current':
      return 'text-success'
    default:
      return ''
  }
})

const statusLabel = computed(() => {
  if (props.isCurrent) return $t('platform.antigravity.status.current')
  return ''
})

// 选择和点击
const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleCardClick = () => {
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

// 复制备注
const copyNotes = async () => {
  if (!props.account.notes) return
  try {
    await navigator.clipboard.writeText(props.account.notes)
    window.$notify?.success($t('common.copySuccess'))
  } catch (error) {
    window.$notify?.error($t('common.copyFailed'))
  }
}
</script>
