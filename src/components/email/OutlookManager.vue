<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="h-full flex flex-col overflow-hidden bg-page">
      <div class="px-6 py-5 border-b border-border shrink-0 bg-muted/30 flex items-center justify-between">
        <h3 class="m-0 text-2xl font-bold text-text">{{ $t('outlookManager.title') }}</h3>
        <button @click="showAddModal = true" class="btn btn--primary btn--sm">
          + {{ $t('outlookManager.addAccountBtn') }}
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        <!-- 账户列表 -->
        <section class="rounded-xl border border-border bg-muted/50 p-5">
          <div class="flex items-center justify-between mb-4 flex-wrap gap-2.5">
            <div class="flex items-center gap-3">
              <h4 class="m-0 text-base font-semibold text-text whitespace-nowrap">{{ $t('outlookManager.accountList') }} ({{ filteredAccounts.length }}/{{ accounts.length }})</h4>
              <div v-if="accounts.length > 0" class="relative">
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="搜索邮箱..."
                  class="input py-1 px-2.5 text-sm w-48 pr-7"
                />
                <button
                  v-if="searchQuery"
                  @click="searchQuery = ''"
                  class="absolute right-1.5 top-1/2 -translate-y-1/2 text-text-muted hover:text-text text-sm leading-none cursor-pointer"
                >✕</button>
              </div>
            </div>
            <div class="flex gap-2.5">
              <button
                @click="refreshAllTokens"
                :disabled="isRefreshingAll || accounts.length === 0"
                class="btn btn--secondary btn--sm"
              >
                <span v-if="isRefreshingAll" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
                {{ isRefreshingAll ? '刷新中...' : '全量刷新 Token' }}
              </button>
              <button
                @click="checkAllStatuses"
                :disabled="isCheckingStatus || accounts.length === 0"
                class="btn btn--primary btn--sm"
              >
                <span v-if="isCheckingStatus" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
                {{ isCheckingStatus ? `检查中 (${statusCheckProgress}/${accounts.length})` : $t('outlookManager.checkStatus') }}
              </button>
            </div>
          </div>

          <!-- 批量刷新结果 -->
          <div v-if="refreshResults" class="mb-4 p-4 rounded-xl border border-border bg-muted/30">
            <div class="flex items-center justify-between mb-3">
              <span class="text-sm font-semibold text-text">
                Token 刷新结果: {{ refreshResults.success_count }}/{{ refreshResults.total }} 成功
              </span>
              <div class="flex items-center gap-2.5">
                <button
                  v-if="refreshResults.failed_count > 0"
                  @click="retryFailedTokens"
                  :disabled="isRefreshingAll"
                  class="btn btn--secondary btn--sm text-xs"
                >
                  <span v-if="isRefreshingAll" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
                  重试失败 ({{ refreshResults.failed_count }})
                </button>
                <button @click="refreshResults = null" class="text-xs text-text-muted hover:text-text cursor-pointer">关闭</button>
              </div>
            </div>
            <div v-if="refreshResults.failed_count > 0" class="flex flex-col gap-1.5">
              <div
                v-for="r in refreshResults.results.filter(r => !r.success)"
                :key="r.email"
                class="text-xs text-red-400 bg-red-500/10 px-3 py-1.5 rounded-lg truncate"
                :title="r.error"
              >
                ✘ {{ r.email }}: {{ r.error?.substring(0, 80) }}
              </div>
            </div>
            <div v-else class="text-xs text-green-400">✔ 所有账号 Token 均有效</div>
          </div>

          <div v-if="isLoading" class="text-center py-10 text-text-muted">
            <span class="btn-spinner btn-spinner--lg inline-block mb-3" aria-hidden="true"></span>
            <p>{{ $t('outlookManager.status.checking') }}</p>
          </div>


          <div v-if="accounts.length === 0 && !isLoading" class="text-center py-10 text-text-muted">
            <p>{{ $t('outlookManager.emptyState') }}</p>
            <p class="text-xs mt-2 opacity-70">{{ $t('outlookManager.emptyDescription') }}</p>
          </div>

          <div v-else class="flex flex-col gap-3.5">
            <div
              v-for="account in paginatedAccounts"
              :key="account.email"
              class="flex justify-between items-center p-4 bg-muted/50 border border-border rounded-xl cursor-pointer transition-all hover:border-accent/50 hover:shadow-accent hover:translate-x-1"
              @click="viewEmails(account.email)"
            >
              <div class="flex-1">
                <div class="font-semibold text-text mb-2">{{ account.email }}</div>
                <div class="flex items-center gap-3.5 flex-wrap">
                  <span :class="getStatusClass(account.email)">
                    {{ getStatusText(account.email) }}
                  </span>
                  <span class="text-[11px] font-mono text-text-muted bg-muted/70 px-2 py-0.5 rounded-lg">
                    {{ formatDate(account.created_at) }}
                  </span>
                </div>
              </div>
              <div class="flex gap-2.5" @click.stop>
                <button
                  @click="checkStatus(account.email)"
                  :disabled="isCheckingStatus"
                  class="btn btn--secondary btn--sm"
                >
                  {{ $t('outlookManager.checkStatus') }}
                </button>
                <button
                  @click="deleteAccount(account.email)"
                  class="btn btn-tech-danger btn--sm"
                >
                  {{ $t('outlookManager.deleteAccount') }}
                </button>
              </div>
            </div>

            <!-- 分页控件 -->
            <div v-if="accountTotalPages > 1" class="flex items-center justify-center gap-3 pt-3">
              <button
                @click="accountPage > 1 && accountPage--"
                :disabled="accountPage <= 1"
                class="btn btn--secondary btn--sm disabled:opacity-40"
              >
                ← 上一页
              </button>
              <span class="text-sm text-text-secondary whitespace-nowrap">
                {{ accountPage }} / {{ accountTotalPages }}
              </span>
              <button
                @click="accountPage < accountTotalPages && accountPage++"
                :disabled="accountPage >= accountTotalPages"
                class="btn btn--secondary btn--sm disabled:opacity-40"
              >
                下一页 →
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>

  <!-- 添加账户弹窗 -->
  <BaseModal
    :visible="showAddModal"
    :title="$t('outlookManager.addAccount')"
    modal-class="max-w-[600px]"
    @close="showAddModal = false"
  >
    <!-- Tab 切换 -->
    <div class="flex border-b border-border mb-4">
      <button
        @click="addTab = 'manual'"
        class="px-4 py-2 text-sm font-semibold transition-colors"
        :class="addTab === 'manual' ? 'text-accent border-b-2 border-accent' : 'text-text-muted hover:text-text'"
      >
        手动导入
      </button>
      <button
        @click="addTab = 'oauth'"
        class="px-4 py-2 text-sm font-semibold transition-colors"
        :class="addTab === 'oauth' ? 'text-accent border-b-2 border-accent' : 'text-text-muted hover:text-text'"
      >
        OAuth 授权
      </button>
    </div>

    <!-- 手动导入 Tab -->
    <div v-if="addTab === 'manual'">
      <div class="form-group">
        <label class="label">{{ $t('outlookManager.accountInfo') }}:</label>
        <textarea
          v-model="accountInput"
          rows="6"
          :placeholder="$t('outlookManager.placeholder') + '\n' + $t('outlookManager.placeholder')"
          class="input resize-y min-h-[120px] font-mono text-sm"
        ></textarea>
        <p class="text-xs text-text-muted mt-1">{{ $t('outlookManager.inputHint') }}（支持多行批量导入）</p>
      </div>
      <div class="mt-4 flex justify-end">
        <button
          @click="addAccount"
          :disabled="!canAddAccount || isAdding"
          class="btn btn--primary"
        >
          <span v-if="isAdding" class="btn-spinner" aria-hidden="true"></span>
          {{ isAdding ? $t('outlookManager.status.checking') : $t('outlookManager.addAccountBtn') }}
        </button>
      </div>
    </div>

    <!-- OAuth 授权 Tab -->
    <div v-if="addTab === 'oauth'">
      <!-- Step 1: 生成授权链接 -->
      <div v-if="oauthStep === 1" class="space-y-4">
        <p class="text-sm text-text-muted">点击下方按钮生成微软授权链接，在浏览器中登录微软账号完成授权。</p>
        <button
          @click="startOAuth"
          :disabled="isOAuthLoading"
          class="btn btn--primary w-full"
        >
          <span v-if="isOAuthLoading" class="btn-spinner" aria-hidden="true"></span>
          {{ isOAuthLoading ? '生成中...' : '生成授权链接' }}
        </button>
      </div>

      <!-- Step 2: 用户去授权，粘贴回调 URL -->
      <div v-if="oauthStep === 2" class="space-y-4">
        <p class="text-sm text-text-muted">请在浏览器中打开以下链接，登录微软账号并授权。授权完成后，将浏览器地址栏中的完整 URL 粘贴到下方。</p>
        <div class="flex gap-2">
          <input
            :value="oauthAuthUrl"
            readonly
            class="input text-xs font-mono flex-1"
          />
          <button @click="copyToClipboard(oauthAuthUrl)" class="btn btn--secondary btn--sm whitespace-nowrap">
            复制链接
          </button>
        </div>
        <div class="form-group">
          <label class="label">授权后的回调 URL:</label>
          <input
            v-model="oauthRedirectedUrl"
            type="text"
            placeholder="粘贴授权完成后浏览器地址栏中的完整 URL..."
            class="input font-mono text-sm"
          />
        </div>
        <div class="flex justify-between">
          <button @click="oauthStep = 1" class="btn btn--secondary btn--sm">返回</button>
          <button
            @click="exchangeOAuthToken"
            :disabled="!oauthRedirectedUrl.trim() || isOAuthLoading"
            class="btn btn--primary"
          >
            <span v-if="isOAuthLoading" class="btn-spinner" aria-hidden="true"></span>
            {{ isOAuthLoading ? '获取中...' : '获取 Token' }}
          </button>
        </div>
      </div>

      <!-- Step 3: 成功获取 token，确认保存 -->
      <div v-if="oauthStep === 3" class="space-y-4">
        <div class="p-4 rounded-xl bg-green-500/10 border border-green-500/30">
          <p class="text-sm text-green-400 font-semibold mb-2">✔ Token 获取成功</p>
          <div v-if="oauthResult?.email" class="text-sm text-text">
            邮箱: <span class="font-mono font-semibold">{{ oauthResult.email }}</span>
          </div>
          <div v-else class="text-xs text-text-muted">
            <label class="label">请输入邮箱地址:</label>
            <input v-model="oauthManualEmail" type="text" placeholder="example@outlook.com" class="input text-sm mt-1" />
          </div>
        </div>
        <div class="flex justify-between">
          <button @click="oauthStep = 1; oauthRedirectedUrl = ''" class="btn btn--secondary btn--sm">重新授权</button>
          <button
            @click="saveOAuthAccount"
            :disabled="isOAuthLoading || (!oauthResult?.email && !oauthManualEmail.trim())"
            class="btn btn--primary"
          >
            <span v-if="isOAuthLoading" class="btn-spinner" aria-hidden="true"></span>
            保存账户
          </button>
        </div>
      </div>
    </div>
  </BaseModal>

  <!-- 邮件查看器 -->
  <EmailViewer
    v-if="showEmailViewer"
    :email="selectedEmail"
    @close="showEmailViewer = false"
  />
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import EmailViewer from './EmailViewer.vue'
import BaseModal from '@/components/common/BaseModal.vue'

const props = defineProps({
  isPageMode: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close'])

// i18n
const { t } = useI18n()

// 响应式数据
const accounts = ref([])
const accountStatuses = ref({})
const isLoading = ref(false)
const isAdding = ref(false)
const isCheckingStatus = ref(false)
const statusCheckProgress = ref(0)
const isRefreshingAll = ref(false)
const refreshResults = ref(null)
const showAddModal = ref(false)
const showEmailViewer = ref(false)
const selectedEmail = ref('')

const accountInput = ref('')
const addTab = ref('manual')
const searchQuery = ref('')
const accountPage = ref(1)
const accountPageSize = 20

// OAuth 状态
const oauthStep = ref(1)
const oauthAuthUrl = ref('')
const oauthRedirectedUrl = ref('')
const oauthResult = ref(null)
const oauthManualEmail = ref('')
const isOAuthLoading = ref(false)

// 计算属性
const filteredAccounts = computed(() => {
  if (!searchQuery.value.trim()) return accounts.value
  const q = searchQuery.value.trim().toLowerCase()
  return accounts.value.filter(a => a.email.toLowerCase().includes(q))
})

const accountTotalPages = computed(() => Math.ceil(filteredAccounts.value.length / accountPageSize) || 1)

const paginatedAccounts = computed(() => {
  const start = (accountPage.value - 1) * accountPageSize
  return filteredAccounts.value.slice(start, start + accountPageSize)
})

watch(searchQuery, () => { accountPage.value = 1 })

const canAddAccount = computed(() => {
  const lines = accountInput.value.trim().split('\n').filter(l => l.trim())
  return lines.length > 0 && lines.some(line => {
    const parts = line.trim().split('----')
    return parts.length >= 3
  })
})

// 方法
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const refreshAccounts = async () => {
  isLoading.value = true
  try {
    accounts.value = await invoke('outlook_get_all_accounts_info')
  } catch (error) {
    showStatus(`刷新失败: ${error}`, 'error')
  } finally {
    isLoading.value = false
  }
}

const checkAllStatuses = async () => {
  await refreshAccounts()
  if (accounts.value.length === 0) return
  isCheckingStatus.value = true
  statusCheckProgress.value = 0

  let successCount = 0
  let failCount = 0

  for (const account of accounts.value) {
    accountStatuses.value[account.email] = 'checking'
    try {
      const result = await invoke('outlook_check_account_status', { email: account.email })
      accountStatuses.value[account.email] = result.status
      if (result.status === 'active') successCount++
      else failCount++
    } catch {
      accountStatuses.value[account.email] = 'error'
      failCount++
    }
    statusCheckProgress.value++
  }

  isCheckingStatus.value = false
  if (failCount === 0) {
    showStatus(`全部 ${successCount} 个账号状态正常`, 'success')
  } else {
    showStatus(`${successCount} 个正常，${failCount} 个异常`, 'warning')
  }
}

const addAccount = async () => {
  isAdding.value = true
  try {
    const lines = accountInput.value.trim().split('\n').filter(l => l.trim())
    if (lines.length === 0) {
      throw new Error('请输入账号信息')
    }

    let successCount = 0
    let failCount = 0
    const errors = []

    for (const line of lines) {
      try {
        // 格式: 邮箱----密码----Refresh Token----Client ID (密码可选)
        const parts = line.trim().split('----').map(p => p.trim())
        let email, refreshToken, clientId

        if (parts.length === 4) {
          // 四字段: 邮箱----密码----refreshToken----clientId
          email = parts[0]
          refreshToken = parts[2]
          clientId = parts[3]
        } else if (parts.length === 3) {
          // 三字段: 邮箱----refreshToken----clientId
          email = parts[0]
          refreshToken = parts[1]
          clientId = parts[2]
        } else {
          failCount++
          errors.push(`格式错误: ${line.substring(0, 30)}...`)
          continue
        }

        if (!email || !refreshToken || !clientId) {
          failCount++
          errors.push(`字段不完整: ${email || '(空)'}`)
          continue
        }

        await invoke('outlook_save_credentials', { email, refreshToken, clientId })
        successCount++
      } catch (err) {
        failCount++
        errors.push(`${err}`)
      }
    }

    // 重置表单
    accountInput.value = ''
    await refreshAccounts()

    if (failCount === 0) {
      showAddModal.value = false
      showStatus(`成功添加 ${successCount} 个账号`, 'success')
    } else if (successCount > 0) {
      showAddModal.value = false
      showStatus(`添加 ${successCount} 个成功，${failCount} 个失败`, 'warning')
    } else {
      showStatus(`全部失败 (${failCount} 个): ${errors[0]}`, 'error')
    }
  } catch (error) {
    showStatus(`添加失败: ${error}`, 'error')
  } finally {
    isAdding.value = false
  }
}

const deleteAccount = async (email) => {
  if (!confirm(`确定要从当前会话中移除账户 ${email} 吗？`)) {
    return
  }

  try {
    const deleted = await invoke('outlook_delete_account', { email })
    if (deleted) {
      await refreshAccounts()
      delete accountStatuses.value[email]
      showStatus(t('outlookManager.messages.deleteSuccess'), 'success')
    } else {
      showStatus(t('outlookManager.messages.invalidFormat'), 'warning')
    }
  } catch (error) {
    showStatus(`删除失败: ${error}`, 'error')
  }
}

const checkStatus = async (email) => {
  isCheckingStatus.value = true
  try {
    const status = await invoke('outlook_check_account_status', { email })
    accountStatuses.value[email] = status.status
    showStatus(`${email} 状态: ${status.status}`, 'info')
  } catch (error) {
    accountStatuses.value[email] = 'error'
    showStatus(`${t('outlookManager.messages.statusCheckFailed')}: ${error}`, 'error')
  } finally {
    isCheckingStatus.value = false
  }
}

const viewEmails = (email) => {
  selectedEmail.value = email
  showEmailViewer.value = true
}

// OAuth 方法
const startOAuth = async () => {
  isOAuthLoading.value = true
  try {
    const result = await invoke('outlook_get_oauth_auth_url')
    oauthAuthUrl.value = result.auth_url
    oauthStep.value = 2
  } catch (error) {
    showStatus(`生成授权链接失败: ${error}`, 'error')
  } finally {
    isOAuthLoading.value = false
  }
}

const exchangeOAuthToken = async () => {
  isOAuthLoading.value = true
  try {
    const result = await invoke('outlook_exchange_oauth_token', {
      redirectedUrl: oauthRedirectedUrl.value.trim()
    })
    oauthResult.value = result
    oauthManualEmail.value = ''
    oauthStep.value = 3
  } catch (error) {
    showStatus(`获取 Token 失败: ${error}`, 'error')
  } finally {
    isOAuthLoading.value = false
  }
}

const saveOAuthAccount = async () => {
  isOAuthLoading.value = true
  try {
    const email = oauthResult.value?.email || oauthManualEmail.value.trim()
    if (!email) throw new Error('请输入邮箱地址')

    await invoke('outlook_save_credentials', {
      email,
      refreshToken: oauthResult.value.refresh_token,
      clientId: oauthResult.value.client_id
    })

    await refreshAccounts()
    showAddModal.value = false
    oauthStep.value = 1
    oauthRedirectedUrl.value = ''
    oauthResult.value = null
    showStatus(`账号 ${email} 添加成功`, 'success')
  } catch (error) {
    showStatus(`保存失败: ${error}`, 'error')
  } finally {
    isOAuthLoading.value = false
  }
}

const copyToClipboard = async (text) => {
  try {
    await navigator.clipboard.writeText(text)
    showStatus('已复制到剪贴板', 'success')
  } catch {
    showStatus('复制失败', 'error')
  }
}

const refreshAllTokens = async () => {
  isRefreshingAll.value = true
  refreshResults.value = null
  try {
    const result = await invoke('outlook_refresh_all_tokens', { emails: null })
    refreshResults.value = result

    for (const r of result.results) {
      accountStatuses.value[r.email] = r.success ? 'active' : 'inactive'
    }

    if (result.failed_count === 0) {
      showStatus(`全部 ${result.total} 个账号 Token 刷新成功`, 'success')
    } else {
      showStatus(`${result.success_count} 个成功，${result.failed_count} 个失败`, 'warning')
    }
  } catch (error) {
    showStatus(`批量刷新失败: ${error}`, 'error')
  } finally {
    isRefreshingAll.value = false
  }
}

const retryFailedTokens = async () => {
  if (!refreshResults.value) return
  const failedEmails = refreshResults.value.results
    .filter(r => !r.success)
    .map(r => r.email)
  if (failedEmails.length === 0) return

  isRefreshingAll.value = true
  try {
    const result = await invoke('outlook_refresh_all_tokens', { emails: failedEmails })

    for (const r of result.results) {
      accountStatuses.value[r.email] = r.success ? 'active' : 'inactive'
    }

    // 合并结果：替换之前失败的记录
    const prev = refreshResults.value
    const updatedResults = prev.results.map(r => {
      const retry = result.results.find(rr => rr.email === r.email)
      return retry || r
    })
    const newFailed = updatedResults.filter(r => !r.success).length
    const newSuccess = updatedResults.filter(r => r.success).length
    refreshResults.value = {
      total: prev.total,
      success_count: newSuccess,
      failed_count: newFailed,
      results: updatedResults
    }

    if (result.failed_count === 0) {
      showStatus(`重试成功，${result.success_count} 个账号已恢复`, 'success')
    } else {
      showStatus(`重试: ${result.success_count} 成功，${result.failed_count} 仍失败`, 'warning')
    }
  } catch (error) {
    showStatus(`重试失败: ${error}`, 'error')
  } finally {
    isRefreshingAll.value = false
  }
}

const getStatusClass = (email) => {
  const status = accountStatuses.value[email]
  switch (status) {
    case 'active': return 'badge badge--sm badge--success-tech'
    case 'inactive': return 'badge badge--sm badge--danger-tech'
    case 'banned': return 'badge badge--sm badge--danger-tech'
    case 'error': return 'badge badge--sm badge--warning-tech'
    case 'checking': return 'badge badge--sm'
    default: return 'badge badge--sm opacity-60'
  }
}

const getStatusText = (email) => {
  const status = accountStatuses.value[email]
  switch (status) {
    case 'active': return t('outlookManager.status.online')
    case 'inactive': return t('outlookManager.status.offline')
    case 'banned': return t('outlookManager.status.banned')
    case 'error': return t('outlookManager.status.error')
    case 'checking': return t('outlookManager.status.checking')
    default: return t('outlookManager.status.unchecked')
  }
}

const formatDate = (dateString) => {
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 加载持久化的账号状态
const loadPersistedStatuses = async () => {
  try {
    const statuses = await invoke('outlook_get_account_statuses')
    for (const [email, status] of Object.entries(statuses)) {
      accountStatuses.value[email] = status
    }
  } catch {
    // 忽略，首次使用可能无数据
  }
}

// 生命周期
onMounted(async () => {
  await refreshAccounts()
  await loadPersistedStatuses()
})
</script>
