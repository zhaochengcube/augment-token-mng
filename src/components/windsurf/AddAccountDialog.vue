<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.windsurf.addAccount')"
    :show-close="true"
    :close-on-overlay="!isLoading"
    :close-on-esc="!isLoading"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="handleClose"
  >
    <!-- 添加方式选择 Tab -->
    <div class="mb-6 flex gap-2 rounded-lg bg-muted p-1">
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-1.5 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
          addMethod === 'login'
            ? 'bg-surface text-accent shadow-sm'
            : 'text-text-secondary hover:bg-hover hover:text-text'
        ]"
        @click="addMethod = 'login'"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
        </svg>
        {{ $t('platform.windsurf.loginWithEmail') }}
      </button>
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-1.5 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
          addMethod === 'token'
            ? 'bg-surface text-accent shadow-sm'
            : 'text-text-secondary hover:bg-hover hover:text-text'
        ]"
        @click="addMethod = 'token'"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/>
        </svg>
        {{ $t('platform.windsurf.addByToken') }}
      </button>
    </div>

    <!-- 邮箱密码登录方式 -->
    <div v-if="addMethod === 'login'" class="animate-fade-in">
      <div class="form-group">
        <label class="label">{{ $t('common.email') }}</label>
        <input
          v-model="email"
          type="email"
          :placeholder="$t('platform.windsurf.emailPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('common.password') }}</label>
        <input
          v-model="password"
          type="password"
          :placeholder="$t('platform.windsurf.passwordPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>
    </div>

    <!-- Token 添加方式 -->
    <div v-else class="animate-fade-in">
      <div class="form-group mb-0">
        <label class="label">Refresh Token</label>
        <textarea
          v-model="refreshToken"
          :placeholder="$t('platform.windsurf.tokenPlaceholder')"
          class="input resize-none"
          rows="4"
          :disabled="isLoading"
        ></textarea>
        <p class="mt-1.5 text-xs text-text-muted">
          {{ $t('platform.windsurf.tokenHint') }}
        </p>
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
      <button @click="handleClose" class="btn btn--secondary">
        {{ $t('common.cancel') }}
      </button>
      <button
        @click="handleSubmit"
        class="btn btn--primary"
        :disabled="!canSubmit"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">
            {{ addMethod === 'login' ? $t('common.login') : $t('common.add') }}
          </span>
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

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'added'])

const addMethod = ref('login') // 'login' or 'token'
const email = ref('')
const password = ref('')
const refreshToken = ref('')
const isLoading = ref(false)
const error = ref('')

const canSubmit = computed(() => {
  if (addMethod.value === 'login') {
    return email.value.trim() && password.value.trim()
  }
  return refreshToken.value.trim()
})

const handleClose = () => {
  if (isLoading.value) return
  emit('close')
}

const handleSubmit = async () => {
  if (!canSubmit.value || isLoading.value) return

  error.value = ''
  isLoading.value = true

  try {
    let account
    if (addMethod.value === 'login') {
      account = await invoke('windsurf_login', {
        email: email.value.trim(),
        password: password.value
      })
    } else {
      account = await invoke('windsurf_add_account', {
        refreshToken: refreshToken.value.trim()
      })
    }
    emit('added', account)
  } catch (err) {
    console.error('Add account error:', err)
    error.value = err?.message || err || $t('platform.windsurf.messages.addFailed')
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
/* Fade-in animation for tab content */
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
