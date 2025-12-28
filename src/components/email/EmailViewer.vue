<template>
  <div class="modal-overlay">
    <div class="modal-content email-viewer" @click.stop>
      <div class="modal-header">
        <h3>{{ $t('emailViewer.title') }} - {{ email }}</h3>
        <button @click="$emit('close')" class="modal-close">×</button>
      </div>

      <div class="modal-body">
        <!-- 文件夹选择和控制 -->
        <div class="controls-section">
          <div class="folder-selector">
            <label>{{ $t('emailViewer.folder') }}:</label>
            <select v-model="selectedFolder" @change="loadEmails" class="folder-select">
              <option value="inbox">{{ $t('emailViewer.inbox') }}</option>
              <option value="junk">{{ $t('emailViewer.junk') }}</option>
            </select>
          </div>

          <div class="page-controls">
            <button
              @click="previousPage"
              :disabled="currentPage <= 1 || isLoading"
              class="btn primary small"
            >
              {{ $t('emailViewer.previousPage') }}
            </button>
            <span class="page-info">
              {{ $t('emailViewer.pageInfo', { current: currentPage, total: totalPages }) }}
            </span>
            <button
              @click="nextPage"
              :disabled="currentPage >= totalPages || isLoading"
              class="btn primary small"
            >
              {{ $t('emailViewer.nextPage') }}
            </button>
          </div>

          <button
            @click="refreshEmails"
            :disabled="isLoading"
            class="btn primary small"
          >
            {{ isLoading ? $t('emailViewer.loading') : $t('emailViewer.reload') }}
          </button>

        </div>

        <!-- 邮件列表 -->
        <div class="emails-section">
          <div v-if="isLoading" class="loading-state">
            <div class="spinner"></div>
            <p>{{ $t('emailViewer.loading') }}</p>
          </div>

          <div v-else-if="emails.length === 0" class="empty-state">
            <p>{{ $t('emailViewer.noEmails') }}</p>
            <p class="empty-hint">{{ $t('emailViewer.noEmails') }}</p>
          </div>

          <div v-else class="emails-list">
            <div
              v-for="emailItem in emails"
              :key="emailItem.message_id"
              class="email-item"
              @click="selectEmail(emailItem)"
              :class="{ selected: selectedEmailId === emailItem.message_id }"
            >
              <div class="email-sender">
                <div class="sender-avatar">{{ emailItem.sender_initial }}</div>
                <div class="sender-info">
                  <div class="sender-name">{{ emailItem.from_email }}</div>
                  <div class="email-date">{{ formatDate(emailItem.date) }}</div>
                </div>
              </div>
              <div class="email-content">
                <div class="email-subject">{{ emailItem.subject }}</div>
                <div class="email-folder">{{ emailItem.folder }}</div>
              </div>
              <div class="email-actions">
                <button
                  @click.stop="viewEmailDetails(emailItem)"
                  class="btn primary small"
                >
                  {{ $t('emailViewer.viewDetails') }}
                </button>
              </div>
              <div class="email-status">
                <span v-if="!emailItem.is_read" class="unread-indicator">●</span>
                <span v-if="emailItem.has_attachments" class="attachment-indicator">📎</span>
              </div>
            </div>
          </div>

          <!-- 分页信息 -->
          <div v-if="emails.length > 0" class="pagination-info">
            <p>
              {{ $t('emailViewer.totalEmails', { count: totalEmails }) }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 邮件详情查看器 -->
  <EmailDetails
    v-if="showEmailDetails"
    :email="email"
    :message-id="selectedEmailForDetails"
    @close="showEmailDetails = false"
  />
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import EmailDetails from './EmailDetails.vue'

const props = defineProps({
  email: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

// i18n
const { t } = useI18n()

// 响应式数据
const emails = ref([])
const selectedFolder = ref('inbox')
const currentPage = ref(1)
const pageSize = ref(20)
const totalEmails = ref(0)
const isLoading = ref(false)
const selectedEmailId = ref('')
const showEmailDetails = ref(false)
const selectedEmailForDetails = ref('')

// 计算属性
const totalPages = computed(() => {
  return Math.ceil(totalEmails.value / pageSize.value)
})

// 方法
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const loadEmails = async () => {
  isLoading.value = true
  try {
    const response = await invoke('outlook_get_emails', {
      email: props.email,
      folder: selectedFolder.value,
      page: currentPage.value,
      pageSize: pageSize.value
    })
    
    emails.value = response.emails
    totalEmails.value = response.total_emails
    
    if (emails.value.length === 0 && currentPage.value > 1) {
      // 如果当前页没有邮件且不是第一页，回到第一页
      currentPage.value = 1
      await loadEmails()
    }
  } catch (error) {
    showStatus(`加载邮件失败: ${error}`, 'error')
    emails.value = []
    totalEmails.value = 0
  } finally {
    isLoading.value = false
  }
}

const refreshEmails = async () => {
  currentPage.value = 1
  await loadEmails()
  showStatus('邮件列表已重新加载', 'success')
}



const previousPage = async () => {
  if (currentPage.value > 1) {
    currentPage.value--
    await loadEmails()
  }
}

const nextPage = async () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
    await loadEmails()
  }
}

const selectEmail = (emailItem) => {
  selectedEmailId.value = emailItem.message_id
  showStatus(`选中邮件: ${emailItem.subject}`, 'info')
}

const viewEmailDetails = (emailItem) => {
  selectedEmailForDetails.value = emailItem.message_id
  showEmailDetails.value = true
}

const formatDate = (dateString) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return dateString
  }
}

// 生命周期
onMounted(() => {
  loadEmails()
})
</script>

<style scoped>
/* ============================================
   EmailViewer - Modern Tech Style
   ============================================ */

.email-viewer {
  width: 95vw;
  max-width: 1020px;
  max-height: 90vh;
}

/* 控制区域 - 科技风 */
.controls-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 22px;
  padding: 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  flex-wrap: wrap;
  gap: 14px;
}

.folder-selector {
  display: flex;
  align-items: center;
  gap: 10px;
}

.folder-selector label {
  font-weight: 600;
  color: var(--text);
}

/* 下拉选择 - 科技风 */
.folder-select {
  padding: 8px 14px;
  border: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--text);
  border-radius: 10px;
  font-size: 14px;
  transition: all 0.2s ease;
}

.folder-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 15%, transparent),
              0 0 12px var(--tech-glow-primary);
}

.page-controls {
  display: flex;
  align-items: center;
  gap: 14px;
}

.page-info {
  font-size: 14px;
  color: var(--text-muted);
  white-space: nowrap;
  font-family: var(--tech-mono-font);
}

.emails-section {
  min-height: 400px;
}

.emails-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* 邮件项 - 科技风 */
.email-item {
  display: flex;
  align-items: center;
  padding: 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.email-item:hover {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  transform: translateX(4px);
}

.email-item.selected {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.email-sender {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 200px;
}

/* 头像 - 科技风渐变 */
.sender-avatar {
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 16px;
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.sender-info {
  flex: 1;
  min-width: 0;
}

.sender-name {
  font-weight: 600;
  color: var(--text-strong);
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.email-date {
  font-size: 12px;
  color: var(--text-muted);
  font-family: var(--tech-mono-font);
}

.email-content {
  flex: 1;
  margin: 0 18px;
  min-width: 0;
}

.email-actions {
  margin-right: 14px;
}

.email-subject {
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 文件夹标签 - 科技风 */
.email-folder {
  font-size: 11px;
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
  padding: 3px 10px;
  border-radius: 20px;
  display: inline-block;
  font-weight: 600;
}

.email-status {
  display: flex;
  align-items: center;
  gap: 10px;
}

.unread-indicator {
  color: var(--accent);
  font-size: 12px;
  filter: drop-shadow(0 0 4px var(--tech-glow-primary));
}

.attachment-indicator {
  font-size: 14px;
  color: var(--text-muted);
}

/* 分页信息 - 科技风 */
.pagination-info {
  text-align: center;
  margin-top: 22px;
  padding: 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
}

.pagination-info p {
  margin: 0;
  color: var(--text-muted);
  font-size: 14px;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 65px 22px;
  color: var(--text-muted);
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 10px;
  opacity: 0.7;
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

@media (max-width: 768px) {
  .controls-section {
    flex-direction: column;
    align-items: stretch;
  }

  .email-item {
    flex-direction: column;
    align-items: stretch;
    gap: 14px;
  }

  .email-sender {
    min-width: auto;
  }

  .email-content {
    margin: 0;
  }
}
</style>
