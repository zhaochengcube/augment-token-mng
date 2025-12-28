<template>
  <div class="bookmark-page">
    <div class="page-header">
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
      </div>
    </div>

    <div class="page-body">
      <div class="bookmarks-grid">
        <div
          v-for="bookmark in allBookmarks"
          :key="bookmark.id"
          class="bookmark-card"
          @click="handleBookmarkAction(bookmark)"
        >
          <div class="bookmark-actions" @click.stop>
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
          </div>
        </div>
      </div>

      <div v-if="allBookmarks.length === 0" class="empty-state">
        <p>{{ $t('bookmarkManager.emptyState') }}</p>
        <p>{{ $t('bookmarkManager.emptyDescription') }}</p>
      </div>
    </div>

    <!-- Add/Edit Form Modal -->
    <div v-if="showForm" class="form-overlay" @click="hideForm">
      <div class="form-content" @click.stop>
        <div class="form-header">
          <h3>{{ editingBookmark ? $t('bookmarkManager.editBookmark') : $t('bookmarkManager.addBookmark') }}</h3>
          <button class="modal-close" @click="hideForm">×</button>
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
              rows="3"
            ></textarea>
          </div>
        </div>

        <div class="form-footer">
          <button @click="saveBookmark" class="btn primary" :disabled="!canSave">
            {{ editingBookmark ? $t('bookmarkManager.form.update') : $t('bookmarkManager.form.save') }}
          </button>
          <button @click="hideForm" class="btn secondary">{{ $t('bookmarkManager.form.cancel') }}</button>
        </div>
      </div>
    </div>

    <!-- Bookmark Dialog -->
    <ExternalLinkDialog
      :show="showBookmarkDialog"
      :title="currentBookmark?.name || ''"
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
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'

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
      category: 'bookmark'
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

const handleBookmarkAction = (bookmark) => {
  currentBookmark.value = bookmark
  showBookmarkDialog.value = true
}

const openDataFolder = async () => {
  try {
    await invoke('open_data_folder')
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
/* ============================================
   BookmarkPage - Modern Tech Style
   ============================================ */

.bookmark-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 26px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 26px;
  flex-shrink: 0;
}

.page-header h2 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-strong);
  margin: 0;
  letter-spacing: -0.5px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.page-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.bookmarks-grid {
  flex: 1;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 14px;
  padding: 4px;
}

/* 书签卡片 - 科技风 */
.bookmark-card {
  position: relative;
  aspect-ratio: 1;
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  background: var(--tech-card-bg);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 顶部发光边线 */
.bookmark-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 25%;
  right: 25%;
  height: 2px;
  background: var(--accent);
  opacity: 0;
  transition: all 0.3s ease;
  border-radius: 0 0 2px 2px;
}

.bookmark-card:hover::before {
  opacity: 1;
  left: 15%;
  right: 15%;
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.bookmark-card:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 30%, transparent),
              0 6px 24px -6px rgba(0, 0, 0, 0.2),
              0 0 25px -8px var(--tech-glow-primary);
  transform: translateY(-3px);
}

.bookmark-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.2s;
  z-index: 10;
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
  padding: 14px;
  text-align: center;
}

.bookmark-icon {
  color: var(--accent);
  margin-bottom: 8px;
  transition: transform 0.3s ease;
}

.bookmark-card:hover .bookmark-icon {
  transform: scale(1.1);
}

.bookmark-icon svg {
  width: 22px;
  height: 22px;
  filter: drop-shadow(0 0 6px var(--tech-glow-primary));
}

.bookmark-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.bookmark-desc {
  font-size: 11px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-muted);
}

.empty-state p {
  margin: 8px 0;
}

/* Form Overlay - 科技风 */
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
  z-index: 3000;
  padding: 20px;
  backdrop-filter: blur(4px);
}

.form-content {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 18px;
  width: 100%;
  max-width: 520px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
  display: flex;
  flex-direction: column;
  max-height: 90vh;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 22px 26px;
  border-bottom: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-radius: 18px 18px 0 0;
}

.form-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
}


.form-body {
  flex: 1;
  overflow-y: auto;
  padding: 26px;
}

.form-footer {
  display: flex;
  justify-content: flex-end;
  gap: 14px;
  padding: 22px 26px;
  border-top: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-radius: 0 0 18px 18px;
}

</style>

