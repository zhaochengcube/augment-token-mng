<template>
  <BaseModal
    :visible="visible"
    :title="title"
    :show-close="true"
    :close-on-overlay="!loading"
    :close-on-esc="!loading"
    modal-class="max-w-[400px]"
    @close="handleClose"
  >
    <!-- Message -->
    <p class="mb-4 text-sm text-text-secondary">{{ message }}</p>

    <!-- Stats Slot -->
    <div v-if="$slots.stats" class="mb-4 flex flex-col gap-2 rounded-lg border border-border bg-muted/50 p-4">
      <slot name="stats" />
    </div>

    <!-- Warning -->
    <p class="text-xs font-medium text-danger">{{ $t('tokenList.cannotUndo') }}</p>

    <template #footer>
      <button
        @click="handleConfirm"
        class="btn btn--danger"
        :disabled="loading"
      >
        <span v-if="loading" class="btn-spinner btn-spinner--sm" aria-hidden="true"></span>
        {{ loading ? $t('tokenList.deleting') : $t('tokenList.confirmDelete') }}
      </button>
      <button @click="handleClose" class="btn btn--secondary" :disabled="loading">
        {{ $t('common.cancel') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from './BaseModal.vue'

const { t } = useI18n()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: ''
  },
  message: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:visible', 'confirm'])

const loading = ref(false)

const handleClose = () => {
  if (loading.value) return
  emit('update:visible', false)
}

const handleConfirm = async () => {
  loading.value = true
  try {
    await emit('confirm')
  } finally {
    loading.value = false
    emit('update:visible', false)
  }
}
</script>
