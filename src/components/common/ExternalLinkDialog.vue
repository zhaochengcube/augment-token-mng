<template>
  <div v-if="show" class="portal-dialog-overlay" @click="handleClose">
    <div class="portal-dialog" @click.stop>
      <h3>{{ title }}</h3>
      <div class="dialog-buttons">
        <button @click="handleExternalOpen" class="btn primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
          {{ $t('dialogs.openExternal') }}
        </button>
        <button @click="handleInternalOpen" class="btn primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
          </svg>
          {{ $t('dialogs.openInternal') }}
        </button>
        <button @click="handleCopyUrl" class="btn primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
          </svg>
          {{ $t('dialogs.copyLink') }}
        </button>
        <button @click="handleClose" class="btn secondary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
          {{ $t('dialogs.cancel') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Props
const props = defineProps({
  show: {
    type: Boolean,
    required: true
  },
  title: {
    type: String,
    required: true
  },
  url: {
    type: String,
    required: true
  },
  browserTitle: {
    type: String,
    required: true
  }
})

// Emits
const emit = defineEmits(['close'])

// Methods
const handleClose = () => {
  emit('close')
}

const handleCopyUrl = async () => {
  if (!props.url) return

  try {
    await navigator.clipboard.writeText(props.url)
    window.$notify.success(t('messages.copySuccess'))
    emit('close') // 复制成功后关闭对话框
  } catch (error) {
    console.error('Failed to copy URL to clipboard:', error)
    window.$notify.error(t('messages.copyFailed'))
  }
}

const handleExternalOpen = async () => {
  emit('close')
  if (!props.url) return

  try {
    await invoke('open_url', { url: props.url })
  } catch (error) {
    console.error('Failed to open URL externally:', error)
    window.$notify.error(t('messages.openUrlFailed'))
  }
}

const handleInternalOpen = async () => {
  emit('close')
  if (!props.url) return

  try {
    await invoke('open_internal_browser', {
      url: props.url,
      title: props.browserTitle
    })
  } catch (error) {
    console.error('Failed to open URL internally:', error)
    window.$notify.error(t('messages.openUrlFailed'))
  }
}
</script>

<style scoped>
/* ============================================
   ExternalLinkDialog - Modern Tech Style
   ============================================ */

.portal-dialog-overlay {
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
  backdrop-filter: blur(4px);
}

/* 对话框 - 磨砂玻璃 */
.portal-dialog {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 18px;
  padding: 28px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
  min-width: 320px;
  max-width: 420px;
}

.portal-dialog h3 {
  margin: 0 0 24px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
  text-align: center;
}

.dialog-buttons {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* 对话框按钮 - 科技风 */
.dialog-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  justify-content: center;
}

.dialog-btn svg {
  flex-shrink: 0;
  display: block;
  width: 16px;
  height: 16px;
}

.dialog-btn:hover {
  transform: translateY(-2px);
}

.dialog-btn.copy {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 35%, transparent);
}

.dialog-btn.copy:hover {
  background: color-mix(in srgb, var(--accent) 18%, transparent);
  border-color: color-mix(in srgb, var(--accent) 55%, transparent);
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.dialog-btn.external {
  background: color-mix(in srgb, #3b82f6 12%, transparent);
  color: #3b82f6;
  border-color: color-mix(in srgb, #3b82f6 35%, transparent);
}

.dialog-btn.external:hover {
  background: color-mix(in srgb, #3b82f6 18%, transparent);
  border-color: color-mix(in srgb, #3b82f6 55%, transparent);
  box-shadow: 0 0 15px rgba(59, 130, 246, 0.4);
}

.dialog-btn.internal {
  background: color-mix(in srgb, #10b981 12%, transparent);
  color: #10b981;
  border-color: color-mix(in srgb, #10b981 35%, transparent);
}

.dialog-btn.internal:hover {
  background: color-mix(in srgb, #10b981 18%, transparent);
  border-color: color-mix(in srgb, #10b981 55%, transparent);
  box-shadow: 0 0 15px rgba(16, 185, 129, 0.4);
}

.dialog-btn.cancel {
  background: color-mix(in srgb, #ef4444 12%, transparent);
  color: #ef4444;
  border-color: color-mix(in srgb, #ef4444 35%, transparent);
}

.dialog-btn.cancel:hover {
  background: color-mix(in srgb, #ef4444 18%, transparent);
  border-color: color-mix(in srgb, #ef4444 55%, transparent);
  box-shadow: 0 0 15px var(--tech-glow-danger);
}
</style>
