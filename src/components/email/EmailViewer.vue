<template>
  <div class="modal-overlay">
    <div class="modal-content email-viewer" @click.stop>
      <div class="modal-header">
        <h3>{{ $t('emailViewer.title') }} - {{ email }}</h3>
        <button @click="$emit('close')" class="close-btn">×</button>
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
              class="btn secondary small"
            >
              {{ $t('emailViewer.previousPage') }}
            </button>
            <span class="page-info">
              {{ $t('emailViewer.pageInfo', { current: currentPage, total: totalPages }) }}
            </span>
            <button
              @click="nextPage"
              :disabled="currentPage >= totalPages || isLoading"
              class="btn secondary small"
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
  z-index: 2000;
}

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 90vw;
  max-height: 95vh;
  overflow-y: auto;
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

.email-viewer {
  width: 95vw;
  max-width: 1000px;
  max-height: 90vh;
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
}

.controls-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 16px;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 8px;
  flex-wrap: wrap;
  gap: 12px;
}

.folder-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.folder-selector label {
  font-weight: 500;
  color: var(--color-text-primary, #374151);
}

.folder-select {
  padding: 6px 12px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 4px;
  font-size: 14px;
}

.page-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-info {
  font-size: 14px;
  color: var(--color-text-muted, #6b7280);
  white-space: nowrap;
}

.emails-section {
  min-height: 400px;
}

.emails-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.email-item {
  display: flex;
  align-items: center;
  padding: 16px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.email-item:hover {
  background: var(--color-surface-alt, #f9fafb);
  border-color: var(--color-border-strong, #d1d5db);
}

.email-item.selected {
  background: var(--color-accent-muted, #eff6ff);
  border-color: var(--color-accent, #3b82f6);
}

.email-sender {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 200px;
}

.sender-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--color-accent, #3b82f6);
  color: var(--color-text-inverse, #ffffff);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 16px;
}

.sender-info {
  flex: 1;
  min-width: 0;
}

.sender-name {
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.email-date {
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
}

.email-content {
  flex: 1;
  margin: 0 16px;
  min-width: 0;
}

.email-actions {
  margin-right: 12px;
}

.email-subject {
  font-weight: 500;
  color: var(--color-text-strong, #111827);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.email-folder {
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
  background: var(--color-surface-hover, #f3f4f6);
  padding: 2px 8px;
  border-radius: 12px;
  display: inline-block;
}

.email-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.unread-indicator {
  color: var(--color-accent, #3b82f6);
  font-size: 12px;
}

.attachment-indicator {
  font-size: 14px;
}

.pagination-info {
  text-align: center;
  margin-top: 20px;
  padding: 16px;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 8px;
}

.pagination-info p {
  margin: 0;
  color: var(--color-text-muted, #6b7280);
  font-size: 14px;
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
  background: var(--color-accent, #3b82f6);
  color: var(--color-text-inverse, #ffffff);
}

.btn.primary:hover:not(:disabled) {
  background: var(--color-accent-hover, #2563eb);
}

.btn.secondary {
  background: var(--color-text-muted, #6b7280);
  color: var(--color-text-inverse, #ffffff);
}

.btn.secondary:hover:not(:disabled) {
  background: var(--color-text-secondary, #4b5563);
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--color-text-muted, #6b7280);
}

.empty-hint {
  font-size: 12px;
  color: var(--color-text-soft, #9ca3af);
  margin-top: 8px;
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

@media (max-width: 768px) {
  .controls-section {
    flex-direction: column;
    align-items: stretch;
  }
  
  .email-item {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }
  
  .email-sender {
    min-width: auto;
  }
  
  .email-content {
    margin: 0;
  }
}
</style>
