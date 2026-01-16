<template>
  <BaseModal
    :visible="visible"
    :title="$t('platform.antigravity.modelsModalTitle')"
    :show-close="false"
    :close-on-overlay="true"
    :close-on-esc="true"
    :body-scroll="true"
    modal-class="max-w-[640px]"
    @close="handleClose"
  >
    <template #header>
      <h3 class="modal-title">{{ $t('platform.antigravity.modelsModalTitle') }}</h3>
      <div class="flex items-center gap-2">
        <button
          class="btn btn--ghost btn--icon"
          :disabled="refreshing"
          @click="$emit('refresh')"
          v-tooltip="$t('platform.antigravity.refresh')"
        >
          <svg
            v-if="!refreshing"
            class="h-4 w-4"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
          <div v-else class="h-4 w-4 border-2 border-accent/30 border-t-accent rounded-full animate-spin"></div>
        </button>
        <button
          class="btn btn--ghost btn--icon"
          @click="handleClose"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
          </svg>
        </button>
      </div>
    </template>

    <!-- Quota Forbidden -->
    <div v-if="account?.quota?.is_forbidden" class="flex items-center justify-center gap-2 py-6 text-text-muted">
      <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
      </svg>
      <span>{{ $t('platform.antigravity.quotaForbidden') }}</span>
    </div>

    <!-- Empty State -->
    <div v-else-if="allModels.length === 0" class="flex items-center justify-center py-6 text-text-muted">
      <span>{{ $t('platform.antigravity.noQuotaData') }}</span>
    </div>

    <!-- Quota List -->
    <template v-else>
      <div class="grid grid-cols-2 gap-3">
        <div v-for="model in allModels" :key="model.name" class="flex w-full">
          <QuotaBar
            :label="formatModelName(model.name)"
            :percent="model.percentage"
            :refresh="formatResetCountdown(model.reset_time)"
            :low-threshold="20"
            :medium-threshold="50"
          />
        </div>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import QuotaBar from '../common/QuotaBar.vue'
import BaseModal from '@/components/common/BaseModal.vue'

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

const emit = defineEmits(['close', 'refresh'])

const handleClose = () => {
  emit('close')
}

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

</script>
