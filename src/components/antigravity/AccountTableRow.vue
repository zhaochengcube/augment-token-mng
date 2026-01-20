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
    <td class="w-11 text-center px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text relative first-cell">
      <div class="inline-flex cursor-pointer" @click.stop="toggleSelection">
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
      </div>
    </td>

    <!-- 账号信息 -->
    <td class="min-w-[220px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center gap-1.5">
          <span :class="['badge', statusBadgeClass]">
            <span class="status-dot" :class="statusDotClass"></span>
            {{ statusLabel }}
          </span>
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
        </div>
        <div class="flex items-center gap-1.5">
          <div class="text-copyable" @click.stop="copyEmail" v-tooltip="account.email">
            <span class="text-copyable__content">{{ showRealEmail ? account.email : maskedEmail }}</span>
          </div>
          <span :class="tierBadgeClasses">{{ subscriptionTier.label }}</span>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-meta" v-tooltip="$t('platform.antigravity.createdAt') + ': ' + formatDate(account.created_at)">C: {{ formatDate(account.created_at) }}</span>
        </div>
      </div>
    </td>

    <!-- 配额 -->
    <td class="min-w-[360px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div v-if="account.quota && !account.quota.is_forbidden && filteredModels.length > 0" class="flex flex-col gap-2">
        <div v-for="(row, rowIndex) in quotaRows" :key="rowIndex" class="grid grid-cols-2 gap-2">
          <div v-for="model in row" :key="model.name" class="flex w-full">
            <QuotaBar
              :label="formatModelNameShort(model.name)"
              :tooltip-label="formatModelNameFull(model.name)"
              :percent="model.percentage"
              :refresh="formatResetCountdown(model.reset_time)"
              :low-threshold="20"
              :medium-threshold="50"
            />
          </div>
        </div>
      </div>
      <span v-else-if="account.quota?.is_forbidden" class="text-text-muted text-xs">{{ $t('platform.antigravity.status.forbidden') }}</span>
      <span v-else class="text-text-muted text-xs">-</span>
    </td>

    <!-- 操作 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center gap-1.5 w-[96px]">
          <!-- 切换账号 -->
          <button
            v-if="!isCurrent"
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

          <!-- 删除 -->
          <button
            @click.stop="$emit('delete', account.id)"
            class="btn btn--ghost btn--icon-sm text-danger hover:bg-danger/10"
            v-tooltip="$t('platform.antigravity.actions.delete')"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
            </svg>
          </button>
        </div>
        <div class="flex items-center gap-1.5">
          <!-- 复制菜单 -->
          <FloatingDropdown ref="copyMenuRef" placement="bottom-end" :close-on-select="true" @click.stop>
            <template #trigger>
              <button class="btn btn--ghost btn--icon-sm" v-tooltip="$t('tokenCard.copyMenu')">
                <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
              </button>
            </template>
            <template #default="{ close }">
              <button @click="handleCopyMenuClick('refreshToken', close)" class="dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
                <span>{{ $t('accountCard.copyRefreshToken') }}</span>
              </button>
              <button v-if="account.token?.project_id" @click="handleCopyMenuClick('projectId', close)" class="dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z"/>
                </svg>
                <span>{{ $t('accountCard.copyProjectId') }}</span>
              </button>
            </template>
          </FloatingDropdown>
          <button
            class="bg-transparent border-0 text-accent text-[11px] font-semibold py-0.5 px-0 cursor-pointer hover:text-accent/80 hover:underline transition-all"
            @click.stop="$emit('view-models', account)"
          >
            {{ $t('platform.antigravity.viewAllModels') }}
          </button>
        </div>
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
import QuotaBar from '../common/QuotaBar.vue'
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

// 复制菜单状态
const copyMenuRef = ref(null)
// 标签编辑器状态
const showTagEditor = ref(false)

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@antigravity.com'
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
    case 'refreshToken':
      await copyRefreshToken()
      break
    case 'projectId':
      await copyProjectId()
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

// 复制 project ID
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
    console.error('Failed to copy project ID:', err)
    window.$notify?.error($t('messages.copyFailed'))
  }
}

// 过滤并排序模型,只显示优先模型
const filteredModels = computed(() => {
  if (!props.account.quota || !props.account.quota.models) return []

  const priorityModels = [
    'claude-opus-4-5-thinking',
    'gemini-3-pro-high',
    'gemini-3-pro-image',
    'gemini-3-flash'
  ]
  const maxDisplay = 4
  const normalizedPriority = priorityModels.map((model) => model.toLowerCase())

  const prioritized = props.account.quota.models
    .map((model) => {
      const index = normalizedPriority.findIndex((target) => model.name.toLowerCase().includes(target))
      return { model, index }
    })
    .filter((entry) => entry.index !== -1)
    .sort((a, b) => a.index - b.index)
    .map((entry) => entry.model)

  const fallback = props.account.quota.models
    .filter((model) => !prioritized.some((entry) => entry.name === model.name))
    .sort((a, b) => a.name.localeCompare(b.name))

  return [...prioritized, ...fallback].slice(0, maxDisplay)
})

const quotaRows = computed(() => {
  const rows = []
  const models = filteredModels.value
  for (let i = 0; i < models.length; i += 2) {
    rows.push(models.slice(i, i + 2))
  }
  return rows
})

const isAvailable = computed(() => {
  if (!props.account.quota || props.account.quota.is_forbidden) return false
  const gemini = props.account.quota.models.find(m => m.name.toLowerCase().includes('gemini'))?.percentage || 0
  const claude = props.account.quota.models.find(m => m.name.toLowerCase().includes('claude'))?.percentage || 0
  return gemini >= 20 && claude >= 20
})

const statusClass = computed(() => {
  if (props.isCurrent) return 'current'
  if (props.account.quota?.is_forbidden) return 'forbidden'
  if (isAvailable.value) return 'available'
  return 'low'
})

// Badge classes for status - using Tailwind
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

// Tier badge Tailwind classes - 完整的内联样式
const tierBadgeClasses = computed(() => {
  const base = 'inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wide leading-none border'
  switch (subscriptionTier.value.class) {
    case 'ultra':
      return `${base} text-amber-400 border-amber-400/50 bg-amber-400/12`
    case 'pro':
      return `${base} text-sky-400 border-sky-400/50 bg-sky-400/12`
    default:
      return `${base} text-slate-400 border-slate-400/45 bg-slate-400/12`
  }
})

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    console.error('Failed to copy email:', err)
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

// 模型名称简写 - 只处理4个优先模型
const formatModelNameShort = (name) => {
  const lowerName = name.toLowerCase()

  if (lowerName.includes('claude-opus-4-5-thinking')) return 'C O4.5 T'
  if (lowerName.includes('gemini-3-pro-high')) return 'G3 Pro'
  if (lowerName.includes('gemini-3-pro-image')) return 'G3 Image'
  if (lowerName.includes('gemini-3-flash')) return 'G3 Flash'

  return name
}

// 模型名称完整显示 - 只处理4个优先模型
const formatModelNameFull = (name) => {
  const lowerName = name.toLowerCase()

  if (lowerName.includes('claude-opus-4-5-thinking')) return 'Claude Opus 4.5 Thinking'
  if (lowerName.includes('gemini-3-pro-high')) return 'Gemini 3 Pro (High)'
  if (lowerName.includes('gemini-3-pro-image')) return 'Gemini 3 Pro (Image)'
  if (lowerName.includes('gemini-3-flash')) return 'Gemini 3 Flash'

  return name
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
