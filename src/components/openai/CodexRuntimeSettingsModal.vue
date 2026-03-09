<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.openai.codexRuntimeSettings.title')"
    :show-close="true"
    :body-scroll="false"
    modal-class="max-w-[560px]"
    @close="$emit('close')"
  >
    <div class="space-y-4">
      <div class="rounded-lg border border-border p-4">
        <div class="flex items-start justify-between gap-3">
          <div>
            <div class="text-sm font-semibold">{{ $t('platform.openai.codexRuntimeSettings.quotaRefresh.title') }}</div>
            <p class="mt-1 text-xs text-text-muted">
              {{ $t('platform.openai.codexRuntimeSettings.quotaRefresh.description') }}
            </p>
          </div>
          <input
            v-model="form.quota_refresh_enabled"
            type="checkbox"
            class="mt-1 h-4 w-4 accent-accent"
            :disabled="isLoading || isSaving"
          />
        </div>
        <div class="mt-3">
          <label class="label">{{ $t('platform.openai.codexRuntimeSettings.quotaRefresh.intervalLabel') }}</label>
          <input
            v-model.number="form.quota_refresh_interval_minutes"
            type="number"
            min="1"
            max="1440"
            class="input"
            :disabled="isLoading || isSaving || !form.quota_refresh_enabled"
          />
        </div>
      </div>

      <div class="rounded-lg border border-border p-4">
        <div class="flex items-start justify-between gap-3">
          <div>
            <div class="text-sm font-semibold">{{ $t('platform.openai.codexRuntimeSettings.fastMode.title') }}</div>
            <p class="mt-1 text-xs text-text-muted">
              {{ $t('platform.openai.codexRuntimeSettings.fastMode.description') }}
            </p>
          </div>
          <input
            v-model="form.fast_mode_enabled"
            type="checkbox"
            class="mt-1 h-4 w-4 accent-accent"
            :disabled="isLoading || isSaving"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <button
          class="btn btn--secondary btn--sm"
          :disabled="isLoading || isSaving"
          @click="$emit('close')"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          class="btn btn--primary btn--sm"
          :disabled="isLoading || isSaving"
          @click="saveSettings"
        >
          <span v-if="isSaving" class="btn-spinner mr-2" aria-hidden="true"></span>
          {{ $t('common.save') }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '../common/BaseModal.vue'

const { t: $t } = useI18n()
const emit = defineEmits(['close'])

const isLoading = ref(true)
const isSaving = ref(false)

const form = reactive({
  quota_refresh_enabled: true,
  quota_refresh_interval_minutes: 30,
  fast_mode_enabled: false
})

const clampInt = (value, min, max, fallback) => {
  const parsed = Number.parseInt(String(value), 10)
  if (!Number.isFinite(parsed)) return fallback
  if (parsed < min) return min
  if (parsed > max) return max
  return parsed
}

const applySettings = (settings) => {
  form.quota_refresh_enabled = Boolean(settings.quota_refresh_enabled)
  const quotaSec = clampInt(settings.quota_refresh_interval_seconds, 60, 86400, 1800)
  form.quota_refresh_interval_minutes = Math.round(quotaSec / 60)
  form.fast_mode_enabled = Boolean(settings.fast_mode_enabled)
}

const loadSettings = async () => {
  isLoading.value = true
  try {
    const settings = await invoke('get_codex_runtime_settings')
    applySettings(settings || {})
  } catch (error) {
    console.error('Failed to load codex runtime settings:', error)
    window.$notify?.error($t('platform.openai.codexRuntimeSettings.messages.loadFailed', { error: error?.message || error }))
  } finally {
    isLoading.value = false
  }
}

const buildPayload = () => ({
  quota_refresh_enabled: form.quota_refresh_enabled,
  quota_refresh_interval_seconds: clampInt(form.quota_refresh_interval_minutes * 60, 60, 86400, 1800),
  fast_mode_enabled: form.fast_mode_enabled
})

const saveSettings = async () => {
  isSaving.value = true
  try {
    const payload = buildPayload()
    const saved = await invoke('set_codex_runtime_settings', { settings: payload })
    applySettings(saved || payload)
    window.$notify?.success($t('platform.openai.codexRuntimeSettings.messages.saveSuccess'))
    emit('close')
  } catch (error) {
    console.error('Failed to save codex runtime settings:', error)
    window.$notify?.error($t('platform.openai.codexRuntimeSettings.messages.saveFailed', { error: error?.message || error }))
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  loadSettings()
})
</script>
