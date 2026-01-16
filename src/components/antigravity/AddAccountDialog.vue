<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.antigravity.addAccountDialog.title')"
    :show-close="true"
    :close-on-overlay="!isLoading"
    :close-on-esc="!isLoading"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="handleClose"
  >
    <!-- 添加方式选择 -->
    <div class="mb-6 flex gap-2 rounded-lg bg-muted p-1">
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-1.5 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
          addMethod === 'oauth'
            ? 'bg-surface text-accent shadow-sm'
            : 'text-text-secondary hover:bg-hover hover:text-text'
        ]"
        @click="switchToOAuth"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
        {{ $t('platform.antigravity.addAccountDialog.oauthMethod') }}
      </button>
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-1.5 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
          addMethod === 'manual'
            ? 'bg-surface text-accent shadow-sm'
            : 'text-text-secondary hover:bg-hover hover:text-text'
        ]"
        @click="addMethod = 'manual'"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
        </svg>
        {{ $t('platform.antigravity.addAccountDialog.manualMethod') }}
      </button>
    </div>

    <!-- OAuth 授权方式 -->
    <div v-if="addMethod === 'oauth'" class="animate-fade-in">
      <!-- OAuth Info -->
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-accent/20 bg-accent/10 p-4">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <p class="text-[13px] leading-relaxed text-text-secondary">{{ $t('platform.antigravity.addAccountDialog.oauthInfo') }}</p>
      </div>

      <!-- Google OAuth Button -->
      <button
        @click="handleOAuthLogin"
        class="flex w-full items-center justify-center gap-2.5 rounded-lg border border-border bg-white px-5 py-3.5 text-[15px] font-medium text-neutral-800 transition-all hover:border-border-strong hover:bg-neutral-50 hover:shadow-sm disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="isLoading"
      >
        <div v-if="isLoading" class="h-5 w-5 border-2 border-current/30 border-t-current rounded-full animate-spin"></div>
        <svg v-else class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12.545,10.239v3.821h5.445c-0.712,2.315-2.647,3.972-5.445,3.972c-3.332,0-6.033-2.701-6.033-6.032s2.701-6.032,6.033-6.032c1.498,0,2.866,0.549,3.921,1.453l2.814-2.814C17.503,2.988,15.139,2,12.545,2C7.021,2,2.543,6.477,2.543,12s4.478,10,10.002,10c8.396,0,10.249-7.85,9.426-11.748L12.545,10.239z"/>
        </svg>
        <span v-if="isLoading">{{ $t('platform.antigravity.addAccountDialog.adding') }}</span>
        <span v-else>{{ $t('platform.antigravity.addAccountDialog.googleLogin') }}</span>
      </button>

      <!-- Manual OAuth Section -->
      <div class="mt-5 rounded-lg bg-muted p-4">
        <div class="mb-3 text-[13px] font-semibold text-text">{{ $t('platform.antigravity.addAccountDialog.oauthManualTitle') }}</div>

        <div class="mb-3 flex gap-2.5">
          <button class="btn btn--primary" @click="generateAuthUrl" :disabled="isLoading || isManualLoading">
            {{ $t('platform.antigravity.addAccountDialog.generateAuthLink') }}
          </button>
        </div>

        <div v-if="oauthAuthUrl" class="mb-3 flex items-center gap-2">
          <input type="text" :value="oauthAuthUrl" readonly class="input flex-1" />
          <button
            class="btn btn--secondary btn--icon shrink-0"
            @click="copyAuthUrl"
            :disabled="isLoading || isManualLoading"
            v-tooltip="$t('platform.antigravity.addAccountDialog.copyAuthLink')"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
        </div>

        <div class="form-group mb-3">
          <label class="label">{{ $t('platform.antigravity.addAccountDialog.callbackLabel') }}</label>
          <div class="relative flex items-center">
            <input
              v-model="oauthCallbackInput"
              type="text"
              :placeholder="$t('platform.antigravity.addAccountDialog.callbackPlaceholder')"
              class="input w-full pr-9"
              :disabled="isLoading || isManualLoading"
            />
            <button
              v-if="oauthCallbackInput"
              class="absolute right-1.5 flex h-7 w-7 items-center justify-center rounded text-text-muted transition-colors hover:bg-hover hover:text-text"
              type="button"
              @click="oauthCallbackInput = ''"
            >
              <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
          </div>
          <p class="mt-1.5 text-xs text-text-muted">
            {{ $t('platform.antigravity.addAccountDialog.callbackHint') }}
          </p>
        </div>

        <button class="btn btn--primary" @click="handleOAuthExchange" :disabled="!canExchange || isLoading || isManualLoading">
          {{ $t('platform.antigravity.addAccountDialog.exchangeCode') }}
        </button>
      </div>
    </div>

    <!-- 手动添加方式 -->
    <div v-else class="animate-fade-in">
      <div class="form-group">
        <label class="label">{{ $t('platform.antigravity.addAccountDialog.email') }}</label>
        <input
          v-model="email"
          type="email"
          :placeholder="$t('platform.antigravity.addAccountDialog.emailPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.antigravity.addAccountDialog.refreshToken') }}</label>
        <textarea
          v-model="refreshToken"
          :placeholder="$t('platform.antigravity.addAccountDialog.refreshTokenPlaceholder')"
          class="input resize-none"
          rows="4"
          :disabled="isLoading"
        ></textarea>
        <p class="mt-1.5 text-xs text-text-muted">
          {{ $t('platform.antigravity.addAccountDialog.refreshTokenHint') }}
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
      <button @click="handleClose" class="btn btn--secondary" :disabled="isLoading">
        {{ $t('common.cancel') }}
      </button>
      <button
        v-if="addMethod === 'manual'"
        @click="handleAdd"
        class="btn btn--primary"
        :disabled="!canSubmit || isLoading"
      >
        <div v-if="isLoading" class="h-4 w-4 border-2 border-current/30 border-t-current rounded-full animate-spin"></div>
        <span v-else>{{ $t('platform.antigravity.addAccountDialog.add') }}</span>
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'add', 'added'])

const handleClose = () => {
  if (isLoading.value) return
  emit('close')
}

const addMethod = ref('oauth') // 'oauth' or 'manual'
const email = ref('')
const refreshToken = ref('')
const isLoading = ref(false)
const isManualLoading = ref(false)
const error = ref('')
const oauthAuthUrl = ref('')
const oauthRedirectUri = ref('')
const oauthCallbackInput = ref('')

const canSubmit = computed(() => {
  if (addMethod.value === 'oauth') return true
  return email.value.trim() && refreshToken.value.trim()
})

const canExchange = computed(() => {
  const raw = oauthCallbackInput.value.trim()
  if (!raw) return false
  if (/^https?:\/\//i.test(raw)) return true
  return !!oauthRedirectUri.value
})

const resetOAuthState = () => {
  oauthAuthUrl.value = ''
  oauthRedirectUri.value = ''
  oauthCallbackInput.value = ''
}

const switchToOAuth = () => {
  addMethod.value = 'oauth'
  resetOAuthState()
}

let unlistenOAuthUrl = null

onMounted(async () => {
  unlistenOAuthUrl = await listen('oauth-url-generated', event => {
    const url = typeof event.payload === 'string' ? event.payload : ''
    if (!url) return
    oauthAuthUrl.value = url
    try {
      const parsed = new URL(url)
      const redirect = parsed.searchParams.get('redirect_uri')
      if (redirect) {
        oauthRedirectUri.value = redirect
      }
    } catch (err) {
      console.error('Parse oauth url error:', err)
    }
  })
})

onUnmounted(() => {
  if (unlistenOAuthUrl) {
    unlistenOAuthUrl()
    unlistenOAuthUrl = null
  }
})

const handleOAuthLogin = async () => {
  error.value = ''
  isLoading.value = true
  resetOAuthState()

  try {
    // 使用 OAuth 服务器模式，自动完成整个流程
    const account = await invoke('antigravity_start_oauth_login')

    emit('added', account)

  } catch (err) {
    console.error('OAuth login error:', err)
    error.value = formatOAuthError(err)
  } finally {
    isLoading.value = false
  }
}

const generateAuthUrl = async () => {
  error.value = ''
  isManualLoading.value = true

  try {
    const port = Math.floor(Math.random() * 16383) + 49152
    const redirectUri = `http://localhost:${port}/oauth-callback`
    const authUrl = await invoke('antigravity_get_auth_url', { redirectUri })
    oauthAuthUrl.value = authUrl
    oauthRedirectUri.value = redirectUri
  } catch (err) {
    console.error('Generate auth url error:', err)
    error.value = err?.message || err || '生成授权链接失败'
  } finally {
    isManualLoading.value = false
  }
}

const copyAuthUrl = async () => {
  if (!oauthAuthUrl.value) return

  try {
    await navigator.clipboard.writeText(oauthAuthUrl.value)
    window.$notify?.success($t('platform.antigravity.addAccountDialog.authLinkCopied'))
  } catch (err) {
    console.error('Copy auth url error:', err)
    error.value = err?.message || err || '复制授权链接失败'
  }
}

const formatOAuthError = (err) => {
  const message = err?.message || err || ''
  if (message.includes('ANTIGRAVITY_EMAIL_EXISTS')) {
    return $t('platform.antigravity.addAccountDialog.emailExists')
  }
  return message || $t('platform.antigravity.addAccountDialog.oauthExchangeFailed')
}

const handleOAuthExchange = async () => {
  const rawInput = oauthCallbackInput.value.trim()
  if (!rawInput) return

  error.value = ''
  isManualLoading.value = true

  try {
    let code = rawInput
    let redirectUri = oauthRedirectUri.value

    if (/^https?:\/\//i.test(rawInput)) {
      const url = new URL(rawInput)
      code = url.searchParams.get('code') || ''
      redirectUri = `${url.origin}${url.pathname}`
    }

    if (!code) {
      throw new Error($t('platform.antigravity.addAccountDialog.invalidCallback'))
    }

    if (!redirectUri) {
      throw new Error($t('platform.antigravity.addAccountDialog.missingRedirectUri'))
    }

    const account = await invoke('antigravity_exchange_code', { code, redirectUri })
    emit('added', account)
  } catch (err) {
    console.error('Exchange code error:', err)
    error.value = formatOAuthError(err)
  } finally {
    isManualLoading.value = false
  }
}

const handleAdd = async () => {
  if (!canSubmit.value) return

  error.value = ''
  isLoading.value = true

  try {
    await emit('add', email.value.trim(), refreshToken.value.trim())
  } catch (err) {
    console.error('Add account error:', err)
    error.value = err?.message || err || '添加账号失败'
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
