<template>
  <div v-if="visible" class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content models-modal">
      <div class="modal-header">
        <div class="modal-title">
          <h3>{{ $t('platform.antigravity.modelsModalTitle') }}</h3>
        </div>
        <div class="modal-actions">
          <button class="btn-icon refresh-btn" :disabled="refreshing" @click="$emit('refresh')" v-tooltip="$t('platform.antigravity.refresh')">
            <svg v-if="!refreshing" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
            </svg>
            <svg v-else class="spin" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
            </svg>
          </button>
          <button class="modal-close" @click="$emit('close')">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
            </svg>
          </button>
        </div>
      </div>

      <div class="modal-body">
        <div v-if="account?.quota?.is_forbidden" class="quota-forbidden">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
          </svg>
          <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
        </div>

        <div v-else-if="allModels.length === 0" class="quota-empty">
          <span>{{ $t('platform.antigravity.noQuotaData') }}</span>
        </div>

        <template v-else>
          <div class="quota-list">
            <div v-for="model in allModels" :key="model.name" class="quota-item">
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
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  account: {
    type: Object,
    default: null
  },
  refreshing: {
    type: Boolean,
    default: false
  }
})

const allModels = computed(() => {
  if (!props.account?.quota?.models) return []
  return [...props.account.quota.models].sort((a, b) => {
    const groupA = getModelGroup(a.name)
    const groupB = getModelGroup(b.name)

    if (groupA !== groupB) {
      return groupA - groupB
    }

    return a.name.localeCompare(b.name)
  })
})

const getModelGroup = (name) => {
  const lowerName = name.toLowerCase()
  if (lowerName.includes('claude')) return 0
  if (lowerName.includes('gemini')) return 1
  return 2
}

const formatModelName = (name) => {
  const lowerName = name.toLowerCase()

  if (lowerName.includes('claude-opus-4-5-thinking')) return 'Claude Opus 4.5 Thinking'
  if (lowerName.includes('claude-sonnet-4-5-thinking')) return 'Claude Sonnet 4.5 Thinking'
  if (lowerName.includes('claude-sonnet-4-5')) return 'Claude Sonnet 4.5'
  if (lowerName.includes('claude-opus')) return 'Claude Opus'
  if (lowerName.includes('claude-sonnet')) return 'Claude Sonnet'
  if (lowerName.includes('claude')) return 'Claude'

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

</script>

<style scoped>
.models-modal {
  width: 640px;
  max-width: calc(100vw - 32px);
  max-height: calc(100vh - 48px);
  background: var(--bg-surface);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface);
}

.modal-title h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.modal-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-body {
  padding: 16px 20px 20px;
  overflow: auto;
}

.quota-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.quota-item {
  display: flex;
  width: 100%;
}

.quota-bar {
  width: 100%;
  height: 24px;
  background: color-mix(in srgb, var(--bg-muted) 75%, transparent);
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  position: relative;
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

:global([data-theme='dark']) .model-time {
  color: var(--text-secondary, #cbd5e1);
}

:global([data-theme='dark']) .quota-overlay {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

.quota-fill.medium {
  background: linear-gradient(90deg, #f59e0b, #d97706);
}

.quota-fill.low {
  background: linear-gradient(90deg, #ef4444, #dc2626);
}

.reset-time {
  margin-top: 6px;
  font-size: 11px;
  color: var(--text-muted);
}

.quota-forbidden,
.quota-empty {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  padding: 24px 0;
  justify-content: center;
}
</style>
