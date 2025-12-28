<template>
  <div :class="['account-card', { 'current': isCurrent, 'switching': isSwitching, 'selected': isSelected }]" @click="handleCardClick">
    <!-- 选择框 -->
    <div v-if="selectionMode" class="selection-checkbox" @click.stop="toggleSelection">
      <div class="checkbox-inner" :class="{ 'checked': isSelected }">
        <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
        </svg>
      </div>
    </div>

    <!-- Current Badge -->
    <div v-if="isCurrent" class="current-badge">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
      </svg>
      {{ $t('platform.antigravity.currentAccount') }}
    </div>

    <!-- Account Info -->
    <div class="account-info">
      <div class="email-wrapper" @click.stop="copyEmail" :title="account.email">
        <span class="email-text">{{ showRealEmail ? account.email : maskedEmail }}</span>
      </div>
    </div>

    <!-- Quota Display -->
    <div v-if="account.quota && !account.quota.is_forbidden && filteredModels.length > 0" class="quota-section">
      <div v-for="model in filteredModels" :key="model.name" class="quota-item">
        <div class="quota-header">
          <span class="model-name">{{ formatModelName(model.name) }}</span>
          <span :class="['quota-percentage', getQuotaClass(model.percentage)]">
            {{ model.percentage }}%
          </span>
        </div>
        <div class="quota-bar">
          <div
            class="quota-fill"
            :class="getQuotaClass(model.percentage)"
            :style="{ width: model.percentage + '%' }"
          ></div>
        </div>
        <div v-if="model.reset_time" class="reset-time">
          {{ $t('platform.antigravity.quota.resetTime', { time: formatResetTime(model.reset_time) }) }}
        </div>
      </div>
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
        class="btn-icon switch-btn"
        :disabled="isSwitching"
      >
        <svg v-if="!isSwitching" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <svg v-else class="spin" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
        </svg>
        {{ isSwitching ? $t('platform.antigravity.switching') : $t('platform.antigravity.switch') }}
      </button>
      
      <button
        @click="$emit('refresh', account.id)"
        class="btn-icon refresh-btn"
        :disabled="isRefreshing"
        :title="$t('platform.antigravity.refresh')"
      >
        <svg :class="{ 'spin': isRefreshing }" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
      </button>
      
      <button
        @click="$emit('delete', account.id)"
        class="btn-icon delete-btn"
        :title="$t('platform.antigravity.delete')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
      </button>
    </div>

    <!-- Last Used -->
    <div class="card-footer">
      <span class="last-used">{{ $t('platform.antigravity.lastUsed') }}: {{ formatLastUsed(account.last_used) }}</span>
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

const formatResetTime = (timeStr) => {
  if (!timeStr) return ''
  try {
    const date = new Date(timeStr)
    return date.toLocaleString('zh-CN', { 
      month: 'short', 
      day: 'numeric', 
      hour: '2-digit', 
      minute: '2-digit' 
    })
  } catch {
    return timeStr
  }
}

const formatLastUsed = (timestamp) => {
  const date = new Date(timestamp * 1000)
  const now = new Date()
  const diff = now - date
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (minutes < 1) return $t('platform.antigravity.quota.timeAgo.justNow')
  if (minutes < 60) return $t('platform.antigravity.quota.timeAgo.minutesAgo', { count: minutes })
  if (hours < 24) return $t('platform.antigravity.quota.timeAgo.hoursAgo', { count: hours })
  if (days < 7) return $t('platform.antigravity.quota.timeAgo.daysAgo', { count: days })
  return date.toLocaleDateString()
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
  min-height: 140px;
  z-index: 1;
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

.current-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: var(--accent);
  color: white;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.account-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quota-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.quota-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.quota-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.model-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.quota-percentage {
  font-size: 14px;
  font-weight: 600;
}

.quota-percentage.high { color: #10b981; }
.quota-percentage.medium { color: #f59e0b; }
.quota-percentage.low { color: #ef4444; }

.quota-bar {
  height: 6px;
  background: var(--bg-muted);
  border-radius: 3px;
  overflow: hidden;
}

.quota-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s;
}

.quota-fill.high { background: linear-gradient(90deg, #10b981, #059669); }
.quota-fill.medium { background: linear-gradient(90deg, #f59e0b, #d97706); }
.quota-fill.low { background: linear-gradient(90deg, #ef4444, #dc2626); }

.reset-time {
  font-size: 11px;
  color: var(--text-soft);
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

.btn-icon {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 60%, transparent);
  backdrop-filter: blur(4px);
  color: var(--text-muted);
}

.btn-icon svg {
  transition: transform 0.2s ease;
}

.btn-icon:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  color: var(--accent);
  box-shadow: 0 0 15px var(--tech-glow-primary);
  transform: translateY(-2px);
}

.btn-icon:hover:not(:disabled) svg {
  transform: scale(1.1);
}

.btn-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.switch-btn {
  background: var(--accent);
  color: white;
  border: none;
}

.switch-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 0 20px var(--tech-glow-primary);
}

.refresh-btn, .delete-btn {
  flex: 0 0 auto;
  padding: 8px;
  min-width: 34px;
  min-height: 34px;
}

.delete-btn:hover:not(:disabled) {
  background: color-mix(in srgb, #ef4444 15%, transparent);
  border-color: color-mix(in srgb, #ef4444 50%, transparent);
  color: #f87171;
  box-shadow: 0 0 15px var(--tech-glow-danger);
}

.card-footer {
  padding-top: 12px;
  border-top: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
}

.last-used {
  font-size: 11px;
  font-family: var(--tech-mono-font);
  color: var(--text-muted);
  opacity: 0.7;
}

.spin {
  animation: tech-spin 0.8s linear infinite;
}

@keyframes tech-spin {
  to { transform: rotate(360deg); }
}
</style>
