<template>
  <div :class="isPageMode ? 'page-container' : 'modal-overlay'">
    <div :class="isPageMode ? 'page-content gptmail-manager' : ['modal-content', 'modal-content-shell', 'gptmail-manager']" @click.stop>
      <div :class="isPageMode ? 'page-header' : 'modal-header'">
        <h3>{{ $t('gptMailManager.title') }}</h3>
        <button v-if="!isPageMode" @click="$emit('close')" class="modal-close">×</button>
      </div>

      <div :class="isPageMode ? 'page-body' : ['modal-body', 'modal-body-scroll']">
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
              <button @click="copyEmail" class="btn primary small">
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
            <button @click="toggleAutoPolling" class="btn primary"
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
              <button @click="copyCode" class="btn primary small">
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
/* ============================================
   GPTMailManager - Modern Tech Style
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
.modal-content.gptmail-manager {
  width: 100%;
  max-width: 920px;
  height: 90vh;
  position: relative;
}

.modal-content.gptmail-manager .modal-body {
  padding: 26px;
  height: calc(100% - 73px);
}

/* 区块 - 科技风 */
.generate-email-section,
.fetch-emails-section {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  padding: 22px;
  margin-bottom: 26px;
}

.generate-email-section h4,
.fetch-emails-section h4 {
  margin: 0 0 18px 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

.form-actions {
  display: flex;
  gap: 14px;
  margin-top: 18px;
  flex-wrap: wrap;
}

.generated-email-display {
  margin-top: 18px;
}

.generated-email-display label {
  display: block;
  margin-bottom: 10px;
  font-weight: 600;
  color: var(--text);
  font-size: 14px;
}

.email-display-box {
  display: flex;
  gap: 10px;
  align-items: center;
}

.email-display-box input {
  flex: 1;
}

/* 轮询指示器 - 科技风 */
.polling-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px;
  margin-top: 14px;
  background: color-mix(in srgb, #3b82f6 12%, transparent);
  border: 1px solid color-mix(in srgb, #3b82f6 35%, transparent);
  border-radius: 10px;
  font-size: 13px;
  color: #3b82f6;
}

.polling-dot {
  width: 10px;
  height: 10px;
  background: #3b82f6;
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
  box-shadow: 0 0 8px rgba(59, 130, 246, 0.5);
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.4;
    transform: scale(0.8);
  }
}

/* 验证码显示 - 科技风 */
.verification-code-display {
  margin-top: 18px;
  padding: 18px;
  background: color-mix(in srgb, #10b981 12%, transparent);
  border: 2px solid color-mix(in srgb, #10b981 40%, transparent);
  border-radius: 12px;
  box-shadow: 0 0 15px rgba(16, 185, 129, 0.2);
}

.verification-code-display label {
  display: block;
  margin-bottom: 10px;
  font-weight: 600;
  color: #10b981;
  font-size: 14px;
}

.code-box {
  display: flex;
  gap: 14px;
  align-items: center;
}

.code-value {
  font-size: 26px;
  font-weight: 700;
  font-family: var(--tech-mono-font);
  color: #10b981;
  letter-spacing: 5px;
  padding: 10px 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, #10b981 35%, transparent);
  text-shadow: 0 0 10px rgba(16, 185, 129, 0.4);
}

/* 邮件列表 */
.emails-section {
  margin-top: 22px;
}

.emails-list h4 {
  margin: 0 0 18px 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

/* 邮件项 - 科技风 */
.email-item {
  padding: 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  margin-bottom: 14px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  user-select: none;
}

.email-item:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  box-shadow: 0 0 15px var(--tech-glow-primary);
  transform: translateX(4px);
}

.email-item.expanded {
  border-color: var(--accent);
  box-shadow: 0 0 20px var(--tech-glow-primary);
}

.email-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  gap: 14px;
}

.email-from {
  font-size: 14px;
  color: var(--text);
  flex: 1;
  min-width: 0;
}

.email-time {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.expand-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  transition: transform 0.3s ease, color 0.2s ease;
}

.expand-indicator svg {
  transition: transform 0.3s ease;
}

.expand-indicator svg.rotated {
  transform: rotate(180deg);
}

.email-item:hover .expand-indicator {
  color: var(--accent);
}

.email-subject {
  font-size: 14px;
  color: var(--text);
  margin-bottom: 0;
  font-weight: 500;
}

/* 邮件内容 - 科技风 */
.email-content {
  font-size: 13px;
  color: var(--text);
  line-height: 1.7;
  max-height: 400px;
  overflow-y: auto;
  padding: 14px;
  background: color-mix(in srgb, var(--bg-muted) 70%, transparent);
  border-radius: 10px;
  border: 1px solid var(--tech-glass-border);
  margin-top: 14px;
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

/* 加载和空状态 */
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
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modal-content.gptmail-manager {
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
  .modal-content.gptmail-manager {
    max-height: 95vh;
  }

  .code-value {
    font-size: 20px;
    letter-spacing: 3px;
  }
}
</style>
