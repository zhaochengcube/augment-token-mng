<template>
  <BaseModal
    :visible="true"
    :title="''"
    :modal-class="'max-w-[1100px]'"
    :body-scroll="false"
    @close="$emit('close')"
  >
    <template #header>
      <div class="flex w-full flex-wrap items-center justify-between gap-2 pr-2">
        <div class="flex items-center gap-1 rounded-md border border-border p-1">
          <button
            class="btn btn--sm"
            :class="activeTab === 'overview' ? 'btn--primary' : 'btn--ghost'"
            @click="activeTab = 'overview'"
          >
            {{ $t('platform.openai.codexDialog.tabOverview') }}
          </button>
          <button
            class="btn btn--sm"
            :class="activeTab === 'logs' ? 'btn--primary' : 'btn--ghost'"
            @click="activeTab = 'logs'"
          >
            {{ $t('platform.openai.codexDialog.tabRequestLogs') }}
          </button>
        </div>
        <div class="flex items-center gap-2">
          <button class="btn btn--secondary btn--sm" :disabled="isLoading" @click="manualRefresh">{{ $t('platform.openai.codexDialog.refresh') }}</button>
          <button
            class="btn btn--sm"
            :class="serverStatus.running ? 'btn--danger' : 'btn--primary'"
            :disabled="isToggling"
            @click="toggleServer"
          >
            <span v-if="isToggling" class="btn-spinner" aria-hidden="true"></span>
            {{ serverStatus.running ? $t('platform.openai.codexDialog.stopService') : $t('platform.openai.codexDialog.startService') }}
          </button>
        </div>
      </div>
    </template>

    <div class="h-[80vh] overflow-hidden">
      <div v-if="activeTab === 'overview'" class="h-full space-y-4 p-1">
        <div class="grid gap-4 md:grid-cols-2">
          <div class="space-y-2 rounded-lg border border-border p-3">
            <label class="label mb-0">{{ $t('platform.openai.codexDialog.serverUrl') }}</label>
            <div class="flex gap-2">
              <input class="input font-mono" :value="accessConfig.serverUrl" readonly />
              <button
                class="btn btn--icon btn--ghost !h-[34px] !w-[34px] shrink-0"
                :title="$t('common.copy')"
                v-tooltip="$t('common.copy')"
                @click="copyText(accessConfig.serverUrl)"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>

          <div class="space-y-2 rounded-lg border border-border p-3">
            <label class="label mb-0">
              {{ $t('platform.openai.codexDialog.apiKey') }}
              <span class="text-danger">*</span>
            </label>
            <div class="flex gap-2">
              <input
                v-model="apiKeyInput"
                :type="showApiKey ? 'text' : 'password'"
                class="input font-mono"
                :class="{ '!border-danger': !apiKeyInput?.trim() }"
                :placeholder="$t('platform.openai.codexDialog.apiKeyPlaceholder')"
              />
              <button
                class="btn btn--icon btn--ghost !h-[34px] !w-[34px] shrink-0"
                :title="showApiKey ? $t('platform.openai.codexDialog.hideApiKey') : $t('platform.openai.codexDialog.showApiKey')"
                v-tooltip="showApiKey ? $t('platform.openai.codexDialog.hideApiKey') : $t('platform.openai.codexDialog.showApiKey')"
                @click="showApiKey = !showApiKey"
              >
                <svg v-if="showApiKey" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27z"/>
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
                </svg>
              </button>
              <button
                class="btn btn--icon btn--ghost !h-[34px] !w-[34px] shrink-0"
                :title="$t('platform.openai.codexDialog.generateApiKey')"
                v-tooltip="$t('platform.openai.codexDialog.generateApiKey')"
                @click="generateApiKey"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="3" y="3" width="18" height="18" rx="3"></rect>
                  <path d="M12 8v8M8 12h8"></path>
                </svg>
              </button>
              <button
                class="btn btn--icon btn--ghost !h-[34px] !w-[34px] shrink-0"
                :title="$t('common.copy')"
                v-tooltip="$t('common.copy')"
                @click="copyText(apiKeyInput)"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- 策略选择器 + 快捷切换 -->
        <div class="grid grid-cols-2 gap-4">
          <div class="flex items-center justify-between gap-2 rounded-lg border border-border bg-muted/20 px-3 py-2">
            <span class="text-[13px] text-text-secondary">{{ $t('platform.openai.codexDialog.poolStrategy') }}</span>
            <div class="flex items-center gap-2">
              <FloatingDropdown placement="bottom-end" :offset="4" :disabled="isChangingStrategy">
                <template #trigger="{ isOpen }">
                  <button
                    class="btn btn--secondary btn--sm h-8 flex items-center gap-1 px-2"
                    :class="{ 'btn--light': !isOpen }"
                    type="button"
                  >
                    <span class="text-[13px]">{{ getStrategyLabel(poolStrategy) }}</span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M6 9l6 6 6-6"/>
                    </svg>
                  </button>
                </template>
                <template #default="{ close }">
                  <div class="py-1">
                    <button
                      v-for="strategy in strategyOptions"
                      :key="strategy.value"
                      class="dropdown-item flex items-center gap-2 px-3 py-1.5 text-[13px]"
                      :class="{ 'bg-primary/10': strategy.value === poolStrategy }"
                      :disabled="isChangingStrategy"
                      @click="selectStrategy(strategy.value, close)"
                    >
                      <span>{{ strategy.label }}</span>
                    </button>
                  </div>
                </template>
              </FloatingDropdown>
              <!-- 单个模式下的账号选择器 -->
              <FloatingDropdown v-if="poolStrategy === 'single'" placement="bottom-end" :offset="4">
                <template #trigger="{ isOpen }">
                  <button
                    class="btn btn--secondary btn--sm h-8 flex items-center gap-1 px-2"
                    :title="poolStatus.selectedAccountEmail || $t('platform.openai.codexDialog.selectAccount')"
                    :class="{ 'btn--light': !isOpen }"
                    type="button"
                  >
                    <span class="truncate max-w-[100px]">{{ poolStatus.selectedAccountEmail || $t('platform.openai.codexDialog.selectAccount') }}</span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M6 9l6 6 6-6"/>
                    </svg>
                  </button>
                </template>
                <template #default="{ close }">
                  <div class="py-1">
                    <button
                      v-for="account in availableAccounts"
                      :key="account.id"
                      class="dropdown-item flex items-center gap-2 px-3 py-1.5 text-[13px]"
                      :class="{ 'bg-primary/10': account.id === selectedAccountId }"
                      @click="selectAccount(account.id, close)"
                    >
                      <span class="truncate">{{ account.email }}</span>
                    </button>
                  </div>
                </template>
              </FloatingDropdown>
            </div>
          </div>
          <!-- 快捷切换 -->
          <div class="flex items-center justify-between gap-2 rounded-lg border border-border bg-muted/20 px-3 py-2">
            <span class="text-[13px] text-text-secondary">{{ $t('platform.openai.codexDialog.quickSwitch') }}</span>
            <div class="flex items-center gap-2">
              <button
                class="btn btn--secondary btn--sm h-8 px-3"
                v-tooltip="$t('platform.openai.codexDialog.clickToUseAccount')"
                @click="showQuickSwitchModal = 'codex'"
              >
                Codex
              </button>
              <button
                class="btn btn--secondary btn--sm h-8 px-3"
                v-tooltip="$t('platform.openai.codexDialog.clickToUseAccount')"
                @click="showQuickSwitchModal = 'droid'"
              >
                Droid
              </button>
            </div>
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-3">
          <div class="rounded-lg border border-border p-3">
            <div class="text-[12px] text-text-muted">{{ $t('platform.openai.codexDialog.totalAccounts') }}</div>
            <div class="text-[18px] font-semibold">{{ poolStatus.totalAccounts }}</div>
          </div>
          <div class="rounded-lg border border-border p-3">
            <div class="flex items-center justify-between">
              <div class="text-[12px] text-text-muted">{{ $t('platform.openai.codexDialog.totalRequests') }}</div>
              <div class="text-[11px] text-success">+{{ formatNumber(periodStats.todayRequests) }}</div>
            </div>
            <div class="text-[18px] font-semibold">{{ formatNumber(allTimeStats.requests) }}</div>
          </div>
          <div class="rounded-lg border border-border p-3">
            <div class="flex items-center justify-between">
              <div class="text-[12px] text-text-muted">{{ $t('platform.openai.codexDialog.allTimeTokens') }}</div>
              <div class="text-[11px] text-success">+{{ formatTokens(periodStats.todayTokens) }}</div>
            </div>
            <div class="text-[18px] font-semibold">{{ formatTokens(allTimeStats.tokens) }}</div>
          </div>
        </div>

        <!-- 月度趋势图 -->
        <CodexUsageChart :loading="isLoadingChart" :chart-data="dailyStats" />
      </div>

      <div v-else class="flex h-full flex-col gap-2 p-1">
        <div class="flex min-h-0 flex-1 flex-col gap-2 rounded-lg border border-border p-3">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <h4 class="text-[13px] font-semibold">{{ $t('platform.openai.codexDialog.logsTitle') }}</h4>
            <div class="flex flex-wrap items-center gap-2">
              <!-- 时间范围筛选 -->
              <FloatingDropdown placement="bottom-end" :offset="4">
                <template #trigger="{ isOpen }">
                  <button
                    class="btn btn--secondary btn--sm h-8 flex items-center gap-1 px-2"
                    :class="{ 'btn--light': !isOpen }"
                    type="button"
                  >
                    <span class="text-[13px]">{{ getLogRangeLabel(logRange) }}</span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M6 9l6 6 6-6"/>
                    </svg>
                  </button>
                </template>
                <template #default="{ close }">
                  <div class="py-1">
                    <button
                      v-for="range in logRangeOptions"
                      :key="range.value"
                      class="dropdown-item flex items-center gap-2 px-3 py-1.5 text-[13px]"
                      :class="{ 'bg-primary/10': range.value === logRange }"
                      @click="selectLogRange(range.value, close)"
                    >
                      <span>{{ range.label }}</span>
                    </button>
                  </div>
                </template>
              </FloatingDropdown>
              <!-- 账号筛选 -->
              <FloatingDropdown placement="bottom-end" :offset="4">
                <template #trigger="{ isOpen }">
                  <button
                    class="btn btn--secondary btn--sm h-8 flex items-center gap-1 px-2"
                    :class="{ 'btn--light': !isOpen }"
                    type="button"
                  >
                    <span class="text-[13px] truncate max-w-[120px]">{{ getLogAccountLabel() }}</span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M6 9l6 6 6-6"/>
                    </svg>
                  </button>
                </template>
                <template #default="{ close }">
                  <div class="py-1">
                    <button
                      class="dropdown-item flex items-center gap-2 px-3 py-1.5 text-[13px]"
                      :class="{ 'bg-primary/10': !logAccountFilter }"
                      @click="selectLogAccount('', close)"
                    >
                      <span>{{ $t('platform.openai.codexDialog.allAccounts') }}</span>
                    </button>
                    <button
                      v-for="account in availableAccounts"
                      :key="account.id"
                      class="dropdown-item flex items-center gap-2 px-3 py-1.5 text-[13px]"
                      :class="{ 'bg-primary/10': account.id === logAccountFilter }"
                      @click="selectLogAccount(account.id, close)"
                    >
                      <span class="truncate">{{ account.email }}</span>
                    </button>
                  </div>
                </template>
              </FloatingDropdown>
              <!-- 模型筛选 -->
              <input v-model="logModelFilter" class="input h-8 w-[140px]" :placeholder="$t('platform.openai.codexDialog.modelFilterPlaceholder')" />
              <!-- 状态筛选 -->
              <FloatingDropdown placement="bottom-end" :offset="4">
                <template #trigger="{ isOpen }">
                  <button
                    class="btn btn--secondary btn--sm h-8 flex items-center gap-1 px-2"
                    :class="{ 'btn--light': !isOpen }"
                    type="button"
                  >
                    <span class="text-[13px]">{{ getLogStatusLabel(logStatusFilter) }}</span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M6 9l6 6 6-6"/>
                    </svg>
                  </button>
                </template>
                <template #default="{ close }">
                  <div class="py-1">
                    <button
                      v-for="status in logStatusOptions"
                      :key="status.value"
                      class="dropdown-item flex items-center gap-2 px-3 py-1.5 text-[13px]"
                      :class="{ 'bg-primary/10': status.value === logStatusFilter }"
                      @click="selectLogStatus(status.value, close)"
                    >
                      <span>{{ status.label }}</span>
                    </button>
                  </div>
                </template>
              </FloatingDropdown>
            </div>
          </div>

          <div class="min-h-0 flex-1 overflow-y-auto rounded-lg">
            <table class="table table-fixed">
              <thead class="sticky top-0 z-10 bg-bg-base rounded-t-lg overflow-hidden">
                <tr>
                  <th class="w-[12%] first:rounded-tl-lg">{{ $t('platform.openai.codexDialog.time') }}</th>
                  <th class="w-[16%]">{{ $t('platform.openai.codexDialog.account') }}</th>
                  <th class="w-[15%]">{{ $t('platform.openai.codexDialog.model') }}</th>
                  <th class="w-[8%]">{{ $t('platform.openai.codexDialog.format') }}</th>
                  <th class="w-[8%] text-right">{{ $t('platform.openai.codexDialog.inputTokens') }}</th>
                  <th class="w-[8%] text-right">{{ $t('platform.openai.codexDialog.outputTokens') }}</th>
                  <th class="w-[8%] text-right">{{ $t('platform.openai.codexDialog.totalTokens') }}</th>
                  <th class="w-[7%]">{{ $t('platform.openai.codexDialog.status') }}</th>
                  <th class="w-[7%] text-right">{{ $t('platform.openai.codexDialog.duration') }}</th>
                  <th class="w-[11%] last:rounded-tr-lg">{{ $t('platform.openai.codexDialog.error') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="logPage.items.length === 0">
                  <td colspan="10" class="text-center text-text-muted">{{ $t('platform.openai.codexDialog.noLogs') }}</td>
                </tr>
                <tr v-for="log in logPage.items" :key="log.id">
                  <td class="font-mono text-[11px]">{{ formatTs(log.timestamp) }}</td>
                  <td class="text-[11px] truncate"><span class="inline-block -mb-1" v-tooltip="log.accountEmail">{{ log.accountEmail || '-' }}</span></td>
                  <td class="font-mono text-[11px] truncate"><span class="inline-block -mb-1" v-tooltip="log.model">{{ log.model }}</span></td>
                  <td class="text-[11px]">{{ log.format }}</td>
                  <td class="text-right text-[11px]">{{ formatTokens(log.inputTokens) }}</td>
                  <td class="text-right text-[11px]">{{ formatTokens(log.outputTokens) }}</td>
                  <td class="text-right text-[11px] font-semibold">{{ formatTokens(log.totalTokens) }}</td>
                  <td>
                    <span :class="['badge badge--sm', log.status === 'success' ? 'badge--success-tech' : 'badge--danger-tech']">{{ log.status }}</span>
                  </td>
                  <td class="text-right text-[11px]">{{ formatDuration(log.requestDurationMs) }}</td>
                  <td class="text-[11px] text-danger truncate"><span class="inline-block -mb-1" v-tooltip="log.errorMessage || ''">{{ log.errorMessage || '-' }}</span></td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="flex items-center justify-between">
            <span class="text-[12px] text-text-muted">{{ $t('platform.openai.codexDialog.totalRecords', { total: logPage.total }) }}</span>
            <div class="flex items-center gap-2">
              <button class="btn btn--secondary btn--sm" :disabled="logOffset === 0" @click="prevLogPage">{{ $t('platform.openai.codexDialog.prevPage') }}</button>
              <button
                class="btn btn--secondary btn--sm"
                :disabled="logOffset + logLimit >= logPage.total"
                @click="nextLogPage"
              >{{ $t('platform.openai.codexDialog.nextPage') }}</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 快捷切换 Modal -->
    <CodexQuickSwitchModal
      v-if="showQuickSwitchModal"
      :type="showQuickSwitchModal"
      :base-url="accessConfig.serverUrl + '/v1'"
      :api-key="apiKeyInput"
      @close="showQuickSwitchModal = ''"
      @switched="showQuickSwitchModal = ''"
    />
  </BaseModal>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import FloatingDropdown from '@/components/common/FloatingDropdown.vue'
import CodexUsageChart from '@/components/openai/CodexUsageChart.vue'
import CodexQuickSwitchModal from '@/components/openai/CodexQuickSwitchModal.vue'

defineEmits(['close'])

const { t: $t } = useI18n()

const isLoading = ref(false)
const isToggling = ref(false)
const isSavingConfig = ref(false)
const showApiKey = ref(false)
const activeTab = ref('overview')
const showQuickSwitchModal = ref('') // 'codex' | 'droid' | ''
const SHARED_PORT = 8766

const serverStatus = ref({ running: false, address: `http://127.0.0.1:${SHARED_PORT}`, port: SHARED_PORT, poolStatus: null })
const accessConfig = ref({
  serverUrl: `http://127.0.0.1:${SHARED_PORT}`,
  apiKey: ''
})
const poolStatus = ref({
  totalAccounts: 0,
  activeAccounts: 0,
  expiredAccounts: 0,
  coolingAccounts: 0,
  unauthorizedAccounts: 0,
  paymentRequiredAccounts: 0,
  totalRequestsToday: 0,
  totalTokensUsed: 0,
  strategy: 'round-robin'
})
const periodStats = ref({ todayRequests: 0, todayTokens: 0, weekRequests: 0, weekTokens: 0, monthRequests: 0, monthTokens: 0 })
const allTimeStats = ref({ requests: 0, tokens: 0 })

const apiKeyInput = ref('')
const poolStrategy = ref('round-robin')
const selectedAccountId = ref('')
const isChangingStrategy = ref(false)
const availableAccounts = ref([])

// 策略选项
const strategyOptions = [
  { value: 'round-robin', label: $t('platform.openai.codexDialog.strategyRoundRobin') },
  { value: 'single', label: $t('platform.openai.codexDialog.strategySingle') },
  { value: 'smart', label: $t('platform.openai.codexDialog.strategySmart') }
]

// 日志时间范围选项
const logRangeOptions = [
  { value: 'today', label: $t('platform.openai.codexDialog.rangeToday') },
  { value: '7d', label: $t('platform.openai.codexDialog.range7d') },
  { value: '30d', label: $t('platform.openai.codexDialog.range30d') },
  { value: 'all', label: $t('platform.openai.codexDialog.rangeAll') }
]

// 日志状态选项
const logStatusOptions = [
  { value: '', label: $t('platform.openai.codexDialog.allStatus') },
  { value: 'success', label: 'success' },
  { value: 'error', label: 'error' }
]

const logPage = ref({ total: 0, items: [] })
const logLimit = ref(50)
const logOffset = ref(0)
const logAccountFilter = ref('')
const logModelFilter = ref('')
const logStatusFilter = ref('')
const logRange = ref('7d')

// 图表相关状态
const isLoadingChart = ref(false)
const dailyStats = ref([])

let pollTimer = null
let apiKeySaveTimer = null

// 防抖保存 API Key
const saveApiKeyDebounced = () => {
  if (apiKeySaveTimer) {
    clearTimeout(apiKeySaveTimer)
  }
  apiKeySaveTimer = setTimeout(() => {
    saveAccessConfig()
  }, 800)
}

const toCamel = (obj) => {
  if (Array.isArray(obj)) return obj.map(toCamel)
  if (!obj || typeof obj !== 'object') return obj
  const out = {}
  for (const [key, value] of Object.entries(obj)) {
    const camel = key.replace(/_([a-z])/g, (_, c) => c.toUpperCase())
    out[camel] = toCamel(value)
  }
  return out
}

const getLogRange = () => {
  const now = Math.floor(Date.now() / 1000)
  if (logRange.value === 'all') {
    return { startTs: null, endTs: null }
  }
  if (logRange.value === 'today') {
    const startDate = new Date()
    startDate.setHours(0, 0, 0, 0)
    return { startTs: Math.floor(startDate.getTime() / 1000), endTs: now }
  }
  if (logRange.value === '30d') {
    return { startTs: now - 30 * 24 * 3600, endTs: now }
  }
  return { startTs: now - 7 * 24 * 3600, endTs: now }
}

const formatCompactNumber = (v) => {
  const n = Number(v || 0)
  if (n < 1000) return n.toLocaleString()
  if (n < 1000000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'K'
  if (n < 1000000000) return (n / 1000000).toFixed(2).replace(/\.00$/, '') + 'M'
  if (n < 1000000000000) return (n / 1000000000).toFixed(2).replace(/\.00$/, '') + 'B'
  return (n / 1000000000000).toFixed(2).replace(/\.00$/, '') + 'T'
}

const formatNumber = (v) => formatCompactNumber(v)

const formatTokens = (v) => formatCompactNumber(v)

const formatTs = (ts) => {
  if (!ts) return '-'
  const date = new Date(ts * 1000)
  if (Number.isNaN(date.getTime())) return '-'
  const pad = (n) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

const formatDuration = (ms) => {
  const n = Number(ms || 0)
  if (n < 1000) return `${n}ms`
  if (n < 60000) return `${(n / 1000).toFixed(1)}s`
  if (n < 3600000) return `${(n / 60000).toFixed(1)}m`
  return `${(n / 3600000).toFixed(1)}h`
}

const copyText = async (text) => {
  const value = String(text || '').trim()
  if (!value) {
    window.$notify?.warning($t('platform.openai.codexDialog.copyEmpty'))
    return
  }
  try {
    await navigator.clipboard.writeText(value)
    window.$notify?.success($t('common.copySuccess'))
  } catch (error) {
    window.$notify?.error($t('common.copyFailed'))
  }
}

const randomBytesHex = (size = 20) => {
  if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
    const bytes = new Uint8Array(size)
    crypto.getRandomValues(bytes)
    return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
  }
  let out = ''
  for (let i = 0; i < size; i += 1) {
    out += Math.floor(Math.random() * 256).toString(16).padStart(2, '0')
  }
  return out
}

const generateApiKey = () => {
  apiKeyInput.value = `sk-${randomBytesHex(24)}`
  showApiKey.value = true
  window.$notify?.success($t('platform.openai.codexDialog.generateApiKeySuccess'))
}

const loadServerStatus = async () => {
  const raw = await invoke('get_codex_server_status')
  const data = toCamel(raw)
  serverStatus.value = {
    running: !!data.running,
    address: data.address || `http://127.0.0.1:${SHARED_PORT}`,
    port: data.port || SHARED_PORT,
    poolStatus: data.poolStatus || null
  }
}

const loadAccessConfig = async () => {
  const raw = await invoke('get_codex_access_config')
  const data = toCamel(raw)
  accessConfig.value = {
    serverUrl: data.serverUrl || `http://127.0.0.1:${SHARED_PORT}`,
    apiKey: data.apiKey || ''
  }
  apiKeyInput.value = accessConfig.value.apiKey
}

const loadPoolStatus = async () => {
  try {
    const raw = await invoke('get_codex_pool_status')
    poolStatus.value = toCamel(raw)

    // 同步策略到前端
    if (poolStatus.value.strategy) {
      const strategyMap = {
        'RoundRobin': 'round-robin',
        'Single': 'single',
        'Smart': 'smart'
      }
      poolStrategy.value = strategyMap[poolStatus.value.strategy] || 'round-robin'
    }

    // 加载账号列表
    await loadAccounts()
  } catch {
    poolStatus.value = toCamel(serverStatus.value.poolStatus || poolStatus.value)
  }
}

const loadAccounts = async () => {
  try {
    const raw = await invoke('openai_list_accounts')
    const accounts = toCamel(raw)
    // 过滤出可用的 OAuth 账号
    availableAccounts.value = accounts.filter((a) =>
      a.accountType === 'oauth' && a.token && !a.token.isExpired
    )
  } catch {
    availableAccounts.value = []
  }
}

const onStrategyChange = async () => {
  isChangingStrategy.value = true
  try {
    await invoke('set_codex_pool_strategy', { strategy: poolStrategy.value })
    await loadPoolStatus()
  } catch (error) {
    window.$notify?.error($t('platform.openai.codexDialog.toggleFailed', { error: error?.message || error }))
  } finally {
    isChangingStrategy.value = false
  }
}

const getStrategyLabel = (value) => {
  const option = strategyOptions.find(s => s.value === value)
  return option?.label || value
}

const selectStrategy = async (value, close) => {
  if (value === poolStrategy.value || isChangingStrategy.value) {
    close()
    return
  }
  isChangingStrategy.value = true
  try {
    await invoke('set_codex_pool_strategy', { strategy: value })
    poolStrategy.value = value
    await loadPoolStatus()
    close()
  } catch (error) {
    window.$notify?.error($t('platform.openai.codexDialog.toggleFailed', { error: error?.message || error }))
  } finally {
    isChangingStrategy.value = false
  }
}

const selectAccount = async (accountId, close) => {
  selectedAccountId.value = accountId
  try {
    await invoke('set_codex_selected_account', { accountId })
    await loadPoolStatus()
    close()
  } catch (error) {
    window.$notify?.error($t('platform.openai.codexDialog.toggleFailed', { error: error?.message || error }))
  }
}

// 日志筛选相关方法
const getLogRangeLabel = (value) => {
  const option = logRangeOptions.find(r => r.value === value)
  return option?.label || value
}

const getLogStatusLabel = (value) => {
  const option = logStatusOptions.find(s => s.value === value)
  return option?.label || value
}

const getLogAccountLabel = () => {
  if (!logAccountFilter.value) {
    return $t('platform.openai.codexDialog.allAccounts')
  }
  const account = availableAccounts.value.find(a => a.id === logAccountFilter.value)
  return account?.email || logAccountFilter.value
}

const selectLogRange = async (value, close) => {
  logRange.value = value
  close()
  await reloadLogs()
}

const selectLogStatus = async (value, close) => {
  logStatusFilter.value = value
  close()
  await reloadLogs()
}

const selectLogAccount = async (value, close) => {
  logAccountFilter.value = value
  close()
  await reloadLogs()
}

// 模型筛选防抖
let modelFilterTimer = null
watch(logModelFilter, () => {
  if (modelFilterTimer) {
    clearTimeout(modelFilterTimer)
  }
  modelFilterTimer = setTimeout(() => {
    reloadLogs()
  }, 500)
})

const loadPeriodStats = async () => {
  const raw = await invoke('get_codex_period_stats_from_storage')
  periodStats.value = toCamel(raw)
}

const loadAllTimeStats = async () => {
  try {
    const raw = await invoke('get_codex_all_time_stats')
    allTimeStats.value = toCamel(raw)
  } catch {
    allTimeStats.value = { requests: 0, tokens: 0 }
  }
}

const loadDailyStats = async () => {
  isLoadingChart.value = true
  try {
    const raw = await invoke('get_codex_daily_stats_from_storage', { days: 30 })
    dailyStats.value = toCamel(raw).stats || []
  } catch {
    dailyStats.value = []
  } finally {
    isLoadingChart.value = false
  }
}

const loadLogs = async () => {
  try {
    const range = getLogRange()
    const query = {
      limit: logLimit.value,
      offset: logOffset.value,
      startTs: range.startTs,
      endTs: range.endTs,
      model: logModelFilter.value.trim() || null,
      status: logStatusFilter.value || null,
      accountId: logAccountFilter.value.trim() || null
    }
    const raw = await invoke('query_codex_logs_from_storage', { query })
    logPage.value = toCamel(raw)
  } catch {
    // 静默失败
  }
}

const reloadLogs = async () => {
  logOffset.value = 0
  await loadLogs()
}

const prevLogPage = async () => {
  logOffset.value = Math.max(0, logOffset.value - logLimit.value)
  await loadLogs()
}

const nextLogPage = async () => {
  if (logOffset.value + logLimit.value >= logPage.value.total) return
  logOffset.value += logLimit.value
  await loadLogs()
}

const saveAccessConfig = async () => {
  if (isSavingConfig.value) return
  isSavingConfig.value = true
  try {
    await invoke('set_codex_access_config', {
      apiKey: apiKeyInput.value
    })
    accessConfig.value.apiKey = apiKeyInput.value
  } catch (error) {
    window.$notify?.error($t('platform.openai.codexDialog.saveConfigFailed', { error: error?.message || error }))
  } finally {
    isSavingConfig.value = false
  }
}

// 监听 API Key 变化，自动保存
watch(apiKeyInput, (newVal, oldVal) => {
  if (newVal !== oldVal && newVal !== accessConfig.value.apiKey) {
    saveApiKeyDebounced()
  }
})

onMounted(async () => {
  await Promise.all([loadServerStatus(), loadAccessConfig()])
  await Promise.all([loadPoolStatus(), loadPeriodStats(), loadAllTimeStats(), loadLogs(), loadAccounts(), loadDailyStats()])
  pollTimer = window.setInterval(() => {
    refreshAllData()
  }, 1000)
})

onBeforeUnmount(() => {
  if (pollTimer) {
    window.clearInterval(pollTimer)
    pollTimer = null
  }
  if (apiKeySaveTimer) {
    clearTimeout(apiKeySaveTimer)
    apiKeySaveTimer = null
  }
  if (modelFilterTimer) {
    clearTimeout(modelFilterTimer)
    modelFilterTimer = null
  }
})

const toggleServer = async () => {
  isToggling.value = true
  try {
    if (serverStatus.value.running) {
      await invoke('stop_codex_server')
      window.$notify?.success($t('platform.openai.codexDialog.stopSuccess'))
    } else {
      await invoke('start_codex_server', {
        config: {
          port: SHARED_PORT,
          poolStrategy: 'round-robin',
          logRequests: true,
          maxLogEntries: 3000,
          apiKey: apiKeyInput.value || null
        }
      })
      window.$notify?.success($t('platform.openai.codexDialog.startSuccess'))
    }
    await refreshAllData()
  } catch (error) {
    window.$notify?.error($t('platform.openai.codexDialog.toggleFailed', { error: error?.message || error }))
  } finally {
    isToggling.value = false
  }
}

const refreshAllData = async ({ refreshPool = false } = {}) => {
  isLoading.value = true
  try {
    await Promise.all([loadServerStatus(), loadAccessConfig()])
    if (refreshPool) {
      const refreshed = await invoke('refresh_codex_pool')
      poolStatus.value = toCamel(refreshed)
    }
    await Promise.all([loadPoolStatus(), loadPeriodStats(), loadAllTimeStats(), loadLogs(), loadDailyStats()])
  } catch (error) {
    console.error('Failed to load codex dialog data:', error)
  } finally {
    isLoading.value = false
  }
}

const manualRefresh = async () => {
  // 先 flush 内存日志到 SQLite 存储
  try {
    await invoke('flush_codex_logs')
  } catch {
    // 忽略 flush 错误
  }
  await refreshAllData({ refreshPool: true })
}
</script>
