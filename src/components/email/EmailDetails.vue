<template>
  <div class="modal-overlay">
    <div class="modal-content modal-content-shell email-details" @click.stop>
      <div class="modal-header">
        <h3>邮件详情</h3>
        <button @click="$emit('close')" class="modal-close">×</button>
      </div>

      <div class="modal-body modal-body-scroll">
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
/* ============================================
   EmailDetails - Modern Tech Style
   ============================================ */

.email-details {
  width: 90vw;
  max-width: 920px;
  max-height: 90vh;
}

.email-details .modal-body {
  padding: 26px;
}

.loading-state,
.error-state {
  text-align: center;
  padding: 65px 22px;
  color: var(--text-muted);
}

/* 加载动画 - 科技风 */
.spinner {
  width: 42px;
  height: 42px;
  border: 3px solid var(--tech-glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 22px;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.email-header {
  margin-bottom: 26px;
  padding-bottom: 22px;
  border-bottom: 1px solid var(--tech-glass-border);
}

.email-subject {
  margin: 0 0 18px 0;
  color: var(--text-strong);
  font-size: 24px;
  font-weight: 700;
  line-height: 1.3;
}

.email-meta {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.meta-row {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.meta-label {
  font-weight: 600;
  color: var(--text-muted);
  min-width: 65px;
  flex-shrink: 0;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.meta-value {
  color: var(--text);
  word-break: break-all;
}

.email-body {
  line-height: 1.7;
}

.body-section {
  margin-bottom: 26px;
}

.body-section h4 {
  margin: 0 0 14px 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

/* 内容区域 - 科技风 */
.html-content {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  padding: 18px;
  max-height: 400px;
  overflow-y: auto;
  word-wrap: break-word;
}

.plain-content {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  padding: 18px;
  font-size: 14px;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 400px;
  overflow-y: auto;
  margin: 0;
}

.no-content {
  text-align: center;
  padding: 45px;
  color: var(--text-muted);
  font-style: italic;
}

.no-content-hint {
  font-size: 12px;
  margin-top: 10px;
  color: var(--text-muted);
  opacity: 0.7;
}

@media (max-width: 768px) {
  .email-details {
    width: 95vw;
    max-width: none;
  }

  .email-details .modal-body {
    padding: 18px;
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
