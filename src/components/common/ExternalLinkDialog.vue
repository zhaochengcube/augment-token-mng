<template>
  <BaseModal
    :visible="show"
    :title="title"
    :close-on-overlay="true"
    :body-scroll="false"
    modal-class="!max-w-[240px]"
    @close="handleClose"
  >
    <div class="flex flex-col gap-2">
      <button @click="handleExternalOpen" class="btn btn--primary btn--md w-full justify-start">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
        </svg>
        {{ $t('dialogs.openExternal') }}
      </button>
      <button @click="handleInternalOpen" class="btn btn--primary btn--md w-full justify-start">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
        {{ $t('dialogs.openInternal') }}
      </button>
      <button @click="handleCopyUrl" class="btn btn--primary btn--md w-full justify-start">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
        </svg>
        {{ $t('dialogs.copyLink') }}
      </button>
      <button @click="handleClose" class="btn btn--secondary btn--md w-full justify-start">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
        {{ $t('dialogs.cancel') }}
      </button>
    </div>
  </BaseModal>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from './BaseModal.vue'

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
