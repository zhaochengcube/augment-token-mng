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

    <!-- 邮箱 -->
    <td class="cell-email">
      <div class="email-wrapper" @click.stop="copyEmail" :title="account.email">
        <span class="email-text">{{ showRealEmail ? account.email : maskedEmail }}</span>
      </div>
    </td>

    <!-- 状态 -->
    <td class="cell-status">
      <span v-if="isCurrent" class="status-badge current">
        {{ $t('platform.antigravity.status.current') }}
      </span>
      <span v-else-if="account.quota?.is_forbidden" class="status-badge forbidden">
        {{ $t('platform.antigravity.status.forbidden') }}
      </span>
      <span v-else-if="isAvailable" class="status-badge available">
        {{ $t('platform.antigravity.status.available') }}
      </span>
      <span v-else class="status-badge low">
        {{ $t('platform.antigravity.status.low') }}
      </span>
    </td>

    <!-- 配额 -->
    <td class="cell-quota">
      <div v-if="account.quota && !account.quota.is_forbidden && filteredModels.length > 0" class="quota-wrapper">
        <div v-for="model in filteredModels" :key="model.name" class="quota-item">
          <span class="model-name">{{ formatModelName(model.name) }}</span>
          <div class="quota-bar-wrapper">
            <div class="quota-bar">
              <div class="quota-fill" :class="getQuotaClass(model.percentage)" :style="{ width: `${model.percentage}%` }"></div>
            </div>
            <span class="quota-percentage">{{ model.percentage }}%</span>
          </div>
        </div>
      </div>
      <span v-else-if="account.quota?.is_forbidden" class="no-quota">{{ $t('platform.antigravity.status.forbidden') }}</span>
      <span v-else class="no-quota">-</span>
    </td>

    <!-- 日期 -->
    <td class="cell-dates">
      <div class="dates-wrapper">
        <span class="date-text created">C: {{ formatDate(account.created_at) }}</span>
        <span class="date-text used">U: {{ formatDate(account.last_used) }}</span>
      </div>
    </td>

    <!-- 操作 -->
    <td class="cell-actions">
      <div class="actions-wrapper">
        <!-- 切换账号 -->
        <button v-if="!isCurrent" @click.stop="$emit('switch', account.id)" class="btn-icon switch" :disabled="isSwitching" :title="$t('platform.antigravity.actions.switch')">
          <svg v-if="!isSwitching" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" />
          </svg>
          <div v-else class="loading-spinner-small"></div>
        </button>

        <!-- 刷新配额 -->
        <button @click.stop="$emit('refresh', account.id)" class="btn-icon refresh" :disabled="isRefreshing" :title="$t('platform.antigravity.actions.refresh')">
          <svg v-if="!isRefreshing" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
          <div v-else class="loading-spinner-small"></div>
        </button>

        <!-- 删除 -->
        <button @click.stop="$emit('delete', account.id)" class="btn-icon delete" :title="$t('platform.antigravity.actions.delete')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
          </svg>
        </button>
      </div>
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

const emit = defineEmits(['switch', 'refresh', 'delete', 'select'])

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
    'claude-sonnet-4-5',
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

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    // 可以添加复制成功提示
  } catch (err) {
    console.error('Failed to copy email:', err)
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

const formatDate = (timestamp) => {
  if (!timestamp) return '-'
  const date = new Date(timestamp * 1000)
  const now = new Date()
  const diff = now - date
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) return $t('common.today')
  if (days === 1) return $t('common.yesterday')
  if (days < 7) return $t('common.daysAgo', { days })

  return date.toLocaleDateString()
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
  vertical-align: middle;
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
.quota-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quota-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-name {
  min-width: 60px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

.quota-bar-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.quota-bar {
  flex: 1;
  height: 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  overflow: hidden;
}

.quota-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s;
}

.quota-fill.high {
  background: linear-gradient(90deg, #22c55e, #16a34a);
}

.quota-fill.medium {
  background: linear-gradient(90deg, #fb923c, #f97316);
}

.quota-fill.low {
  background: linear-gradient(90deg, #ef4444, #dc2626);
}

.quota-percentage {
  min-width: 40px;
  font-size: 12px;
  font-weight: 600;
  font-family: 'Monaco', 'Courier New', monospace;
  color: var(--text);
  text-align: right;
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

.date-text {
  font-size: 11px;
  color: var(--text-secondary);
  font-family: 'Monaco', 'Courier New', monospace;
}

.date-text.created {
  color: var(--text-soft);
}

/* 操作按钮 */
.actions-wrapper {
  display: flex;
  align-items: center;
  gap: 5px;
}

.btn-icon {
  width: 30px;
  height: 30px;
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
