<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content proxy-config-modal">
      <div class="modal-header">
        <h3>{{ $t('proxyConfig.title') }}</h3>
        <button @click="$emit('close')" class="close-btn" :title="$t('common.close')">×</button>
      </div>
      
      <div class="modal-body">
        <div class="config-form">
          <!-- 启用代理开关 -->
          <div class="form-group checkbox-group">
            <label>
              <input type="checkbox" v-model="config.enabled" :disabled="isLoading">
              <span>{{ $t('proxyConfig.enableProxy') }}</span>
            </label>
          </div>
          
          <!-- 代理类型选择 -->
          <div class="form-group">
            <label for="proxyType">{{ $t('proxyConfig.proxyType') }}:</label>
            <select
              id="proxyType"
              v-model="config.proxyType"
              :disabled="!config.enabled || isLoading"
            >
              <option value="system">{{ $t('proxyConfig.types.system') }}</option>
              <option value="http">{{ $t('proxyConfig.types.http') }}</option>
              <option value="https">{{ $t('proxyConfig.types.https') }}</option>
              <option value="socks5">{{ $t('proxyConfig.types.socks5') }}</option>
              <option value="custom_url">{{ $t('proxyConfig.types.customUrl') }}</option>
            </select>
            <small class="help-text">{{ getProxyTypeHelp }}</small>
          </div>

          <!-- 代理服务器配置 (仅当选择 http/https/socks5 时显示) -->
          <template v-if="needsServerConfig">
            <div class="form-group">
              <label for="host">{{ $t('proxyConfig.host') }}:</label>
              <input
                id="host"
                v-model="config.host"
                type="text"
                :placeholder="$t('proxyConfig.placeholders.host')"
                :disabled="!config.enabled || isLoading"
              >
            </div>
            
            <div class="form-group">
              <label for="port">{{ $t('proxyConfig.port') }}:</label>
              <input
                id="port"
                v-model.number="config.port"
                type="number"
                min="1"
                max="65535"
                :placeholder="$t('proxyConfig.placeholders.port')"
                :disabled="!config.enabled || isLoading"
              >
            </div>
            
            <!-- 认证信息 (可选) -->
            <div class="form-group">
              <label for="username">
                {{ $t('proxyConfig.username') }} 
                <span class="optional">({{ $t('proxyConfig.optional') }})</span>
              </label>
              <input
                id="username"
                v-model="config.username"
                type="text"
                :placeholder="$t('proxyConfig.placeholders.username')"
                :disabled="!config.enabled || isLoading"
              >
            </div>
            
            <div class="form-group">
              <label for="password">
                {{ $t('proxyConfig.password') }} 
                <span class="optional">({{ $t('proxyConfig.optional') }})</span>
              </label>
              <input
                id="password"
                v-model="config.password"
                type="password"
                :placeholder="$t('proxyConfig.placeholders.password')"
                :disabled="!config.enabled || isLoading"
              >
            </div>
          </template>
          
          <!-- 自定义 URL 配置 (仅当选择 custom_url 时显示) -->
          <template v-if="needsCustomUrl">
            <div class="form-group">
              <label for="customUrl">{{ $t('proxyConfig.customUrl') }}:</label>
              <input
                id="customUrl"
                v-model="config.customUrl"
                type="text"
                :placeholder="$t('proxyConfig.placeholders.customUrl')"
                :disabled="!config.enabled || isLoading"
              >
              <small class="help-text">{{ $t('proxyConfig.detailedHelp.customUrl') }}</small>
            </div>
          </template>
        </div>
      </div>
      
      <div class="modal-footer">
        <div class="footer-left">
          <button 
            @click="testConnection" 
            :class="['btn', 'secondary', { loading: isTesting }]"
            :disabled="!canTest || isTesting || isLoading"
          >
            <svg v-if="!isTesting" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
            {{ $t('proxyConfig.testConnection') }}
          </button>
          
          <span v-if="lastTestResult && lastTestResult.success" :class="['latency-badge', getLatencyClass(lastTestResult.latency)]">
            <span class="status-dot"></span>
            {{ lastTestResult.latency }}ms
          </span>
          <span v-else-if="lastTestResult && !lastTestResult.success" class="latency-badge failed">
            <span class="status-dot"></span>
            {{ $t('proxyConfig.messages.testFailedStatus') }}
          </span>
        </div>
        
        <button 
          @click="saveConfig" 
          :class="['btn', 'primary', { loading: isSaving }]"
          :disabled="!canSave || isSaving || isLoading"
        >
          <svg v-if="!isSaving" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"/>
          </svg>
          {{ $t('proxyConfig.save') }}
        </button>
        
        <button 
          v-if="hasExistingConfig"
          @click="showConfirmDelete = true" 
          class="btn danger"
          :disabled="isLoading || isSaving"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
          {{ $t('proxyConfig.delete') }}
        </button>
      </div>
      
      <!-- 删除确认对话框 -->
      <div v-if="showConfirmDelete" class="confirm-dialog-overlay" @click.self="showConfirmDelete = false">
        <div class="confirm-dialog">
          <h3>{{ $t('proxyConfig.confirmDelete') }}</h3>
          <p>{{ $t('proxyConfig.confirmDeleteMessage') }}</p>
          <div class="confirm-dialog-buttons">
            <button @click="showConfirmDelete = false" class="btn secondary">
              {{ $t('common.cancel') }}
            </button>
            <button @click="deleteConfig" class="btn danger">
              {{ $t('common.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

// Emits
const emit = defineEmits(['close', 'config-saved', 'config-deleted'])

// i18n
const { t } = useI18n()

// Reactive data
const config = ref({
  enabled: false,
  proxyType: 'system',
  host: '',
  port: 7890,
  username: '',
  password: '',
  customUrl: ''
})

const isLoading = ref(false)
const isSaving = ref(false)
const isTesting = ref(false)
const hasExistingConfig = ref(false)
const showConfirmDelete = ref(false)
const lastTestResult = ref(null) // { success: boolean, latency: number }

// Computed
const needsServerConfig = computed(() => {
  return config.value.enabled &&
         ['http', 'https', 'socks5'].includes(config.value.proxyType)
})

const needsCustomUrl = computed(() => {
  return config.value.enabled && config.value.proxyType === 'custom_url'
})

const canTest = computed(() => {
  if (!config.value.enabled) return false
  if (config.value.proxyType === 'system') return true
  if (needsServerConfig.value) {
    return config.value.host.trim() !== '' && config.value.port > 0 && config.value.port <= 65535
  }
  if (needsCustomUrl.value) {
    return config.value.customUrl.trim() !== '' && isValidUrl(config.value.customUrl)
  }
  return true
})

const canSave = computed(() => {
  if (!config.value.enabled) return true // 可以保存禁用状态
  if (config.value.proxyType === 'system') return true
  if (needsServerConfig.value) {
    return config.value.host.trim() !== '' && config.value.port > 0 && config.value.port <= 65535
  }
  if (needsCustomUrl.value) {
    return config.value.customUrl.trim() !== '' && isValidUrl(config.value.customUrl)
  }
  return true
})

const getProxyTypeHelp = computed(() => {
  switch (config.value.proxyType) {
    case 'system':
      return t('proxyConfig.help.system')
    case 'http':
      return t('proxyConfig.help.http')
    case 'https':
      return t('proxyConfig.help.https')
    case 'socks5':
      return t('proxyConfig.help.socks5')
    case 'custom_url':
      return t('proxyConfig.help.customUrl')
    default:
      return ''
  }
})

// 根据延迟返回对应的 CSS 类
const getLatencyClass = (latency) => {
  if (latency < 100) return 'excellent'
  if (latency < 300) return 'good'
  if (latency < 500) return 'fair'
  return 'poor'
}

// Methods
const isValidUrl = (urlString) => {
  if (!urlString) return false
  try {
    const url = new URL(urlString)
    return url.protocol === 'http:' || url.protocol === 'https:'
  } catch {
    return false
  }
}

const loadConfig = async () => {
  isLoading.value = true
  try {
    // 检查配置文件是否存在
    const configExists = await invoke('proxy_config_exists')
    hasExistingConfig.value = configExists

    // 加载配置
    const loadedConfig = await invoke('load_proxy_config')
    if (loadedConfig && loadedConfig.enabled !== undefined) {
      config.value = {
        enabled: loadedConfig.enabled || false,
        proxyType: loadedConfig.proxyType || 'system',  // 现在使用 camelCase
        host: loadedConfig.host || '',
        port: loadedConfig.port || 8080,
        username: loadedConfig.username || '',
        password: '',  // 密码字段不显示，但后端会保留已保存的密码
        customUrl: loadedConfig.customUrl || ''  // 现在使用 camelCase
      }
      // 记录是否有保存的密码
      if (loadedConfig.password) {
        config.value.hasPassword = true
      }
    }
  } catch (error) {
    console.error('Failed to load proxy config:', error)
  } finally {
    isLoading.value = false
  }
}

const saveConfig = async () => {
  isSaving.value = true

  try {
    await invoke('save_proxy_config', {
      enabled: config.value.enabled,
      proxyType: config.value.proxyType,
      host: config.value.host,
      port: config.value.port,
      username: config.value.username || null,
      password: config.value.password || null,
      customUrl: config.value.customUrl || null
    })

    hasExistingConfig.value = true
    window.$notify.success(t('proxyConfig.messages.saveSuccess'))
    emit('config-saved')
    emit('close')
  } catch (error) {
    window.$notify.error(`${t('proxyConfig.messages.saveFailed')}: ${error}`)
  } finally {
    isSaving.value = false
  }
}

const deleteConfig = async () => {
  showConfirmDelete.value = false
  isLoading.value = true

  try {
    await invoke('delete_proxy_config')
    hasExistingConfig.value = false
    window.$notify.success(t('proxyConfig.messages.deleteSuccess'))
    emit('config-deleted')
    emit('close')
  } catch (error) {
    window.$notify.error(`${t('proxyConfig.messages.deleteFailed')}: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const testConnection = async () => {
  isTesting.value = true
  const startTime = performance.now()
  
  try {
    await invoke('test_proxy_config', {
      enabled: config.value.enabled,
      proxyType: config.value.proxyType,
      host: config.value.host,
      port: config.value.port,
      username: config.value.username || null,
      password: config.value.password || null,
      customUrl: config.value.customUrl || null
    })
    
    const endTime = performance.now()
    const latency = Math.round(endTime - startTime)
    lastTestResult.value = { success: true, latency }
    
    window.$notify.success(t('proxyConfig.messages.testSuccess'))
  } catch (error) {
    lastTestResult.value = { success: false, latency: 0 }
    window.$notify.error(`${t('proxyConfig.messages.testFailed')}: ${error}`)
  } finally {
    isTesting.value = false
  }
}

onMounted(() => {
  loadConfig()
})
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
  border-radius: 12px;
  max-width: 90vw;
  max-height: 95vh;
  overflow: hidden;
  position: relative;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.proxy-config-modal {
  max-width: 600px;
  width: 90%;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
  border-radius: 12px 12px 0 0;
  flex-shrink: 0;
}

.modal-header h3 {
  margin: 0;
  color: var(--color-text-primary, #374151);
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
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
  background: var(--color-border, #e5e7eb);
  color: var(--color-text-primary, #374151);
}

.modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
  background: var(--color-surface, #ffffff);
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  font-size: 14px;
}

.form-group input[type="text"],
.form-group input[type="number"],
.form-group input[type="password"],
.form-group select {
  padding: 10px 12px;
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
  transition: all 0.2s;
}

.form-group select {
  padding-right: 36px;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23374151' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  background-size: 12px;
  cursor: pointer;
}

.form-group select:disabled {
  cursor: not-allowed;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%236b7280' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--color-primary, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-group input:disabled,
.form-group select:disabled {
  background: var(--color-surface-alt, #f9fafb);
  color: var(--color-text-muted, #6b7280);
  cursor: not-allowed;
}

.checkbox-group {
  margin-bottom: 0;
}

.checkbox-group label {
  display: flex;
  align-items: center;
  cursor: pointer;
  font-weight: 500;
  color: var(--color-text-primary, #374151);
}

.checkbox-group input[type="checkbox"] {
  margin-right: 10px;
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.checkbox-group input[type="checkbox"]:disabled {
  cursor: not-allowed;
}

.help-text {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
  font-style: italic;
  line-height: 1.4;
}

.optional {
  font-size: 12px;
  color: var(--color-text-muted, #9ca3af);
  font-weight: normal;
}

.modal-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  align-items: center;
  padding: 16px 24px;
  border-top: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
}

.footer-left {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  margin-right: auto;
}

.footer-left .btn {
  margin-right: 0;
}

.latency-badge {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  transition: all 0.2s;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: currentColor;
  display: inline-block;
  flex-shrink: 0;
}

.latency-badge.excellent {
  background: #d1fae5;
  color: #065f46;
  border: 1px solid #a7f3d0;
}

.latency-badge.good {
  background: #dbeafe;
  color: #1e40af;
  border: 1px solid #93c5fd;
}

.latency-badge.fair {
  background: #fef3c7;
  color: #92400e;
  border: 1px solid #fcd34d;
}

.latency-badge.poor {
  background: #fee2e2;
  color: #991b1b;
  border: 1px solid #fca5a5;
}

.latency-badge.failed {
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #d1d5db;
}

.modal-footer .btn {
  min-width: 100px;
  padding: 10px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid transparent;
}

.modal-footer .btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-footer .btn.primary {
  background: var(--color-primary, #3b82f6);
  color: white;
}

.modal-footer .btn.primary:hover:not(:disabled) {
  background: var(--color-primary-dark, #2563eb);
}

.modal-footer .btn.secondary {
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
  border-color: var(--color-border, #e5e7eb);
}

.modal-footer .btn.secondary:hover:not(:disabled) {
  background: var(--color-surface-alt, #f9fafb);
}

.modal-footer .btn.danger {
  background: #ef4444;
  color: white;
}

.modal-footer .btn.danger:hover:not(:disabled) {
  background: #dc2626;
}

.modal-footer .btn.loading {
  opacity: 0.7;
  pointer-events: none;
}

.confirm-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
}

.confirm-dialog {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  padding: 24px;
  max-width: 400px;
  width: 90%;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.confirm-dialog h3 {
  margin: 0 0 12px 0;
  color: var(--color-text-primary, #374151);
  font-size: 18px;
  font-weight: 600;
}

.confirm-dialog p {
  margin: 0 0 20px 0;
  color: var(--color-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.5;
}

.confirm-dialog-buttons {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

/* Dark theme support */
[data-theme='dark'] .modal-content {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .modal-header {
  background: var(--color-surface-alt, #111827);
  border-bottom-color: var(--color-border, #374151);
}

[data-theme='dark'] .modal-header h3 {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .close-btn {
  color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .close-btn:hover {
  background: var(--color-border, #374151);
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .modal-body {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .modal-footer {
  background: var(--color-surface-alt, #111827);
  border-top-color: var(--color-border, #374151);
}

[data-theme='dark'] .form-group label {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .form-group input {
  background: var(--color-surface, #1f2937);
  color: var(--color-text-primary, #f9fafb);
  border-color: var(--color-border, #374151);
}

[data-theme='dark'] .form-group select {
  background-color: var(--color-surface, #1f2937);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23f9fafb' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  background-size: 12px;
  color: var(--color-text-primary, #f9fafb);
  border-color: var(--color-border, #374151);
}

[data-theme='dark'] .form-group input:disabled {
  background: var(--color-surface-alt, #111827);
  color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .form-group select:disabled {
  background-color: var(--color-surface-alt, #111827);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%239ca3af' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  background-size: 12px;
  color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .confirm-dialog {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .confirm-dialog h3 {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .confirm-dialog p {
  color: var(--color-text-secondary, #9ca3af);
}

/* Dark theme latency badge styles */
[data-theme='dark'] .latency-badge.excellent {
  background: rgba(16, 185, 129, 0.2);
  color: #6ee7b7;
  border-color: rgba(110, 231, 183, 0.4);
}

[data-theme='dark'] .latency-badge.good {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
  border-color: rgba(147, 197, 253, 0.4);
}

[data-theme='dark'] .latency-badge.fair {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  border-color: rgba(251, 191, 36, 0.4);
}

[data-theme='dark'] .latency-badge.poor {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  border-color: rgba(252, 165, 165, 0.4);
}

[data-theme='dark'] .latency-badge.failed {
  background: rgba(107, 114, 128, 0.2);
  color: #9ca3af;
  border-color: rgba(156, 163, 175, 0.4);
}
</style>

