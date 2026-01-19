<template>
  <div
    :class="[
      'card group relative flex flex-col gap-4 overflow-hidden rounded-2xl border p-5 transition-all duration-300',
      'min-h-[140px] h-fit z-[1]',
      isSwitching ? 'opacity-60 pointer-events-none' : '',
      isSelected ? 'selected border-accent' : ''
    ]"
    @click="handleCardClick"
  >
    <!-- 选择框 -->
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

    <!-- Status Badge -->
    <div class="absolute right-3 top-3 z-10 flex items-center gap-1.5">
      <span :class="['badge', statusBadgeClass]">
        <span class="status-dot" :class="statusDotClass"></span>
        {{ statusLabel }}
      </span>
    </div>

    <!-- Account Info -->
    <div class="flex flex-col gap-1 h-full mt-3.5">
      <div class="flex-1 min-w-0">
        <!-- 第一行：邮箱 -->
        <div class="mb-2.5 min-h-[26px] flex items-center">
          <div class="text-copyable" @click.stop="copyEmail" v-tooltip="account.email">
            <span class="text-copyable__content">{{ showRealEmail ? account.email : maskedEmail }}</span>
            <span :class="tierBadgeClasses">{{ subscriptionTier.label }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <!-- 第二行：日期信息 -->
          <div class="flex flex-wrap items-center justify-start gap-2.5">
            <span class="text-meta" v-tooltip="$t('platform.antigravity.createdAt') + ': ' + formatDate(account.created_at)">C: {{ formatDate(account.created_at) }}</span>
            <span v-if="account.quota?.last_updated" class="text-meta" v-tooltip="$t('platform.antigravity.quotaRefreshedAt') + ': ' + formatDate(account.quota.last_updated)">R: {{ formatDate(account.quota.last_updated) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Quota Display -->
    <div v-if="account.quota && !account.quota.is_forbidden && filteredModels.length > 0" class="flex flex-col gap-3">
      <div v-for="model in filteredModels" :key="model.name" class="flex w-full">
        <QuotaBar
          :label="formatModelName(model.name)"
          :percent="model.percentage"
          :refresh="formatResetCountdown(model.reset_time)"
          :low-threshold="20"
          :medium-threshold="50"
        />
      </div>
      <button
        class="self-end border-0 bg-transparent px-0 py-0.5 text-xs font-semibold text-accent cursor-pointer hover:text-accent/80 hover:underline transition-all"
        @click.stop="$emit('view-models', account)"
      >
        {{ $t('platform.antigravity.viewAllModels') }}
      </button>
    </div>

    <div v-else-if="account.quota?.is_forbidden" class="flex items-center justify-center gap-2 rounded-lg bg-muted p-4 text-[13px] text-text-secondary">
      <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
      </svg>
      <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
    </div>

    <div v-else class="flex items-center justify-center gap-2 rounded-lg bg-muted p-4 text-[13px] text-text-secondary">
      <span>{{ $t('platform.antigravity.noQuotaData') }}</span>
    </div>

    <!-- Actions -->
    <div class="mt-auto flex gap-1.5 justify-end">
      <button
        v-if="!isCurrent"
        @click.stop="$emit('switch', account.id)"
        class="btn btn--primary btn--sm flex-1"
        :disabled="isSwitching"
      >
        <svg v-if="!isSwitching" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <span v-else class="btn-spinner btn-spinner--sm" aria-hidden="true"></span>
        {{ isSwitching ? $t('platform.antigravity.switching') : $t('platform.antigravity.switch') }}
      </button>

      <button
        @click.stop="$emit('refresh', account.id)"
        class="btn btn--ghost btn--icon shrink-0"
        :disabled="isRefreshing"
        v-tooltip="$t('platform.antigravity.refresh')"
      >
        <svg v-if="!isRefreshing" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
      </button>

      <button
        @click.stop="$emit('delete', account.id)"
        class="btn btn--ghost btn--icon shrink-0 text-danger hover:bg-danger/10 hover:text-danger"
        v-tooltip="$t('platform.antigravity.actions.delete')"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
      </button>
    </div>

  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import QuotaBar from '../common/QuotaBar.vue'

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
  }
})

const emit = defineEmits(['switch', 'refresh', 'delete', 'select', 'view-models'])

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@augmentcode.com'
})

// 过滤并排序模型,只显示优先模型
const filteredModels = computed(() => {
  if (!props.account.quota || !props.account.quota.models) return []

  const priorityModels = [
    'claude-opus-4-5-thinking',
    'claude-sonnet-4-5-thinking',
    'claude-sonnet-4-5',
    'gemini-3-pro-high',
    'gemini-3-pro-image',
    'gemini-3-flash'
  ]
  const maxDisplay = 6
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
  const base = 'ml-2 rounded-full px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide border'
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
    console.error('Failed to copy email:', err)
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

const formatModelName = (name) => {
  const lowerName = name.toLowerCase()

  // Claude 模型
  if (lowerName.includes('claude-opus-4-5-thinking')) return 'Claude Opus 4.5 Thinking'
  if (lowerName.includes('claude-sonnet-4-5-thinking')) return 'Claude Sonnet 4.5 Thinking'
  if (lowerName.includes('claude-sonnet-4-5')) return 'Claude Sonnet 4.5'
  if (lowerName.includes('claude-opus')) return 'Claude Opus'
  if (lowerName.includes('claude-sonnet')) return 'Claude Sonnet'
  if (lowerName.includes('claude')) return 'Claude'

  // Gemini 模型
  if (lowerName.includes('gemini-2.5-pro')) return 'Gemini 2.5 Pro'
  if (lowerName.includes('gemini-2.5-flash-thinking')) return 'Gemini 2.5 Flash Thinking'
  if (lowerName.includes('gemini-2.5-flash-lite')) return 'Gemini 2.5 Flash Lite'
  if (lowerName.includes('gemini-2.5-flash')) return 'Gemini 2.5 Flash'
  if (lowerName.includes('gemini-3-pro-high')) return 'Gemini 3 Pro (High)'
  if (lowerName.includes('gemini-3-pro-low')) return 'Gemini 3 Pro (Low)'
  if (lowerName.includes('gemini-3-pro-image')) return 'Gemini 3 Pro (Image)'
  if (lowerName.includes('gemini-3-flash')) return 'Gemini 3 Flash'
  if (lowerName.includes('gemini-3-pro')) return 'Gemini 3 Pro'
  if (lowerName.includes('gemini')) return 'Gemini'

  if (lowerName.includes('gpt-oss-120b-medium')) return 'GPT OSS 120B Medium'
  if (lowerName.includes('rev19-uic3-1p')) return 'Rev19 UIC3 1P'
  if (lowerName.startsWith('chat_')) return `Chat ${name.replace(/^chat_/, '')}`

  // 返回原始名称
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
</script>
