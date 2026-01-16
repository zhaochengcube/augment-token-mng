<template>
  <BaseModal
    :visible="true"
    :title="$t('credit.title')"
    :close-on-overlay="true"
    :body-scroll="true"
    modal-class="!max-w-[820px]"
    @close="handleClose"
  >
    <template #header>
      <div class="flex items-center gap-3 flex-1">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" class="text-accent">
          <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
        </svg>
        <h3 class="modal-title">{{ $t('credit.title') }}</h3>
      </div>
      <button @click="refresh" class="btn btn--ghost btn--icon" :disabled="loading">
        <svg v-if="!loading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <span v-else class="spinner-sm"></span>
      </button>
    </template>

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
  </BaseModal>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '../common/BaseModal.vue'
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
