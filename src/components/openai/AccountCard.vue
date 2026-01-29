<template>
  <div
    class="bg-surface border border-border rounded-lg p-3 cursor-pointer relative group transition-all duration-150 hover:bg-hover hover:border-border-strong"
    :class="{
      'opacity-60 pointer-events-none': isDeleting,
      'border-accent bg-accent/5': isSelected
    }"
    @click="handleCardClick"
  >
    <!-- 头部：选择框 + 邮箱标题 -->
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

      <!-- 邮箱（可点击复制） -->
      <div
        class="inline-flex items-center gap-1 cursor text-[15px] font-semibold text-text truncate flex-1"
        v-tooltip="account.email"
        @click.stop="copyEmail"
      >
        <span>{{ showRealEmail ? account.email : maskedEmail }}</span>
      </div>
    </div>

    <!-- 右上角按钮组（悬停显示） -->
    <div
      class="absolute top-2 right-2 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
      :class="{ 'opacity-100': isMenuOpen }"
      @click.stop
    >
      <!-- 切换按钮（非当前账号） -->
      <button
        v-if="!isCurrent"
        @click="$emit('switch', account.id)"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors"
        :disabled="isSwitching"
        v-tooltip="$t('platform.openai.switch')"
      >
        <svg v-if="!isSwitching" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <span v-else class="w-3.5 h-3.5 border-2 border-accent border-t-transparent rounded-full animate-spin"></span>
      </button>

      <!-- 刷新配额按钮 -->
      <button
        @click="$emit('refresh-quota', account.id)"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors"
        :disabled="isRefreshing"
        v-tooltip="$t('platform.openai.actions.refreshQuota')"
      >
        <svg v-if="!isRefreshing" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <span v-else class="w-3.5 h-3.5 border-2 border-accent border-t-transparent rounded-full animate-spin"></span>
      </button>

      <!-- 操作菜单 -->
      <FloatingDropdown
        ref="menuRef"
        placement="bottom-end"
        :close-on-select="true"
        @open="isMenuOpen = true"
        @close="isMenuOpen = false"
      >
        <template #trigger="{ isOpen }">
          <button
            class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-text transition-colors"
            v-tooltip="$t('app.moreOptions')"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
            </svg>
          </button>
        </template>
        <template #default="{ close }">
          <button @click="handleMenuClick('refreshToken', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <span>{{ $t('platform.openai.actions.refreshToken') }}</span>
          </button>
          <button v-if="account.token?.refresh_token" @click="handleMenuClick('copyRefreshToken', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
            <span>{{ $t('accountCard.copyRefreshToken') }}</span>
          </button>
          <button v-if="account.token?.access_token" @click="handleMenuClick('copyAccessToken', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
            <span>{{ $t('accountCard.copyAccessToken') }}</span>
          </button>
          <button @click="handleMenuClick('delete', close)" class="dropdown-item text-danger hover:bg-danger/10">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
            <span>{{ $t('platform.openai.actions.delete') }}</span>
          </button>
        </template>
      </FloatingDropdown>
    </div>

    <!-- 属性列表 -->
    <div class="flex flex-col gap-1.5">
      <!-- Token 过期时间 -->
      <div v-if="account.token?.expires_at" class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
          </svg>
          <span>{{ $t('platform.openai.tokenExpiresAt') }}</span>
        </div>
        <div class="flex-1 text-[13px] truncate text-text">
          {{ formatDate(account.token.expires_at) }}
        </div>
      </div>

      <!-- 5h 配额 -->
      <div v-if="account.quota?.codex_5h_used_percent !== null && account.quota?.codex_5h_used_percent !== undefined"
           class="flex items-center gap-1 min-h-6">
        <div class="flex flex-col gap-0.5 w-[90px] shrink-0 text-text-muted text-xs">
          <div class="flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>{{ $t('platform.openai.quota5h') }}</span>
          </div>
          <span v-if="account.quota.codex_5h_reset_after_seconds" class="text-[10px] opacity-70 pl-5">
            {{ formatResetTimeShort(account.quota.codex_5h_reset_after_seconds) }}
          </span>
        </div>
        <div class="flex-1 flex items-center gap-1">
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div class="h-full rounded transition-all"
                 :class="getQuotaBarClass(100 - account.quota.codex_5h_used_percent)"
                 :style="{ width: (100 - account.quota.codex_5h_used_percent) + '%' }">
            </div>
          </div>
          <span class="text-[11px] font-medium tabular nums text-text-muted w-8 text-right">
            {{ 100 - account.quota.codex_5h_used_percent }}%
          </span>
        </div>
      </div>

      <!-- 7d 配额 -->
      <div v-if="account.quota?.codex_7d_used_percent !== null && account.quota?.codex_7d_used_percent !== undefined"
           class="flex items-center gap-1 min-h-6">
        <div class="flex flex-col gap-0.5 w-[90px] shrink-0 text-text-muted text-xs">
          <div class="flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>{{ $t('platform.openai.quota7d') }}</span>
          </div>
          <span v-if="account.quota.codex_7d_reset_after_seconds" class="text-[10px] opacity-70 pl-5">
            {{ formatResetTimeLong(account.quota.codex_7d_reset_after_seconds) }}
          </span>
        </div>
        <div class="flex-1 flex items-center gap-1">
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div class="h-full rounded transition-all"
                 :class="getQuotaBarClass(100 - account.quota.codex_7d_used_percent)"
                 :style="{ width: (100 - account.quota.codex_7d_used_percent) + '%' }">
            </div>
          </div>
          <span class="text-[11px] font-medium tabular nums text-text-muted w-8 text-right">
            {{ 100 - account.quota.codex_7d_used_percent }}%
          </span>
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
          <!-- 标签（有标签时显示） -->
          <span
            v-else
            class="badge editable badge--sm"
            :style="{ '--tag-color': account.tag_color || DEFAULT_TAG_COLOR }"
            @click.stop="openTagEditor"
          >
            {{ account.tag }}
          </span>
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
import { useI18n } from 'vue-i18n'
import FloatingDropdown from '../common/FloatingDropdown.vue'
import TagEditorModal from '../token/TagEditorModal.vue'

const { t: $t } = useI18n()

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
  isRefreshing: {
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
  showRealEmail: {
    type: Boolean,
    default: true
  },
  allAccounts: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['refresh', 'refresh-quota', 'delete', 'select', 'switch', 'account-updated'])

const menuRef = ref(null)
const showTagEditor = ref(false)
const isMenuOpen = ref(false)
const DEFAULT_TAG_COLOR = '#f97316'

// 隐藏邮箱
const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@openai.com'
})

// 状态判断
const isActive = computed(() => {
  if (!props.account.token) return false
  if (props.account.token.expires_at) {
    const now = Math.floor(Date.now() / 1000)
    return props.account.token.expires_at > now
  }
  return true
})

// 配额相关
const getQuotaBarClass = (percent) => {
  if (percent === null || percent === undefined) return 'bg-text-muted'
  if (percent < 10) return 'bg-danger'
  if (percent < 30) return 'bg-warning'
  return 'bg-success'
}

const formatResetTimeShort = (seconds) => {
  if (!seconds) return ''
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) return `${hours}h${minutes}m`
  return `${minutes}m`
}

const formatResetTimeLong = (seconds) => {
  if (!seconds) return ''
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const parts = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes}m`)
  return parts.join('')
}

const formatDate = (timestamp) => {
  if (!timestamp) return '-'
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 选择和点击
const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleCardClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  }
}

// 标签操作
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

// 复制操作
const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

const copyRefreshToken = async () => {
  try {
    const refreshToken = props.account.token?.refresh_token
    if (!refreshToken) {
      window.$notify?.error($t('messages.noRefreshToken'))
      return
    }
    await navigator.clipboard.writeText(refreshToken)
    window.$notify?.success($t('messages.refreshTokenCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyFailed'))
  }
}

const copyAccessToken = async () => {
  try {
    const accessToken = props.account.token?.access_token
    if (!accessToken) {
      window.$notify?.error($t('messages.noAccessToken'))
      return
    }
    await navigator.clipboard.writeText(accessToken)
    window.$notify?.success($t('messages.accessTokenCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyFailed'))
  }
}

const handleMenuClick = async (type, close) => {
  close?.()
  switch (type) {
    case 'refreshToken':
      emit('refresh', props.account.id)
      break
    case 'copyRefreshToken':
      await copyRefreshToken()
      break
    case 'copyAccessToken':
      await copyAccessToken()
      break
    case 'delete':
      emit('delete', props.account.id)
      break
  }
}
</script>
