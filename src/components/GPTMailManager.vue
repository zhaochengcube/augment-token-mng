<template>
  <div class="modal-overlay">
    <div class="modal-content gptmail-manager" @click.stop>
      <div class="modal-header">
        <h3>{{ $t('gptMailManager.title') }}</h3>
        <button @click="$emit('close')" class="close-btn">×</button>
      </div>

      <div class="modal-body">
        <!-- 生成随机邮箱区域 -->
        <div class="generate-email-section">
          <h4>{{ $t('gptMailManager.generateEmail') }}</h4>
          <div class="form-actions">
            <button @click="generateRandomEmail" :disabled="isGenerating"
              :class="['btn', 'primary', { loading: isGenerating }]">
              {{ isGenerating ? $t('gptMailManager.generating') : $t('gptMailManager.generateBtn') }}
            </button>
          </div>
          <div v-if="generatedEmail" class="generated-email-display">
            <label>{{ $t('gptMailManager.generatedEmail') }}:</label>
            <div class="email-display-box">
              <input type="text" :value="generatedEmail" readonly class="form-input">
              <button @click="copyEmail" class="btn secondary small">
                {{ $t('gptMailManager.copy') }}
              </button>
            </div>
          </div>
        </div>

        <!-- 获取邮件和验证码区域 -->
        <div class="fetch-emails-section">
          <h4>{{ $t('gptMailManager.fetchEmails') }}</h4>

          <div class="form-group">
            <label>{{ $t('gptMailManager.emailAddress') }}:</label>
            <input v-model="emailInput" type="text" :placeholder="$t('gptMailManager.emailPlaceholder')"
              class="form-input">
            <div class="input-hint">
              {{ $t('gptMailManager.emailHint') }}
            </div>
          </div>

          <div class="form-actions">
            <button @click="fetchEmails" :disabled="!canFetchEmails || isFetching"
              :class="['btn', 'primary', { loading: isFetching }]">
              {{ isFetching ? $t('gptMailManager.fetching') : $t('gptMailManager.fetchBtn') }}
            </button>
            <button @click="toggleAutoPolling" :class="['btn', isPolling ? 'danger' : 'secondary']"
              :disabled="!canFetchEmails">
              {{ isPolling ? $t('gptMailManager.stopPolling') : $t('gptMailManager.startPolling') }}
            </button>
          </div>

          <!-- 轮询状态指示 -->
          <div v-if="isPolling" class="polling-indicator">
            <span class="polling-dot"></span>
            {{ $t('gptMailManager.pollingActive') }} ({{ pollingCountdown }}s)
          </div>

          <!-- 验证码显示 -->
          <div v-if="verificationCode" class="verification-code-display">
            <label>{{ $t('gptMailManager.verificationCode') }}:</label>
            <div class="code-box">
              <span class="code-value">{{ verificationCode }}</span>
              <button @click="copyCode" class="btn success small">
                {{ $t('gptMailManager.copy') }}
              </button>
            </div>
          </div>

          <!-- 邮件列表 -->
          <div class="emails-section">
            <div v-if="isFetching && emails.length === 0" class="loading-state">
              <div class="spinner"></div>
              <p>{{ $t('gptMailManager.loadingEmails') }}</p>
            </div>

            <div v-else-if="emails.length === 0 && hasFetched" class="empty-state">
              <p>{{ $t('gptMailManager.noEmails') }}</p>
            </div>

            <div v-else-if="emails.length > 0" class="emails-list">
              <h4>{{ $t('gptMailManager.emailList') }} ({{ emails.length }})</h4>
              <div v-for="email in emails" :key="email.id"
                :class="['email-item', { 'expanded': isEmailExpanded(email.id) }]"
                @click="toggleEmailExpansion(email.id)">
                <div class="email-header">
                  <div class="email-from">
                    <strong>{{ $t('gptMailManager.from') }}:</strong> {{ email.from }}
                  </div>
                  <div class="email-time">
                    {{ formatTimestamp(email.timestamp) }}
                  </div>
                  <div class="expand-indicator">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"
                      :class="{ 'rotated': isEmailExpanded(email.id) }">
                      <path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6 1.41-1.41z" />
                    </svg>
                  </div>
                </div>
                <div class="email-subject">
                  <strong>{{ $t('gptMailManager.subject') }}:</strong> {{ email.subject }}
                </div>
                <transition name="expand">
                  <div v-if="isEmailExpanded(email.id)" class="email-content" @click.stop>
                    <!-- 优先显示 HTML 内容 -->
                    <div v-if="email.htmlContent" v-html="email.htmlContent" class="email-html-content"></div>
                    <!-- 否则显示纯文本内容 -->
                    <pre v-else class="email-text-content">{{ email.content || $t('gptMailManager.noContent') }}</pre>
                  </div>
                </transition>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['close'])

// i18n
const { t } = useI18n()

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
const expandedEmails = ref(new Set())

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
    const data = await invoke('generate_random_email')
    generatedEmail.value = data.email
    emailInput.value = data.email // 自动填充到输入框
    showStatus(t('gptMailManager.generateSuccess'), 'success')
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
    const data = await invoke('get_emails', { email })
    emails.value = data.emails || []

    // 提取验证码
    extractVerificationCode()

    if (emails.value.length > 0) {
      showStatus(t('gptMailManager.fetchSuccess', { count: emails.value.length }), 'success')
    } else {
      showStatus(t('gptMailManager.noEmails'), 'info')
    }
  } catch (error) {
    showStatus(`${t('gptMailManager.fetchFailed')}: ${error}`, 'error')
    emails.value = []
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

// 停止轮询
const stopPolling = () => {
  isPolling.value = false

  if (pollingInterval.value) {
    clearInterval(pollingInterval.value)
    pollingInterval.value = null
  }

  if (countdownInterval.value) {
    clearInterval(countdownInterval.value)
    countdownInterval.value = null
  }

  showStatus(t('gptMailManager.pollingStopped'), 'info')
}

// 格式化时间戳
const formatTimestamp = (timestamp) => {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  } catch {
    return timestamp
  }
}

// 切换邮件展开/收缩状态
const toggleEmailExpansion = (emailId) => {
  if (expandedEmails.value.has(emailId)) {
    expandedEmails.value.delete(emailId)
  } else {
    expandedEmails.value.add(emailId)
  }
  // 触发响应式更新
  expandedEmails.value = new Set(expandedEmails.value)
}

// 检查邮件是否展开
const isEmailExpanded = (emailId) => {
  return expandedEmails.value.has(emailId)
}

// 生命周期
onMounted(() => {
  // 组件挂载时不自动开始轮询
})

onBeforeUnmount(() => {
  // 组件卸载时清理定时器
  stopPolling()
})

</script>

<style scoped>
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
  background: var(--color-surface, #ffffff);
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
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
  border-radius: 12px 12px 0 0;
}

.modal-header h3 {
  margin: 0;
  color: var(--color-text-primary, #374151);
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
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
  background: var(--color-border, #e5e7eb);
  color: var(--color-text-primary, #374151);
}

.gptmail-manager {
  width: 100%;
  max-width: 900px;
  height: 90vh;
}

.modal-body {
  padding: 24px;
  overflow-y: auto;
  height: calc(100% - 73px);
}

.generate-email-section,
.fetch-emails-section {
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 24px;
}

.generate-email-section h4,
.fetch-emails-section h4 {
  margin: 0 0 16px 0;
  color: var(--color-text-heading, #333);
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
  color: var(--color-text-primary, #374151);
  font-size: 14px;
}

.form-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s ease;
  box-sizing: border-box;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
}

.form-input:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-input:read-only {
  background-color: var(--color-surface-muted, #f8f9fa);
  color: var(--color-text-muted, #6c757d);
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 16px;
  flex-wrap: wrap;
}

.input-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
  font-style: italic;
}

.generated-email-display {
  margin-top: 16px;
}

.generated-email-display label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  font-size: 14px;
}

.email-display-box {
  display: flex;
  gap: 8px;
  align-items: center;
}

.email-display-box input {
  flex: 1;
}

/* 轮询指示器 */
.polling-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  margin-top: 12px;
  background: var(--color-info-surface, #dbeafe);
  border: 1px solid var(--color-info-border, #93c5fd);
  border-radius: 6px;
  font-size: 13px;
  color: var(--color-info-text, #1e40af);
}

.polling-dot {
  width: 8px;
  height: 8px;
  background: var(--color-info-text, #1e40af);
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.3;
  }
}

/* 验证码显示 */
.verification-code-display {
  margin-top: 16px;
  padding: 16px;
  background: var(--color-success-surface, #d1fae5);
  border: 2px solid var(--color-success-border, #a7f3d0);
  border-radius: 8px;
}

.verification-code-display label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  color: var(--color-success-text, #065f46);
  font-size: 14px;
}

.code-box {
  display: flex;
  gap: 12px;
  align-items: center;
}

.code-value {
  font-size: 24px;
  font-weight: 700;
  font-family: 'Courier New', monospace;
  color: var(--color-success-text, #065f46);
  letter-spacing: 4px;
  padding: 8px 16px;
  background: var(--color-surface, #ffffff);
  border-radius: 6px;
  border: 1px solid var(--color-success-border, #a7f3d0);
}

/* 邮件列表 */
.emails-section {
  margin-top: 20px;
}

.emails-list h4 {
  margin: 0 0 16px 0;
  color: var(--color-text-heading, #333);
  font-size: 16px;
  font-weight: 600;
}

.email-item {
  padding: 16px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  margin-bottom: 12px;
  transition: all 0.2s;
  cursor: pointer;
  user-select: none;
}

.email-item:hover {
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.1);
  background: var(--color-surface-hover, #f9fafb);
}

.email-item.expanded {
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
}

.email-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  gap: 12px;
}

.email-from {
  font-size: 14px;
  color: var(--color-text-primary, #374151);
  flex: 1;
  min-width: 0;
}

.email-time {
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
  white-space: nowrap;
}

.expand-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted, #6b7280);
  transition: transform 0.3s ease, color 0.2s ease;
}

.expand-indicator svg {
  transition: transform 0.3s ease;
}

.expand-indicator svg.rotated {
  transform: rotate(180deg);
}

.email-item:hover .expand-indicator {
  color: var(--color-accent, #3b82f6);
}

.email-subject {
  font-size: 14px;
  color: var(--color-text-primary, #374151);
  margin-bottom: 0;
}

.email-content {
  font-size: 13px;
  color: var(--color-text-primary, #374151);
  line-height: 1.6;
  max-height: 400px;
  overflow-y: auto;
  padding: 12px;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 6px;
  border: 1px solid var(--color-border, #e5e7eb);
  margin-top: 12px;
}

.email-html-content {
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.email-html-content * {
  max-width: 100%;
}

.email-text-content {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: inherit;
  font-size: inherit;
  color: inherit;
  background: transparent;
  border: none;
  padding: 0;
}

/* 展开/收缩过渡动画 */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
  margin-top: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 500px;
}

/* 按钮样式 */
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
  background: var(--color-accent, #3b82f6);
  color: var(--color-text-inverse, #ffffff);
}

.btn.primary:hover:not(:disabled) {
  background: var(--color-accent-hover, #2563eb);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(59, 130, 246, 0.3);
}

.btn.secondary {
  background: var(--color-text-muted, #6b7280);
  color: var(--color-text-inverse, #ffffff);
}

.btn.secondary:hover:not(:disabled) {
  background: var(--color-text-secondary, #4b5563);
}

.btn.danger {
  background: var(--color-danger-bg, #dc2626);
  color: var(--color-text-inverse, #ffffff);
}

.btn.danger:hover:not(:disabled) {
  background: var(--color-danger-bg-hover, #b91c1c);
}

.btn.success {
  background: var(--color-success-bg, #10b981);
  color: var(--color-text-inverse, #ffffff);
}

.btn.success:hover:not(:disabled) {
  background: var(--color-success-bg-hover, #059669);
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
}

.btn.loading {
  opacity: 0.7;
  cursor: wait;
}

/* 加载和空状态 */
.loading-state,
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--color-text-muted, #6b7280);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-surface-hover, #f3f3f3);
  border-top: 3px solid var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modal-content {
    margin: 10px;
    max-width: calc(100vw - 20px);
    height: calc(100vh - 20px);
  }

  .form-actions {
    flex-direction: column;
  }

  .form-actions .btn {
    width: 100%;
  }

  .email-display-box {
    flex-direction: column;
  }

  .email-display-box .btn {
    width: 100%;
  }
}

@media (max-width: 480px) {
  .modal-content {
    max-height: 95vh;
  }

  .code-value {
    font-size: 20px;
    letter-spacing: 2px;
  }
}

/* 深色主题样式 */
[data-theme='dark'] .polling-indicator {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.4);
  color: #60a5fa;
}

[data-theme='dark'] .polling-dot {
  background: #60a5fa;
}

[data-theme='dark'] .verification-code-display {
  background: rgba(16, 185, 129, 0.15);
  border-color: rgba(16, 185, 129, 0.4);
}

[data-theme='dark'] .verification-code-display label {
  color: #6ee7b7;
}

[data-theme='dark'] .code-value {
  color: #6ee7b7;
  background: var(--color-surface, #1e293b);
  border-color: rgba(16, 185, 129, 0.4);
}
</style>
