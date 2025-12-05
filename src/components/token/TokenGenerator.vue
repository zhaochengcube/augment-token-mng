<template>
  <div class="token-generator">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>生成Augment Token</h2>
          <button class="close-btn" @click="$emit('close')">×</button>
        </div>
        
        <div class="modal-body">
          <!-- Step 1: Generate Authorization URL -->
          <div class="section">
            <h3>步骤 1: 生成Augment授权URL</h3>
            <button 
              @click="generateAuthUrl" 
              :class="['btn', 'primary', { loading: isGenerating }]"
              :disabled="isGenerating"
            >
              生成Augment授权URL
            </button>
            
            <div v-if="authUrl" class="url-section">
              <p>Augment授权URL已生成:</p>
              <div class="url-input-container">
                <input
                  type="text"
                  :value="authUrl"
                  readonly
                  ref="authUrlInput"
                >
              </div>
              <div class="url-buttons">
                <button @click="copyAuthUrl" class="btn secondary">复制</button>
                <button @click="openAuthUrl" class="btn secondary" title="在系统浏览器中打开">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
                  </svg>
                  外部打开
                </button>
                <button @click="openAuthUrlInternal" class="btn primary" title="在内置浏览器中打开">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 4H5c-1.11 0-2 .9-2 2v12c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm-5 14H5V8h9v10z"/>
                  </svg>
                  内置打开
                </button>
              </div>
            </div>
          </div>

          <!-- Step 2: Enter Authorization Code -->
          <div class="section">
            <h3>步骤 2: 输入授权码</h3>
            <textarea 
              v-model="authCode"
              placeholder="在此粘贴授权码JSON..." 
              rows="4"
            ></textarea>
            <div class="button-container">
              <button 
                @click="getAccessToken" 
                :class="['btn', 'primary', { loading: isGettingToken }]"
                :disabled="!canGetToken || isGettingToken"
              >
                获取访问令牌
              </button>
            </div>
          </div>

          <!-- Step 3: Access Token -->
          <div class="section" v-if="tokenResult">
            <h3>步骤 3: Augment访问令牌</h3>
            <div class="token-section">
              <div class="result-container">
                <label>访问令牌:</label>
                <div class="token-container">
                  <input 
                    type="text" 
                    :value="tokenResult.access_token" 
                    readonly
                    ref="accessTokenInput"
                  >
                  <button @click="copyAccessToken" class="btn secondary">复制</button>
                </div>
              </div>
              <div class="result-container">
                <label>租户URL:</label>
                <div class="token-container">
                  <input 
                    type="text" 
                    :value="tokenResult.tenant_url" 
                    readonly
                    ref="tenantUrlInput"
                  >
                  <button @click="copyTenantUrl" class="btn secondary">复制</button>
                </div>
              </div>

              <!-- Additional Fields -->
              <div class="additional-fields">
                <div class="field-container">
                  <label>Portal URL:</label>
                  <input
                    type="text"
                    v-model="portalUrl"
                    placeholder="请输入 Portal 地址（可选）"
                    class="field-input"
                  >
                </div>
                <div class="field-container">
                  <label>邮箱备注:</label>
                  <input
                    type="text"
                    v-model="emailNote"
                    placeholder="请输入邮箱相关备注（可选）"
                    class="field-input"
                  >
                </div>
              </div>

              <div class="button-container">
                <button @click="saveAndClose" class="btn success">保存并关闭</button>
              </div>
            </div>
          </div>
        </div>


      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Emits
const emit = defineEmits(['close', 'token-saved', 'save-token'])

// Reactive data
const authUrl = ref('')
const authCode = ref('')
const tokenResult = ref(null)
const isGenerating = ref(false)
const isGettingToken = ref(false)
const portalUrl = ref('')
const emailNote = ref('')

// Template refs
const authUrlInput = ref(null)
const accessTokenInput = ref(null)
const tenantUrlInput = ref(null)

// Computed properties
const canGetToken = computed(() => {
  return authUrl.value && authCode.value.trim().length > 0
})

// Methods
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const copyToClipboard = async (text) => {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch (err) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.select()
    const success = document.execCommand('copy')
    document.body.removeChild(textArea)
    return success
  }
}

const generateAuthUrl = async () => {
  isGenerating.value = true
  showStatus('正在生成Augment授权URL...', 'info')
  
  try {
    const url = await invoke('generate_augment_auth_url')
    authUrl.value = url
    showStatus('Augment授权URL生成成功!', 'success')
  } catch (error) {
    showStatus(`错误: ${error}`, 'error')
  } finally {
    isGenerating.value = false
  }
}

const copyAuthUrl = async () => {
  const success = await copyToClipboard(authUrl.value)
  showStatus(
    success ? 'URL已复制到剪贴板!' : '复制URL失败',
    success ? 'success' : 'error'
  )
}

const openAuthUrl = async () => {
  try {
    await invoke('open_url', { url: authUrl.value })
    showStatus('正在浏览器中打开授权URL...', 'info')
  } catch (error) {
    showStatus(`打开URL错误: ${error}`, 'error')
  }
}

const openAuthUrlInternal = async () => {
  try {
    const windowLabel = await invoke('open_internal_browser', {
      url: authUrl.value,
      title: 'Augment OAuth 授权'
    })
    showStatus('已在内置浏览器中打开授权URL', 'info')
  } catch (error) {
    showStatus(`打开内置浏览器失败: ${error}`, 'error')
  }
}

const getAccessToken = async () => {
  if (!authCode.value.trim()) {
    showStatus('请输入授权码', 'error')
    return
  }
  
  isGettingToken.value = true
  showStatus('正在获取Augment访问令牌...', 'info')
  
  try {
    const result = await invoke('get_augment_token', { code: authCode.value.trim() })
    tokenResult.value = result
    showStatus('Augment访问令牌获取成功!', 'success')
  } catch (error) {
    showStatus(`错误: ${error}`, 'error')
  } finally {
    isGettingToken.value = false
  }
}

const copyAccessToken = async () => {
  const success = await copyToClipboard(tokenResult.value.access_token)
  showStatus(
    success ? '访问令牌已复制到剪贴板!' : '复制令牌失败',
    success ? 'success' : 'error'
  )
}

const copyTenantUrl = async () => {
  const success = await copyToClipboard(tokenResult.value.tenant_url)
  showStatus(
    success ? '租户URL已复制到剪贴板!' : '复制URL失败',
    success ? 'success' : 'error'
  )
}

const saveAndClose = async () => {
  try {
    // 通知父组件保存 token 到内存
    emit('save-token', {
      tenantUrl: tokenResult.value.tenant_url,
      accessToken: tokenResult.value.access_token,
      portalUrl: portalUrl.value.trim() || null,
      emailNote: emailNote.value.trim() || null
    })
    showStatus('Token已保存', 'success')
    emit('token-saved')
    setTimeout(() => {
      emit('close')
    }, 1000)
  } catch (error) {
    showStatus(`保存失败: ${error}`, 'error')
  }
}

// Initialize
// showStatus('准备生成OAuth令牌', 'info')
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 8px;
  width: 90%;
  max-width: 600px;
  max-height: 95vh;
  overflow-y: auto;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #eee;
}

.modal-header h2 {
  margin: 0;
  color: var(--color-text-heading, #333);
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-muted, #666);
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: var(--color-text-heading, #333);
}

.modal-body {
  padding: 20px;
}

.section {
  margin-bottom: 30px;
}

.section h3 {
  margin: 0 0 15px 0;
  color: var(--color-text-heading, #333);
  font-size: 18px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.btn.primary {
  background: var(--color-blue-primary, #007bff);
  color: var(--color-text-inverse, #ffffff);
}

.btn.primary:hover:not(:disabled) {
  background: var(--color-blue-primary-hover, #0056b3);
}

.btn.secondary {
  background: var(--color-text-muted, #6c757d);
  color: var(--color-text-inverse, #ffffff);
}

.btn.secondary:hover {
  background: #545b62;
}

.btn.success {
  background: var(--color-success-bg, #28a745);
  color: var(--color-text-inverse, #ffffff);
}

.btn.success:hover {
  background: var(--color-success-bg-hover, #1e7e34);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn.loading {
  position: relative;
}

.btn.loading::after {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  margin: auto;
  border: 2px solid transparent;
  border-top-color: var(--color-surface, #ffffff);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.url-section, .token-section {
  margin-top: 15px;
  padding: 0;
  background: transparent;
  border-radius: 4px;
  text-align: left;
}

.url-section p, .token-section p {
  margin: 0 0 10px 0;
  text-align: left;
  font-weight: 500;
  color: var(--color-text-heading, #333);
}

.url-input-container {
  margin-top: 10px;
}

.url-input-container input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-btn-secondary-border, #ddd);
  border-radius: 4px;
  font-family: monospace;
  font-size: 12px;
}

.url-buttons {
  display: flex;
  gap: 10px;
  margin-top: 10px;
  justify-content: flex-start;
}

.url-buttons .btn {
  padding: 8px 16px;
  font-size: 14px;
  min-width: 80px;
}

.token-container {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  align-items: center;
}

.token-container input {
  flex: 1;
  padding: 8px 10px;
  border: 1px solid var(--color-btn-secondary-border, #ddd);
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
  min-width: 0;
}

.token-container .btn {
  flex-shrink: 0;
  padding: 8px 12px;
  font-size: 12px;
  white-space: nowrap;
}

.result-container {
  margin-bottom: 15px;
}

.result-container label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
  color: var(--color-text-heading, #333);
}

textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid var(--color-btn-secondary-border, #ddd);
  border-radius: 4px;
  font-family: monospace;
  font-size: 12px;
  resize: vertical;
}

.additional-fields {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--color-divider, #e1e5e9);
}

.field-container {
  margin-bottom: 15px;
}

.field-container label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  font-size: 14px;
}

.field-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s ease;
}

.field-input:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.field-input::placeholder {
  color: var(--color-text-soft, #9ca3af);
}

.button-container {
  margin-top: 15px;
}


</style>
