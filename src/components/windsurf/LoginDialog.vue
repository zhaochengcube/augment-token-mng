<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.windsurf.loginWithEmail')"
    :show-close="true"
    :close-on-overlay="!isLogging"
    :close-on-esc="!isLogging"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="handleClose"
  >
    <div class="form-group">
      <label class="label">{{ $t('common.email') }}</label>
      <input
        v-model="email"
        type="email"
        :placeholder="$t('platform.windsurf.emailPlaceholder')"
        class="input"
        :disabled="isLogging"
      />
    </div>

    <div class="form-group mb-0">
      <label class="label">{{ $t('common.password') }}</label>
      <input
        v-model="password"
        type="password"
        :placeholder="$t('platform.windsurf.passwordPlaceholder')"
        class="input"
        :disabled="isLogging"
      />
    </div>

    <!-- Error Message -->
    <div v-if="error" class="mt-4 flex items-center gap-2 rounded-lg border border-danger/30 bg-danger/10 p-3 text-[13px] text-danger">
      <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      {{ error }}
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary" :disabled="isLogging">
        {{ $t('common.cancel') }}
      </button>
      <button
        @click="handleLogin"
        class="btn btn--primary"
        :disabled="!canLogin || isLogging"
      >
        <span v-if="isLogging" class="btn-spinner" aria-hidden="true"></span>
        <span v-else>{{ $t('common.login') }}</span>
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'added'])

const email = ref('')
const password = ref('')
const isLogging = ref(false)
const error = ref('')

const canLogin = computed(() => email.value.trim() && password.value.trim())

const handleClose = () => {
  if (isLogging.value) return
  emit('close')
}

const handleLogin = async () => {
  if (!canLogin.value || isLogging.value) return

  error.value = ''
  isLogging.value = true

  try {
    const account = await invoke('windsurf_login', { 
      email: email.value.trim(), 
      password: password.value 
    })
    emit('added', account)
  } catch (err) {
    console.error('Login error:', err)
    error.value = err?.message || err || $t('platform.windsurf.messages.loginFailed')
  } finally {
    isLogging.value = false
  }
}
</script>
