<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.openai.editApiAccountDialog.title')"
    :show-close="true"
    :close-on-overlay="!isLoading"
    :close-on-esc="!isLoading"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="handleClose"
  >
    <div class="animate-fade-in">
      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.modelProvider') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.modelProviderHint') }})</span></label>
        <input
          v-model="formData.model_provider"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.modelProviderPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.model') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.modelHint') }})</span></label>
        <input
          v-model="formData.model"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.modelPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="flex gap-3">
        <div class="form-group flex-1">
          <label class="label">{{ $t('platform.openai.addAccountDialog.reasoningEffort') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.reasoningEffortHint') }})</span></label>
          <FloatingDropdown v-model="formData.model_reasoning_effort" placement="bottom-start" :close-on-select="true">
            <template #trigger="{ isOpen }">
              <button type="button" class="input flex items-center justify-between text-left" :disabled="isLoading">
                <span>{{ formData.model_reasoning_effort || 'medium' }}</span>
                <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': isOpen }" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M7 10l5 5 5-5z"/>
                </svg>
              </button>
            </template>
            <template #default="{ close }">
              <button @click="selectReasoningEffort('low', close)" class="dropdown-item">low</button>
              <button @click="selectReasoningEffort('medium', close)" class="dropdown-item">medium</button>
              <button @click="selectReasoningEffort('high', close)" class="dropdown-item">high</button>
              <button @click="selectReasoningEffort('xhigh', close)" class="dropdown-item">xhigh</button>
            </template>
          </FloatingDropdown>
        </div>

        <div class="form-group flex-1">
          <label class="label">{{ $t('platform.openai.addAccountDialog.wireApi') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.wireApiHint') }})</span></label>
          <FloatingDropdown v-model="formData.wire_api" placement="bottom-start" :close-on-select="true">
            <template #trigger="{ isOpen }">
              <button type="button" class="input flex items-center justify-between text-left" :disabled="isLoading">
                <span>{{ formData.wire_api || 'responses' }}</span>
                <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': isOpen }" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M7 10l5 5 5-5z"/>
                </svg>
              </button>
            </template>
            <template #default="{ close }">
              <button @click="selectWireApi('responses', close)" class="dropdown-item">responses</button>
              <button @click="selectWireApi('chat', close)" class="dropdown-item">chat</button>
            </template>
          </FloatingDropdown>
        </div>
      </div>

      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.baseUrl') }}</label>
        <input
          v-model="formData.base_url"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.baseUrlPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.openai.addAccountDialog.apiKey') }}</label>
        <textarea
          v-model="formData.key"
          :placeholder="$t('platform.openai.addAccountDialog.apiKeyPlaceholder')"
          class="input resize-none"
          rows="3"
          :disabled="isLoading"
        ></textarea>
      </div>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="mt-4 flex items-center gap-2 rounded-lg border border-danger/30 bg-danger/10 p-3 text-[13px] text-danger">
      <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      {{ error }}
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary" :disabled="isLoading">
        {{ $t('common.cancel') }}
      </button>
      <button
        @click="handleSave"
        class="btn btn--primary"
        :disabled="!canSubmit || isLoading"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">
            {{ $t('common.save') }}
          </span>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'
import FloatingDropdown from '@/components/common/FloatingDropdown.vue'

const { t: $t } = useI18n()

const props = defineProps({
  account: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close', 'saved'])

const isLoading = ref(false)
const error = ref('')

const formData = ref({
  model_provider: '',
  model: '',
  model_reasoning_effort: 'medium',
  wire_api: 'responses',
  base_url: '',
  key: ''
})

const canSubmit = computed(() => {
  return formData.value.model_provider.trim() &&
         formData.value.base_url.trim() &&
         formData.value.key.trim()
})

const handleClose = () => {
  if (isLoading.value) return
  emit('close')
}

const selectReasoningEffort = (value, close) => {
  formData.value.model_reasoning_effort = value
  close?.()
}

const selectWireApi = (value, close) => {
  formData.value.wire_api = value
  close?.()
}

const handleSave = async () => {
  if (!canSubmit.value) return

  error.value = ''
  isLoading.value = true

  try {
    const updatedAccount = await invoke('openai_update_api_account', {
      accountId: props.account.id,
      modelProvider: formData.value.model_provider.trim(),
      model: formData.value.model.trim(),
      reasoningEffort: formData.value.model_reasoning_effort,
      wireApi: formData.value.wire_api,
      baseUrl: formData.value.base_url.trim(),
      key: formData.value.key.trim()
    })
    emit('saved', updatedAccount)
  } catch (err) {
    console.error('Update API account error:', err)
    error.value = err?.message || err || $t('platform.openai.messages.updateFailed')
    isLoading.value = false
  }
}

onMounted(() => {
  if (props.account?.api_config) {
    const config = props.account.api_config
    formData.value = {
      model_provider: config.model_provider || '',
      model: config.model || '',
      model_reasoning_effort: config.model_reasoning_effort || 'medium',
      wire_api: config.wire_api || 'responses',
      base_url: config.base_url || '',
      key: config.key || ''
    }
  }
})
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
