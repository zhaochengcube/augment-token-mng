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
    <td class="w-11 text-center px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text relative first-cell">
      <div class="inline-flex items-center justify-center h-5 cursor-pointer align-middle leading-none" @click.stop="toggleSelection">
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
      </div>
    </td>

    <!-- 标签 -->
    <td class="w-[60px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
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
        class="badge editable badge--sm max-w-[50px]"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ tagDisplayName }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div class="flex items-center gap-1.5">
        <div class="text-copyable" @click.stop="copyEmail" v-tooltip="account.email">
          <span class="text-copyable__content">{{ showRealEmail ? account.email : maskedEmail }}</span>
        </div>
        <span :class="getTierBadgeClass(subscriptionTier.class)">
          <svg v-if="subscriptionTier.class === 'ultra'" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5L2 9l10 12L22 9l-3-6zM9.62 8l1.5-3h1.76l1.5 3H9.62zM11 10v6.68L5.44 10H11zm2 0h5.56L13 16.68V10zm6.26-2h-2.65l-1.5-3h2.65l1.5 3zM6.24 5h2.65l-1.5 3H4.74l1.5-3z"/>
          </svg>
          <svg v-else-if="subscriptionTier.class === 'pro'" class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M5 16L3 5l5.5 5L12 4l3.5 6L21 5l-2 11H5zm14 3c0 .6-.4 1-1 1H6c-.6 0-1-.4-1-1v-1h14v1z"/>
          </svg>
          {{ subscriptionTier.label }}
        </span>
      </div>
    </td>

    <!-- 时间 -->
    <td class="w-[140px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text-muted">
      <span v-tooltip="$t('platform.antigravity.createdAt')">{{ formatDate(account.created_at) }}</span>
    </td>

    <!-- 配额 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div v-if="account.quota && !account.quota.is_forbidden" class="flex flex-col gap-1.5 min-w-[200px]">
        <!-- Claude 配额 -->
        <div class="flex items-center gap-2">
          <span class="text-text-muted text-xs w-12 shrink-0">Claude</span>
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div
              class="h-full rounded transition-all"
              :class="getQuotaBarClass(claudeQuota.percent)"
              :style="{ width: claudeQuota.percent + '%' }"
            ></div>
          </div>
          <span class="text-[11px] font-medium tabular-nums text-text-muted w-8 text-right">{{ claudeQuota.percent }}%</span>
        </div>
        <!-- Gemini 配额 -->
        <div class="flex items-center gap-2">
          <span class="text-text-muted text-xs w-12 shrink-0">Gemini</span>
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div
              class="h-full rounded transition-all"
              :class="getQuotaBarClass(geminiQuota.percent)"
              :style="{ width: geminiQuota.percent + '%' }"
            ></div>
          </div>
          <span class="text-[11px] font-medium tabular-nums text-text-muted w-8 text-right">{{ geminiQuota.percent }}%</span>
        </div>
      </div>
      <span v-else-if="account.quota?.is_forbidden" class="flex items-center gap-1.5 text-xs text-danger">
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
        </svg>
        <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
      </span>
      <span v-else class="text-text-muted text-xs">-</span>
    </td>

    <!-- 操作 -->
    <td class="w-[110px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text text-center">
      <div class="flex items-center justify-end gap-1">
        <!-- 切换账号 -->
        <button
          @click.stop="$emit('switch', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isSwitching"
          v-tooltip="$t('platform.antigravity.actions.switch')"
        >
          <svg v-if="!isSwitching" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z" />
          </svg>
          <span v-else class="btn-spinner btn-spinner--xs text-accent" aria-hidden="true"></span>
        </button>

        <!-- 刷新配额 -->
        <button
          @click.stop="$emit('refresh', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isRefreshing"
          v-tooltip="$t('platform.antigravity.actions.refresh')"
        >
          <svg v-if="!isRefreshing" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
          <span v-else class="btn-spinner btn-spinner--xs text-accent" aria-hidden="true"></span>
        </button>

        <!-- 操作菜单 -->
        <FloatingDropdown
          ref="menuRef"
          placement="bottom-end"
          :close-on-select="true"
          @click.stop
        >
          <template #trigger>
            <button class="btn btn--ghost btn--icon-sm" v-tooltip="$t('app.moreOptions')">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
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

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@antigravity.com'
})

// Claude 配额
const claudeQuota = computed(() => {
  if (!props.account.quota?.models) return { percent: 0, resetTime: null }
  const model = props.account.quota.models.find(m =>
    m.name.toLowerCase().includes('claude')
  )
  return {
    percent: model?.percentage ?? 0,
    resetTime: model?.reset_time || null
  }
})

// Gemini 配额
const geminiQuota = computed(() => {
  if (!props.account.quota?.models) return { percent: 0, resetTime: null }
  const model = props.account.quota.models.find(m =>
    m.name.toLowerCase().includes('gemini')
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
      return 'badge--accent-tech'
    case 'forbidden':
      return 'badge--danger-tech'
    case 'available':
      return 'badge--success-tech'
    case 'low':
      return 'badge--warning-tech'
    default:
      return ''
  }
})

const statusDotClass = computed(() => {
  switch (statusClass.value) {
    case 'current':
      return 'text-accent'
    case 'forbidden':
      return 'text-danger'
    case 'available':
      return 'text-success'
    case 'low':
      return 'text-warning'
    default:
      return ''
  }
})

const statusLabel = computed(() => {
  if (props.isCurrent) return $t('platform.antigravity.status.current')
  if (props.account.quota?.is_forbidden) return $t('platform.antigravity.status.forbidden')
  if (isAvailable.value) return $t('platform.antigravity.status.available')
  return $t('platform.antigravity.status.low')
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

// 订阅等级徽章样式
const getTierBadgeClass = (tierClass) => {
  const base = 'badge badge--sm uppercase shrink-0'
  switch (tierClass) {
    case 'ultra':
      return `${base} bg-gradient-to-r from-rose-400 to-pink-500 text-white border-pink-500/50 shadow-sm shadow-pink-500/30`
    case 'pro':
      return `${base} bg-gradient-to-r from-amber-400 to-amber-500 text-amber-900 border-amber-500/50`
    default:
      return base
  }
}

// 标签相关
const tagDisplayName = computed(() => (props.account.tag ?? '').trim())
const hasTag = computed(() => tagDisplayName.value.length > 0)

const tagBadgeStyle = computed(() => {
  if (!hasTag.value) return {}
  return { '--tag-color': props.account.tag_color || DEFAULT_TAG_COLOR }
})

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

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
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

const formatDate = (timestamp) => {
  if (!timestamp) return '-'
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
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
