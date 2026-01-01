<template>
  <div :class="['account-card', { 'current': isCurrent, 'switching': isSwitching, 'selected': isSelected }]" @click="handleCardClick">
    <!-- 选择框 -->
    <div class="selection-checkbox" :class="{ 'visible': selectionMode || isSelected }" @click.stop="toggleSelection">
      <div class="checkbox-inner" :class="{ 'checked': isSelected }">
        <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
        </svg>
      </div>
    </div>

    <!-- Status Badge -->
    <div class="status-indicator">
      <span :class="['status-badge', statusClass]">
        <span :class="['status-dot', statusClass]"></span>
        {{ statusLabel }}
      </span>
    </div>

    <!-- Account Info -->
    <div class="account-info">
      <div class="email-wrapper" @click.stop="copyEmail" v-tooltip="account.email">
        <span class="email-text">{{ showRealEmail ? account.email : maskedEmail }}</span>
        <span :class="['tier-badge', `tier-${subscriptionTier.class}`]">{{ subscriptionTier.label }}</span>
      </div>
      <div class="account-meta">
        <div class="meta-row">
          <span class="created-date" v-tooltip="$t('platform.antigravity.createdAt') + ': ' + formatDate(account.created_at)">C: {{ formatDate(account.created_at) }}</span>
          <span v-if="account.quota?.last_updated" class="created-date" v-tooltip="$t('platform.antigravity.quotaRefreshedAt') + ': ' + formatDate(account.quota.last_updated)">R: {{ formatDate(account.quota.last_updated) }}</span>
        </div>
      </div>
    </div>

    <!-- Quota Display -->
    <div v-if="account.quota && !account.quota.is_forbidden && filteredModels.length > 0" class="quota-section">
      <div v-for="model in filteredModels" :key="model.name" class="quota-item">
        <div class="quota-bar">
          <div
            class="quota-fill"
            :class="getQuotaClass(model.percentage)"
            :style="{ width: model.percentage + '%' }"
          ></div>
          <div class="quota-overlay">
            <span class="model-label">{{ formatModelName(model.name) }}</span>
            <span class="model-time">{{ formatResetCountdown(model.reset_time) }}</span>
            <span :class="['model-percent', getQuotaClass(model.percentage)]">
              {{ model.percentage }}%
            </span>
          </div>
        </div>
      </div>
      <button class="view-all-models" @click.stop="$emit('view-models', account)">
        {{ $t('platform.antigravity.viewAllModels') }}
      </button>
    </div>

    <div v-else-if="account.quota?.is_forbidden" class="quota-forbidden">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
      </svg>
      <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
    </div>

    <div v-else class="quota-empty">
      <span>{{ $t('platform.antigravity.noQuotaData') }}</span>
    </div>

    <!-- Actions -->
    <div class="card-actions">
      <button
        v-if="!isCurrent"
        @click="$emit('switch', account.id)"
        class="btn primary small"
        :disabled="isSwitching"
      >
        <svg v-if="!isSwitching" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <svg v-else class="spin" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
        </svg>
        {{ isSwitching ? $t('platform.antigravity.switching') : $t('platform.antigravity.switch') }}
      </button>

      <button
        @click="$emit('refresh', account.id)"
        class="btn-icon refresh-btn"
        :disabled="isRefreshing"
        v-tooltip="$t('platform.antigravity.refresh')"
      >
        <svg :class="{ 'spin': isRefreshing }" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
      </button>

      <button
        @click="$emit('delete', account.id)"
        class="btn-icon delete-btn"
        v-tooltip="$t('platform.antigravity.delete')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
      </button>
    </div>

  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

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

// 过滤并排序模型,只显示指定的4个模型
const filteredModels = computed(() => {
  if (!props.account.quota || !props.account.quota.models) return []

  const targetModels = [
    'gemini-3-pro-high',
    'gemini-3-pro-image',
    'gemini-3-flash',
    'claude-sonnet-4-5',
    'claude-sonnet-4-5-thinking',
    'claude-opus-4-5-thinking'
  ]

  return props.account.quota.models
    .filter(model => {
      const lowerName = model.name.toLowerCase()
      return targetModels.some(target => lowerName.includes(target))
    })
    .sort((a, b) => {
      // 按照 targetModels 的顺序排序
      const indexA = targetModels.findIndex(target => a.name.toLowerCase().includes(target))
      const indexB = targetModels.findIndex(target => b.name.toLowerCase().includes(target))
      return indexA - indexB
    })
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
    window.$notify?.success($t('messages.copySuccess'))
  } catch (err) {
    console.error('Failed to copy email:', err)
    window.$notify?.error($t('messages.copyFailed'))
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
  if (lowerName.includes('gemini-3-pro-high')) return 'Gemini 3 Pro (High)'
  if (lowerName.includes('gemini-3-pro-image')) return 'Gemini 3 Pro (Image)'
  if (lowerName.includes('gemini-3-flash')) return 'Gemini 3 Flash'
  if (lowerName.includes('gemini-3-pro')) return 'Gemini 3 Pro'
  if (lowerName.includes('gemini')) return 'Gemini'

  // 返回原始名称
  return name
}

const getQuotaClass = (percentage) => {
  if (percentage >= 50) return 'high'
  if (percentage >= 20) return 'medium'
  return 'low'
}

const parseResetTime = (timeStr) => {
  if (!timeStr) return null
  const normalized = /Z$|[+-]\d{2}:\d{2}$/.test(timeStr) ? timeStr : `${timeStr}Z`
  const target = new Date(normalized).getTime()
  return Number.isNaN(target) ? null : target
}

const formatResetCountdown = (timeStr) => {
  const target = parseResetTime(timeStr)
  if (!target) return '--:--'

  const diffMs = Math.max(0, target - Date.now())
  const days = Math.floor(diffMs / 86400000)
  const hours = Math.floor((diffMs % 86400000) / 3600000)
  const minutes = Math.floor((diffMs % 3600000) / 60000)

  if (days > 0) {
    return `R: ${days}d ${String(hours).padStart(2, '0')}h ${String(minutes).padStart(2, '0')}m`
  }

  return `R: ${hours}h ${String(minutes).padStart(2, '0')}m`
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

<style scoped>
.account-card {
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  padding: 20px;
  box-shadow: var(--tech-border-glow);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: fit-content;
  min-height: 140px;
  z-index: 1;
  overflow: hidden;
  /* 优化滚动性能,防止样式丢失 */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-font-smoothing: subpixel-antialiased;
}

.account-card:hover {
  box-shadow: 0 0 0 1px var(--tech-glass-border), 0 8px 32px -8px rgba(0, 0, 0, 0.2), 0 0 20px -5px var(--tech-glow-primary);
  border-color: color-mix(in srgb, var(--accent) 50%, var(--tech-glass-border));
  transform: translateY(-2px);
}

.account-card.current {
  border-color: var(--accent);
  background: var(--tech-card-bg);
  box-shadow: 0 0 0 1px var(--accent), 0 0 25px -5px var(--tech-glow-primary);
}

.account-card.current::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: var(--accent);
  border-radius: 16px 0 0 16px;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.account-card.switching {
  opacity: 0.6;
  pointer-events: none;
}

.account-card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent), 0 0 25px -5px var(--tech-glow-primary);
}

.account-card.selected::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: var(--accent);
  border-radius: 16px 0 0 16px;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

/* 选择框样式 */
.selection-checkbox {
  position: absolute;
  top: 12px;
  left: 12px;
  z-index: 15;
  opacity: 0;
  transform: scale(0.85);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
}

.selection-checkbox.visible,
.account-card:hover .selection-checkbox {
  opacity: 1;
  transform: scale(1);
}

.checkbox-inner {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  border: 1.5px solid var(--border);
  background: var(--bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.checkbox-inner:hover {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
}

.checkbox-inner.checked {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

.checkbox-inner.checked svg {
  color: #fff;
}

.status-indicator {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 6px;
}

.account-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.email-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tier-badge {
  padding: 2px 6px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.2px;
  text-transform: uppercase;
  border: 1px solid transparent;
}

.tier-free {
  color: #94a3b8;
  border-color: rgba(148, 163, 184, 0.45);
  background: rgba(148, 163, 184, 0.12);
}

.tier-pro {
  color: #38bdf8;
  border-color: rgba(56, 189, 248, 0.5);
  background: rgba(56, 189, 248, 0.12);
}

.tier-ultra {
  color: #fbbf24;
  border-color: rgba(251, 191, 36, 0.5);
  background: rgba(251, 191, 36, 0.12);
}

.account-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.meta-row {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.created-date {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--tech-mono-font);
  opacity: 0.7;
}

.quota-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.view-all-models {
  grid-column: 1 / -1;
  align-self: flex-end;
  background: transparent;
  border: none;
  color: var(--accent);
  font-size: 12px;
  font-weight: 600;
  padding: 2px 0;
  cursor: pointer;
}

.view-all-models:hover {
  color: color-mix(in srgb, var(--accent) 85%, white);
}

.quota-item {
  display: flex;
  width: 100%;
}

.quota-bar {
  position: relative;
  width: 100%;
  height: 24px;
  background: color-mix(in srgb, var(--bg-muted) 75%, transparent);
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
}

.quota-fill {
  height: 100%;
  border-radius: 10px;
  transition: width 0.3s;
  opacity: 0.95;
  position: relative;
  z-index: 1;
}

.quota-fill.high { background: rgba(16, 185, 129, 0.45); }
.quota-fill.medium { background: linear-gradient(90deg, #f59e0b, #d97706); }
.quota-fill.low { background: linear-gradient(90deg, #ef4444, #dc2626); }

.quota-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  gap: 8px;
  color: var(--text-strong, #0f172a);
  font-size: 11px;
  font-weight: 600;
  z-index: 2;
  text-shadow: none;
}

.model-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-time {
  font-variant-numeric: tabular-nums;
  color: var(--text-secondary, #4b5563);
}

.model-percent {
  font-variant-numeric: tabular-nums;
}

.model-percent.high { color: #10b981; }
.model-percent.medium { color: #f59e0b; }
.model-percent.low { color: #ef4444; }

:global([data-theme='dark']) .quota-overlay {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

:global([data-theme='dark']) .model-time {
  color: var(--text-secondary, #cbd5e1);
}

:global(:root:not([data-theme='dark'])) .quota-bar {
  background: color-mix(in srgb, var(--bg-muted) 90%, white);
  border-color: var(--border);
}

.quota-forbidden, .quota-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px;
  background: var(--bg-muted);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 13px;
}

.card-actions {
  display: flex;
  gap: 6px;
  margin-top: auto;
}

/* 使用全局 .btn 和 .btn-icon 样式 */
.card-actions .btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: 0;
}

.card-actions .btn-icon {
  flex: 0 0 auto;
}

.spin {
  animation: tech-spin 0.8s linear infinite;
}

@keyframes tech-spin {
  to { transform: rotate(360deg); }
}
</style>
