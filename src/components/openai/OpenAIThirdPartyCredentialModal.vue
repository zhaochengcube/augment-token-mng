<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.openai.thirdPartyCredentials.title')"
    :show-close="true"
    :close-on-overlay="true"
    :close-on-esc="true"
    :body-scroll="false"
    modal-class="max-w-[720px]"
    @close="emit('close')"
  >
    <div class="flex flex-col gap-4">
      <div class="rounded-lg border border-border bg-muted/40 px-3 py-2.5">
        <div class="text-xs text-text-muted">{{ $t('platform.openai.thirdPartyCredentials.account') }}</div>
        <div class="mt-1 break-all text-sm font-medium text-text">{{ account?.email || '-' }}</div>
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.openai.thirdPartyCredentials.template') }}</label>
        <select v-model="selectedTemplateId" class="input">
          <option
            v-for="template in availableTemplates"
            :key="template.id"
            :value="template.id"
          >
            {{ template.label }}
          </option>
        </select>
        <p class="mt-1.5 text-xs text-text-muted">
          {{ $t('platform.openai.thirdPartyCredentials.hint') }}
        </p>
      </div>

      <div class="rounded-lg border border-border bg-muted/30 p-3">
        <div class="mb-2 flex items-center justify-between gap-3">
          <span class="text-xs font-medium text-text-secondary">{{ $t('platform.openai.thirdPartyCredentials.preview') }}</span>
          <span class="text-[11px] uppercase text-text-muted">{{ selectedTemplateLabel }}</span>
        </div>
        <textarea
          :value="previewContent"
          readonly
          class="input min-h-[320px] resize-none font-mono text-[12px] leading-5"
        ></textarea>
      </div>
    </div>

    <template #footer>
      <button class="btn btn--secondary" @click="emit('close')">
        {{ $t('common.cancel') }}
      </button>
      <button class="btn btn--primary" :disabled="!canCopy" @click="copyContent">
        {{ $t('platform.openai.thirdPartyCredentials.copy') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import {
  buildOpenAIThirdPartyCredentialPreview,
  getAvailableOpenAIThirdPartyCredentialTemplates
} from '@/utils/openaiThirdPartyCredentials'

const props = defineProps({
  account: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close'])
const { t: $t } = useI18n()

const selectedTemplateId = ref('')

const availableTemplates = computed(() => {
  return getAvailableOpenAIThirdPartyCredentialTemplates(props.account)
})

watch(
  availableTemplates,
  (templates) => {
    if (templates.some((template) => template.id === selectedTemplateId.value)) {
      return
    }
    selectedTemplateId.value = templates[0]?.id || ''
  },
  { immediate: true }
)

const selectedTemplate = computed(() => {
  return availableTemplates.value.find((template) => template.id === selectedTemplateId.value) || null
})

const selectedTemplateLabel = computed(() => {
  return selectedTemplate.value?.label || '-'
})

const previewContent = computed(() => {
  if (!selectedTemplate.value) {
    return $t('platform.openai.thirdPartyCredentials.empty')
  }

  try {
    return buildOpenAIThirdPartyCredentialPreview(props.account, selectedTemplate.value.id)
  } catch (error) {
    return error?.message || $t('platform.openai.thirdPartyCredentials.copyFailed')
  }
})

const canCopy = computed(() => Boolean(selectedTemplate.value))

const copyContent = async () => {
  if (!selectedTemplate.value) return

  try {
    await navigator.clipboard.writeText(previewContent.value)
    window.$notify?.success(
      $t('platform.openai.thirdPartyCredentials.copySuccess', {
        name: selectedTemplate.value.label
      })
    )
    emit('close')
  } catch {
    window.$notify?.error($t('platform.openai.thirdPartyCredentials.copyFailed'))
  }
}
</script>
