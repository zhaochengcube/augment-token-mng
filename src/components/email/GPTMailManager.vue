<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="h-full flex flex-col overflow-hidden bg-page">
      <div class="px-6 py-5 border-b border-border shrink-0 bg-muted/30">
        <h3 class="m-0 text-2xl font-bold text-text">{{ $t('gptMailManager.title') }}</h3>
      </div>

      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <!-- API Key + 生成邮箱 同行 -->
        <div class="flex items-center gap-2.5 px-5 py-3 bg-muted/30 border border-border rounded-xl text-sm flex-wrap max-md:flex-col max-md:items-stretch">
          <label class="font-semibold text-text whitespace-nowrap">{{ $t('gptMailManager.apiKeyLabel') }}</label>
          <input v-model="apiKey" type="text" placeholder="gpt-test" class="input flex-1 min-w-[120px]">
          <button v-if="apiKey !== 'gpt-test'" @click="resetApiKey" class="btn btn--primary btn--sm">
            {{ $t('gptMailManager.apiKeyReset') }}
          </button>
          <div class="w-px h-6 bg-border mx-1 max-md:hidden"></div>
          <button @click="generateRandomEmail" :disabled="isGenerating" class="btn btn--primary btn--sm max-md:w-full">
            <span v-if="isGenerating" class="btn-spinner" aria-hidden="true"></span>
            {{ isGenerating ? $t('gptMailManager.generating') : $t('gptMailManager.generateBtn') }}
          </button>
        </div>

        <!-- API Key 提示 + 额度信息 -->
        <div class="flex items-start justify-between gap-4 -mt-4 px-1 flex-wrap">
          <p class="text-xs text-text-muted m-0">
            {{ $t('gptMailManager.apiKeyHint') }}
            <a href="https://mail.chatgpt.org.uk/api" target="_blank" rel="noopener noreferrer" class="text-accent hover:underline">https://mail.chatgpt.org.uk/api</a>
          </p>
          <div v-if="hasUsage" class="flex items-center gap-3 text-xs text-text-muted shrink-0">
            <span>{{ $t('gptMailManager.usageRemainingLabel') }}：{{ formatRemaining(usage.remaining_total) }}，{{ $t('gptMailManager.usageTotalLabel') }}：{{ formatTotalLimit(usage.total_limit) }}</span>
          </div>
        </div>

        <!-- 生成的邮箱展示 -->
        <section v-if="generatedEmail" class="rounded-xl border border-border bg-muted/50 p-5 -mt-2">
          <label class="block mb-2.5 font-semibold text-text text-sm">{{ $t('gptMailManager.generatedEmail') }}:</label>
          <div class="flex gap-2.5 items-center max-md:flex-col">
            <input type="text" :value="generatedEmail" readonly class="input flex-1 max-md:w-full">
            <button @click="copyEmail" class="btn btn--primary btn--sm max-md:w-full">
              {{ $t('gptMailManager.copy') }}
            </button>
          </div>
        </section>

        <!-- Tab 区域 -->
        <section class="rounded-xl border border-border bg-muted/50 p-5">
          <!-- Tab 栏 -->
          <div class="flex rounded-lg border border-border overflow-hidden mb-4">
            <button
              v-for="tab in [
                { key: 'mail', label: $t('gptMailManager.tabMail') },
                { key: 'history', label: $t('gptMailManager.tabHistoryEmails') }
              ]"
              :key="tab.key"
              @click="activeTab = tab.key"
              :class="['px-4 py-1.5 text-sm font-medium transition', activeTab === tab.key ? 'bg-accent text-text-inverse' : 'bg-surface text-text-secondary hover:bg-hover']"
            >
              {{ tab.label }}
            </button>
          </div>

          <!-- Tab: 邮件+验证码 -->
          <div v-if="activeTab === 'mail'">
            <div class="form-group">
              <label class="label">{{ $t('gptMailManager.emailAddress') }}:</label>
              <input v-model="emailInput" type="text" :placeholder="$t('gptMailManager.emailPlaceholder')"
                class="input">
              <p class="text-xs text-text-muted mt-1">{{ $t('gptMailManager.emailHint') }}</p>
            </div>

            <div class="flex gap-3.5 mt-4 flex-wrap max-md:flex-col">
              <button @click="fetchEmails" :disabled="!canFetchEmails || isFetching" class="btn btn--primary max-md:w-full">
                <span v-if="isFetching" class="btn-spinner" aria-hidden="true"></span>
                {{ isFetching ? $t('gptMailManager.fetching') : $t('gptMailManager.fetchBtn') }}
              </button>
              <button @click="toggleAutoPolling" class="btn btn--primary max-md:w-full"
                :disabled="!canFetchEmails">
                {{ isPolling ? $t('gptMailManager.stopPolling') : $t('gptMailManager.startPolling') }}
              </button>
            </div>

            <!-- 轮询状态指示 -->
            <div v-if="isPolling"
              class="flex items-center gap-2.5 px-3.5 py-3 mt-3.5 rounded-xl text-[13px] text-blue-500 bg-blue-500/12 border border-blue-500/35">
              <span class="w-2.5 h-2.5 bg-blue-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(59,130,246,0.5)]"></span>
              {{ $t('gptMailManager.pollingActive') }} ({{ pollingCountdown }}s)
            </div>

            <!-- 验证码 -->
            <div v-if="verificationCode" class="mt-5 p-4 rounded-xl bg-emerald-500/12 border-2 border-emerald-500/40 shadow-[0_0_15px_rgba(16,185,129,0.2)]">
              <label class="block mb-2.5 font-semibold text-emerald-500 text-sm">{{ $t('gptMailManager.verificationCode') }}:</label>
              <div class="flex gap-3.5 items-center">
                <span class="text-[26px] font-bold font-mono text-emerald-500 tracking-[5px] px-4 py-2.5 bg-muted/50 rounded-xl border border-emerald-500/35 [text-shadow:0_0_10px_rgba(16,185,129,0.4)]">
                  {{ verificationCode }}
                </span>
                <button @click="copyCode" class="btn btn--primary btn--sm">
                  {{ $t('gptMailManager.copy') }}
                </button>
              </div>
            </div>

            <!-- 邮件列表 -->
            <div class="mt-5">
              <div v-if="isFetching && emails.length === 0" class="text-center py-10 text-text-muted">
                <span class="btn-spinner btn-spinner--lg inline-block mb-3" aria-hidden="true"></span>
                <p>{{ $t('gptMailManager.loadingEmails') }}</p>
              </div>

              <div v-else-if="emails.length === 0 && hasFetched" class="text-center py-10 text-text-muted">
                <p>{{ $t('gptMailManager.noEmails') }}</p>
              </div>

              <div v-else-if="emails.length > 0">
                <h4 class="m-0 mb-4 text-base font-semibold text-text">{{ $t('gptMailManager.emailList') }} ({{ emails.length }})</h4>
                <div v-for="email in emails" :key="email.id"
                  class="group p-4 bg-muted/50 border border-border rounded-xl mb-3.5 cursor-pointer select-none transition-all hover:border-accent/50 hover:shadow-accent hover:translate-x-1"
                  @click="selectedEmail = email">
                  <div class="flex justify-between items-center mb-2.5 gap-3.5">
                    <div class="text-sm text-text flex-1 min-w-0 truncate">
                      <strong>{{ $t('gptMailManager.from') }}:</strong> {{ email.from }}
                    </div>
                    <div class="text-xs text-text-muted whitespace-nowrap">
                      {{ formatTimestamp(email.timestamp) }}
                    </div>
                  </div>
                  <div class="text-sm text-text font-medium truncate">
                    <strong>{{ $t('gptMailManager.subject') }}:</strong> {{ email.subject }}
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Tab: 历史邮箱 -->
          <div v-if="activeTab === 'history'">
            <div class="flex items-center gap-3 mb-4">
              <h4 class="m-0 text-base font-semibold text-text">{{ $t('gptMailManager.tabHistoryEmails') }}</h4>
              <span class="text-sm text-text-muted">({{ historyList.length }})</span>
            </div>
            <!-- 搜索 + 刷新 -->
            <div class="flex items-center gap-2.5 mb-4">
              <input
                v-model="searchKeyword"
                type="text"
                :placeholder="$t('gptMailManager.historySearchPlaceholder')"
                class="input flex-1"
                @keyup.enter="loadHistoryList"
              >
              <button @click="loadHistoryList" :disabled="historyLoading" class="btn btn--primary btn--sm">
                <span v-if="historyLoading" class="btn-spinner" aria-hidden="true"></span>
                {{ $t('common.refresh') }}
              </button>
            </div>

            <!-- 表格 -->
            <div v-if="historyList.length > 0" class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="border-b border-border text-text-secondary text-left">
                    <th class="py-2 px-3 whitespace-nowrap">{{ $t('gptMailManager.historyCreatedAt') }}</th>
                    <th class="py-2 px-3 whitespace-nowrap">{{ $t('gptMailManager.historyEmail') }}</th>
                    <th class="px-3 py-2 font-medium text-text-secondary w-[60px]">{{ $t('gptMailManager.historyTag') }}</th>
                    <th class="py-2 px-3 whitespace-nowrap">{{ $t('gptMailManager.historyDescription') }}</th>
                    <th class="py-2 px-3 whitespace-nowrap text-center">{{ $t('common.actions') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="item in pagedHistoryList"
                    :key="item.id"
                    class="border-b border-border/50 hover:bg-hover transition"
                  >
                    <td class="py-2.5 px-3 text-text-muted whitespace-nowrap text-xs">
                      {{ formatDateStr(item.created_at) }}
                    </td>
                    <td
                      class="py-2.5 px-3 font-mono text-xs max-w-[220px] truncate cursor-pointer text-text hover:text-accent transition"
                      @click="copyHistoryEmail(item.email)"
                    >
                      {{ item.email }}
                    </td>
                    <td class="px-3 py-2 w-[60px]">
                      <span
                        v-if="!item.tag"
                        class="inline-flex items-center justify-center w-6 h-6 border border-dashed border-border rounded text-text-muted cursor-pointer hover:border-accent hover:text-accent transition-colors"
                        v-tooltip="$t('tokenList.clickToAddTag')"
                        @click.stop="openTagEditor(item)"
                      >
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                        </svg>
                      </span>
                      <span
                        v-else
                        class="badge editable badge--sm max-w-[50px]"
                        :style="{ '--tag-color': item.tag_color || DEFAULT_TAG_COLOR }"
                        v-tooltip="$t('tokenList.clickToEditTag')"
                        @click.stop="openTagEditor(item)"
                      >
                        {{ item.tag }}
                      </span>
                    </td>
                    <td class="py-2.5 px-3 text-text-muted text-xs max-w-[180px]">
                      <input
                        v-if="editingDescId === item.id"
                        v-model="editingDescValue"
                        type="text"
                        class="input input--sm w-full text-xs"
                        @blur="saveDescription(item)"
                        @keyup.enter="$event.target.blur()"
                        @keyup.escape="editingDescId = null"
                      >
                      <span
                        v-else
                        class="cursor-pointer hover:text-accent transition truncate block"
                        @click="startEditDesc(item)"
                      >
                        {{ item.description || '-' }}
                      </span>
                    </td>
                    <td class="py-2.5 px-3 text-center whitespace-nowrap">
                      <button
                        class="text-accent hover:text-accent/80 text-xs transition"
                        @click="fetchMailFromHistory(item.email)"
                      >
                        {{ $t('gptMailManager.historyFetchMail') }}
                      </button>
                      <span class="inline-block w-px h-3 bg-border mx-1.5 align-middle"></span>
                      <button
                        class="text-red-500 hover:text-red-400 text-xs transition"
                        @click="deleteHistoryItem(item.id)"
                      >
                        {{ $t('gptMailManager.historyDelete') }}
                      </button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <!-- 分页 -->
            <div v-if="historyTotalPages > 1" class="flex items-center justify-center gap-3 mt-4">
              <button
                @click="historyCurrentPage--"
                :disabled="historyCurrentPage === 1"
                class="btn btn--secondary btn--sm disabled:opacity-40">
                ← {{ $t('pagination.prev') }}
              </button>
              <span class="text-sm text-text-secondary">
                {{ historyCurrentPage }} / {{ historyTotalPages }}
              </span>
              <button
                @click="historyCurrentPage++"
                :disabled="historyCurrentPage === historyTotalPages"
                class="btn btn--secondary btn--sm disabled:opacity-40">
                {{ $t('pagination.next') }} →
              </button>
            </div>

            <!-- 空态 -->
            <div v-else-if="!historyLoading" class="text-center py-10 text-text-muted">
              <p>{{ $t('gptMailManager.historyEmpty') }}</p>
            </div>

            <!-- 加载态 -->
            <div v-if="historyLoading && historyList.length === 0" class="text-center py-10 text-text-muted">
              <span class="btn-spinner btn-spinner--lg inline-block mb-3" aria-hidden="true"></span>
            </div>
          </div>
        </section>
      </div>
    </div>

    <BaseModal
      :visible="!!selectedEmail"
      :title="selectedEmail?.subject"
      modal-class="max-w-[720px]"
      body-scroll
      @close="selectedEmail = null"
    >
      <div v-if="selectedEmail" class="space-y-4">
        <div class="flex flex-col gap-2 text-sm text-text-muted">
          <div><strong class="text-text">{{ $t('gptMailManager.from') }}:</strong> {{ selectedEmail.from }}</div>
          <div><strong class="text-text">{{ $t('gptMailManager.time') }}:</strong> {{ formatTimestamp(selectedEmail.timestamp) }}</div>
        </div>
        <hr class="border-border">
        <div class="text-sm text-text leading-relaxed">
          <iframe
            v-if="selectedEmail.htmlContent"
            :srcdoc="selectedEmail.htmlContent"
            sandbox="allow-same-origin"
            class="w-full border-0 min-h-[200px] overflow-hidden opacity-0 transition-opacity duration-150"
            @load="resizeIframe"
          />
          <pre v-else class="m-0 whitespace-pre-wrap break-words [font:inherit] text-inherit bg-transparent border-0 p-0">{{ selectedEmail.content || $t('gptMailManager.noContent') }}</pre>
        </div>
      </div>
    </BaseModal>

    <!-- 标签编辑 -->
    <TagEditorModal
      v-model:visible="showTagEditor"
      :token="editingTagToken"
      :all-tokens="allHistoryAsTokens"
      @save="handleTagSave"
      @clear="handleTagClear"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'
import TagEditorModal from '@/components/token/TagEditorModal.vue'

const emit = defineEmits(['close'])

// i18n
const { t } = useI18n()

// API Key
const API_KEY_STORAGE_KEY = 'gptmail_api_key'
const API_KEY_DEFAULT = 'gpt-test'
const apiKey = ref(API_KEY_DEFAULT)

// Usage / quota
const USAGE_STORAGE_KEY = 'gptmail_usage'
const usage = ref(null)
const hasUsage = computed(() => usage.value != null)

const updateUsage = (newUsage) => {
  if (!newUsage) return
  usage.value = newUsage
  try { localStorage.setItem(USAGE_STORAGE_KEY, JSON.stringify(newUsage)) } catch {}
}

const formatRemaining = (v) => {
  if (v == null) return t('gptMailManager.usageNA')
  if (v === -1 || v < 0) return t('gptMailManager.usageUnlimited')
  return String(v)
}

const formatTotalLimit = (v) => {
  if (v == null || v <= 0) return t('gptMailManager.usageUnlimited')
  return String(v)
}

// 响应式数据
const generatedEmail = ref('')
const emailInput = ref('')
const emails = ref([])
const verificationCode = ref('')
const isGenerating = ref(false)
const isFetching = ref(false)
const isPolling = ref(false)
const hasFetched = ref(false)
const pollingInterval = ref(null)
const pollingCountdown = ref(30)
const countdownInterval = ref(null)
const selectedEmail = ref(null)
const activeTab = ref('mail')

// Tag
const DEFAULT_TAG_COLOR = '#f97316'

// 历史邮箱
const HISTORY_PAGE_SIZE = 20
const searchKeyword = ref('')
const historyList = ref([])
const historyLoading = ref(false)
const historyCurrentPage = ref(1)
const showTagEditor = ref(false)
const editingHistoryItem = ref(null)
const editingDescId = ref(null)
const editingDescValue = ref('')

const editingTagToken = computed(() => ({
  tag_name: editingHistoryItem.value?.tag || '',
  tag_color: editingHistoryItem.value?.tag_color || ''
}))

const allHistoryAsTokens = computed(() =>
  historyList.value.map(item => ({ tag_name: item.tag || '', tag_color: item.tag_color || '' }))
)

const historyTotalPages = computed(() => Math.max(1, Math.ceil(historyList.value.length / HISTORY_PAGE_SIZE)))
const pagedHistoryList = computed(() => {
  const start = (historyCurrentPage.value - 1) * HISTORY_PAGE_SIZE
  return historyList.value.slice(start, start + HISTORY_PAGE_SIZE)
})

const resizeIframe = (event) => {
  const iframe = event.target
  try {
    const doc = iframe.contentDocument || iframe.contentWindow?.document
    if (doc) {
      iframe.style.height = doc.documentElement.scrollHeight + 'px'
    }
  } catch { /* cross-origin fallback: keep min-height */ }
  iframe.style.opacity = '1'
}

// 计算属性
const canFetchEmails = computed(() => {
  return emailInput.value.trim().length > 0
})

// 方法
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

// 生成随机邮箱
const generateRandomEmail = async () => {
  isGenerating.value = true
  try {
    const data = await invoke('generate_random_email', { apiKey: apiKey.value || undefined })
    generatedEmail.value = data.email
    emailInput.value = data.email
    updateUsage(data.usage)
    showStatus(t('gptMailManager.generateSuccess'), 'success')

    // T6: 自动保存到历史（静默失败）
    try {
      await invoke('gptmail_save_email', { email: data.email, label: null, description: null })
      if (activeTab.value === 'history') loadHistoryList()
    } catch { /* silent */ }
  } catch (error) {
    showStatus(`${t('gptMailManager.generateFailed')}: ${error}`, 'error')
  } finally {
    isGenerating.value = false
  }
}

// 复制邮箱
const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(generatedEmail.value)
    showStatus(t('gptMailManager.copySuccess'), 'success')
  } catch (error) {
    showStatus(t('gptMailManager.copyFailed'), 'error')
  }
}

const fetchMailFromHistory = (email) => {
  emailInput.value = email
  activeTab.value = 'mail'
}

// 复制历史邮箱到剪贴板
const copyHistoryEmail = async (email) => {
  try {
    await navigator.clipboard.writeText(email)
    showStatus(t('gptMailManager.copySuccess'), 'success')
  } catch {
    showStatus(t('gptMailManager.copyFailed'), 'error')
  }
}

// 复制验证码
const copyCode = async () => {
  try {
    await navigator.clipboard.writeText(verificationCode.value)
    showStatus(t('gptMailManager.copySuccess'), 'success')
  } catch (error) {
    showStatus(t('gptMailManager.copyFailed'), 'error')
  }
}

// 获取邮件
const fetchEmails = async () => {
  if (!canFetchEmails.value) return

  isFetching.value = true
  hasFetched.value = true

  try {
    const email = emailInput.value.trim()
    const data = await invoke('get_emails', { email, apiKey: apiKey.value || undefined })
    emails.value = data.emails || []
    updateUsage(data.usage)

    extractVerificationCode()

    if (emails.value.length > 0) {
      showStatus(t('gptMailManager.fetchSuccess', { count: emails.value.length }), 'success')
    } else {
      showStatus(t('gptMailManager.noEmails'), 'info')
    }

    // Auto-save to history if not already present (silent failure)
    try {
      const history = await invoke('gptmail_list_history', { search: email })
      const exists = (history || []).some(item => item.email === email)
      if (!exists) {
        await invoke('gptmail_save_email', { email, label: null, description: null })
        if (activeTab.value === 'history') loadHistoryList()
      }
    } catch { /* silent */ }
  } catch (error) {
    showStatus(`${t('gptMailManager.fetchFailed')}: ${error}`, 'error')
  } finally {
    isFetching.value = false
  }
}

// 提取验证码
const extractVerificationCode = () => {
  verificationCode.value = ''

  for (const email of emails.value) {
    // 尝试从 content 和 htmlContent 中提取验证码
    const content = email.content || ''
    const htmlContent = email.htmlContent || ''
    const combinedContent = content + ' ' + htmlContent

    // 匹配常见的验证码模式
    const patterns = [
      /verification code is:?\s*(\d{6})/i,
      /code is:?\s*(\d{6})/i,
      /your code:?\s*(\d{6})/i,
      /(\d{6})\s*is your verification code/i,
      /(\d{6})\s*is your code/i,
      /\b(\d{6})\b/
    ]

    for (const pattern of patterns) {
      const match = combinedContent.match(pattern)
      if (match && match[1]) {
        verificationCode.value = match[1]
        return
      }
    }
  }
}

// 切换自动轮询
const toggleAutoPolling = () => {
  if (isPolling.value) {
    stopPolling()
  } else {
    startPolling()
  }
}

// 开始轮询
const startPolling = () => {
  isPolling.value = true
  pollingCountdown.value = 30

  // 立即获取一次
  fetchEmails()

  // 设置30秒轮询
  pollingInterval.value = setInterval(() => {
    fetchEmails()
    pollingCountdown.value = 30
  }, 30000)

  // 设置倒计时
  countdownInterval.value = setInterval(() => {
    if (pollingCountdown.value > 0) {
      pollingCountdown.value--
    }
  }, 1000)

  showStatus(t('gptMailManager.pollingStarted'), 'info')
}

// 停止轮询；silent 为 true 时不弹出「已停止自动轮询」（如卸载时清理）
const stopPolling = (silent = false) => {
  isPolling.value = false

  if (pollingInterval.value) {
    clearInterval(pollingInterval.value)
    pollingInterval.value = null
  }

  if (countdownInterval.value) {
    clearInterval(countdownInterval.value)
    countdownInterval.value = null
  }

  if (!silent) {
    showStatus(t('gptMailManager.pollingStopped'), 'info')
  }
}

// 格式化时间戳（API 返回秒级 Unix timestamp，Date 需要毫秒）
const formatTimestamp = (timestamp) => {
  try {
    if (!timestamp) return '-'
    const ms = timestamp > 1e12 ? timestamp : timestamp * 1000
    const date = new Date(ms)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  } catch {
    return '-'
  }
}

// 格式化 RFC3339 / ISO 日期字符串
const formatDateStr = (str) => {
  try {
    if (!str) return '-'
    const date = new Date(str)
    if (isNaN(date.getTime())) return str
    return date.toLocaleString('zh-CN', {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit'
    })
  } catch {
    return str || '-'
  }
}

// 历史邮箱加载
const loadHistoryList = async () => {
  historyLoading.value = true
  historyCurrentPage.value = 1
  try {
    const data = await invoke('gptmail_list_history', {
      search: searchKeyword.value.trim() || null
    })
    historyList.value = data || []
  } catch (error) {
    showStatus(`${t('gptMailManager.fetchFailed')}: ${error}`, 'error')
  } finally {
    historyLoading.value = false
  }
}

// 删除历史邮箱
const deleteHistoryItem = async (id) => {
  if (!confirm(t('gptMailManager.historyDeleteConfirm'))) return
  try {
    await invoke('gptmail_delete_emails', { ids: [id] })
    await loadHistoryList()
  } catch (error) {
    showStatus(`${t('gptMailManager.historyDelete')}: ${error}`, 'error')
  }
}

// 标签编辑
const openTagEditor = (item) => {
  editingHistoryItem.value = item
  showTagEditor.value = true
}

const handleTagSave = async ({ tagName, tagColor }) => {
  if (!editingHistoryItem.value) return
  try {
    await invoke('gptmail_update_tag', {
      id: editingHistoryItem.value.id,
      tag: tagName || null,
      tagColor: tagColor || null
    })
    showStatus(t('messages.tagUpdated'), 'success')
    await loadHistoryList()
  } catch (error) {
    showStatus(`${error}`, 'error')
  }
}

const handleTagClear = async () => {
  if (!editingHistoryItem.value) return
  try {
    await invoke('gptmail_update_tag', {
      id: editingHistoryItem.value.id,
      tag: null,
      tagColor: null
    })
    showStatus(t('messages.tagCleared'), 'success')
    await loadHistoryList()
  } catch (error) {
    showStatus(`${error}`, 'error')
  }
}

// Description 内联编辑
const startEditDesc = (item) => {
  editingDescId.value = item.id
  editingDescValue.value = item.description || ''
}

const saveDescription = async (item) => {
  const newVal = editingDescValue.value.trim()
  editingDescId.value = null
  if (newVal === (item.description || '')) return
  try {
    await invoke('gptmail_update_email', {
      id: item.id,
      label: item.label || null,
      description: newVal || null
    })
    item.description = newVal || null
    showStatus(t('gptMailManager.updateSuccess'), 'success')
  } catch (error) {
    showStatus(`${t('messages.updateFailed')}: ${error}`, 'error')
  }
}

const resetApiKey = () => {
  apiKey.value = API_KEY_DEFAULT
}

// 生命周期
onMounted(() => {
  const saved = localStorage.getItem(API_KEY_STORAGE_KEY)
  if (saved) {
    apiKey.value = saved
  }
  try {
    const savedUsage = localStorage.getItem(USAGE_STORAGE_KEY)
    if (savedUsage) usage.value = JSON.parse(savedUsage)
  } catch {}

  loadHistoryList()
})

// 切到 history tab 时刷新列表
watch(activeTab, (tab) => {
  if (tab === 'history') loadHistoryList()
})

watch(apiKey, (val) => {
  if (val && val !== API_KEY_DEFAULT) {
    localStorage.setItem(API_KEY_STORAGE_KEY, val)
  } else {
    localStorage.removeItem(API_KEY_STORAGE_KEY)
  }
})

onBeforeUnmount(() => {
  // 组件卸载时清理定时器，不弹出提示
  stopPolling(true)
})

</script>

