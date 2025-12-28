<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content proxy-config-modal">
      <div class="modal-header">
        <h3>{{ $t('proxyConfig.title') }}</h3>
        <button @click="$emit('close')" class="modal-close" :title="$t('common.close')">×</button>
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
import { useSettingsStore } from '../../stores/settings'

// Emits
const emit = defineEmits(['close', 'config-saved', 'config-deleted'])

// i18n
const { t } = useI18n()

// Settings store
const settingsStore = useSettingsStore()

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
    // 从store加载配置,强制刷新以获取最新数据
    await settingsStore.loadProxyConfig(true)

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

    // 保存成功后刷新store中的配置
    await settingsStore.loadProxyConfig(true)

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

    // 删除成功后刷新store中的配置
    await settingsStore.loadProxyConfig(true)

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
/* ============================================
   ProxyConfig - Modern Tech Style
   ============================================ */

.proxy-config-modal {
  max-width: 600px;
  width: 90%;
  display: flex;
  flex-direction: column;
}

.proxy-config-modal .modal-body {
  padding: 26px;
  overflow-y: auto;
  flex: 1;
  background: transparent;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.checkbox-group {
  margin-bottom: 0;
}

.checkbox-group label {
  display: flex;
  align-items: center;
  cursor: pointer;
  font-weight: 500;
  color: var(--text);
}

.checkbox-group input[type="checkbox"] {
  margin-right: 10px;
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: var(--accent);
}

.checkbox-group input[type="checkbox"]:disabled {
  cursor: not-allowed;
}

.help-text {
  display: block;
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-muted);
  font-style: italic;
  line-height: 1.5;
}

.optional {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: normal;
  opacity: 0.7;
}

/* 底部按钮区 */
.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  align-items: center;
  padding: 18px 26px;
  border-top: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.footer-left {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  margin-right: auto;
}

.footer-left .btn {
  margin-right: 0;
}

/* 延迟徽章 - 科技风 */
.latency-badge {
  padding: 7px 14px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  font-family: var(--tech-mono-font);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.latency-badge.excellent {
  background: color-mix(in srgb, #10b981 15%, transparent);
  color: #10b981;
  border: 1px solid color-mix(in srgb, #10b981 30%, transparent);
}

.latency-badge.good {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
  border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
}

.latency-badge.fair {
  background: color-mix(in srgb, #f59e0b 15%, transparent);
  color: #f59e0b;
  border: 1px solid color-mix(in srgb, #f59e0b 30%, transparent);
}

.latency-badge.poor {
  background: color-mix(in srgb, #ef4444 15%, transparent);
  color: #ef4444;
  border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
}

.latency-badge.failed {
  background: color-mix(in srgb, var(--text-muted) 15%, transparent);
  color: var(--text-muted);
  border: 1px solid var(--tech-glass-border);
}

/* Footer button sizing */
.modal-footer .btn {
  min-width: 100px;
}

/* 确认对话框 - 科技风 */
.confirm-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(4px);
  z-index: 3000;
}

.confirm-dialog {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  padding: 26px;
  max-width: 400px;
  width: 90%;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
}

.confirm-dialog h3 {
  margin: 0 0 14px 0;
  color: var(--text-strong);
  font-size: 18px;
  font-weight: 600;
}

.confirm-dialog p {
  margin: 0 0 22px 0;
  color: var(--text-muted);
  font-size: 14px;
  line-height: 1.6;
}

.confirm-dialog-buttons {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

/* 暗黑模式样式已通过 CSS 变量自动适配 */
</style>
