<template>
  <div class="modal-overlay">
    <div class="modal-content email-details" @click.stop>
      <div class="modal-header">
        <h3>邮件详情</h3>
        <button @click="$emit('close')" class="close-btn">×</button>
      </div>

      <div class="modal-body">
        <div v-if="isLoading" class="loading-state">
          <div class="spinner"></div>
          <p>加载邮件详情中...</p>
        </div>

        <div v-else-if="error" class="error-state">
          <p>{{ error }}</p>
          <button @click="loadEmailDetails" class="btn primary">重新加载</button>
        </div>

        <div v-else-if="emailDetails" class="email-content">
          <!-- 邮件头部信息 -->
          <div class="email-header">
            <h2 class="email-subject">{{ emailDetails.subject }}</h2>
            
            <div class="email-meta">
              <div class="meta-row">
                <span class="meta-label">发件人:</span>
                <span class="meta-value">{{ emailDetails.from_email }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">收件人:</span>
                <span class="meta-value">{{ emailDetails.to_email }}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">日期:</span>
                <span class="meta-value">{{ formatDate(emailDetails.date) }}</span>
              </div>
            </div>
          </div>

          <!-- 邮件正文 -->
          <div class="email-body">
            <div v-if="emailDetails.body_html" class="body-section">
              <h4>HTML 内容:</h4>
              <div class="html-content" v-html="emailDetails.body_html"></div>
            </div>
            
            <div v-if="emailDetails.body_plain" class="body-section">
              <h4>纯文本内容:</h4>
              <pre class="plain-content">{{ emailDetails.body_plain }}</pre>
            </div>

            <div v-if="!emailDetails.body_html && !emailDetails.body_plain" class="no-content">
              <p>此邮件没有可显示的内容</p>
              <p class="no-content-hint">可能是特殊格式或编码问题</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  email: {
    type: String,
    required: true
  },
  messageId: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

// 响应式数据
const emailDetails = ref(null)
const isLoading = ref(false)
const error = ref('')

// 方法
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const loadEmailDetails = async () => {
  isLoading.value = true
  error.value = ''
  
  try {
    emailDetails.value = await invoke('outlook_get_email_details', {
      email: props.email,
      messageId: props.messageId
    })
  } catch (err) {
    error.value = `加载邮件详情失败: ${err}`
    showStatus(error.value, 'error')
  } finally {
    isLoading.value = false
  }
}

const formatDate = (dateString) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  } catch {
    return dateString
  }
}

// 生命周期
onMounted(() => {
  loadEmailDetails()
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
  z-index: 1000;
}

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 90vw;
  max-height: 95vh;
  overflow: hidden;
  position: relative;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
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

.email-details {
  width: 90vw;
  max-width: 900px;
  max-height: 90vh;
}

.modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.loading-state,
.error-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--color-text-muted, #6b7280);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--color-surface-hover, #f3f3f3);
  border-top: 4px solid var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.email-header {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
}

.email-subject {
  margin: 0 0 16px 0;
  color: var(--color-text-strong, #111827);
  font-size: 24px;
  font-weight: 600;
  line-height: 1.3;
}

.email-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.meta-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.meta-label {
  font-weight: 600;
  color: var(--color-text-primary, #374151);
  min-width: 60px;
  flex-shrink: 0;
}

.meta-value {
  color: var(--color-text-muted, #6b7280);
  word-break: break-all;
}

.email-body {
  line-height: 1.6;
}

.body-section {
  margin-bottom: 24px;
}

.body-section h4 {
  margin: 0 0 12px 0;
  color: var(--color-text-primary, #374151);
  font-size: 16px;
  font-weight: 600;
}

.html-content {
  background: var(--color-surface-alt, #f9fafb);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
  word-wrap: break-word;
}

.plain-content {
  background: var(--color-surface-alt, #f9fafb);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  padding: 16px;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 400px;
  overflow-y: auto;
  margin: 0;
}

.no-content {
  text-align: center;
  padding: 40px;
  color: var(--color-text-soft, #9ca3af);
  font-style: italic;
}

.no-content-hint {
  font-size: 12px;
  margin-top: 8px;
  color: var(--color-border-strong, #d1d5db);
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
}

.btn.primary {
  background: var(--color-accent, #3b82f6);
  color: var(--color-text-inverse, #ffffff);
}

.btn.primary:hover {
  background: var(--color-accent-hover, #2563eb);
}

@media (max-width: 768px) {
  .email-details {
    width: 95vw;
    max-width: none;
  }
  
  .modal-body {
    padding: 16px;
  }
  
  .email-subject {
    font-size: 20px;
  }
  
  .meta-row {
    flex-direction: column;
    gap: 4px;
  }
  
  .meta-label {
    min-width: auto;
  }
}
</style>
