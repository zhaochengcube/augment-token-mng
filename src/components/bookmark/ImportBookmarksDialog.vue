<template>
  <BaseModal
    :visible="true"
    :title="$t('bookmarks.import.title')"
    :show-close="true"
    :close-on-overlay="!isImporting"
    :close-on-esc="!isImporting"
    :body-scroll="false"
    modal-class="max-w-[600px]"
    @close="handleClose"
  >
    <div class="animate-fade-in">
      <!-- Info -->
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-accent/20 bg-accent/10 p-4">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <div class="flex-1">
          <p class="text-[13px] leading-relaxed text-text-secondary">
            {{ $t('bookmarks.import.info') }}
          </p>
          <p class="mt-1.5 text-[12px] leading-relaxed text-text-muted">
            {{ $t('bookmarks.import.infoDetail', { example: '' }) }}<code class="rounded bg-muted px-1 py-0.5 text-[11px]">开发工具/前端</code>
          </p>
        </div>
      </div>

      <!-- 隐藏的文件输入 -->
      <input
        ref="fileInputRef"
        type="file"
        accept=".html,.htm"
        class="hidden"
        @change="handleFileChange"
      />

      <!-- 文件选择 -->
      <div v-if="!previewData" class="form-group">
        <label class="label">{{ $t('bookmarks.import.selectFile') }}</label>
        <div
          class="flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed border-border p-8 cursor-pointer hover:border-accent hover:bg-accent/5 transition-colors"
          @click="selectFile"
        >
          <svg class="h-10 w-10 text-text-muted" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/>
          </svg>
          <span class="text-sm text-text-secondary">{{ $t('bookmarks.import.clickToSelect') }}</span>
          <span class="text-xs text-text-muted">{{ $t('bookmarks.import.supportFormat') }}</span>
        </div>
      </div>

      <!-- 预览数据 -->
      <div v-else class="space-y-4">
        <!-- 文件信息 -->
        <div class="flex items-center justify-between rounded-lg bg-muted p-3">
          <div class="flex items-center gap-2">
            <svg class="h-5 w-5 text-accent" viewBox="0 0 24 24" fill="currentColor">
              <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
            </svg>
            <span class="text-sm text-text">{{ fileName }}</span>
          </div>
          <button
            @click="clearFile"
            class="text-xs text-text-muted hover:text-danger transition-colors"
          >
            {{ $t('bookmarks.import.reselect') }}
          </button>
        </div>

        <!-- 书签预览列表 -->
        <div class="rounded-lg border border-border">
          <div class="flex items-center justify-between border-b border-border px-4 py-2 bg-muted">
            <span class="text-sm font-medium text-text">
              {{ $t('bookmarks.import.parsedCount', { count: previewData.length }) }}
            </span>
            <span v-if="tagCount > 0" class="text-xs text-text-muted">
              {{ $t('bookmarks.import.tagCount', { count: tagCount }) }}
            </span>
          </div>
          <div class="max-h-[300px] overflow-y-auto">
            <div
              v-for="(item, index) in previewData.slice(0, 100)"
              :key="index"
              class="flex items-center justify-between px-4 py-2.5 border-b border-border last:border-b-0"
            >
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <span class="text-sm text-text truncate">{{ item.name }}</span>
                <span
                  v-if="item.tag"
                  class="shrink-0 rounded bg-accent/10 px-1.5 py-0.5 text-[10px] text-accent truncate max-w-[150px]"
                >
                  {{ item.tag }}
                </span>
              </div>
              <span class="shrink-0 text-xs text-text-muted truncate max-w-[200px] ml-2">
                {{ item.url }}
              </span>
            </div>
            <div v-if="previewData.length > 100" class="px-4 py-2.5 text-center text-xs text-text-muted">
              {{ $t('bookmarks.import.moreHidden', { count: previewData.length - 100 }) }}
            </div>
          </div>
        </div>

        <!-- 导入结果 -->
        <div v-if="importResult" class="rounded-lg border border-border p-4">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-text">{{ $t('bookmarks.import.resultTitle') }}</span>
            <span class="text-xs text-text-muted">
              {{ $t('bookmarks.import.resultSummary', { success: importResult.success_count, skipped: importResult.skipped_count, failed: importResult.failed_count }) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Error Message -->
      <div v-if="error" class="mt-4 flex items-center gap-2 rounded-lg border border-danger/30 bg-danger/10 p-3 text-[13px] text-danger">
        <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        {{ error }}
      </div>
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary" :disabled="isImporting">
        {{ importResult ? $t('common.close') : $t('common.cancel') }}
      </button>
      <button
        v-if="previewData && !importResult"
        @click="handleImport"
        class="btn btn--primary"
        :disabled="isImporting || !previewData.length"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isImporting ? 'hidden' : 'visible' }">
            {{ $t('bookmarks.import.importBtn', { count: previewData.length }) }}
          </span>
          <span v-if="isImporting" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'imported'])

const fileName = ref('')
const fileContent = ref('')
const previewData = ref(null)
const importResult = ref(null)
const isImporting = ref(false)
const error = ref('')
const fileInputRef = ref(null)

const tagCount = computed(() => {
  if (!previewData.value) return 0
  const tags = new Set(previewData.value.filter(b => b.tag).map(b => b.tag))
  return tags.size
})

const handleClose = () => {
  if (isImporting.value) return
  emit('close')
}

const selectFile = () => {
  fileInputRef.value?.click()
}

// 前端预解析：简单提取书签信息用于预览
const preParseBookmarks = (content) => {
  const bookmarks = []
  const folderStack = []
  let pendingFolder = null

  const folderRe = /<H3[^>]*>(.*?)<\/H3>/gi
  const bookmarkRe = /<A\s[^>]*HREF="([^"]*)"[^>]*>(.*?)<\/A>/gi
  const dlOpenRe = /<DL>/gi
  const dlCloseRe = /<\/DL>/gi

  for (const line of content.split('\n')) {
    const trimmed = line.trim()

    // 检测文件夹
    const folderMatch = /<H3[^>]*>(.*?)<\/H3>/i.exec(trimmed)
    if (folderMatch) {
      pendingFolder = folderMatch[1].replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>')
    }

    // 进入文件夹
    if (/<DL>/i.test(trimmed)) {
      if (pendingFolder !== null) {
        folderStack.push(pendingFolder)
        pendingFolder = null
      }
    }

    // 离开文件夹
    if (/<\/DL>/i.test(trimmed)) {
      folderStack.pop()
    }

    // 检测书签
    const bmMatch = /<A\s[^>]*HREF="([^"]*)"[^>]*>(.*?)<\/A>/i.exec(trimmed)
    if (bmMatch) {
      const url = bmMatch[1].replace(/&amp;/g, '&')
      const name = bmMatch[2].replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>')

      if (!url || url.startsWith('javascript:')) continue

      // 跳过第一层文件夹（书签栏、其他书签等浏览器固定根文件夹）
      bookmarks.push({
        name,
        url,
        tag: folderStack.length > 1 ? folderStack.slice(1).join('/') : null
      })
    }
  }

  return bookmarks
}

const handleFileChange = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  try {
    const content = await file.text()

    // 前端预解析用于预览
    const parsed = preParseBookmarks(content)

    if (parsed.length === 0) {
      error.value = $t('bookmarks.import.emptyFile')
      return
    }

    fileName.value = file.name
    fileContent.value = content
    previewData.value = parsed
    error.value = ''
  } catch (err) {
    console.error('Failed to read file:', err)
    error.value = err?.message || $t('bookmarks.import.readError')
  } finally {
    event.target.value = ''
  }
}

const clearFile = () => {
  fileName.value = ''
  fileContent.value = ''
  previewData.value = null
  importResult.value = null
  error.value = ''
}

const handleImport = async () => {
  if (isImporting.value || !fileContent.value) return

  error.value = ''
  isImporting.value = true

  try {
    const result = await invoke('bookmark_import_from_browser', {
      content: fileContent.value
    })

    importResult.value = result

    if (result.success_count > 0) {
      emit('imported', result)
    }
  } catch (err) {
    console.error('Import error:', err)
    error.value = err?.message || err || $t('bookmarks.import.importFailed')
  } finally {
    isImporting.value = false
  }
}
</script>

<style scoped>
.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
