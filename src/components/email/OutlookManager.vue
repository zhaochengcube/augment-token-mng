<template>
  <div :class="isPageMode ? 'page-container' : 'modal-overlay'">
    <div :class="isPageMode ? 'page-content outlook-manager' : ['modal-content', 'modal-content-shell', 'outlook-manager']" @click.stop>
      <div :class="isPageMode ? 'page-header' : 'modal-header'">
        <h3>{{ $t('outlookManager.title') }}</h3>
        <button v-if="!isPageMode" @click="$emit('close')" class="modal-close">×</button>
      </div>

      <div :class="isPageMode ? 'page-body' : ['modal-body', 'modal-body-scroll']">
        <!-- 添加邮箱表单 -->
        <div class="add-account-section">
          <h4>{{ $t('outlookManager.addAccount') }}</h4>
          <div class="session-notice">
            <span class="notice-icon">ℹ️</span>
            {{ $t('outlookManager.sessionNotice') }}
          </div>
          <div class="form-group">
            <label>{{ $t('outlookManager.accountInfo') }}:</label>
            <input
              v-model="accountInput"
              type="text"
              :placeholder="$t('outlookManager.placeholder')"
              class="form-input"
            >
            <div class="input-hint">
              {{ $t('outlookManager.inputHint') }}
            </div>
          </div>
          <div class="form-actions">
            <button
              @click="addAccount"
              :disabled="!canAddAccount || isAdding"
              class="btn primary"
            >
              <span v-if="isAdding" class="btn-spinner" aria-hidden="true"></span>
              {{ isAdding ? $t('outlookManager.status.checking') : $t('outlookManager.addAccountBtn') }}
            </button>
          </div>
        </div>

        <!-- 账户列表 -->
        <div class="accounts-section">
          <div class="section-header">
            <h4>{{ $t('outlookManager.accountList') }} ({{ accounts.length }})</h4>
            <button
              @click="refreshAccounts"
              :disabled="isLoading"
              class="btn primary small"
            >
              {{ $t('outlookManager.checkStatus') }}
            </button>

          </div>

          <div v-if="isLoading" class="loading-state">
            <div class="spinner"></div>
            <p>{{ $t('outlookManager.status.checking') }}</p>
          </div>

          <div v-else-if="accounts.length === 0" class="empty-state">
            <p>{{ $t('outlookManager.emptyState') }}</p>
            <p class="empty-hint">{{ $t('outlookManager.emptyDescription') }}</p>
          </div>

          <div v-else class="accounts-list">
            <div
              v-for="account in accounts"
              :key="account.email"
              class="account-item"
            >
              <div class="account-info">
                <div class="account-email">{{ account.email }}</div>
                <div class="account-meta">
                  <div class="account-status" :class="getStatusClass(account.email)">
                    {{ getStatusText(account.email) }}
                  </div>
                  <div class="account-created">
                    {{ formatDate(account.created_at) }}
                  </div>
                </div>
              </div>
              <div class="account-actions">
                <button
                  @click="viewEmails(account.email)"
                  class="btn primary small"
                >
                  {{ $t('outlookManager.viewEmails') }}
                </button>
                <button
                  @click="checkStatus(account.email)"
                  :disabled="isCheckingStatus"
                  class="btn primary small"
                >
                  {{ $t('outlookManager.checkStatus') }}
                </button>
                <button
                  @click="deleteAccount(account.email)"
                  class="btn danger small"
                >
                  {{ $t('outlookManager.deleteAccount') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 邮件查看器 -->
  <EmailViewer
    v-if="showEmailViewer"
    :email="selectedEmail"
    @close="showEmailViewer = false"
  />
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import EmailViewer from './EmailViewer.vue'

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
const showEmailViewer = ref(false)
const selectedEmail = ref('')

const accountInput = ref('')

// 计算属性
const canAddAccount = computed(() => {
  return accountInput.value.trim() &&
         accountInput.value.includes('----') &&
         accountInput.value.split('----').length === 4
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

const addAccount = async () => {
  isAdding.value = true
  try {
    // 解析输入的账户信息（四字段格式：邮箱----密码----Refresh Token----Client ID）
    const parts = accountInput.value.trim().split('----')
    if (parts.length !== 4) {
      throw new Error('格式错误，请按照 邮箱地址----密码----Refresh Token----Client ID 的格式输入')
    }

    const [email, password, refreshToken, clientId] = parts.map(part => part.trim())

    if (!email || !password || !refreshToken || !clientId) {
      throw new Error(t('outlookManager.messages.invalidFormat'))
    }

    // 回退到IMAP版本（Graph API需要不同的权限）
    await invoke('outlook_save_credentials', {
      email,
      refreshToken,
      clientId
    })

    // 重置表单
    accountInput.value = ''

    // 刷新账户列表
    await refreshAccounts()
    showStatus(t('outlookManager.messages.addSuccess'), 'success')
  } catch (error) {
    showStatus(`${t('outlookManager.messages.addSuccess')}: ${error}`, 'error')
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
    showStatus(`${t('outlookManager.messages.deleteSuccess')}: ${error}`, 'error')
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



const getStatusClass = (email) => {
  const status = accountStatuses.value[email]
  return {
    'status-active': status === 'active',
    'status-inactive': status === 'inactive',
    'status-error': status === 'error',
    'status-unknown': !status
  }
}

const getStatusText = (email) => {
  const status = accountStatuses.value[email]
  switch (status) {
    case 'active': return t('outlookManager.status.online')
    case 'inactive': return t('outlookManager.status.offline')
    case 'error': return t('outlookManager.status.error')
    default: return t('outlookManager.status.checking')
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

// 生命周期
onMounted(() => {
  refreshAccounts()
})
</script>

<style scoped>
/* ============================================
   OutlookManager - Modern Tech Style
   ============================================ */

/* Page Mode Styles */
.page-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.page-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-page);
}

.page-header {
  padding: 22px 26px;
  border-bottom: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  flex-shrink: 0;
}

.page-header h3 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--text-strong);
}

.page-body {
  flex: 1;
  overflow-y: auto;
  padding: 26px;
}

/* Modal Mode Styles - 科技风 */
.modal-content.outlook-manager {
  width: 100%;
  max-width: 920px;
  height: 90vh;
}

.modal-content.outlook-manager .modal-body {
  padding: 26px;
}

/* 区块 - 科技风 */
.add-account-section {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  padding: 22px;
  margin-bottom: 26px;
}

.add-account-section h4 {
  margin: 0 0 18px 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

.form-actions {
  margin-top: 22px;
}

/* 警告提示 - 科技风 */
.session-notice {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px;
  background: color-mix(in srgb, #f59e0b 12%, transparent);
  border: 1px solid color-mix(in srgb, #f59e0b 40%, transparent);
  border-radius: 10px;
  margin-bottom: 18px;
  font-size: 13px;
  color: #f59e0b;
}

.notice-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 10px;
  opacity: 0.7;
}

.accounts-section h4 {
  margin: 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
}

.accounts-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* 账户项 - 科技风 */
.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.account-item:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.account-info {
  flex: 1;
}

.account-email {
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 8px;
}

.account-meta {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
}

/* 状态标签 - 科技风 */
.account-status {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 20px;
  display: inline-block;
}

.account-created {
  font-size: 11px;
  font-family: var(--tech-mono-font);
  color: var(--text-muted);
  background: color-mix(in srgb, var(--bg-muted) 70%, transparent);
  padding: 3px 8px;
  border-radius: 8px;
}

.status-active {
  background: color-mix(in srgb, #10b981 15%, transparent);
  color: #10b981;
  border: 1px solid color-mix(in srgb, #10b981 30%, transparent);
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.3);
}

.status-inactive {
  background: color-mix(in srgb, #ef4444 15%, transparent);
  color: #ef4444;
  border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
}

.status-error {
  background: color-mix(in srgb, #f59e0b 15%, transparent);
  color: #f59e0b;
  border: 1px solid color-mix(in srgb, #f59e0b 30%, transparent);
}

.status-unknown {
  background: color-mix(in srgb, var(--bg-muted) 70%, transparent);
  color: var(--text-muted);
  border: 1px solid var(--tech-glass-border);
}

.account-actions {
  display: flex;
  gap: 10px;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 45px 22px;
  color: var(--text-muted);
}

/* 加载动画 - 科技风 */
.spinner {
  width: 36px;
  height: 36px;
  border: 3px solid var(--tech-glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 18px;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Responsive Design */
@media (max-width: 768px) {
  .modal-content.outlook-manager {
    margin: 10px;
    max-width: calc(100vw - 20px);
    height: calc(100vh - 20px);
  }
}

@media (max-width: 480px) {
  .modal-content.outlook-manager {
    max-height: 95vh;
  }
}
</style>
