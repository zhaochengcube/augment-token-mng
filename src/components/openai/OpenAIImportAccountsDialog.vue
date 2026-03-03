<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.openai.importDialog.title')"
    :show-close="true"
    :close-on-overlay="!isImporting"
    :close-on-esc="!isImporting"
    :body-scroll="true"
    modal-class="max-w-[600px]"
    @close="handleClose"
  >
    <div class="animate-fade-in">
      <!-- Info -->
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-accent/20 bg-accent/10 p-4">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <p class="text-[13px] leading-relaxed text-text-secondary">{{ $t('platform.openai.importDialog.info') }}</p>
        <button
          @click="showFormatModal = true"
          class="shrink-0 rounded p-1 text-text-muted hover:bg-accent/20 hover:text-accent transition-colors"
          v-tooltip="$t('platform.openai.importDialog.formatExample')"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm2.07-7.75l-.9.92C13.45 12.9 13 13.5 13 15h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H8c0-2.21 1.79-4 4-4s4 1.79 4 4c0 .88-.36 1.68-.93 2.25z"/>
          </svg>
        </button>
      </div>

      <!-- 隐藏的文件输入 -->
      <input
        ref="fileInputRef"
        type="file"
        accept=".json"
        class="hidden"
        @change="handleFileChange"
      />

      <!-- 文件选择 -->
      <div v-if="!previewTokens.length" class="form-group">
        <label class="label">{{ $t('platform.openai.importDialog.selectFile') }}</label>
        <div
          class="flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed border-border p-8 cursor-pointer hover:border-accent hover:bg-accent/5 transition-colors"
          @click="selectFile"
        >
          <svg class="h-10 w-10 text-text-muted" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/>
          </svg>
          <span class="text-sm text-text-secondary">{{ $t('platform.openai.importDialog.clickToSelect') }}</span>
          <span class="text-xs text-text-muted">{{ $t('platform.openai.importDialog.supportFormat') }}</span>
        </div>
      </div>

      <!-- 预览数据 -->
      <div v-else class="space-y-4">
        <!-- 文件信息 -->
        <div class="flex items-center justify-between rounded-lg bg-surface-alt p-3">
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
            {{ $t('platform.openai.importDialog.reselect') }}
          </button>
        </div>

        <!-- 预览列表 -->
        <div class="rounded-lg border border-border">
          <div class="flex items-center justify-between border-b border-border px-4 py-2 bg-surface-alt">
            <span class="text-sm font-medium text-text">
              {{ $t('platform.openai.importDialog.previewTitle', { count: previewTokens.length }) }}
            </span>
          </div>
          <div class="max-h-[300px] overflow-y-auto">
            <div
              v-for="(token, index) in previewTokens"
              :key="index"
              class="flex items-center gap-2 px-4 py-2.5 border-b border-border last:border-b-0"
            >
              <span class="text-xs text-text-muted shrink-0">#{{ index + 1 }}</span>
              <span class="text-sm text-text truncate font-mono">{{ maskToken(token) }}</span>
            </div>
          </div>
        </div>

        <!-- 导入进度 -->
        <div v-if="importResult" class="rounded-lg border border-border p-4">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium text-text">{{ $t('platform.openai.importDialog.resultTitle') }}</span>
            <span class="text-xs text-text-muted">
              {{ $t('platform.openai.importDialog.resultSummary', {
                success: importResult.success_count,
                failed: importResult.failed_count,
                total: importResult.total
              }) }}
            </span>
          </div>
          <div class="max-h-[200px] overflow-y-auto space-y-1">
            <div
              v-for="(result, index) in importResult.results"
              :key="index"
              class="flex items-center gap-2 text-xs py-1"
            >
              <svg
                v-if="result.success"
                class="h-4 w-4 shrink-0 text-success"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
              <svg
                v-else
                class="h-4 w-4 shrink-0 text-danger"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
              <span class="text-text truncate">{{ result.email || `#${index + 1}` }}</span>
              <span v-if="result.error" class="text-danger truncate">{{ result.error }}</span>
            </div>
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
        v-if="previewTokens.length && !importResult"
        @click="handleImport"
        class="btn btn--primary"
        :disabled="isImporting"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isImporting ? 'hidden' : 'visible' }">
            {{ $t('platform.openai.importDialog.import', { count: previewTokens.length }) }}
          </span>
          <span v-if="isImporting" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
      </button>
    </template>
  </BaseModal>

  <!-- 格式说明 Modal -->
  <BaseModal
    :visible="showFormatModal"
    :title="$t('platform.openai.importDialog.formatExample')"
    :show-close="true"
    close-on-overlay
    close-on-esc
    modal-class="max-w-[500px]"
    @close="showFormatModal = false"
  >
    <div class="space-y-4">
      <pre class="rounded-lg bg-surface-alt p-4 text-xs text-text overflow-x-auto">{{ formatExampleJson }}</pre>
    </div>
    <template #footer>
      <button @click="showFormatModal = false" class="btn btn--primary">{{ $t('common.close') }}</button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'imported'])

const showFormatModal = ref(false)
const formatExampleJson = `[
  "refresh_token_1",
  "refresh_token_2",
  "refresh_token_3"
]`

const fileName = ref('')
const previewTokens = ref([])
const importResult = ref(null)
const isImporting = ref(false)
const error = ref('')
const fileInputRef = ref(null)

const handleClose = () => {
  if (isImporting.value) return
  emit('close')
}

const maskToken = (token) => {
  if (!token || token.length < 12) return '***'
  return token.slice(0, 6) + '...' + token.slice(-4)
}

const selectFile = () => {
  fileInputRef.value?.click()
}

const handleFileChange = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  try {
    const content = await file.text()
    const data = JSON.parse(content)

    const tokens = Array.isArray(data) ? data : [data]

    const validTokens = tokens.filter(item => typeof item === 'string' && item.trim().length > 0)

    if (validTokens.length === 0) {
      error.value = $t('platform.openai.importDialog.emptyFile')
      return
    }

    fileName.value = file.name
    previewTokens.value = validTokens.map(s => s.trim())
    importResult.value = null
    error.value = ''
  } catch (err) {
    console.error('Failed to read file:', err)
    error.value = err?.message || $t('platform.openai.importDialog.readError')
  } finally {
    event.target.value = ''
  }
}

const clearFile = () => {
  fileName.value = ''
  previewTokens.value = []
  importResult.value = null
  error.value = ''
}

const handleImport = async () => {
  if (!previewTokens.value.length || isImporting.value) return

  error.value = ''
  isImporting.value = true
  const results = []
  let success_count = 0
  let failed_count = 0

  try {
    for (let i = 0; i < previewTokens.value.length; i++) {
      const token = previewTokens.value[i]
      try {
        const account = await invoke('openai_add_account', { refreshToken: token })
        try {
          await invoke('openai_fetch_quota', { accountId: account?.id })
        } catch (quotaErr) {
          console.warn('Fetch quota after add failed:', quotaErr)
        }
        results.push({ success: true, email: account?.email || '' })
        success_count++
      } catch (err) {
        const msg = err?.message || err || String(err)
        results.push({ success: false, email: '', error: msg })
        failed_count++
      }
    }

    importResult.value = {
      success_count,
      failed_count,
      total: previewTokens.value.length,
      results
    }

    if (success_count > 0) {
      emit('imported', importResult.value)
    }
  } catch (err) {
    console.error('Import error:', err)
    error.value = err?.message || err || $t('platform.openai.importDialog.importFailed')
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
