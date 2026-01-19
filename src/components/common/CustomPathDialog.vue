<template>
  <BaseModal
    :visible="visible"
    :title="title"
    @close="$emit('close')"
    modal-class="!max-w-[500px]"
  >
    <div class="flex flex-col gap-4">
      <div class="text-sm text-text-muted">
        {{ description }}
      </div>

      <!-- 当前路径显示 -->
      <div class="flex flex-col gap-2">
        <label class="label">{{ $t('customPath.currentPath') }}</label>
        <div class="flex items-center gap-2">
          <input
            type="text"
            v-model="localPathInput"
            :placeholder="defaultPath || $t('customPath.defaultPath')"
            class="input flex-1"
            readonly
            v-tooltip="localPathInput || defaultPath || ''"
          />
          <button
            class="btn btn--secondary btn--sm"
            @click="handleSelectPath"
            :disabled="isSelecting"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z"/>
            </svg>
            <span>{{ isSelecting ? $t('customPath.selecting') : $t('customPath.browse') }}</span>
          </button>
        </div>
      </div>

      <!-- 路径状态 -->
      <div v-if="validationStatus" class="text-sm" :class="validationStatus.valid ? 'text-green-500' : 'text-red-500'">
        {{ validationStatus.message }}
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center justify-end gap-2 pt-2">
        <button
          class="btn btn--ghost btn--sm"
          @click="handleReset"
          :disabled="!currentPath"
        >
          {{ $t('customPath.resetToDefault') }}
        </button>
        <button
          class="btn btn--primary btn--sm"
          @click="handleSave"
          :disabled="isSaving"
        >
          {{ isSaving ? $t('customPath.saving') : $t('customPath.save') }}
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from './BaseModal.vue'

const { t: $t } = useI18n()

const props = defineProps({
  visible: { type: Boolean, default: false },
  title: { type: String, required: true },
  description: { type: String, required: true },
  currentPath: { type: String, default: null },
  defaultPath: { type: String, default: '' },
  // 后端命令名称
  selectCommand: { type: String, required: true },
  validateCommand: { type: String, required: true },
  saveCommand: { type: String, required: true },
})

const emit = defineEmits(['close', 'saved'])

const localPathInput = ref('')
const isSelecting = ref(false)
const isSaving = ref(false)
const validationStatus = ref(null)

// 同步外部路径到本地
watch(() => props.visible, (val) => {
  if (val) {
    localPathInput.value = props.currentPath || ''
    validationStatus.value = null
  }
})

const handleSelectPath = async () => {
  isSelecting.value = true
  validationStatus.value = null

  try {
    const selected = await invoke(props.selectCommand)
    if (selected) {
      localPathInput.value = selected
      const isValid = await invoke(props.validateCommand, { path: selected })
      validationStatus.value = isValid
        ? { valid: true, message: $t('customPath.pathValid') }
        : { valid: false, message: $t('customPath.pathInvalid') }
    }
  } catch (error) {
    if (error !== 'User cancelled') {
      window.$notify?.error($t('customPath.selectFailed') + ': ' + error)
    }
  } finally {
    isSelecting.value = false
  }
}

const handleSave = async () => {
  isSaving.value = true
  try {
    const pathToSave = localPathInput.value.trim() || null
    if (pathToSave) {
      const isValid = await invoke(props.validateCommand, { path: pathToSave })
      if (!isValid) {
        validationStatus.value = { valid: false, message: $t('customPath.pathInvalid') }
        return
      }
    }
    await invoke(props.saveCommand, { path: pathToSave })
    window.$notify?.success($t('customPath.pathSaved'))
    emit('saved', pathToSave)
    emit('close')
  } catch (error) {
    window.$notify?.error($t('customPath.saveFailed') + ': ' + error)
  } finally {
    isSaving.value = false
  }
}

const handleReset = async () => {
  localPathInput.value = ''
  validationStatus.value = null
  try {
    await invoke(props.saveCommand, { path: null })
    window.$notify?.success($t('customPath.resetSuccess'))
    emit('saved', null)
  } catch (error) {
    window.$notify?.error($t('customPath.resetFailed') + ': ' + error)
  }
}
</script>

