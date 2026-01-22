<template>
  <BaseModal
    :visible="true"
    :title="$t('cursorUsage.title')"
    :close-on-overlay="true"
    :body-scroll="false"
    modal-class="!max-w-[900px]"
    @close="handleClose"
  >
    <template #header>
      <div class="flex items-center gap-3 flex-1">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" class="text-accent">
          <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
        </svg>
        <h3 class="modal-title">{{ $t('cursorUsage.title') }}</h3>
      </div>
      <button @click="refresh" class="btn btn--ghost btn--icon" :disabled="loading">
        <svg v-if="!loading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <span v-else class="btn-spinner"></span>
      </button>
    </template>

    <!-- 基本信息区域 -->
    <div class="mb-6">
      <div v-if="loading && !subscriptionInfo" class="flex justify-center py-8">
        <span class="btn-spinner btn-spinner--lg"></span>
      </div>
      <div v-else-if="error" class="text-danger text-center py-4">{{ error }}</div>
      <div v-else class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div class="card hover:translate-y-0 p-4">
          <div class="text-xs text-text-muted mb-1">{{ $t('cursorUsage.membershipType') }}</div>
          <div class="text-lg font-semibold text-text-strong">{{ subscriptionInfo?.membershipType || 'Free' }}</div>
        </div>
        <div class="card hover:translate-y-0 p-4">
          <div class="text-xs text-text-muted mb-1">{{ $t('cursorUsage.totalInputTokens') }}</div>
          <div class="text-lg font-semibold text-text-strong">{{ formatNumber(aggregatedData?.totalInputTokens) }}</div>
        </div>
        <div class="card hover:translate-y-0 p-4">
          <div class="text-xs text-text-muted mb-1">{{ $t('cursorUsage.totalOutputTokens') }}</div>
          <div class="text-lg font-semibold text-text-strong">{{ formatNumber(aggregatedData?.totalOutputTokens) }}</div>
        </div>
        <div class="card hover:translate-y-0 p-4">
          <div class="text-xs text-text-muted mb-1">{{ $t('cursorUsage.totalCost') }}</div>
          <div class="text-lg font-semibold text-text-strong">${{ (aggregatedData?.totalCostCents / 100).toFixed(2) || '0.00' }}</div>
        </div>
      </div>
    </div>

    <!-- Tab 切换区域 -->
    <div class="border-b border-border mb-4">
      <div class="flex gap-4">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          :class="[
            'px-4 py-2 text-sm font-medium border-b-2 transition-colors',
            activeTab === tab.key
              ? 'text-accent border-accent'
              : 'text-text-muted border-transparent hover:text-text-strong'
          ]"
          @click="switchTab(tab.key)"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <!-- 使用事件表格 -->
    <div class="overflow-x-auto max-h-[400px] overflow-y-auto">
      <div v-if="eventsLoading" class="flex justify-center py-8">
        <span class="btn-spinner btn-spinner--lg"></span>
      </div>
      <div v-else-if="eventsError" class="text-danger text-center py-4">{{ eventsError }}</div>
      <div v-else-if="!usageEvents?.length" class="text-text-muted text-center py-8">
        {{ $t('cursorUsage.noEvents') }}
      </div>
      <table v-else class="w-full text-sm">
        <thead>
          <tr>
            <th class="text-left text-xs font-medium text-text-muted uppercase tracking-wider py-3 px-4 border-b border-border">{{ $t('cursorUsage.timestamp') }}</th>
            <th class="text-left text-xs font-medium text-text-muted uppercase tracking-wider py-3 px-4 border-b border-border">{{ $t('cursorUsage.model') }}</th>
            <th class="text-right text-xs font-medium text-text-muted uppercase tracking-wider py-3 px-4 border-b border-border">{{ $t('cursorUsage.inputTokens') }}</th>
            <th class="text-right text-xs font-medium text-text-muted uppercase tracking-wider py-3 px-4 border-b border-border">{{ $t('cursorUsage.outputTokens') }}</th>
            <th class="text-right text-xs font-medium text-text-muted uppercase tracking-wider py-3 px-4 border-b border-border">{{ $t('cursorUsage.cost') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(event, index) in usageEvents" :key="index" class="hover:bg-hover">
            <td class="py-3 px-4 border-b border-border text-text-strong">{{ formatTimestamp(event.timestamp) }}</td>
            <td class="py-3 px-4 border-b border-border text-text-strong">{{ event.model }}</td>
            <td class="py-3 px-4 border-b border-border text-text-strong text-right">{{ formatNumber(event.tokenUsage?.inputTokens) }}</td>
            <td class="py-3 px-4 border-b border-border text-text-strong text-right">{{ formatNumber(event.tokenUsage?.outputTokens) }}</td>
            <td class="py-3 px-4 border-b border-border text-text-strong text-right">{{ event.tokenUsage?.totalCents ? `$${(event.tokenUsage.totalCents / 100).toFixed(4)}` : '-' }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

const { t: $t } = useI18n()

const props = defineProps({
  account: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close'])

// 状态
const loading = ref(false)
const error = ref(null)
const eventsLoading = ref(false)
const eventsError = ref(null)
const subscriptionInfo = ref(null)
const aggregatedData = ref(null)
const usageEvents = ref([])
const activeTab = ref('1day')

const tabs = [
  { key: '1day', label: $t('cursorUsage.tab1Day'), days: 1 },
  { key: '1week', label: $t('cursorUsage.tab1Week'), days: 7 },
  { key: '1month', label: $t('cursorUsage.tab1Month'), days: 30 }
]

const sessionToken = computed(() => props.account.token?.workos_cursor_session_token)

// 日期格式化
const getDateRange = (days) => {
  const end = new Date()
  const start = new Date()
  start.setDate(start.getDate() - days)
  return {
    startDate: start.toISOString().split('T')[0],
    endDate: end.toISOString().split('T')[0],
    startTimestamp: start.getTime(),  // 毫秒
    endTimestamp: end.getTime()       // 毫秒
  }
}

const formatNumber = (value) => {
  if (!value) return '0'
  return parseInt(value).toLocaleString()
}

const formatTimestamp = (timestamp) => {
  if (!timestamp) return '-'
  // 处理毫秒时间戳（数字或字符串）和 ISO 字符串
  let date
  if (typeof timestamp === 'number') {
    date = new Date(timestamp)
  } else if (typeof timestamp === 'string') {
    // 尝试作为数字解析（毫秒时间戳字符串）
    const num = Number(timestamp)
    if (!isNaN(num) && num > 1000000000000) {
      date = new Date(num)
    } else {
      // 尝试作为 ISO 字符串解析
      date = new Date(timestamp)
    }
  } else {
    return '-'
  }
  return isNaN(date.getTime()) ? '-' : date.toLocaleString()
}

// 从 events 计算聚合数据
const calculateAggregatedFromEvents = (events) => {
  if (!events?.length) return null

  let totalInputTokens = 0
  let totalOutputTokens = 0
  let totalCostCents = 0

  for (const event of events) {
    if (event.tokenUsage) {
      totalInputTokens += event.tokenUsage.inputTokens || 0
      totalOutputTokens += event.tokenUsage.outputTokens || 0
      totalCostCents += (event.tokenUsage.totalCents || 0) * 100
    }
  }

  return {
    total_input_tokens: totalInputTokens.toString(),
    total_output_tokens: totalOutputTokens.toString(),
    total_cost_cents: totalCostCents
  }
}

// 获取订阅和聚合数据
const fetchBasicData = async () => {
  if (!sessionToken.value) {
    error.value = 'No session token'
    return
  }

  loading.value = true
  error.value = null

  try {
    const { startTimestamp, endTimestamp } = getDateRange(30)

    const [subInfo, aggData] = await Promise.all([
      invoke('cursor_get_subscription_info', { sessionToken: sessionToken.value }),
      invoke('cursor_get_aggregated_usage', {
        sessionToken: sessionToken.value,
        startDate: startTimestamp,
        endDate: endTimestamp,
        teamId: 0
      })
    ])
    subscriptionInfo.value = subInfo
    aggregatedData.value = aggData
  } catch (e) {
    error.value = e.toString()
    console.error('Failed to fetch basic data:', e)
  } finally {
    loading.value = false
  }
}

// 获取使用事件
const fetchUsageEvents = async () => {
  if (!sessionToken.value) return

  const currentTab = tabs.find(t => t.key === activeTab.value)
  if (!currentTab) return

  eventsLoading.value = true
  eventsError.value = null

  try {
    const { startTimestamp, endTimestamp } = getDateRange(currentTab.days)
    const result = await invoke('cursor_get_filtered_usage_events', {
      sessionToken: sessionToken.value,
      startDate: String(startTimestamp),
      endDate: String(endTimestamp),
      page: 1,
      pageSize: 1000,
      teamId: 0
    })
    const events = result?.usageEventsDisplay || []
    usageEvents.value = events
  } catch (e) {
    eventsError.value = e.toString()
    console.error('Failed to fetch usage events:', e)
  } finally {
    eventsLoading.value = false
  }
}

const switchTab = (tabKey) => {
  activeTab.value = tabKey
  fetchUsageEvents()
}

const refresh = () => {
  if (!loading.value && !eventsLoading.value) {
    fetchBasicData()
    fetchUsageEvents()
  }
}

const handleClose = () => {
  emit('close')
}

onMounted(() => {
  fetchBasicData()
  fetchUsageEvents()
})
</script>

