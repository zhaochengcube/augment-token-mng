<template>
  <div class="bookmark-manager-modal">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>{{ $t('bookmarkManager.title') }}</h2>
          <div class="header-actions">
            <button @click="openDataFolder" class="btn-icon info" :title="$t('bookmarkManager.openDataFolder')">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z"/>
              </svg>
            </button>
            <button @click="showAddForm()" class="btn-icon add" :title="$t('bookmarkManager.addBookmark')">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
            </button>
            <button class="close-btn" @click="$emit('close')">×</button>
          </div>
        </div>
        
        <div class="modal-body">

          <div class="bookmarks-grid">
            <div
              v-for="bookmark in allBookmarks"
              :key="bookmark.id"
              class="bookmark-card"
            >
              <div class="bookmark-actions">
                <button @click="editBookmark(bookmark)" class="btn-icon edit" :title="$t('bookmarkManager.editBookmark')">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
                  </svg>
                </button>
                <button @click="deleteBookmark(bookmark.id)" class="btn-icon delete" :title="$t('bookmarkManager.deleteBookmark')">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                  </svg>
                </button>
              </div>
              <div class="bookmark-content">
                <div class="bookmark-icon">
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z"/>
                  </svg>
                </div>
                <div class="bookmark-name">{{ bookmark.name }}</div>
                <div v-if="bookmark.description" class="bookmark-desc">{{ bookmark.description }}</div>
                <div class="bookmark-buttons">
                  <button
                    @click="handleBookmarkAction(bookmark)"
                    class="bookmark-open-btn"
                    title="打开书签"
                  >
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
                    </svg>
                    打开
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div v-if="allBookmarks.length === 0" class="empty-state">
            <p>{{ $t('bookmarkManager.emptyState') }}</p>
            <p>{{ $t('bookmarkManager.emptyDescription') }}</p>
          </div>
        </div>

        <!-- Add/Edit Form Modal -->
        <div v-if="showForm" class="form-overlay">
          <div class="form-content" @click.stop>
            <div class="form-header">
              <h3>{{ editingBookmark ? $t('bookmarkManager.editBookmark') : $t('bookmarkManager.addBookmark') }}</h3>
              <button class="close-btn" @click="hideForm">×</button>
            </div>
            
            <div class="form-body">
              <div class="form-group">
                <label>{{ $t('bookmarkManager.form.name') }} *</label>
                <input
                  v-model="formData.name"
                  type="text"
                  :placeholder="$t('bookmarkManager.form.name')"
                  required
                >
              </div>

              <div class="form-group">
                <label>{{ $t('bookmarkManager.form.url') }} *</label>
                <input
                  v-model="formData.url"
                  type="url"
                  placeholder="https://example.com"
                  required
                >
              </div>

              <div class="form-group">
                <label>{{ $t('bookmarkManager.form.description') }}</label>
                <textarea
                  v-model="formData.description"
                  :placeholder="$t('bookmarkManager.form.description')"
                  rows="2"
                ></textarea>
              </div>

              <div class="form-actions">
                <button @click="hideForm" class="btn secondary">{{ $t('bookmarkManager.form.cancel') }}</button>
                <button @click="saveBookmark" class="btn primary" :disabled="!canSave">
                  {{ editingBookmark ? $t('bookmarkManager.form.save') : $t('bookmarkManager.form.save') }}
                </button>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>

    <!-- 书签链接对话框 -->
    <ExternalLinkDialog
      :show="showBookmarkDialog"
      :title="$t('bookmarkManager.dialog.selectOpenMethod')"
      :url="currentBookmark?.url || ''"
      :browser-title="currentBookmark?.name || ''"
      @close="showBookmarkDialog = false"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ExternalLinkDialog from './ExternalLinkDialog.vue'

// Emits
const emit = defineEmits(['close'])

// i18n
const { t } = useI18n()

// Reactive data
const allBookmarks = ref([])
const showForm = ref(false)
const editingBookmark = ref(null)

// Bookmark dialog
const showBookmarkDialog = ref(false)
const currentBookmark = ref(null)

const formData = ref({
  name: '',
  url: '',
  description: ''
})

// Computed properties
const canSave = computed(() => {
  return formData.value.name.trim() && formData.value.url.trim()
})

// Methods
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const loadBookmarks = async () => {
  try {
    const result = await invoke('get_all_bookmarks')
    allBookmarks.value = result || []
  } catch (error) {
    showStatus(`加载书签失败: ${error}`, 'error')
  }
}

const showAddForm = () => {
  editingBookmark.value = null
  formData.value = {
    name: '',
    url: '',
    description: ''
  }
  showForm.value = true
}

const editBookmark = (bookmark) => {
  editingBookmark.value = bookmark
  formData.value = {
    name: bookmark.name,
    url: bookmark.url,
    description: bookmark.description || ''
  }
  showForm.value = true
}

const hideForm = () => {
  showForm.value = false
  editingBookmark.value = null
  formData.value = {
    name: '',
    url: '',
    description: ''
  }
}

const saveBookmark = async () => {
  if (!canSave.value) return

  try {
    const bookmarkData = {
      name: formData.value.name.trim(),
      url: formData.value.url.trim(),
      description: formData.value.description.trim(),
      category: 'bookmark' // 统一使用bookmark类别
    }

    if (editingBookmark.value) {
      await invoke('update_bookmark', {
        id: editingBookmark.value.id,
        ...bookmarkData
      })
      showStatus(t('bookmarkManager.messages.updateSuccess'), 'success')
    } else {
      await invoke('add_bookmark', bookmarkData)
      showStatus(t('bookmarkManager.messages.addSuccess'), 'success')
    }

    await loadBookmarks()
    hideForm()
  } catch (error) {
    showStatus(`保存书签失败: ${error}`, 'error')
  }
}

const deleteBookmark = async (id) => {
  if (!confirm('确定要删除这个书签吗？')) return

  try {
    await invoke('delete_bookmark', { id })
    await loadBookmarks()
    showStatus(t('bookmarkManager.messages.deleteSuccess'), 'success')
  } catch (error) {
    showStatus(`${t('bookmarkManager.messages.deleteSuccess')}: ${error}`, 'error')
  }
}

// 书签对话框相关方法
const handleBookmarkAction = (bookmark) => {
  currentBookmark.value = bookmark
  showBookmarkDialog.value = true
}


const openDataFolder = async () => {
  try {
    await invoke('open_data_folder')
    // 静默执行，不显示状态提示
  } catch (error) {
    showStatus(`${t('bookmarkManager.messages.openFolderFailed')}: ${error}`, 'error')
  }
}

// Initialize
onMounted(() => {
  loadBookmarks()
})
</script>

<style scoped>
/* 外层容器样式 */
.bookmark-manager-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2000;
}

/* 隐藏表单弹窗的滚动条 */
.form-overlay * {
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE and Edge */
}

.form-overlay *::-webkit-scrollbar {
  display: none;
}

.modal-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-content {
  background: var(--bg-surface);
  border-radius: 12px;
  width: 100%;
  max-width: 900px;
  height: 90vh;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.modal-header h2 {
  margin: 0;
  color: var(--text-strong);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--text-muted);
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: var(--text-strong);
}

.btn-icon.info {
  background: var(--bg-muted);
  color: var(--accent);
}

.btn-icon.info:hover {
  background: var(--bg-hover);
  color: var(--accent-strong);
}

.btn-icon.add {
  background: var(--bg-muted);
  color: #28a745;
}

.btn-icon.add:hover {
  background: var(--bg-hover);
  color: #1e7e34;
}

.modal-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}



.bookmarks-grid {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 16px;
}

.bookmark-card {
  position: relative;
  aspect-ratio: 1;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--bg-surface);
  transition: all 0.2s;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.bookmark-card:hover {
  border-color: var(--accent);
  box-shadow: 0 4px 12px color-mix(in srgb, var(--accent) 15%, transparent);
  transform: translateY(-2px);
}

.bookmark-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
  z-index: 2;
}

.bookmark-card:hover .bookmark-actions {
  opacity: 1;
}

.bookmark-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px 12px;
  text-align: center;
}

.bookmark-icon {
  margin-bottom: 8px;
  color: var(--accent);
  opacity: 0.8;
}

.bookmark-name {
  font-weight: 600;
  color: var(--text-strong);
  font-size: 13px;
  line-height: 1.3;
  margin-bottom: 4px;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.bookmark-desc {
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.bookmark-buttons {
  display: flex;
  gap: 6px;
  justify-content: center;
  margin-top: 8px;
}

.bookmark-open-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--accent);
  color: var(--text-contrast);
  min-width: 60px;
}

.bookmark-open-btn:hover {
  background: var(--accent-strong);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--accent) 30%, transparent);
}

.btn-small {
  padding: 4px 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: all 0.2s;
}

.btn-small.primary {
  background: var(--accent);
  color: var(--text-contrast);
}

.btn-small.primary:hover {
  background: var(--accent-strong);
}

.btn-small.secondary {
  background: var(--text-muted);
  color: var(--text-contrast);
}

.btn-small.secondary:hover {
  background: color-mix(in srgb, var(--text-muted) 80%, black);
}

.btn-icon {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  backdrop-filter: blur(10px);
}

.btn-icon.edit {
  background: color-mix(in srgb, var(--bg-surface) 90%, transparent);
  color: var(--text-muted);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.btn-icon.edit:hover {
  background: var(--bg-hover);
  color: var(--text);
  transform: scale(1.1);
}

.btn-icon.delete {
  background: color-mix(in srgb, var(--bg-surface) 90%, transparent);
  color: #dc3545;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.btn-icon.delete:hover {
  background: rgba(245, 198, 203, 0.95);
  color: #721c24;
  transform: scale(1.1);
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
}

.empty-state p {
  margin: 8px 0;
  line-height: 1.5;
}

.empty-state p:first-child {
  font-size: 18px;
  font-weight: 500;
  color: var(--text-strong);
}

/* Form Modal Styles */
.form-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}

.form-content {
  background: var(--bg-surface);
  border-radius: 8px;
  width: 90%;
  max-width: 500px;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.form-header h3 {
  margin: 0;
  color: var(--text-strong);
  font-size: 18px;
}

.form-body {
  padding: 20px;
  flex: 1;
  min-height: 0;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--text);
  font-size: 14px;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 14px;
  transition: border-color 0.2s;
  box-sizing: border-box;
  background: var(--bg-surface);
  color: var(--text);
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 10%, transparent);
}

.form-group textarea {
  resize: vertical;
  font-family: inherit;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.form-actions .btn {
  flex: 1;
  justify-content: center;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn.primary {
  background: var(--accent);
  color: var(--text-contrast);
}

.btn.primary:hover:not(:disabled) {
  background: var(--accent-strong);
}

.btn.secondary {
  background: var(--text-muted);
  color: var(--text-contrast);
}

.btn.secondary:hover {
  background: color-mix(in srgb, var(--text-muted) 80%, black);
}

.btn.small {
  padding: 8px 12px;
  font-size: 13px;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.status {
  padding: 12px 20px;
  margin: 0 20px 20px;
  border-radius: 4px;
  font-size: 14px;
}

.status.info {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.status.success {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.status.error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

/* Responsive Design */
@media (max-width: 768px) {
  .modal-content {
    margin: 10px;
    max-width: calc(100vw - 20px);
    height: calc(100vh - 20px);
  }

  .form-content {
    width: 95%;
  }

  .bookmarks-grid {
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 12px;
    padding: 16px;
  }

  .bookmark-card {
    border-radius: 8px;
  }

  .bookmark-content {
    padding: 12px 8px;
  }

  .bookmark-name {
    font-size: 12px;
  }

  .bookmark-desc {
    font-size: 10px;
  }

  .form-actions {
    flex-direction: row;
  }

  .form-actions .btn {
    flex: 1;
  }
}

@media (max-width: 480px) {
  .modal-overlay {
    padding: 10px;
  }

  .modal-content {
    max-height: 95vh;
  }

  .modal-header h2 {
    font-size: 1.25rem;
  }
}

/* Portal Dialog Styles */
.portal-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}

.portal-dialog {
  background: var(--bg-surface);
  border-radius: 12px;
  padding: 24px;
  min-width: 320px;
  max-width: 400px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  text-align: center;
}

.portal-dialog h3 {
  margin: 0 0 20px 0;
  color: var(--text-strong);
  font-size: 18px;
  font-weight: 600;
}

.dialog-buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 12px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-decoration: none;
}

.dialog-btn.copy {
  background: #28a745;
  color: #ffffff;
}

.dialog-btn.copy:hover {
  background: #218838;
  transform: translateY(-1px);
}

.dialog-btn.external {
  background: var(--accent);
  color: var(--text-contrast);
}

.dialog-btn.external:hover {
  background: var(--accent-strong);
  transform: translateY(-1px);
}

.dialog-btn.internal {
  background: var(--text-muted);
  color: var(--text-contrast);
}

.dialog-btn.internal:hover {
  background: color-mix(in srgb, var(--text-muted) 80%, black);
  transform: translateY(-1px);
}

.dialog-btn.cancel {
  background: var(--bg-muted);
  color: var(--text-muted);
  border: 1px solid var(--border);
}

.dialog-btn.cancel:hover {
  background: var(--bg-hover);
  color: var(--text);
}
</style>
