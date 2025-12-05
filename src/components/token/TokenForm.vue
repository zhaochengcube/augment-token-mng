<template>
  <div class="token-form-modal">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>{{ isEditing ? $t('tokenForm.editTitle') : $t('tokenForm.addTitle') }}</h2>
          <button class="close-btn" @click="handleCancel">×</button>
        </div>

        <div class="modal-body">
          <!-- Tab Navigation (只在添加模式显示) -->
          <div v-if="!isEditing" class="tab-navigation">
            <button
              :class="['tab-btn', { active: activeTab === 'manual' }]"
              @click="activeTab = 'manual'"
              type="button"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm0 4c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm6 12H6v-1.4c0-2 4-3.1 6-3.1s6 1.1 6 3.1V19z"/>
              </svg>
              {{ $t('tokenForm.manualTab') }}
            </button>
            <button
              :class="['tab-btn', { active: activeTab === 'session' }]"
              @click="activeTab = 'session'"
              type="button"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/>
              </svg>
              {{ $t('tokenForm.sessionTab') }}
            </button>
          </div>

          <!-- Manual Input Tab -->
          <form v-if="activeTab === 'manual'" @submit.prevent="handleSubmit" class="tab-content">
            <div class="form-group">
              <label for="tenantUrl">{{ $t('tokenForm.tenantUrl') }} *</label>
              <input
                id="tenantUrl"
                v-model="formData.tenantUrl"
                type="url"
                placeholder="https://example.augmentcode.com/"
                required
                :disabled="isLoading"
              >
              <div v-if="errors.tenantUrl" class="error-message">{{ errors.tenantUrl }}</div>
            </div>

            <div class="form-group">
              <label for="accessToken">{{ $t('tokenForm.accessToken') }} *</label>
              <input
                id="accessToken"
                v-model="formData.accessToken"
                type="text"
                :placeholder="$t('tokenForm.accessTokenPlaceholder')"
                required
                :disabled="isLoading"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="none"
                spellcheck="false"
              >
              <div v-if="errors.accessToken" class="error-message">{{ errors.accessToken }}</div>
            </div>

            <div class="form-group tag-group">
              <label for="tagName">{{ $t('tokenForm.tagLabel') }} ({{ $t('tokenForm.optional') }})</label>
              <div class="tag-input-row">
                <div class="tag-autocomplete-wrapper">
                  <input
                    id="tagName"
                    v-model="formData.tagName"
                    type="text"
                    class="tag-name-input"
                    :placeholder="$t('tokenForm.tagPlaceholder')"
                    :disabled="isLoading"
                    @input="handleTagInput"
                    @focus="showTagSuggestions = true"
                    @blur="handleTagBlur"
                    @keydown.down.prevent="navigateSuggestions(1)"
                    @keydown.up.prevent="navigateSuggestions(-1)"
                    @keydown.enter.prevent="selectSuggestion(selectedSuggestionIndex)"
                    @keydown.escape="showTagSuggestions = false"
                  >
                  <button
                    v-if="formData.tagName"
                    type="button"
                    class="tag-clear-btn"
                    :title="$t('tokenForm.clearTag')"
                    @click="clearTag"
                    :disabled="isLoading"
                  >
                    ×
                  </button>
                  <Transition name="dropdown">
                    <div v-if="showTagSuggestions && filteredTagSuggestions.length > 0" class="tag-suggestions">
                      <div
                        v-for="(suggestion, index) in filteredTagSuggestions"
                        :key="suggestion.name"
                        :class="['tag-suggestion-item', { selected: index === selectedSuggestionIndex }]"
                        @mousedown.prevent="selectTagSuggestion(suggestion)"
                        @mouseenter="selectedSuggestionIndex = index"
                      >
                        <span
                          class="tag-preview"
                          :style="{ backgroundColor: suggestion.color }"
                        ></span>
                        <span class="tag-suggestion-name">{{ suggestion.name }}</span>
                      </div>
                    </div>
                  </Transition>
                </div>
                <div class="tag-color-wrapper">
                  <button
                    type="button"
                    class="tag-color-display"
                    :style="{ backgroundColor: formData.tagColor }"
                    :title="$t('tokenForm.tagColorPicker')"
                    :aria-label="$t('tokenForm.tagColorPicker')"
                    @click="openTagColorPicker"
                    :disabled="isLoading"
                  ></button>
                  <input
                    ref="tagColorInput"
                    type="color"
                    v-model="formData.tagColor"
                    class="hidden-color-input"
                    tabindex="-1"
                    aria-hidden="true"
                    @input="handleTagColorInput"
                  >
                </div>
              </div>
            </div>

            <div class="form-group">
              <label for="portalUrl">{{ $t('tokenForm.portalUrl') }} ({{ $t('tokenForm.optional') }})</label>
              <input
                id="portalUrl"
                v-model="formData.portalUrl"
                type="url"
                placeholder="https://portal.withorb.com/view?token=xxx"
                :disabled="isLoading"
              >
              <div class="help-text">{{ $t('tokenForm.portalUrlHelp') }}</div>
              <div v-if="errors.portalUrl" class="error-message">{{ errors.portalUrl }}</div>
            </div>

            <div class="form-group">
              <label for="emailNote">{{ $t('tokenForm.emailNote') }} ({{ $t('tokenForm.optional') }})</label>
              <input
                id="emailNote"
                v-model="formData.emailNote"
                type="text"
                :placeholder="$t('tokenForm.emailNotePlaceholder')"
                :disabled="isLoading"
              >
              <div class="help-text">{{ $t('tokenForm.emailNoteHelp') }}</div>
              <div v-if="errors.emailNote" class="error-message">{{ errors.emailNote }}</div>
            </div>

            <div class="form-actions">
              <button type="button" @click="handleCancel" class="btn secondary" :disabled="isLoading">
                {{ $t('tokenForm.cancel') }}
              </button>
              <button type="submit" class="btn primary" :disabled="isLoading || !isFormValid">
                <span v-if="isLoading" class="loading-spinner"></span>
                {{ isLoading ? $t('loading.saving') : (isEditing ? $t('tokenForm.update') : $t('tokenForm.save')) }}
              </button>
            </div>
          </form>

          <!-- Session Import Tab -->
          <div v-else-if="activeTab === 'session'" class="tab-content">
            <div class="session-section">
              <div class="session-header">
                <h3>{{ $t('tokenForm.sessionImportTitle') }}</h3>
              </div>
              <p class="section-description">{{ $t('tokenForm.sessionImportDescription') }}</p>

              <textarea
                v-model="sessionInput"
                :placeholder="$t('tokenForm.sessionPlaceholder')"
                rows="6"
                :disabled="isImportingSession"
                class="session-textarea"
              ></textarea>

              <div class="button-container">
                <button
                  type="button"
                  @click="importFromSession"
                  class="btn primary"
                  :disabled="!sessionInput.trim() || isImportingSession"
                >
                  {{ isImportingSession ? $t('loading.importing') : $t('tokenForm.importSession') }}
                </button>
                <button
                  type="button"
                  @click="openInternalBrowserForAutoImport"
                  class="btn secondary"
                  :disabled="isImportingSession || isOpeningBrowser"
                >
                  {{ $t('tokenGenerator.autoImportSession') }}
                </button>
                <button
                  type="button"
                  @click="handleCancel"
                  class="btn secondary"
                  :disabled="isImportingSession"
                >
                  {{ $t('tokenForm.cancel') }}
                </button>
              </div>

              <!-- Loading State -->
              <div v-if="isImportingSession" class="session-loading">
                <div class="session-spinner"></div>
                <span>{{ sessionImportProgress }}</span>
              </div>
            </div>
          </div>
        </div>


      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

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
const emit = defineEmits(['close', 'success', 'update-token', 'add-token', 'manual-import-completed'])

// Reactive data
const defaultTagColor = '#f97316'

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
const activeTab = ref(props.token ? 'manual' : 'session')

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

<style scoped>
.token-form-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 3000;
}

.modal-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  width: 100%;
  max-width: 500px;
  max-height: 98vh;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
  background: var(--color-surface-muted, #f8f9fa);
}

.modal-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: var(--color-text-muted, #666);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--color-surface-muted, #e9ecef);
  color: var(--color-text-heading, #333);
}

.modal-body {
  padding: 24px;
  max-height: calc(98vh - 120px);
  overflow-y: auto;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: var(--color-text-heading, #333);
  font-size: 14px;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-btn-secondary-border, #ddd);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s;
  box-sizing: border-box;
  background: var(--color-surface, #ffffff);
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-group input:disabled,
.form-group textarea:disabled {
  background: var(--color-btn-tertiary-bg, #f5f5f5);
  color: var(--color-text-muted, #666);
  cursor: not-allowed;
}

.form-group textarea {
  resize: vertical;
  min-height: 80px;
}

.tag-input-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tag-autocomplete-wrapper {
  flex: 1;
  position: relative;
}

.tag-name-input {
  width: 100%;
  padding-right: 32px; /* 为清空按钮留出空间 */
}

.tag-clear-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: var(--color-text-muted, #666);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  padding: 4px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.tag-clear-btn:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-heading, #333);
}

.tag-clear-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.tag-suggestions {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  max-height: 200px;
  overflow-y: auto;
  z-index: 1000;
}

.tag-suggestion-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.tag-suggestion-item:hover {
  background: var(--color-surface-hover, #f3f4f6);
}

.tag-suggestion-item.selected {
  background: var(--color-accent, #3b82f6);
}

.tag-suggestion-item.selected .tag-suggestion-name {
  color: #ffffff;
}

.tag-preview {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1px solid var(--color-border-strong, #d1d5db);
  flex-shrink: 0;
}

.tag-suggestion-name {
  font-size: 14px;
  color: var(--color-text-primary, #374151);
}

.tag-color-wrapper {
  position: relative;
  flex-shrink: 0;
}

.tag-color-display {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 1px solid var(--color-border-strong, #d1d5db);
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  padding: 0;
}

.tag-color-display:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.tag-color-display:not(:disabled):hover,
.tag-color-display:not(:disabled):focus {
  transform: scale(1.05);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  outline: none;
}

.hidden-color-input {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  width: 1px;
  height: 1px;
  pointer-events: none;
}

/* Dropdown transition */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.help-text {
  font-size: 12px;
  color: var(--color-text-muted, #666);
  margin-top: 4px;
}

.error-message {
  color: var(--color-danger-bg, #dc3545);
  font-size: 12px;
  margin-top: 4px;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--color-divider, #e1e5e9);
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 80px;
  justify-content: center;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn.primary {
  background: var(--color-accent, #3b82f6);
  color: var(--color-text-inverse, #ffffff);
}

.btn.primary:hover:not(:disabled) {
  background: var(--color-accent-hover, #2563eb);
}

.btn.secondary {
  background: var(--color-surface-muted, #f8f9fa);
  color: var(--color-text-secondary, #495057);
  border: 1px solid var(--color-border-strong, #dee2e6);
}

.btn.secondary:hover:not(:disabled) {
  background: var(--color-surface-muted, #e9ecef);
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Tab Navigation Styles */
.tab-navigation {
  display: flex;
  gap: 8px;
  justify-content: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.tab-btn {
  padding: 6px 14px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
  border: 1px solid var(--color-border-strong, #d1d5db);
}

.tab-btn svg {
  width: 14px;
  height: 14px;
}

.tab-btn:hover {
  background: var(--color-border, #e5e7eb);
  border-color: var(--color-border-hover, #9ca3af);
}

.tab-btn.active {
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border: 1px solid var(--color-blue-soft-border, #90caf9);
}

.tab-btn.active:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
}

.tab-btn svg {
  flex-shrink: 0;
}

/* Tab Content */
.tab-content {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Session Import Styles */
.session-section {
  padding: 0;
}

.session-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.session-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.section-description {
  margin: 4px 0 12px 0;
  font-size: 13px;
  color: var(--color-text-muted, #6b7280);
  line-height: 1.4;
}

.session-textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  font-size: 13px;
  font-family: monospace;
  resize: vertical;
  background: var(--color-surface, #ffffff);
  min-height: 100px;
  margin-bottom: 12px;
  box-sizing: border-box;
}

.session-textarea:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.session-textarea:disabled {
  background: var(--color-btn-tertiary-bg, #f5f5f5);
  color: var(--color-text-muted, #666);
  cursor: not-allowed;
}

.button-container {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

/* Session Loading State */
.session-loading {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  margin-top: 12px;
  background: var(--color-surface-hover, #f8f9fa);
  border-radius: 8px;
  border: 1px solid var(--color-border, #e5e7eb);
}

.session-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border, #e5e7eb);
  border-top-color: var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

.session-loading span {
  font-size: 14px;
  color: var(--color-text-secondary, #6b7280);
}

/* Dark theme support for tabs */
[data-theme='dark'] .tab-btn {
  background: rgba(148, 163, 184, 0.2);
  color: #cbd5e1;
  border: 1px solid rgba(148, 163, 184, 0.4);
}

[data-theme='dark'] .tab-btn:hover {
  background: rgba(148, 163, 184, 0.3);
  border-color: rgba(148, 163, 184, 0.6);
}

[data-theme='dark'] .tab-btn.active {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
  border: 1px solid rgba(147, 197, 253, 0.4);
}

[data-theme='dark'] .tab-btn.active:hover {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(96, 165, 250, 0.6);
}

</style>
