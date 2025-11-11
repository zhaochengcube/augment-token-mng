<template>
  <div class="modal-overlay">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <div class="header-title">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
          </svg>
          <h2>{{ $t('credit.title') }}</h2>
        </div>
        <div class="header-actions">
          <button @click="refresh" class="refresh-btn" :disabled="loading">
            <svg v-if="!loading" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <div v-else class="mini-spinner"></div>
          </button>
          <button class="close-btn" @click="handleClose">×</button>
        </div>
      </div>

      <div class="modal-body">
        <CreditUsageStats
          :loading="loading"
          :error="error"
          :stats-data="statsData"
          :remaining-credits="remainingCredits"
        />
        <CreditUsageChart
          :loading="loading"
          :error="error"
          :chart-data="chartData"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import CreditUsageStats from './CreditUsageStats.vue'
import CreditUsageChart from './CreditUsageChart.vue'

const props = defineProps({
  authSession: {
    type: String,
    required: true
  },
  creditsBalance: {
    type: [Number, String],
    default: null
  },
  hasPortalUrl: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'refresh-balance', 'update-portal-url'])

const loading = ref(false)
const error = ref(null)
const statsData = ref(null)
const chartData = ref(null)
const remainingCredits = ref(props.creditsBalance)

const fetchData = async () => {
  loading.value = true
  error.value = null

  try {
    // 使用批量获取接口,只交换一次 app_session
    const result = await invoke('fetch_batch_credit_consumption', {
      authSession: props.authSession
    })

    statsData.value = result.stats_data
    chartData.value = result.chart_data
  } catch (e) {
    error.value = e.toString()
    console.error('Failed to fetch credit data:', e)
  } finally {
    loading.value = false
  }
}

const refresh = () => {
  if (!loading.value) {
    fetchData()
    emit('refresh-balance')
  }
}

const handleClose = () => {
  emit('close')
}

onMounted(() => {
  fetchData()
})

watch(() => props.creditsBalance, (value) => {
  remainingCredits.value = value
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-backdrop, rgba(0, 0, 0, 0.5));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal-content {
  background: var(--color-modal-surface, var(--color-surface, #ffffff));
  border: 1px solid var(--color-border);
  border-radius: 16px;
  max-width: 800px;
  width: 100%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: var(--color-shadow-modal, 0 25px 60px rgba(15, 23, 42, 0.3));
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title svg {
  color: var(--color-btn-primary-bg);
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-heading);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.refresh-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0;
  border-radius: 6px;
  color: var(--color-text-secondary);
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}

.refresh-btn svg {
  width: 24px;
  height: 24px;
}

.refresh-btn:hover:not(:disabled) {
  background: var(--color-surface-hover);
  color: var(--color-btn-primary-bg);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mini-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-border);
  border-top-color: var(--color-btn-primary-bg);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 28px;
  line-height: 1;
  cursor: pointer;
  color: var(--color-text-muted);
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
  margin-top: -2px;
}

.close-btn:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-primary);
}

.modal-body {
  padding: 24px 28px;
  overflow-y: auto;
  flex: 1;
}

/* 自定义滚动条 */
.modal-body::-webkit-scrollbar {
  width: 8px;
}

.modal-body::-webkit-scrollbar-track {
  background: var(--color-surface-alt);
  border-radius: 4px;
}

.modal-body::-webkit-scrollbar-thumb {
  background: var(--color-border-strong);
  border-radius: 4px;
}

.modal-body::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-muted);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modal-overlay {
    padding: 0;
  }

  .modal-content {
    max-width: 100%;
    max-height: 100vh;
    border-radius: 0;
  }

  .modal-header {
    padding: 16px 20px;
  }

  .header-title h2 {
    font-size: 18px;
  }

  .modal-body {
    padding: 16px 20px;
  }
}
</style>
