<template>
  <div
    class="bg-surface border border-border rounded-lg p-3 cursor-pointer relative group transition-all duration-150 hover:bg-hover hover:border-border-strong"
    :class="{
      'opacity-60 pointer-events-none': isSwitching,
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

    <!-- 右上角状态徽章 -->
    <div class="absolute right-3 top-3 z-10 flex items-center gap-1.5">
      <span v-if="isCurrent" :class="['badge', statusBadgeClass]">
        <span class="status-dot" :class="statusDotClass"></span>
        {{ statusLabel }}
      </span>
    </div>

    <!-- 右上角按钮组（悬停显示） -->
    <div
      class="absolute right-3 top-3 z-20 flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity"
      :class="{ 'opacity-100': isMenuOpen }"
      @click.stop
    >
      <!-- 切换按钮（非当前账号） -->
      <button
        v-if="!isCurrent"
        @click="$emit('switch', account.id)"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors"
        :disabled="isSwitching"
        v-tooltip="$t('platform.antigravity.switch')"
      >
        <svg v-if="!isSwitching" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <span v-else class="w-3.5 h-3.5 border-2 border-accent border-t-transparent rounded-full animate-spin"></span>
      </button>

      <!-- 刷新配额按钮 -->
      <button
        @click="$emit('refresh', account.id)"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors"
        :disabled="isRefreshing"
        v-tooltip="$t('platform.antigravity.refresh')"
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
        <template #trigger>
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
          <button @click="handleMenuClick('viewModels', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>{{ $t('platform.antigravity.viewAllModels') }}</span>
          </button>
          <button @click="handleMenuClick('copyRefreshToken', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
            <span>{{ $t('accountCard.copyRefreshToken') }}</span>
          </button>
          <button v-if="account.token?.project_id" @click="handleMenuClick('copyProjectId', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z"/>
            </svg>
            <span>{{ $t('accountCard.copyProjectId') }}</span>
          </button>
          <button @click="handleMenuClick('delete', close)" class="dropdown-item text-danger hover:bg-danger/10">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
            <span>{{ $t('platform.antigravity.actions.delete') }}</span>
          </button>
        </template>
      </FloatingDropdown>
    </div>

    <!-- 属性列表 -->
    <div class="flex flex-col gap-1.5">
      <!-- 订阅等级 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
          </svg>
          <span>{{ $t('platform.antigravity.subscriptionTier') }}</span>
        </div>
        <div class="flex-1 text-[13px]">
          <span :class="tierBadgeClasses">{{ subscriptionTier.label }}</span>
        </div>
      </div>

      <!-- Claude 配额 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex flex-col gap-0.5 w-[90px] shrink-0 text-text-muted text-xs">
          <div class="flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>Claude</span>
          </div>
          <span v-if="claudeQuota.resetTime" class="text-[10px] opacity-70 pl-5">
            {{ formatResetCountdown(claudeQuota.resetTime) }}
          </span>
        </div>
        <div class="flex-1 flex items-center gap-1">
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div
              class="h-full rounded transition-all"
              :class="getQuotaBarClass(claudeQuota.percent)"
              :style="{ width: claudeQuota.percent + '%' }"
            ></div>
          </div>
          <span class="text-[11px] font-medium tabular-nums text-text-muted w-8 text-right">
            {{ claudeQuota.percent }}%
          </span>
        </div>
      </div>

      <!-- Gemini 配额 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex flex-col gap-0.5 w-[90px] shrink-0 text-text-muted text-xs">
          <div class="flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>Gemini</span>
          </div>
          <span v-if="geminiQuota.resetTime" class="text-[10px] opacity-70 pl-5">
            {{ formatResetCountdown(geminiQuota.resetTime) }}
          </span>
        </div>
        <div class="flex-1 flex items-center gap-1">
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div
              class="h-full rounded transition-all"
              :class="getQuotaBarClass(geminiQuota.percent)"
              :style="{ width: geminiQuota.percent + '%' }"
            ></div>
          </div>
          <span class="text-[11px] font-medium tabular-nums text-text-muted w-8 text-right">
            {{ geminiQuota.percent }}%
          </span>
        </div>
      </div>

      <!-- 禁用状态提示 -->
      <div v-if="account.quota?.is_forbidden" class="flex items-center gap-2 rounded bg-danger/10 px-2 py-1.5 text-xs text-danger">
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
        </svg>
        <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
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

const emit = defineEmits(['switch', 'refresh', 'delete', 'select', 'view-models', 'account-updated'])

const menuRef = ref(null)
const showTagEditor = ref(false)
const isMenuOpen = ref(false)
const DEFAULT_TAG_COLOR = '#f97316'

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@antigravity.com'
})

// Claude Opus 4.5 Thinking 配额
const claudeQuota = computed(() => {
  if (!props.account.quota?.models) return { percent: 0, resetTime: null }
  const model = props.account.quota.models.find(m =>
    m.name.toLowerCase().includes('claude-opus-4-5-thinking')
  )
  return {
    percent: model?.percentage ?? 0,
    resetTime: model?.reset_time || null
  }
})

// Gemini Pro 3 配额
const geminiQuota = computed(() => {
  if (!props.account.quota?.models) return { percent: 0, resetTime: null }
  const model = props.account.quota.models.find(m =>
    m.name.toLowerCase().includes('gemini-3-pro')
  )
  return {
    percent: model?.percentage ?? 0,
    resetTime: model?.reset_time || null
  }
})

// 配额进度条颜色
const getQuotaBarClass = (percent) => {
  if (percent === null || percent === undefined) return 'bg-text-muted'
  if (percent < 10) return 'bg-danger'
  if (percent < 30) return 'bg-warning'
  return 'bg-success'
}

// 判断账号可用性
const isAvailable = computed(() => {
  if (!props.account.quota || props.account.quota.is_forbidden) return false
  return claudeQuota.value.percent >= 20 && geminiQuota.value.percent >= 20
})

const statusClass = computed(() => {
  if (props.isCurrent) return 'current'
  if (props.account.quota?.is_forbidden) return 'forbidden'
  if (isAvailable.value) return 'available'
  return 'low'
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

const subscriptionTier = computed(() => {
  const tier = props.account.quota?.subscription_tier
  if (!tier) {
    return { label: 'Free', class: 'free' }
  }
  const lowerTier = tier.toLowerCase()
  if (lowerTier.includes('ultra')) {
    return { label: 'Ultra', class: 'ultra' }
  }
  if (tier === 'g1-pro-tier') {
    return { label: 'Pro', class: 'pro' }
  }
  return { label: 'Free', class: 'free' }
})

const tierBadgeClasses = computed(() => {
  const base = 'rounded-full px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide border'
  switch (subscriptionTier.value.class) {
    case 'ultra':
      return `${base} text-amber-400 border-amber-400/50 bg-amber-400/12`
    case 'pro':
      return `${base} text-sky-400 border-sky-400/50 bg-sky-400/12`
    default:
      return `${base} text-slate-400 border-slate-400/45 bg-slate-400/12`
  }
})

const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleCardClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  }
}

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

const parseResetTime = (timeStr) => {
  if (!timeStr) return null
  const normalized = /Z$|[+-]\d{2}:\d{2}$/.test(timeStr) ? timeStr : `${timeStr}Z`
  const target = new Date(normalized).getTime()
  return Number.isNaN(target) ? null : target
}

const formatResetCountdown = (timeStr) => {
  const target = parseResetTime(timeStr)
  if (!target) return ''

  const diffMs = Math.max(0, target - Date.now())
  const days = Math.floor(diffMs / 86400000)
  const hours = Math.floor((diffMs % 86400000) / 3600000)
  const minutes = Math.floor((diffMs % 3600000) / 60000)

  if (days > 0) {
    return `${days}d ${String(hours).padStart(2, '0')}h ${String(minutes).padStart(2, '0')}m`
  }

  return `${hours}h ${String(minutes).padStart(2, '0')}m`
}

// 菜单操作
const handleMenuClick = async (type, close) => {
  close?.()
  switch (type) {
    case 'viewModels':
      emit('view-models', props.account)
      break
    case 'copyRefreshToken':
      await copyRefreshToken()
      break
    case 'copyProjectId':
      await copyProjectId()
      break
    case 'delete':
      emit('delete', props.account.id)
      break
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

const copyProjectId = async () => {
  try {
    const projectId = props.account.token?.project_id
    if (!projectId) {
      window.$notify?.error($t('messages.noProjectId'))
      return
    }
    await navigator.clipboard.writeText(projectId)
    window.$notify?.success($t('messages.projectIdCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyFailed'))
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
</script>
