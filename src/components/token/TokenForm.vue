<template>
  <BaseModal
    :visible="true"
    :title="isEditing ? $t('tokenForm.editTitle') : $t('tokenForm.addTitle')"
    :show-close="true"
    :close-on-overlay="!isLoading && !isImportingSession"
    :close-on-esc="!isLoading && !isImportingSession"
    :body-scroll="activeTab === 'oauth'"
    modal-class="max-w-[520px]"
    @close="handleCancel"
  >
    <!-- Tab Navigation (只在添加模式显示) -->
    <div v-if="!isEditing" class="mb-4 flex gap-1 justify-center rounded-lg border border-border bg-muted p-1">
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-all',
          activeTab === 'oauth'
            ? 'bg-surface text-text shadow-sm'
            : 'text-text-muted hover:text-text'
        ]"
        @click="activeTab = 'oauth'"
        type="button"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
        </svg>
        {{ $t('tokenForm.oauthTab') }}
      </button>
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-all',
          activeTab === 'manual'
            ? 'bg-surface text-text shadow-sm'
            : 'text-text-muted hover:text-text'
        ]"
        @click="activeTab = 'manual'"
        type="button"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm0 4c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm6 12H6v-1.4c0-2 4-3.1 6-3.1s6 1.1 6 3.1V19z"/>
        </svg>
        {{ $t('tokenForm.manualTab') }}
      </button>
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-all',
          activeTab === 'session'
            ? 'bg-surface text-text shadow-sm'
            : 'text-text-muted hover:text-text'
        ]"
        @click="activeTab = 'session'"
        type="button"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 6h-2.18c.11-.31.18-.65.18-1 0-1.66-1.34-3-3-3-1.05 0-1.96.54-2.5 1.35l-.5.67-.5-.68C10.96 2.54 10.05 2 9 2 7.34 2 6 3.34 6 5c0 .35.07.69.18 1H4c-1.11 0-1.99.89-1.99 2L2 19c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2zm-5-2c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zM9 4c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm11 15H4v-2h16v2zm0-5H4V8h5.08L7 10.83 8.62 12 11 8.76l1-1.36 1 1.36L15.38 12 17 10.83 14.92 8H20v6z"/>
        </svg>
        {{ $t('tokenForm.sessionTab') }}
      </button>
    </div>

    <!-- Manual Input Tab -->
    <form v-if="activeTab === 'manual'" @submit.prevent="handleSubmit" class="space-y-4">
      <div class="form-group">
        <label for="tenantUrl" class="label">{{ $t('tokenForm.tenantUrl') }} *</label>
        <input
          id="tenantUrl"
          v-model="formData.tenantUrl"
          type="url"
          class="input w-full"
          placeholder="https://example.augmentcode.com/"
          required
          :disabled="isLoading"
        >
        <div v-if="errors.tenantUrl" class="text-xs text-error">{{ errors.tenantUrl }}</div>
      </div>

      <div class="form-group">
        <label for="accessToken" class="label">{{ $t('tokenForm.accessToken') }} *</label>
        <input
          id="accessToken"
          v-model="formData.accessToken"
          type="text"
          class="input w-full"
          :placeholder="$t('tokenForm.accessTokenPlaceholder')"
          required
          :disabled="isLoading"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="none"
          spellcheck="false"
        >
        <div v-if="errors.accessToken" class="text-xs text-error">{{ errors.accessToken }}</div>
      </div>

      <div class="form-group">
        <label class="label">{{ $t('tokenForm.tagLabel') }} ({{ $t('tokenForm.optional') }})</label>
        <div class="flex gap-2.5 items-center">
          <div class="dropdown flex-1" @click="showTagSuggestions = true">
            <input
              id="tagName"
              v-model="formData.tagName"
              type="text"
              class="input !pr-9"
              :placeholder="$t('tokenForm.tagPlaceholder')"
              :disabled="isLoading"
              maxlength="20"
              @input="handleTagInput"
              @focus="showTagSuggestions = true"
              @blur="handleTagBlur"
              @click.stop="showTagSuggestions = true"
            >
            <button
              v-if="formData.tagName"
              type="button"
              class="btn btn--ghost btn--icon-sm absolute right-1.5 top-1/2 -translate-y-1/2"
              :title="$t('tokenForm.clearTag')"
              @click="clearTag"
              :disabled="isLoading"
            >
              ×
            </button>
            <Transition name="dropdown">
              <div
                v-if="showTagSuggestions && filteredTagSuggestions.length > 0"
                class="dropdown-menu dropdown-menu--left w-full max-h-[200px] overflow-y-auto"
                @mousedown.prevent
              >
                <div
                  v-for="suggestion in filteredTagSuggestions"
                  :key="suggestion.name"
                  class="dropdown-item"
                  @mousedown.prevent="selectTagSuggestion(suggestion)"
                >
                  <span
                    class="w-4.5 h-4.5 rounded-md shrink-0 shadow-sm"
                    :style="{ backgroundColor: suggestion.color }"
                  ></span>
                  <span class="text-[14px] text-text">{{ suggestion.name }}</span>
                </div>
              </div>
            </Transition>
          </div>
          <div class="relative shrink-0">
            <div
              class="w-[42px] h-[42px] border-2 border-border rounded-full shadow-sm cursor-pointer"
              :style="{ backgroundColor: formData.tagColor }"
              @click="openTagColorPicker"
            ></div>
            <input
              ref="tagColorInput"
              type="color"
              v-model="formData.tagColor"
              :title="$t('tokenForm.tagColorPicker')"
              class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
              @input="handleTagColorInput"
            >
          </div>
        </div>
      </div>

      <div class="form-group">
        <label for="portalUrl" class="label">{{ $t('tokenForm.portalUrl') }} ({{ $t('tokenForm.optional') }})</label>
        <input
          id="portalUrl"
          v-model="formData.portalUrl"
          type="url"
          class="input w-full"
          placeholder="https://portal.withorb.com/view?token=xxx"
          :disabled="isLoading"
        >
        <div class="text-xs text-text-muted mt-1.5">{{ $t('tokenForm.portalUrlHelp') }}</div>
        <div v-if="errors.portalUrl" class="text-xs text-error">{{ errors.portalUrl }}</div>
      </div>

      <div class="form-group">
        <label for="emailNote" class="label">{{ $t('tokenForm.emailNote') }} ({{ $t('tokenForm.optional') }})</label>
        <input
          id="emailNote"
          v-model="formData.emailNote"
          type="text"
          class="input w-full"
          :placeholder="$t('tokenForm.emailNotePlaceholder')"
          :disabled="isLoading"
        >
        <div class="text-xs text-text-muted mt-1.5">{{ $t('tokenForm.emailNoteHelp') }}</div>
        <div v-if="errors.emailNote" class="text-xs text-error">{{ errors.emailNote }}</div>
      </div>

      <div class="flex gap-3 justify-end pt-4 border-t border-border">
        <button type="button" @click="handleCancel" class="btn btn--secondary" :disabled="isLoading">
          {{ $t('tokenForm.cancel') }}
        </button>
        <button type="submit" class="btn btn--primary" :disabled="isLoading || !isFormValid">
          <span v-if="isLoading" class="inline-block w-3.5 h-3.5 border-2 border-transparent border-t-current rounded-full animate-spin"></span>
          {{ isLoading ? $t('loading.saving') : (isEditing ? $t('tokenForm.update') : $t('tokenForm.save')) }}
        </button>
      </div>
    </form>

    <!-- OAuth Tab -->
    <div v-if="activeTab === 'oauth'" class="flex flex-col gap-6">
      <!-- Step 1: Generate Authorization URL -->
      <div class="p-5 bg-muted rounded-lg border border-border">
        <div class="flex items-center justify-between gap-4">
          <h3 class="text-base font-semibold text-text-strong">{{ $t('tokenGenerator.step1') }}</h3>
          <button
            type="button"
            @click="generateAuthUrl"
            class="btn btn--primary"
            :disabled="isGeneratingAuth"
          >
            {{ $t('tokenGenerator.generateUrl') }}
          </button>
        </div>

        <div v-if="oauthData.authUrl" class="mt-4">
          <label class="block mb-2 text-sm font-medium text-text">{{ $t('tokenGenerator.authUrlLabel') }}</label>
          <div class="flex gap-2 items-center">
            <input type="text" :value="oauthData.authUrl" readonly class="input flex-1 bg-surface text-text-muted font-mono text-sm">
            <button type="button" @click="copyAuthUrl" class="btn btn--ghost btn--icon" :title="$t('tokenGenerator.copyUrl')">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
            </button>
            <button type="button" @click="openAuthDialog" class="btn btn--ghost btn--icon" :title="$t('tokenGenerator.openAuthUrl')">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Step 2: Input Authorization Code -->
      <div v-if="oauthData.authUrl" class="p-5 bg-muted rounded-lg border border-border">
        <h3 class="mb-2 text-base font-semibold text-text-strong">{{ $t('tokenGenerator.step2') }}</h3>
        <p class="mb-4 text-sm text-text-muted">{{ $t('tokenForm.oauthStep2Desc') }}</p>
        <input
          type="text"
          v-model="oauthData.authCode"
          :placeholder="$t('tokenGenerator.authCodePlaceholder')"
          :disabled="isGettingToken"
          class="input w-full font-mono text-sm disabled:bg-muted disabled:text-text-muted disabled:cursor-not-allowed"
        >
        <div class="flex gap-3 justify-end mt-4">
          <button
            type="button"
            @click="getAccessToken"
            class="btn btn--primary"
            :disabled="!canGetToken || isGettingToken"
          >
            {{ $t('tokenGenerator.getToken') }}
          </button>
        </div>
      </div>

      <!-- Step 3: Token Result and Save -->
      <div v-if="oauthData.tokenResult" class="p-5 bg-muted rounded-lg border border-border">
        <h3 class="mb-2 text-base font-semibold text-text-strong">{{ $t('tokenGenerator.step3') }}</h3>
        <p class="mb-4 text-sm text-text-muted">{{ $t('tokenForm.oauthStep3Desc') }}</p>

        <div class="flex flex-col gap-4">
          <div class="flex flex-col gap-2">
            <label class="text-sm font-medium text-text">{{ $t('tokenForm.accessToken') }}</label>
            <div class="flex gap-2 items-center">
              <input type="text" :value="oauthData.tokenResult.access_token" readonly class="input flex-1 bg-surface text-text-muted font-mono text-sm">
              <button type="button" @click="copyAccessToken" class="btn btn--ghost btn--icon">
                <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-sm font-medium text-text">{{ $t('tokenForm.tenantUrl') }}</label>
            <input type="text" :value="oauthData.tokenResult.tenant_url" readonly class="input bg-surface text-text-muted font-mono text-sm">
          </div>

          <!-- Optional Fields -->
          <div class="space-y-1.5">
            <label for="oauth-portalUrl" class="block text-sm font-medium text-text">{{ $t('tokenForm.portalUrl') }} ({{ $t('tokenForm.optional') }})</label>
            <input id="oauth-portalUrl" v-model="oauthData.portalUrl" type="url" class="input w-full" :placeholder="$t('tokenForm.portalUrlPlaceholder')">
          </div>

          <div class="space-y-1.5">
            <label for="oauth-emailNote" class="block text-sm font-medium text-text">{{ $t('tokenForm.emailNote') }} ({{ $t('tokenForm.optional') }})</label>
            <input id="oauth-emailNote" v-model="oauthData.emailNote" type="text" class="input w-full" :placeholder="$t('tokenForm.emailNotePlaceholder')">
          </div>

          <div class="space-y-1.5">
            <label for="oauth-tagName" class="block text-sm font-medium text-text">{{ $t('tokenForm.tagLabel') }} ({{ $t('tokenForm.optional') }})</label>
            <div class="flex items-center gap-3">
              <input id="oauth-tagName" v-model="oauthData.tagName" type="text" class="input flex-1" :placeholder="$t('tokenForm.tagPlaceholder')">
              <input type="color" v-model="oauthData.tagColor" class="w-8 h-8 rounded border border-border cursor-pointer" :title="$t('tokenForm.tagColor')">
            </div>
          </div>
        </div>

        <div class="flex gap-3 justify-end mt-4 pt-4 border-t border-border">
          <button type="button" @click="saveOAuthToken" class="btn btn--primary">
            {{ $t('tokenForm.saveToken') }}
          </button>
          <button type="button" @click="resetOAuthForm" class="btn btn--secondary">
            {{ $t('tokenForm.reset') }}
          </button>
          <button type="button" @click="handleCancel" class="btn btn--secondary">
            {{ $t('tokenForm.cancel') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Session Import Tab -->
    <div v-else-if="activeTab === 'session'" class="space-y-4">
      <div>
        <h3 class="text-base font-semibold text-text-strong mb-1.5">{{ $t('tokenForm.sessionImportTitle') }}</h3>
        <p class="text-sm text-text-muted leading-relaxed">{{ $t('tokenForm.sessionImportDescription') }}</p>
      </div>

      <textarea
        v-model="sessionInput"
        :placeholder="$t('tokenForm.sessionPlaceholder')"
        rows="6"
        :disabled="isImportingSession"
        class="input w-full font-mono text-sm resize-y min-h-[100px] disabled:bg-muted disabled:text-text-muted disabled:cursor-not-allowed"
      ></textarea>

      <div class="flex gap-3 justify-end">
        <button
          type="button"
          @click="importFromSession"
          class="btn btn--primary"
          :disabled="!sessionInput.trim() || isImportingSession"
        >
          {{ isImportingSession ? $t('loading.importing') : $t('tokenForm.importSession') }}
        </button>
        <button
          type="button"
          @click="openInternalBrowserForAutoImport"
          class="btn btn--primary"
          :disabled="isImportingSession || isOpeningBrowser"
        >
          {{ $t('tokenGenerator.autoImportSession') }}
        </button>
        <button
          type="button"
          @click="handleCancel"
          class="btn btn--secondary"
          :disabled="isImportingSession"
        >
          {{ $t('tokenForm.cancel') }}
        </button>
      </div>

      <!-- Loading State -->
      <div v-if="isImportingSession" class="flex items-center gap-3 p-4 mt-3 bg-muted rounded-lg border border-border">
        <div class="w-5 h-5 shrink-0 border-2 border-border border-t-accent rounded-full animate-spin"></div>
        <span class="text-sm text-text-muted">{{ sessionImportProgress }}</span>
      </div>
    </div>
  </BaseModal>

  <ExternalLinkDialog
    :show="showAuthUrlDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="oauthData.authUrl"
    :browser-title="$t('dialogs.authUrlTitle')"
    @close="showAuthUrlDialog = false"
  />
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'

const { t } = useI18n()

const resolveCssVar = (name, fallback) => {
  if (typeof window === 'undefined') return fallback
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return value || fallback
}

// Props
const props = defineProps({
  token: {
    type: Object,
    default: null
  },
  allTokens: {
    type: Array,
    default: () => []
  }
})

// Emits
const emit = defineEmits([
  'close',
  'success',
  'update-token',
  'add-token',
  'auto-import-completed',
  'manual-import-completed'
])

// Reactive data
const defaultTagColor = resolveCssVar('--tag-default', '#f97316')

const tagColorInput = ref(null)

const formData = ref({
  tenantUrl: '',
  accessToken: '',
  portalUrl: '',
  emailNote: '',
  tagName: '',
  tagColor: defaultTagColor
})

const errors = ref({
  tenantUrl: '',
  accessToken: '',
  portalUrl: '',
  emailNote: ''
})

const isLoading = ref(false)

// Tab state
const activeTab = ref(props.token ? 'manual' : 'oauth')

// OAuth data
const oauthData = ref({
  authUrl: '',
  authCode: '',
  tokenResult: null,
  portalUrl: '',
  emailNote: '',
  tagName: '',
  tagColor: defaultTagColor
})
const isGeneratingAuth = ref(false)
const isGettingToken = ref(false)
const showAuthUrlDialog = ref(false)

// Session import data
const sessionInput = ref('')
const isImportingSession = ref(false)
const sessionImportProgress = ref('')
const isOpeningBrowser = ref(false)

// Tag autocomplete state
const showTagSuggestions = ref(false)
const selectedSuggestionIndex = ref(0)

// Computed properties
const isEditing = computed(() => !!props.token)

const isFormValid = computed(() => {
  return formData.value.tenantUrl.trim() &&
         formData.value.accessToken.trim() &&
         !errors.value.tenantUrl &&
         !errors.value.accessToken &&
         !errors.value.portalUrl &&
         !errors.value.emailNote
})

// 从所有 tokens 中提取已使用的标签
const existingTags = computed(() => {
  const tagMap = new Map()

  props.allTokens.forEach(token => {
    if (token.tag_name && token.tag_name.trim()) {
      const originalName = token.tag_name.trim()
      const normalizedName = originalName.toLowerCase() // 统一转小写用于去重
      const tagColor = token.tag_color || defaultTagColor

      // 如果标签已存在(不区分大小写),保留第一次出现的原始名称和颜色
      if (!tagMap.has(normalizedName)) {
        tagMap.set(normalizedName, { name: originalName, color: tagColor })
      }
    }
  })

  // 转换为数组并按名称排序
  return Array.from(tagMap.values())
    .sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
})

// 根据输入过滤标签建议
const filteredTagSuggestions = computed(() => {
  const input = formData.value.tagName.trim().toLowerCase()

  // 如果没有输入,显示所有标签
  if (!input) {
    return existingTags.value
  }

  // 过滤匹配的标签
  return existingTags.value.filter(tag =>
    tag.name.toLowerCase().includes(input)
  )
})

// OAuth computed properties
const canGetToken = computed(() => {
  return oauthData.value.authUrl && oauthData.value.authCode.trim().length > 0
})

// Watch for token prop changes (for editing)
watch(() => props.token, (newToken) => {
  if (newToken) {
    activeTab.value = 'manual'
    formData.value = {
      tenantUrl: newToken.tenant_url || '',
      accessToken: newToken.access_token || '',
      portalUrl: newToken.portal_url || '',
      emailNote: newToken.email_note || '',
      tagName: newToken.tag_name || '',
      tagColor: newToken.tag_color || defaultTagColor
    }
  } else {
    activeTab.value = 'session'
    // Reset form for adding new token
    formData.value = {
      tenantUrl: '',
      accessToken: '',
      portalUrl: '',
      emailNote: '',
      tagName: '',
      tagColor: defaultTagColor
    }
  }
  // Clear errors when token changes
  errors.value = {
    tenantUrl: '',
    accessToken: '',
    portalUrl: '',
    emailNote: ''
  }
}, { immediate: true })

// Methods
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const validateForm = () => {
  errors.value = {
    tenantUrl: '',
    accessToken: '',
    portalUrl: '',
    emailNote: ''
  }

  // Validate tenant URL
  if (!formData.value.tenantUrl.trim()) {
    errors.value.tenantUrl = t('validation.tenantUrlRequired')
  } else {
    try {
      new URL(formData.value.tenantUrl)
    } catch {
      errors.value.tenantUrl = t('validation.invalidUrl')
    }
  }

  // Validate access token
  if (!formData.value.accessToken.trim()) {
    errors.value.accessToken = t('validation.accessTokenRequired')
  }

  // Validate portal URL (optional)
  if (formData.value.portalUrl && formData.value.portalUrl.trim()) {
    try {
      new URL(formData.value.portalUrl)
    } catch {
      errors.value.portalUrl = t('validation.invalidUrl')
    }
  }

  return !errors.value.tenantUrl && !errors.value.accessToken && !errors.value.portalUrl
}

const handleSubmit = async () => {
  if (!validateForm()) {
    return
  }

  isLoading.value = true

  try {
    const tagName = formData.value.tagName ? formData.value.tagName.trim() : ''
    const tokenData = {
      tenantUrl: formData.value.tenantUrl.trim(),
      accessToken: formData.value.accessToken.trim(),
      portalUrl: formData.value.portalUrl ? formData.value.portalUrl.trim() || null : null,
      emailNote: formData.value.emailNote ? formData.value.emailNote.trim() || null : null,
      tagName: tagName || null,
      tagColor: tagName ? (formData.value.tagColor || defaultTagColor) : null
    }

    if (isEditing.value) {
      // Update existing token - 通知父组件更新内存中的数据
      emit('update-token', {
        id: props.token.id,
        ...tokenData
      })
    } else {
      // Add new token - 通知父组件添加到内存中的数据
      emit('add-token', tokenData)
    }

    emit('success')
    emit('close')
  } catch (error) {
    showStatus(`${isEditing.value ? t('messages.updateFailed') : t('messages.saveFailed')}: ${error}`, 'error')
  } finally {
    isLoading.value = false
  }
}

const handleCancel = () => {
  emit('close')
}

// ========== OAuth Methods ==========
const generateAuthUrl = async () => {
  isGeneratingAuth.value = true
  try {
    const url = await invoke('generate_augment_auth_url')
    oauthData.value.authUrl = url
  } catch (error) {
    console.error('Failed to generate auth URL:', error)
    showStatus(t('messages.generateUrlError'), 'error')
  } finally {
    isGeneratingAuth.value = false
  }
}

const copyToClipboard = async (text) => {
  if (!text) return false
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch (error) {
    console.warn('Clipboard API failed, falling back:', error)
  }

  try {
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.select()
    const success = document.execCommand('copy')
    document.body.removeChild(textArea)
    return success
  } catch (error) {
    console.error('Fallback copy failed:', error)
    return false
  }
}

const copyAuthUrl = async () => {
  const success = await copyToClipboard(oauthData.value.authUrl)
  showStatus(
    success ? t('messages.copySuccess') : t('messages.copyFailed'),
    success ? 'success' : 'error'
  )
}

const openAuthDialog = () => {
  if (!oauthData.value.authUrl) return
  showAuthUrlDialog.value = true
}

const getAccessToken = async () => {
  isGettingToken.value = true
  try {
    const result = await invoke('get_augment_token', { code: oauthData.value.authCode })
    oauthData.value.tokenResult = result
    // Auto-fill email if returned
    if (result.email) {
      oauthData.value.emailNote = result.email
    }
  } catch (error) {
    console.error('Failed to get access token:', error)
    // TODO: Show error message
  } finally {
    isGettingToken.value = false
  }
}

const copyAccessToken = async () => {
  const success = await copyToClipboard(oauthData.value.tokenResult.access_token)
  showStatus(
    success ? t('messages.accessTokenCopied') : t('messages.copyFailed'),
    success ? 'success' : 'error'
  )
}

const saveOAuthToken = async () => {
  if (!oauthData.value.tokenResult) return

  const tagName = oauthData.value.tagName ? oauthData.value.tagName.trim() : ''
  const tokenData = {
    tenantUrl: oauthData.value.tokenResult.tenant_url,
    accessToken: oauthData.value.tokenResult.access_token,
    portalUrl: oauthData.value.portalUrl.trim(),
    emailNote: oauthData.value.emailNote.trim(),
    tagName: tagName,
    tagColor: tagName ? oauthData.value.tagColor : null
  }

  emit('add-token', tokenData)
  resetOAuthForm()
  emit('close')
}

const resetOAuthForm = () => {
  oauthData.value = {
    authUrl: '',
    authCode: '',
    tokenResult: null,
    portalUrl: '',
    emailNote: '',
    tagName: '',
    tagColor: defaultTagColor
  }
}
// ========== OAuth Methods End ==========

const openTagColorPicker = () => {
  if (isLoading.value) {
    return
  }

  if (tagColorInput.value) {
    tagColorInput.value.click()
  }
}

const handleTagColorInput = (event) => {
  const value = event?.target?.value
  if (typeof value === 'string' && value) {
    formData.value.tagColor = value
  }
}

// 标签输入处理
const handleTagInput = () => {
  showTagSuggestions.value = true
  selectedSuggestionIndex.value = 0
}

// 标签输入框失焦处理
const handleTagBlur = () => {
  // 延迟关闭,以便点击建议项时能够触发
  setTimeout(() => {
    showTagSuggestions.value = false
  }, 200)
}

// 导航建议列表
const navigateSuggestions = (direction) => {
  if (filteredTagSuggestions.value.length === 0) return

  selectedSuggestionIndex.value += direction

  if (selectedSuggestionIndex.value < 0) {
    selectedSuggestionIndex.value = filteredTagSuggestions.value.length - 1
  } else if (selectedSuggestionIndex.value >= filteredTagSuggestions.value.length) {
    selectedSuggestionIndex.value = 0
  }
}

// 选择建议项
const selectSuggestion = (index) => {
  if (index >= 0 && index < filteredTagSuggestions.value.length) {
    selectTagSuggestion(filteredTagSuggestions.value[index])
  }
}

// 选择标签建议
const selectTagSuggestion = (suggestion) => {
  formData.value.tagName = suggestion.name
  formData.value.tagColor = suggestion.color
  showTagSuggestions.value = false
}

// 清空标签
const clearTag = () => {
  formData.value.tagName = ''
  formData.value.tagColor = defaultTagColor
  showTagSuggestions.value = false
}

// Session import method
const importFromSession = async () => {
  if (!sessionInput.value.trim()) {
    showStatus(t('messages.sessionRequired'), 'warning')
    return
  }

  isImportingSession.value = true
  sessionImportProgress.value = t('messages.sessionImportStarting')
  showStatus(t('messages.importingSession'), 'info')

  try {
    const authSession = sessionInput.value.trim()
    const result = await invoke('add_token_from_session', { session: authSession })

    // 创建包含 auth_session 的 tokenData
    const tokenData = {
      tenantUrl: result.tenant_url,
      accessToken: result.access_token,
      portalUrl: null,  // Session 导入不再获取 portal_url
      emailNote: result.email || null,  // 从 get-models API 获取的邮箱
      authSession: authSession,  // 保存 auth_session
      suspensions: null,  // Session 导入不再获取 suspensions
      creditsBalance: null,  // Session 导入不再获取余额
      expiryDate: null,  // Session 导入不再获取过期时间
      banStatus: 'ACTIVE'  // Session 导入默认设置为 ACTIVE 状态
    }

    // 保存 token
    sessionImportProgress.value = t('messages.sessionImportSavingToken')

    // 通知父组件添加 token
    emit('add-token', tokenData)

    // 等待一下让父组件处理完 add-token 事件
    await nextTick()

    // 通知父组件处理成功逻辑
    emit('manual-import-completed')

    // 清空输入
    sessionInput.value = ''
    sessionImportProgress.value = ''
  } catch (error) {
    sessionImportProgress.value = t('messages.sessionImportFailed')
    // 映射后端错误标识符到 i18n key
    let errorMessage = error
    if (error.includes('SESSION_ERROR_OR_ACCOUNT_BANNED')) {
      errorMessage = t('messages.sessionErrorOrAccountBanned')
    }
    showStatus(`${t('messages.error')}: ${errorMessage}`, 'error')
  } finally {
    isImportingSession.value = false
  }
}

// 打开内置浏览器进行自动导入
const openInternalBrowserForAutoImport = async () => {
  if (isOpeningBrowser.value) {
    return // 防止重复点击
  }
  
  isOpeningBrowser.value = true
  try {
    // 打开登录页面,登录后会跳转到 auth.augmentcode.com
    await invoke('open_internal_browser', {
      url: 'https://app.augmentcode.com/',
      title: t('tokenGenerator.autoImportBrowserTitle')
    })
  } catch (error) {
    showStatus(`${t('messages.error')}: ${error}`, 'error')
  } finally {
    // 延迟重置状态，避免窗口创建过程中再次点击
    setTimeout(() => {
      isOpeningBrowser.value = false
    }, 1000)
  }
}

// Event listeners
let unlistenProgress = null
let unlistenAutoImported = null
let unlistenAutoImportFailed = null

onMounted(async () => {
  // 监听 Session 导入进度事件
  unlistenProgress = await listen('session-import-progress', (event) => {
    console.log('Progress event received in TokenForm:', event.payload)
    // 后端发送的是 i18n key,需要转换为翻译文本
    sessionImportProgress.value = t('messages.' + event.payload)
  })

  // 监听 Session 自动导入成功事件
  unlistenAutoImported = await listen('session-auto-imported', async (event) => {
    console.log('Session auto-imported in TokenForm:', event.payload)

    // 添加 token
    if (event.payload.token) {
      const tokenData = {
        tenantUrl: event.payload.token.tenant_url,
        accessToken: event.payload.token.access_token,
        portalUrl: null,  // Session 导入不再获取 portal_url
        emailNote: event.payload.token.email || null,  // 从 get-models API 获取的邮箱
        authSession: event.payload.session || null,  // 保存 auth_session
        suspensions: null,  // Session 导入不再获取 suspensions
        creditsBalance: null,  // Session 导入不再获取余额
        expiryDate: null  // Session 导入不再获取过期时间
      }

      // 通知父组件添加 token
      emit('add-token', tokenData)

      // 等待一下让父组件处理完 add-token 事件
      await nextTick()

      // 通知父组件处理成功逻辑(显示提示、关闭对话框等)
      // 父组件会根据 lastAddTokenSuccess 判断是否真的成功
      emit('auto-import-completed')
    }
  })

  // 监听 Session 自动导入失败事件
  unlistenAutoImportFailed = await listen('session-auto-import-failed', (event) => {
    console.error('Session auto-import failed in TokenForm:', event.payload)
    // 映射后端错误标识符到 i18n key
    let errorMessage = event.payload.error
    if (errorMessage.includes('SESSION_ERROR_OR_ACCOUNT_BANNED')) {
      errorMessage = t('messages.sessionErrorOrAccountBanned')
    }
    showStatus(t('messages.sessionAutoImportFailed') + ': ' + errorMessage, 'error')
  })
})

onUnmounted(() => {
  // 清理事件监听器
  if (unlistenProgress) {
    unlistenProgress()
  }
  if (unlistenAutoImported) {
    unlistenAutoImported()
  }
  if (unlistenAutoImportFailed) {
    unlistenAutoImportFailed()
  }
})
</script>
