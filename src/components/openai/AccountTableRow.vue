<template>
  <tr
    :class="[
      'group transition-colors duration-200',
      isCurrent ? 'bg-accent/8 border-l-2 border-l-accent' : '',
      isSelected ? 'bg-accent/10' : '',
      !isCurrent && !isSelected ? 'hover:bg-accent/6' : ''
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

    <!-- 标签 -->
    <td class="overflow-hidden px-2.5 py-3.5 border-b border-border/50 align-middle whitespace-nowrap text-[13px] text-text">
      <!-- 添加标签按钮（无标签时显示） -->
      <span
        v-if="!hasTag"
        class="btn btn--icon-sm btn--dashed"
        v-tooltip="$t('tokenList.clickToAddTag')"
        @click.stop="openTagEditor"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </span>
      <!-- 标签（有标签时显示，可点击编辑） -->
      <span
        v-else
        class="badge editable max-w-[75px]"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ tagDisplayName }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text truncate">
      <div class="flex min-w-0 items-center gap-2">
        <div class="text-copyable min-w-0" @click.stop="copyEmail" v-tooltip="account.email">
          <span class="text-copyable__content block truncate">{{ showRealEmail ? account.email : maskedEmail }}</span>
        </div>
        <!-- 订阅计划徽章 -->
        <span v-if="planType" :class="getPlanBadgeClass(planType)">
          <svg v-if="isApiAccount" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5L2 9l10 12L22 9l-3-6zM9.62 8l1.5-3h1.76l1.5 3H9.62zM11 10v6.68L5.44 10H11zm2 0h5.56L13 16.68V10zm6.26-2h-2.65l-1.5-3h2.65l1.5 3zM6.24 5h2.65l-1.5 3H4.74l1.5-3z"/>
          </svg>
          <svg v-else-if="planType?.toLowerCase() === 'pro'" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5L2 9l10 12L22 9l-3-6zM9.62 8l1.5-3h1.76l1.5 3H9.62zM11 10v6.68L5.44 10H11zm2 0h5.56L13 16.68V10zm6.26-2h-2.65l-1.5-3h2.65l1.5 3zM6.24 5h2.65l-1.5 3H4.74l1.5-3z"/>
          </svg>
          <svg v-else-if="planType?.toLowerCase() === 'team'" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M5 16L3 5l5.5 5L12 4l3.5 6L21 5l-2 11H5zm14 3c0 .6-.4 1-1 1H6c-.6 0-1-.4-1-1v-1h14v1z"/>
          </svg>
          <svg v-else-if="planType?.toLowerCase() === 'plus'" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2l2.4 7.2L22 12l-7.6 2.8L12 22l-2.4-7.2L2 12l7.6-2.8z"/>
          </svg>
          {{ isApiAccount ? 'API' : planType }}
        </span>
      </div>
    </td>

    <!-- 订阅/时间信息 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle whitespace-nowrap text-[13px] text-text">
      <div class="flex flex-col gap-1">
        <!-- API 账号显示 Model -->
        <template v-if="isApiAccount">
          <span v-if="account.api_config?.model" class="text-xs truncate" v-tooltip="'Model: ' + account.api_config.model">
            {{ account.api_config.model }}
          </span>
          <span v-else class="text-text-muted text-xs">-</span>
        </template>
        <!-- OAuth 账号显示订阅到期时间 -->
        <template v-else>
          <span v-if="authInfo?.chatgpt_subscription_active_until" :class="['text-xs', subscriptionExpiryClass]" v-tooltip="$t('platform.openai.subscriptionExpires')">
            {{ formatISODate(authInfo.chatgpt_subscription_active_until) }}
            <span v-if="subscriptionDaysLeftText" class="text-[11px] opacity-80">({{ subscriptionDaysLeftText }})</span>
          </span>
          <span v-else class="text-text-muted text-xs">-</span>
        </template>
      </div>
    </td>

    <!-- 配额/配置信息 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle text-[13px] text-text">
      <!-- API 账号显示配置信息 -->
      <div v-if="isApiAccount && account.api_config" class="flex flex-col gap-1 text-xs">
        <div v-if="account.api_config.model_reasoning_effort" class="flex items-center gap-1">
          <span class="text-text-muted">{{ $t('platform.openai.reasoningEffort') }}:</span>
          <span class="truncate">{{ account.api_config.model_reasoning_effort }}</span>
        </div>
        <div v-if="account.api_config.base_url" class="flex items-center gap-1">
          <span class="text-text-muted">URL:</span>
          <span class="truncate max-w-[150px]">{{ account.api_config.base_url }}</span>
        </div>
        <span v-if="!account.api_config.model_reasoning_effort && !account.api_config.base_url" class="text-text-muted">-</span>
      </div>
      <!-- 禁用账号显示禁用提示 -->
      <div v-else-if="account.quota?.is_forbidden" class="flex items-center gap-1.5 text-xs text-danger">
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
        </svg>
        <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
      </div>
      <!-- OAuth 账号显示配额信息 -->
      <div v-else-if="account.quota && hasQuotaData" class="flex flex-col gap-1">
        <!-- 5h 配额 -->
        <div v-if="account.quota.codex_5h_used_percent !== null && account.quota.codex_5h_used_percent !== undefined" class="flex items-center gap-1">
          <span class="text-xs text-text-muted flex items-center gap-1">
            <svg class="w-3 h-3 opacity-70" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>{{ $t('platform.openai.quota5h') }}</span>
          </span>
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden flex items-center">
            <div class="h-full rounded transition-all" :class="getQuotaBarClass(100 - account.quota.codex_5h_used_percent)" :style="{ width: (100 - account.quota.codex_5h_used_percent) + '%' }"></div>
          </div>
          <span class="text-xs font-medium tabular nums text-text-muted w-8 text-right" :class="getQuotaTextClass(100 - account.quota.codex_5h_used_percent)">{{ 100 - account.quota.codex_5h_used_percent }}%</span>
          <span v-if="account.quota.codex_5h_reset_after_seconds" class="text-[11px] text-text-muted w-12 text-right">{{ formatResetTimeShort(account.quota.codex_5h_reset_after_seconds) }}</span>
        </div>
        <!-- 7d 配额 -->
        <div v-if="account.quota.codex_7d_used_percent !== null && account.quota.codex_7d_used_percent !== undefined" class="flex items-center gap-1">
          <span class="text-xs text-text-muted flex items-center gap-1">
            <svg class="w-3 h-3 opacity-70" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>{{ $t('platform.openai.quota7d') }}</span>
          </span>
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden flex items-center">
            <div class="h-full rounded transition-all" :class="getQuotaBarClass(100 - account.quota.codex_7d_used_percent)" :style="{ width: (100 - account.quota.codex_7d_used_percent) + '%' }"></div>
          </div>
          <span class="text-xs font-medium tabular nums text-text-muted w-8 text-right" :class="getQuotaTextClass(100 - account.quota.codex_7d_used_percent)">{{ 100 - account.quota.codex_7d_used_percent }}%</span>
          <span v-if="account.quota.codex_7d_reset_after_seconds" class="text-[11px] text-text-muted w-12 text-right">{{ formatResetTimeLong(account.quota.codex_7d_reset_after_seconds) }}</span>
        </div>
      </div>
      <span v-else class="text-text-muted text-xs">-</span>
    </td>

    <!-- 操作 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-middle whitespace-nowrap text-[13px] text-text">
      <div class="flex items-center gap-1.5 justify-end">
        <!-- 切换按钮 -->
        <button
          @click.stop="$emit('switch', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isSwitching"
          v-tooltip="$t('platform.openai.actions.switch')"
        >
          <svg v-if="!isSwitching" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z" />
          </svg>
          <span v-else class="btn-spinner btn-spinner--xs text-accent" aria-hidden="true"></span>
        </button>

        <!-- 刷新配额按钮（仅 OAuth 账号） -->
        <button
          v-if="!isApiAccount"
          @click.stop="$emit('refresh-quota', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isRefreshing"
          v-tooltip="$t('platform.openai.actions.refreshQuota')"
        >
          <svg v-if="!isRefreshing" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
          <span v-else class="btn-spinner btn-spinner--xs text-accent" aria-hidden="true"></span>
        </button>

        <!-- 更多菜单 -->
        <FloatingDropdown ref="copyMenuRef" placement="bottom-end" :close-on-select="true" @click.stop>
          <template #trigger>
            <button class="btn btn--ghost btn--icon-sm" v-tooltip="$t('app.moreOptions')">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
              </svg>
            </button>
          </template>
          <template #default="{ close }">
            <button v-if="account.token?.refresh_token" @click="handleCopyMenuClick('copyRefreshToken', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('accountCard.copyRefreshToken') }}</span>
            </button>
            <button v-if="account.token?.access_token" @click="handleCopyMenuClick('copyAccessToken', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
              </svg>
              <span>{{ $t('accountCard.copyAccessToken') }}</span>
            </button>
            <button v-if="isApiAccount && account.api_config?.key" @click="handleCopyMenuClick('copyApiKey', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('accountCard.copyApiKey') }}</span>
            </button>
            <button v-if="isApiAccount && account.api_config?.base_url" @click="handleCopyMenuClick('copyBaseUrl', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
              </svg>
              <span>{{ $t('accountCard.copyBaseUrl') }}</span>
            </button>
            <button @click="handleCopyMenuClick('delete', close)" class="dropdown-item text-danger hover:bg-danger/10">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
              <span>{{ $t('platform.openai.actions.delete') }}</span>
            </button>
          </template>
        </FloatingDropdown>
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
import { useI18n } from 'vue-i18n'
import FloatingDropdown from '../common/FloatingDropdown.vue'
import TagEditorModal from '../token/TagEditorModal.vue'

const { t: $t } = useI18n()

const DEFAULT_TAG_COLOR = '#6366f1'

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

// 复制菜单状态
const copyMenuRef = ref(null)
// 标签编辑器状态
const showTagEditor = ref(false)

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@openai.com'
})

// 标签相关计算属性
const tagDisplayName = computed(() => (props.account.tag ?? '').trim())
const hasTag = computed(() => tagDisplayName.value.length > 0)

const normalizeHexColor = (color) => {
  if (typeof color !== 'string') {
    return DEFAULT_TAG_COLOR
  }
  const trimmed = color.trim()
  if (/^#[0-9A-Fa-f]{6}$/.test(trimmed)) {
    return trimmed
  }
  return DEFAULT_TAG_COLOR
}

const tagBadgeStyle = computed(() => {
  if (!hasTag.value) {
    return {}
  }
  const color = normalizeHexColor(props.account.tag_color || DEFAULT_TAG_COLOR)
  return {
    '--tag-color': color
  }
})

// 将 account 转换为 TagEditorModal 需要的 token 格式
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

// 打开标签编辑器
const openTagEditor = () => {
  showTagEditor.value = true
}

// 标签保存处理
const handleTagSave = ({ tagName, tagColor }) => {
  props.account.tag = tagName
  props.account.tag_color = tagColor
  props.account.updated_at = Math.floor(Date.now() / 1000)
  emit('account-updated', props.account)
  window.$notify?.success($t('messages.tagUpdated'))
}

// 标签清除处理
const handleTagClear = () => {
  props.account.tag = ''
  props.account.tag_color = ''
  props.account.updated_at = Math.floor(Date.now() / 1000)
  emit('account-updated', props.account)
  window.$notify?.success($t('messages.tagCleared'))
}

// 复制菜单操作
const handleCopyMenuClick = async (type, close) => {
  close?.()

  switch (type) {
    case 'copyRefreshToken':
      await copyRefreshToken()
      break
    case 'copyAccessToken':
      await copyAccessToken()
      break
    case 'copyApiKey':
      await copyApiKey()
      break
    case 'copyBaseUrl':
      await copyBaseUrl()
      break
    case 'delete':
      emit('delete', props.account.id)
      break
  }
}

// 复制 refresh token
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
    console.error('Failed to copy refresh token:', err)
    window.$notify?.error($t('messages.copyFailed'))
  }
}

// 复制 access token
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
    console.error('Failed to copy access token:', err)
    window.$notify?.error($t('messages.copyFailed'))
  }
}

// 复制 API Key
const copyApiKey = async () => {
  try {
    const apiKey = props.account.api_config?.key
    if (!apiKey) {
      window.$notify?.error($t('accountCard.noApiKey'))
      return
    }
    await navigator.clipboard.writeText(apiKey)
    window.$notify?.success($t('accountCard.apiKeyCopied'))
  } catch (err) {
    console.error('Failed to copy API key:', err)
    window.$notify?.error($t('messages.copyFailed'))
  }
}

// 复制 Base URL
const copyBaseUrl = async () => {
  try {
    const baseUrl = props.account.api_config?.base_url
    if (!baseUrl) {
      window.$notify?.error($t('accountCard.noBaseUrl'))
      return
    }
    await navigator.clipboard.writeText(baseUrl)
    window.$notify?.success($t('accountCard.baseUrlCopied'))
  } catch (err) {
    console.error('Failed to copy base URL:', err)
    window.$notify?.error($t('messages.copyFailed'))
  }
}

// 账号类型判断
const isApiAccount = computed(() => {
  return props.account.account_type === 'api'
})

// 解析 openai_auth_json
const authInfo = computed(() => {
  if (!props.account.openai_auth_json) return null
  try {
    return JSON.parse(props.account.openai_auth_json)
  } catch {
    return null
  }
})

// 订阅计划类型
const planType = computed(() => {
  if (isApiAccount.value) return 'api'
  return authInfo.value?.chatgpt_plan_type || null
})

// 订阅计划徽章样式
const getPlanBadgeClass = (type) => {
  const base = 'badge badge--sm uppercase shrink-0'
  switch (type?.toLowerCase()) {
    case 'pro':
    case 'api':
      return `${base} bg-gradient-to-r from-rose-400 to-pink-500 text-white border-pink-500/50 shadow-sm shadow-pink-500/30`
    case 'team':
      return `${base} bg-gradient-to-r from-amber-400 to-amber-500 text-amber-900 border-amber-500/50`
    case 'plus':
      return `${base} bg-gradient-to-r from-emerald-400 to-teal-500 text-white border-teal-500/50`
    default:
      return base
  }
}

// 订阅到期剩余天数
const subscriptionDaysLeft = computed(() => {
  if (!authInfo.value?.chatgpt_subscription_active_until) return null
  const now = new Date()
  const expiry = new Date(authInfo.value.chatgpt_subscription_active_until)
  return Math.ceil((expiry - now) / (1000 * 60 * 60 * 24))
})

const subscriptionDaysLeftText = computed(() => {
  if (subscriptionDaysLeft.value === null) return ''
  if (subscriptionDaysLeft.value < 0) return $t('subscriptions.expired')
  if (subscriptionDaysLeft.value === 0) return $t('subscriptions.expirestoday')
  return $t('subscriptions.daysLeft', { days: subscriptionDaysLeft.value })
})

const subscriptionExpiryClass = computed(() => {
  if (subscriptionDaysLeft.value === null) return 'text-text-muted'
  if (subscriptionDaysLeft.value < 0) return 'text-danger'
  if (subscriptionDaysLeft.value < 10) return 'text-danger'
  if (subscriptionDaysLeft.value < 20) return 'text-warning'
  return 'text-success'
})

const formatISODate = (isoString) => {
  if (!isoString) return '-'
  const date = new Date(isoString)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
}

// 配额相关
const hasQuotaData = computed(() => {
  const quota = props.account.quota
  return quota && (
    (quota.codex_5h_used_percent !== null && quota.codex_5h_used_percent !== undefined) ||
    (quota.codex_7d_used_percent !== null && quota.codex_7d_used_percent !== undefined)
  )
})

const getQuotaTextClass = (percent) => {
  if (percent === null || percent === undefined) return 'text-text-muted'
  if (percent < 10) return 'text-danger'
  if (percent < 30) return 'text-warning'
  return 'text-success'
}

const getQuotaBarClass = (percent) => {
  if (percent === null || percent === undefined) return 'bg-text-muted'
  if (percent < 10) return 'bg-danger'
  if (percent < 30) return 'bg-warning'
  return 'bg-success'
}

const formatResetTime = (seconds) => {
  if (!seconds) return '-'
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) {
    return `${hours}h${minutes}m`
  }
  return `${minutes}m`
}

const formatResetTimeShort = (seconds) => {
  if (!seconds) return ''
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (hours > 0) {
    return `${hours}h${minutes}m`
  }
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

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    console.error('Failed to copy email:', err)
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

const formatDate = (timestamp) => {
  if (!timestamp) return '-'
  const date = new Date(timestamp * 1000)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleRowClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  }
}
</script>
