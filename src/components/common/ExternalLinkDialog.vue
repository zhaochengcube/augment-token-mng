<template>
  <div v-if="show" class="portal-dialog-overlay" @click="handleClose">
    <div class="portal-dialog" @click.stop>
      <h3>{{ title }}</h3>
      <div class="dialog-buttons">
        <button @click="handleExternalOpen" class="dialog-btn external">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
          {{ $t('dialogs.openExternal') }}
        </button>
        <button @click="handleInternalOpen" class="dialog-btn internal">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
          </svg>
          {{ $t('dialogs.openInternal') }}
        </button>
        <button @click="handleCopyUrl" class="dialog-btn copy">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
          </svg>
          {{ $t('dialogs.copyLink') }}
        </button>
        <button @click="handleClose" class="dialog-btn cancel">
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
/* Portal对话框样式 - 与App.vue保持一致 */
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
  z-index: 3000; /* 确保在所有其他元素之上 */
}

.portal-dialog {
  background: var(--bg-surface);
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  min-width: 300px;
  max-width: 400px;
}

.portal-dialog h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
  text-align: center;
}

.dialog-buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border: 2px solid transparent;
  border-radius: 8px;
  background: var(--bg-muted);
  color: var(--text);
  font-size: 14px;
  font-weight: 500;
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
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.dialog-btn.copy {
  background: rgba(124, 58, 237, 0.1);
  color: #7c3aed;
  border-color: rgba(196, 181, 253, 0.3);
}

.dialog-btn.copy:hover {
  background: rgba(124, 58, 237, 0.15);
  border-color: rgba(168, 139, 250, 0.5);
}

.dialog-btn.external {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent-strong);
  border-color: color-mix(in srgb, var(--accent) 30%, transparent);
}

.dialog-btn.external:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
}

.dialog-btn.internal {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border-color: rgba(34, 197, 94, 0.3);
}

.dialog-btn.internal:hover {
  background: rgba(34, 197, 94, 0.15);
  border-color: rgba(34, 197, 94, 0.5);
}

.dialog-btn.cancel {
  background: rgba(236, 72, 153, 0.1);
  color: #ec4899;
  border-color: rgba(236, 72, 153, 0.3);
}

.dialog-btn.cancel:hover {
  background: rgba(236, 72, 153, 0.15);
  border-color: rgba(236, 72, 153, 0.5);
}

</style>
