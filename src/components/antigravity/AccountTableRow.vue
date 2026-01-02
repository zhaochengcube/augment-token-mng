<template>
  <tr :class="['account-table-row', { 'selected': isSelected, 'current': isCurrent }]" @click="handleRowClick">
    <!-- 多选框 -->
    <td class="cell-checkbox">
      <div class="checkbox-wrapper" @click.stop="toggleSelection">
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
      </div>
    </td>

    <!-- 账号信息 -->
    <td class="cell-info">
      <div class="info-wrapper">
        <div class="status-row">
          <span :class="['status-badge', statusClass]">
            <span :class="['status-dot', statusClass]"></span>
            {{ statusLabel }}
          </span>
          <span :class="['tier-badge', `tier-${subscriptionTier.class}`]">{{ subscriptionTier.label }}</span>
        </div>
        <div class="email-wrapper" @click.stop="copyEmail" v-tooltip="account.email">
          <span class="email-text">{{ showRealEmail ? account.email : maskedEmail }}</span>
        </div>
        <div class="dates-wrapper">
          <span class="created-date" v-tooltip="$t('platform.antigravity.createdAt') + ': ' + formatDate(account.created_at)">C: {{ formatDate(account.created_at) }}</span>
          <span v-if="account.quota?.last_updated" class="created-date" v-tooltip="$t('platform.antigravity.quotaRefreshedAt') + ': ' + formatDate(account.quota.last_updated)">R: {{ formatDate(account.quota.last_updated) }}</span>
        </div>
      </div>
    </td>

    <!-- 配额 -->
    <td class="cell-quota">
      <div v-if="account.quota && !account.quota.is_forbidden && filteredModels.length > 0" class="quota-wrapper">
        <div v-for="(row, rowIndex) in quotaRows" :key="rowIndex" class="quota-row">
          <div v-for="model in row" :key="model.name" class="quota-item">
            <div class="quota-bar">
              <div class="quota-fill" :class="getQuotaClass(model.percentage)" :style="{ width: `${model.percentage}%` }"></div>
              <div class="quota-overlay">
                <span class="model-label">{{ formatModelName(model.name) }}</span>
                <span class="model-time">{{ formatResetCountdown(model.reset_time) }}</span>
                <span :class="['model-percent', getQuotaClass(model.percentage)]">
                  {{ model.percentage }}%
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <span v-else-if="account.quota?.is_forbidden" class="no-quota">{{ $t('platform.antigravity.status.forbidden') }}</span>
      <span v-else class="no-quota">-</span>
    </td>

    <!-- 操作 -->
    <td class="cell-actions">
      <div class="actions-wrapper">
        <!-- 切换账号 -->
        <button v-if="!isCurrent" @click.stop="$emit('switch', account.id)" class="btn-icon switch" :disabled="isSwitching" v-tooltip="$t('platform.antigravity.actions.switch')">
          <svg v-if="!isSwitching" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z" />
          </svg>
          <div v-else class="loading-spinner-small"></div>
        </button>

        <!-- 刷新配额 -->
        <button @click.stop="$emit('refresh', account.id)" class="btn-icon refresh" :disabled="isRefreshing" v-tooltip="$t('platform.antigravity.actions.refresh')">
          <svg v-if="!isRefreshing" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
          <div v-else class="loading-spinner-small"></div>
        </button>

        <!-- 删除 -->
        <button @click.stop="$emit('delete', account.id)" class="btn-icon delete" v-tooltip="$t('platform.antigravity.actions.delete')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
          </svg>
        </button>
      </div>
      <button class="view-all-models" @click.stop="$emit('view-models', account)">
        {{ $t('platform.antigravity.viewAllModels') }}
      </button>
    </td>
  </tr>
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

  // Claude 模型 - 缩写为 C
  if (lowerName.includes('claude-opus-4-5-thinking')) return 'C Opus 4.5 Thinking'
  if (lowerName.includes('claude-sonnet-4-5-thinking')) return 'C Sonnet 4.5 Thinking'
  if (lowerName.includes('claude-sonnet-4-5')) return 'C Sonnet 4.5'
  if (lowerName.includes('claude-opus')) return 'C Opus'
  if (lowerName.includes('claude-sonnet')) return 'C Sonnet'
  if (lowerName.includes('claude')) return 'C'

  // Gemini 模型 - Gemini 3 缩写为 G3
  if (lowerName.includes('gemini-3-pro-high')) return 'G3 Pro (High)'
  if (lowerName.includes('gemini-3-pro-image')) return 'G3 Pro (Image)'
  if (lowerName.includes('gemini-3-flash')) return 'G3 Flash'
  if (lowerName.includes('gemini-3-pro')) return 'G3 Pro'
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

const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleRowClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  }
}
</script>

<style scoped>
.account-table-row {
  background: transparent;
  transition: background-color 0.2s ease;
  cursor: pointer;
}

.account-table-row:hover {
  background-color: color-mix(in srgb, var(--accent) 6%, transparent);
}

.account-table-row.selected {
  background-color: color-mix(in srgb, var(--accent) 10%, transparent);
}

.account-table-row.current {
  background-color: color-mix(in srgb, var(--accent) 8%, transparent);
}

.account-table-row td {
  padding: 14px 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  vertical-align: top;
  white-space: nowrap;
  background: inherit;
  font-size: 13px;
  color: var(--text);
}

/* 第一个单元格添加左侧指示条 */
.account-table-row td:first-child {
  padding-left: 16px;
  position: relative;
}

.account-table-row td:first-child::before {
  content: '';
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: transparent;
  transition: all 0.25s ease;
}

.account-table-row:hover td:first-child::before {
  background: var(--accent);
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

.account-table-row.selected td:first-child::before,
.account-table-row.current td:first-child::before {
  background: var(--accent);
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

/* 复选框 */
.cell-checkbox {
  width: 44px;
  text-align: center;
}

.cell-info {
  min-width: 220px;
}

.cell-quota {
  min-width: 360px;
}

.checkbox-wrapper {
  display: inline-flex;
  cursor: pointer;
}

.checkbox-inner {
  width: 18px;
  height: 18px;
  border-radius: 5px;
  border: 1.5px solid var(--border);
  background: var(--bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.checkbox-inner:hover {
  border-color: var(--accent);
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

/* 配额 */
.info-wrapper {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.info-wrapper .status-badge {
  align-self: center;
  padding: 2px 8px;
  font-size: 10px;
}

.tier-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.2px;
  text-transform: uppercase;
  line-height: 1;
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

.quota-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quota-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
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

.no-quota {
  color: var(--text-soft);
  font-size: 12px;
}

/* 日期 */
.dates-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.created-date {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--tech-mono-font);
  opacity: 0.7;
}

/* 操作按钮 */
.actions-wrapper {
  display: flex;
  align-items: center;
  gap: 5px;
}

.view-all-models {
  margin-top: 6px;
  background: transparent;
  border: none;
  color: var(--accent);
  font-size: 11px;
  font-weight: 600;
  padding: 2px 0;
  cursor: pointer;
}

.view-all-models:hover {
  color: color-mix(in srgb, var(--accent) 85%, white);
}

.btn-icon {
  width: 26px;
  height: 26px;
  padding: 0;
}

.loading-spinner-small {
  width: 12px;
  height: 12px;
  border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: tech-spin 0.8s linear infinite;
}

@keyframes tech-spin {
  to { transform: rotate(360deg); }
}
</style>
