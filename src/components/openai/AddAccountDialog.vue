<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.openai.addAccountDialog.title')"
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
        {{ $t('platform.openai.addAccountDialog.oauthMethod') }}
      </button>
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-1.5 rounded-md px-4 py-2.5 text-sm font-medium transition-all',
          addMethod === 'api'
            ? 'bg-surface text-accent shadow-sm'
            : 'text-text-secondary hover:bg-hover hover:text-text'
        ]"
        @click="addMethod = 'api'"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
        API
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
        {{ $t('platform.openai.addAccountDialog.manualMethod') }}
      </button>
    </div>

    <!-- OAuth 授权方式 -->
    <div v-if="addMethod === 'oauth'" class="animate-fade-in">
      <!-- OAuth Info -->
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-accent/20 bg-accent/10 p-4">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <p class="text-[13px] leading-relaxed text-text-secondary">{{ $t('platform.openai.addAccountDialog.oauthInfo') }}</p>
      </div>

      <!-- OpenAI OAuth Button -->
      <button
        @click="handleOAuthLogin"
        class="flex w-full items-center justify-center gap-2.5 rounded-lg border border-border bg-white px-5 py-3.5 text-[15px] font-medium text-neutral-800 transition-all hover:border-border-strong hover:bg-neutral-50 hover:shadow-sm disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="isLoading"
      >
        <span class="relative inline-flex h-5 w-5 items-center justify-center">
          <svg :style="{ visibility: isLoading ? 'hidden' : 'visible' }" class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997z"/>
          </svg>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
        {{ isLoading ? $t('platform.openai.addAccountDialog.adding') : $t('platform.openai.addAccountDialog.openaiLogin') }}
      </button>

      <!-- Manual OAuth Section -->
      <div class="mt-5 rounded-lg bg-muted p-4">
        <div class="mb-3 text-[13px] font-semibold text-text">{{ $t('platform.openai.addAccountDialog.oauthManualTitle') }}</div>

        <div class="mb-3 flex gap-2.5">
          <button class="btn btn--primary" @click="generateAuthUrl" :disabled="isLoading || isManualLoading">
            {{ $t('platform.openai.addAccountDialog.generateAuthLink') }}
          </button>
        </div>

        <div v-if="oauthAuthUrl" class="mb-3 flex items-center gap-2">
          <input type="text" :value="oauthAuthUrl" readonly class="input flex-1" />
          <button
            class="btn btn--secondary btn--icon shrink-0"
            @click="copyAuthUrl"
            :disabled="isLoading || isManualLoading"
            v-tooltip="$t('platform.openai.addAccountDialog.copyAuthLink')"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
        </div>

        <div class="form-group mb-3">
          <label class="label">{{ $t('platform.openai.addAccountDialog.callbackLabel') }}</label>
          <div class="relative flex items-center">
            <input
              v-model="oauthCallbackInput"
              type="text"
              :placeholder="$t('platform.openai.addAccountDialog.callbackPlaceholder')"
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
            {{ $t('platform.openai.addAccountDialog.callbackHint') }}
          </p>
        </div>

        <button class="btn btn--primary" @click="handleOAuthExchange" :disabled="!canExchange || isLoading || isManualLoading">
          {{ $t('platform.openai.addAccountDialog.exchangeCode') }}
        </button>
      </div>
    </div>

    <!-- 手动添加方式 -->
    <div v-else-if="addMethod === 'manual'" class="animate-fade-in">
      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.email') }}</label>
        <input
          v-model="email"
          type="email"
          :placeholder="$t('platform.openai.addAccountDialog.emailPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.openai.addAccountDialog.refreshToken') }}</label>
        <textarea
          v-model="refreshToken"
          :placeholder="$t('platform.openai.addAccountDialog.refreshTokenPlaceholder')"
          class="input resize-none"
          rows="4"
          :disabled="isLoading"
        ></textarea>
        <p class="mt-1.5 text-xs text-text-muted">
          {{ $t('platform.openai.addAccountDialog.refreshTokenHint') }}
        </p>
      </div>
    </div>

    <!-- API 添加方式 -->
    <div v-else-if="addMethod === 'api'" class="animate-fade-in">
      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.modelProvider') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.modelProviderHint') }})</span></label>
        <input
          v-model="apiForm.model_provider"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.modelProviderPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group">
        <label class="label">{{ $t('platform.openai.addAccountDialog.model') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.modelHint') }})</span></label>
        <input
          v-model="apiForm.model"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.modelPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="flex gap-3">
        <div class="form-group flex-1">
          <label class="label">{{ $t('platform.openai.addAccountDialog.reasoningEffort') }} <span class="text-text-muted">({{ $t('platform.openai.addAccountDialog.reasoningEffortHint') }})</span></label>
          <FloatingDropdown v-model="selectedReasoningEffort" placement="bottom-start" :close-on-select="true">
            <template #trigger="{ isOpen }">
              <button type="button" class="input flex items-center justify-between text-left" :disabled="isLoading">
                <span>{{ selectedReasoningEffort || 'medium' }}</span>
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
          <FloatingDropdown v-model="selectedWireApi" placement="bottom-start" :close-on-select="true">
            <template #trigger="{ isOpen }">
              <button type="button" class="input flex items-center justify-between text-left" :disabled="isLoading">
                <span>{{ selectedWireApi || 'responses' }}</span>
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
          v-model="apiForm.base_url"
          type="text"
          :placeholder="$t('platform.openai.addAccountDialog.baseUrlPlaceholder')"
          class="input"
          :disabled="isLoading"
        />
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.openai.addAccountDialog.apiKey') }}</label>
        <textarea
          v-model="apiForm.key"
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
        v-if="addMethod === 'manual'"
        @click="handleAdd"
        class="btn btn--primary"
        :disabled="!canSubmit || isLoading"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">
            {{ $t('platform.openai.addAccountDialog.add') }}
          </span>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
      </button>
      <button
        v-if="addMethod === 'api'"
        @click="handleAddApi"
        class="btn btn--primary"
        :disabled="!canSubmitApi || isLoading"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">
            {{ $t('platform.openai.addAccountDialog.addApiAccount') }}
          </span>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
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
import FloatingDropdown from '@/components/common/FloatingDropdown.vue'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'add', 'added'])

const handleClose = () => {
  if (isLoading.value) return
  emit('close')
}

const addMethod = ref('oauth') // 'oauth', 'manual', or 'api'
const email = ref('')
const refreshToken = ref('')
const isLoading = ref(false)
const isManualLoading = ref(false)
const error = ref('')
const oauthAuthUrl = ref('')
const oauthRedirectUri = ref('')
const oauthSessionId = ref('')
const oauthCallbackInput = ref('')

// API 表单数据
const apiForm = ref({
  model_provider: '',
  model: '',
  base_url: '',
  key: ''
})
const selectedReasoningEffort = ref('medium')
const selectedWireApi = ref('responses')

const canSubmit = computed(() => {
  if (addMethod.value === 'oauth') return true
  return email.value.trim() && refreshToken.value.trim()
})

const canSubmitApi = computed(() => {
  return apiForm.value.model_provider.trim() &&
         apiForm.value.base_url.trim() &&
         apiForm.value.key.trim()
})

const canExchange = computed(() => {
  const raw = oauthCallbackInput.value.trim()
  if (!raw) return false
  if (/^https?:\/\//i.test(raw)) return true
  return !!oauthSessionId.value
})

const resetOAuthState = () => {
  oauthAuthUrl.value = ''
  oauthRedirectUri.value = ''
  oauthSessionId.value = ''
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
    // 使用自动 OAuth 登录流程（会启动本地服务器监听回调）
    const account = await invoke('openai_start_oauth_login')
    emit('added', account)
  } catch (err) {
    console.error('OAuth login error:', err)
    error.value = formatOAuthError(err)
    isLoading.value = false
  }
}

const generateAuthUrl = async () => {
  error.value = ''
  isManualLoading.value = true

  try {
    const result = await invoke('openai_generate_auth_url')
    oauthAuthUrl.value = result.auth_url
    oauthSessionId.value = result.session_id
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
    window.$notify?.success($t('platform.openai.addAccountDialog.authLinkCopied'))
  } catch (err) {
    console.error('Copy auth url error:', err)
    error.value = err?.message || err || '复制授权链接失败'
  }
}

const formatOAuthError = (err) => {
  const message = err?.message || err || ''
  return message || $t('platform.openai.addAccountDialog.oauthExchangeFailed')
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
      throw new Error($t('platform.openai.addAccountDialog.invalidCallback'))
    }

    const account = await invoke('openai_exchange_code', {
      sessionId: oauthSessionId.value,
      code,
      redirectUri
    })
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
    const account = await invoke('openai_add_account', {
      email: email.value.trim(),
      refreshToken: refreshToken.value.trim()
    })
    emit('added', account)
  } catch (err) {
    console.error('Add account error:', err)
    error.value = err?.message || err || '添加账号失败'
    isLoading.value = false
  }
}

const handleAddApi = async () => {
  if (!canSubmitApi.value) return

  error.value = ''
  isLoading.value = true

  try {
    const account = await invoke('openai_add_api_account', {
      modelProvider: apiForm.value.model_provider.trim(),
      model: apiForm.value.model.trim(),
      reasoningEffort: selectedReasoningEffort.value,
      wireApi: selectedWireApi.value,
      baseUrl: apiForm.value.base_url.trim(),
      key: apiForm.value.key.trim()
    })
    emit('added', account)
  } catch (err) {
    console.error('Add API account error:', err)
    error.value = err?.message || err || '添加 API 账号失败'
    isLoading.value = false
  }
}

const selectReasoningEffort = (value, close) => {
  selectedReasoningEffort.value = value
  close?.()
}

const selectWireApi = (value, close) => {
  selectedWireApi.value = value
  close?.()
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
