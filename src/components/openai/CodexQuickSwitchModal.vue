<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.openai.codexDialog.quickSwitch') + ' - ' + (type === 'codex' ? 'Codex' : 'Droid')"
    :show-close="true"
    :close-on-overlay="!isLoading"
    :close-on-esc="!isLoading"
    modal-class="max-w-[460px]"
    @close="handleClose"
  >
    <div class="space-y-4">
      <!-- Codex 专用：modelProvider -->
      <div v-if="type === 'codex'" class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.modelProvider') }}</label>
        <input
          v-model="form.modelProvider"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.modelProviderPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.model') }}</label>
        <input
          v-model="form.model"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.modelPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="flex gap-3">
        <div class="form-group flex-1">
          <label class="label">{{ $t('platform.openai.addAccountDialog.reasoningEffort') }}</label>
          <FloatingDropdown placement="bottom-start">
            <template #trigger="{ isOpen }">
              <button type="button" class="input flex items-center justify-between text-left" :disabled="isLoading">
                <span>{{ form.reasoningEffort }}</span>
                <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': isOpen }" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M7 10l5 5 5-5z"/>
                </svg>
              </button>
            </template>
            <template #default="{ close }">
              <button v-for="opt in reasoningOptions" :key="opt" @click="form.reasoningEffort = opt; close()" class="dropdown-item">{{ opt }}</button>
            </template>
          </FloatingDropdown>
        </div>

        <!-- Codex 专用：wireApi -->
        <div v-if="type === 'codex'" class="form-group flex-1">
          <label class="label">{{ $t('platform.openai.addAccountDialog.wireApi') }}</label>
          <FloatingDropdown placement="bottom-start">
            <template #trigger="{ isOpen }">
              <button type="button" class="input flex items-center justify-between text-left" :disabled="isLoading">
                <span>{{ form.wireApi }}</span>
                <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': isOpen }" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M7 10l5 5 5-5z"/>
                </svg>
              </button>
            </template>
            <template #default="{ close }">
              <button @click="form.wireApi = 'responses'; close()" class="dropdown-item">responses</button>
              <button @click="form.wireApi = 'chat'; close()" class="dropdown-item">chat</button>
            </template>
          </FloatingDropdown>
        </div>
      </div>

      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.baseUrl') }}</label>
        <input :value="baseUrl" type="text" class="input bg-muted" readonly />
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.openai.addAccountDialog.apiKey') }}</label>
        <input :value="apiKey" type="password" class="input bg-muted" readonly />
      </div>

      <div v-if="error" class="flex items-center gap-2 rounded-lg border border-danger/30 bg-danger/10 p-3 text-[13px] text-danger">
        <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        {{ error }}
      </div>
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary" :disabled="isLoading">{{ $t('common.cancel') }}</button>
      <button @click="handleSwitch" class="btn btn--primary" :disabled="!canSubmit || isLoading">
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">{{ $t('platform.openai.codexDialog.switchAccount') }}</span>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
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
import FloatingDropdown from '@/components/common/FloatingDropdown.vue'

const props = defineProps({
  type: { type: String, required: true }, // 'codex' | 'droid'
  baseUrl: { type: String, required: true },
  apiKey: { type: String, required: true }
})

const emit = defineEmits(['close', 'switched'])
const { t: $t } = useI18n()

const form = ref({
  modelProvider: 'codex',
  model: '',
  reasoningEffort: 'medium',
  wireApi: 'responses'
})

const reasoningOptions = ['low', 'medium', 'high', 'xhigh']
const isLoading = ref(false)
const error = ref('')

const canSubmit = computed(() => {
  if (props.type === 'codex') {
    return form.value.modelProvider.trim() && form.value.model.trim()
  }
  return form.value.model.trim()
})

const handleClose = () => {
  if (!isLoading.value) emit('close')
}

const handleSwitch = async () => {
  if (!canSubmit.value) return
  error.value = ''
  isLoading.value = true

  try {
    if (props.type === 'codex') {
      // Codex: 直接写入本地 Codex 配置并切换（不新增账号）
      await invoke('codex_switch_account', {
        modelProvider: form.value.modelProvider.trim(),
        model: form.value.model.trim(),
        reasoningEffort: form.value.reasoningEffort,
        wireApi: form.value.wireApi,
        baseUrl: props.baseUrl,
        apiKey: props.apiKey
      })
    } else {
      // Droid: 直接修改配置文件
      await invoke('droid_switch_account', {
        model: form.value.model.trim(),
        reasoningEffort: form.value.reasoningEffort,
        baseUrl: props.baseUrl,
        apiKey: props.apiKey
      })
    }
    window.$notify?.success($t('platform.openai.codexDialog.switchSuccess'))
    emit('switched')
  } catch (err) {
    console.error('Switch error:', err)
    error.value = err?.message || err || $t('platform.openai.codexDialog.switchFailed')
    isLoading.value = false
  }
}
</script>
