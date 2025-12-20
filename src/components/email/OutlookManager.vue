<template>
  <div :class="isPageMode ? 'page-container' : 'modal-overlay'">
    <div :class="isPageMode ? 'page-content outlook-manager' : 'modal-content outlook-manager'" @click.stop>
      <div :class="isPageMode ? 'page-header' : 'modal-header'">
        <h3>{{ $t('outlookManager.title') }}</h3>
        <button v-if="!isPageMode" @click="$emit('close')" class="close-btn">×</button>
      </div>

      <div :class="isPageMode ? 'page-body' : 'modal-body'">
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
              :class="['btn', 'primary', { loading: isAdding }]"
            >
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
              class="btn secondary small"
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
                  class="btn secondary small"
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
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface);
  flex-shrink: 0;
}

.page-header h3 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--text-strong);
}

.page-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

/* Modal Mode Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  z-index: 2000;
}

.modal-content {
  background: var(--bg-surface);
  border-radius: 12px;
  width: 100%;
  max-width: 900px;
  height: 90vh;
  overflow: hidden;
  position: relative;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-muted);
  border-radius: 12px 12px 0 0;
}

.modal-header h3 {
  margin: 0;
  color: var(--text-strong);
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--text-muted);
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.outlook-manager {
  width: 100%;
  max-width: 900px;
  height: 90vh;
}

.modal-body {
  padding: 24px;
  overflow-y: auto;
}

.add-account-section {
  background: var(--bg-muted);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 24px;
}

.add-account-section h4 {
  margin: 0 0 16px 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: var(--text);
  font-size: 14px;
}

.form-input,
.form-textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s ease;
  box-sizing: border-box;
  background: var(--bg-surface);
  color: var(--text);
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 10%, transparent);
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.form-actions {
  margin-top: 20px;
}

.input-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
  font-style: italic;
}

.session-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--color-warning-surface, #fef3c7);
  border: 1px solid var(--color-warning-bg, #f59e0b);
  border-radius: 6px;
  margin-bottom: 16px;
  font-size: 13px;
  color: var(--color-warning-text, #92400e);
}

.notice-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.empty-hint {
  font-size: 12px;
  color: var(--color-text-soft, #9ca3af);
  margin-top: 8px;
}

.accounts-section h4 {
  margin: 0;
  color: var(--color-text-heading, #333);
  font-size: 16px;
  font-weight: 600;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.accounts-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
}

.account-info {
  flex: 1;
}

.account-email {
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  margin-bottom: 6px;
}

.account-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.account-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 12px;
  display: inline-block;
}

.account-created {
  font-size: 11px;
  color: var(--color-text-muted, #6b7280);
  background: var(--color-surface-hover, #f3f4f6);
  padding: 2px 6px;
  border-radius: 8px;
}

.status-active {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.status-inactive {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.status-error {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.status-unknown {
  background: var(--bg-muted);
  color: var(--text-muted);
}

.account-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  text-decoration: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.btn.primary {
  background: var(--accent);
  color: var(--text-contrast);
}

.btn.primary:hover:not(:disabled) {
  background: var(--accent-strong);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px color-mix(in srgb, var(--accent) 30%, transparent);
}

.btn.secondary {
  background: var(--text-muted);
  color: var(--text-contrast);
}

.btn.secondary:hover:not(:disabled) {
  background: color-mix(in srgb, var(--text-muted) 80%, black);
}

.btn.danger {
  background: #dc2626;
  color: #ffffff;
}

.btn.danger:hover:not(:disabled) {
  background: #b91c1c;
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
}

.btn.loading {
  opacity: 0.7;
  cursor: wait;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--bg-muted);
  border-top: 3px solid var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Responsive Design */
@media (max-width: 768px) {
  .modal-content {
    margin: 10px;
    max-width: calc(100vw - 20px);
    height: calc(100vh - 20px);
  }
}

@media (max-width: 480px) {
  .modal-content {
    max-height: 95vh;
  }
}

</style>
