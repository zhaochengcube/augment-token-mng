<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed inset-0 z-[4000] flex items-center justify-center"
    >
      <!-- Backdrop -->
      <div
        class="absolute inset-0 bg-black/50 backdrop-blur-sm"
        @click="handleCancel"
      />

      <!-- Dialog -->
      <div
        class="relative z-10 w-[90%] max-w-[320px] rounded-lg border border-border bg-page shadow-lg"
      >
        <!-- Content -->
        <div class="px-4 pt-4 pb-3">
          <h3 class="m-0 mb-1.5 text-[14px] font-semibold text-text">
            {{ currentOptions.title }}
          </h3>
          <p class="m-0 text-[13px] leading-relaxed text-text-secondary">
            {{ currentOptions.message }}
          </p>
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-2 px-4 pb-4">
          <button
            @click="handleCancel"
            class="btn btn--secondary btn--sm"
          >
            {{ currentOptions.cancelText }}
          </button>
          <button
            @click="handleConfirm"
            :class="['btn btn--sm', currentOptions.variant === 'danger' ? 'btn--danger' : 'btn--primary']"
          >
            {{ currentOptions.confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const visible = ref(false)
const currentOptions = ref({
  title: '',
  message: '',
  confirmText: '',
  cancelText: '',
  variant: 'danger'
})

let resolvePromise = null

const show = (options = {}) => {
  currentOptions.value = {
    title: options.title || t('common.confirm'),
    message: options.message || '',
    confirmText: options.confirmText || t('common.confirm'),
    cancelText: options.cancelText || t('common.cancel'),
    variant: options.variant || 'danger'
  }
  visible.value = true

  return new Promise((resolve) => {
    resolvePromise = resolve
  })
}

const handleConfirm = () => {
  visible.value = false
  if (resolvePromise) {
    resolvePromise(true)
    resolvePromise = null
  }
}

const handleCancel = () => {
  visible.value = false
  if (resolvePromise) {
    resolvePromise(false)
    resolvePromise = null
  }
}

// 注册全局服务
onMounted(() => {
  if (typeof window !== 'undefined') {
    window.$confirm = show
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    delete window.$confirm
  }
})

defineExpose({
  show
})
</script>
