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
            <span :class="planBadgeClasses">{{ planLabel }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <!-- 第二行：日期信息 -->
          <div class="flex flex-wrap items-center justify-start gap-2.5">
            <span class="text-meta" v-tooltip="$t('platform.windsurf.createdAt') + ': ' + formatDate(account.created_at)">C: {{ formatDate(account.created_at) }}</span>
            <span v-if="expiresText" class="text-meta" v-tooltip="$t('platform.windsurf.quota.expires') + ': ' + expiresText">E: {{ expiresText }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Quota Display -->
    <div v-if="account.quota" class="flex flex-col gap-3">
      <QuotaBar
        :label="quotaLabel"
        :percent="remainingPercentage"
        :show-percent="true"
        :low-threshold="20"
        :medium-threshold="50"
      />
    </div>

    <div v-else class="flex items-center justify-center gap-2 rounded-lg bg-muted p-4 text-[13px] text-text-secondary">
      <span>{{ $t('platform.windsurf.noQuotaData') }}</span>
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
        <span v-else class="btn-spinner" aria-hidden="true"></span>
        {{ isSwitching ? $t('platform.windsurf.switching') : $t('platform.windsurf.switch') }}
      </button>

      <button
        @click.stop="$emit('refresh', account.id)"
        class="btn btn--ghost btn--icon shrink-0"
        :disabled="isRefreshing"
        v-tooltip="$t('platform.windsurf.refresh')"
      >
        <svg v-if="!isRefreshing" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
      </button>

      <button
        @click.stop="$emit('delete', account.id)"
        class="btn btn--ghost btn--icon shrink-0 text-danger hover:bg-danger/10 hover:text-danger"
        v-tooltip="$t('platform.windsurf.delete')"
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
  account: { type: Object, required: true },
  isCurrent: { type: Boolean, default: false },
  isSwitching: { type: Boolean, default: false },
  isRefreshing: { type: Boolean, default: false },
  isSelected: { type: Boolean, default: false },
  selectionMode: { type: Boolean, default: false },
  showRealEmail: { type: Boolean, default: true }
})

const emit = defineEmits(['switch', 'refresh', 'delete', 'select'])

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@windsurf.com'
})

const planLabel = computed(() => {
  return props.account.quota?.plan_name || 'Free'
})

const planBadgeClasses = computed(() => {
  const base = 'ml-2 rounded-full px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide border'
  const plan = props.account.quota?.plan_name?.toLowerCase() || 'free'

  if (plan.includes('enterprise')) {
    return `${base} text-amber-400 border-amber-400/50 bg-amber-400/12`
  }
  if (plan.includes('pro')) {
    return `${base} text-sky-400 border-sky-400/50 bg-sky-400/12`
  }
  return `${base} text-slate-400 border-slate-400/45 bg-slate-400/12`
})

const remainingPercentage = computed(() => {
  if (!props.account.quota) return 100
  return Math.max(0, 100 - (props.account.quota.usage_percentage || 0))
})

const quotaLabel = computed(() => {
  if (!props.account.quota) return ''
  const remaining = Math.max(0, (props.account.quota.total_credits || 0) - (props.account.quota.used_credits || 0))
  const total = props.account.quota.total_credits || 0
  return `${remaining}/${total}`
})

const expiresText = computed(() => {
  if (!props.account.quota?.expires_at) return ''
  try {
    const date = new Date(props.account.quota.expires_at)
    if (isNaN(date.getTime())) return ''
    return date.toLocaleDateString()
  } catch {
    return ''
  }
})

const isAvailable = computed(() => {
  return remainingPercentage.value >= 20
})

const statusClass = computed(() => {
  if (props.isCurrent) return 'current'
  if (props.account.disabled) return 'disabled'
  if (isAvailable.value) return 'available'
  if (remainingPercentage.value > 0) return 'low'
  return 'expired'
})

const statusBadgeClass = computed(() => {
  switch (statusClass.value) {
    case 'current': return 'badge--accent-tech'
    case 'disabled': return 'badge--danger-tech'
    case 'available': return 'badge--success-tech'
    case 'low': return 'badge--warning-tech'
    case 'expired': return 'badge--danger-tech'
    default: return ''
  }
})

const statusDotClass = computed(() => {
  switch (statusClass.value) {
    case 'current': return 'text-accent'
    case 'disabled': return 'text-danger'
    case 'available': return 'text-success'
    case 'low': return 'text-warning'
    case 'expired': return 'text-danger'
    default: return ''
  }
})

const statusLabel = computed(() => {
  if (props.isCurrent) return $t('platform.windsurf.status.current')
  if (props.account.disabled) return $t('platform.windsurf.status.disabled')
  if (isAvailable.value) return $t('platform.windsurf.status.available')
  if (remainingPercentage.value > 0) return $t('platform.windsurf.status.low')
  return $t('platform.windsurf.status.expired')
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
