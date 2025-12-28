<template>
  <div class="modal-overlay credit-usage-overlay">
    <div class="modal-content credit-usage-modal" @click.stop>
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
          <button class="modal-close" @click="handleClose">×</button>
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
/* ============================================
   CreditUsageModal - Modern Tech Style
   ============================================ */

.credit-usage-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
  backdrop-filter: blur(4px);
  animation: fadeIn 0.25s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* 模态框布局 */
.credit-usage-modal {
  max-width: 820px;
  width: 100%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
  animation: slideUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideUp {
  from {
    transform: translateY(20px) scale(0.95);
    opacity: 0;
  }
  to {
    transform: translateY(0) scale(1);
    opacity: 1;
  }
}

.credit-usage-modal .modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 1px solid var(--tech-glass-border);
  flex-shrink: 0;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title svg {
  color: var(--accent);
  filter: drop-shadow(0 0 8px var(--tech-glow-primary));
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-strong);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.refresh-btn {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  cursor: pointer;
  padding: 0;
  border-radius: 10px;
  color: var(--text-muted);
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
}

.refresh-btn svg {
  width: 20px;
  height: 20px;
}

.refresh-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}

.refresh-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.mini-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--tech-glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.credit-usage-modal .modal-body {
  padding: 26px 28px;
  overflow-y: auto;
  flex: 1;
}

/* 自定义滚动条 - 科技风 */
.credit-usage-modal .modal-body::-webkit-scrollbar {
  width: 8px;
}

.credit-usage-modal .modal-body::-webkit-scrollbar-track {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border-radius: 4px;
}

.credit-usage-modal .modal-body::-webkit-scrollbar-thumb {
  background: var(--tech-glass-border);
  border-radius: 4px;
}

.credit-usage-modal .modal-body::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--accent) 50%, transparent);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .credit-usage-overlay {
    padding: 0;
  }

  .credit-usage-modal {
    max-width: 100%;
    max-height: 100vh;
    border-radius: 0;
  }

  .credit-usage-modal .modal-header {
    padding: 18px 20px;
  }

  .header-title h2 {
    font-size: 18px;
  }

  .credit-usage-modal .modal-body {
    padding: 18px 20px;
  }
}
</style>
