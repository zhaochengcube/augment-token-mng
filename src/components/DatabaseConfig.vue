<template>
  <div class="database-config-modal">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>{{ $t('databaseConfig.title') }}</h2>
          <button class="close-btn" @click="$emit('close')">×</button>
        </div>
        
        <div class="modal-body">
          <div class="config-form">
            <div class="form-group">
              <label for="host">{{ $t('databaseConfig.host') }}:</label>
              <input
                id="host"
                v-model="config.host"
                type="text"
                :placeholder="$t('databaseConfig.placeholders.host')"
                :disabled="isLoading"
              >
            </div>

            <div class="form-group">
              <label for="port">{{ $t('databaseConfig.port') }}:</label>
              <input
                id="port"
                v-model.number="config.port"
                type="number"
                :placeholder="$t('databaseConfig.placeholders.port')"
                :disabled="isLoading"
              >
            </div>

            <div class="form-group">
              <label for="database">{{ $t('databaseConfig.database') }}:</label>
              <input
                id="database"
                v-model="config.database"
                type="text"
                :placeholder="$t('databaseConfig.placeholders.database')"
                :disabled="isLoading"
              >
            </div>

            <div class="form-group">
              <label for="username">{{ $t('databaseConfig.username') }}:</label>
              <input
                id="username"
                v-model="config.username"
                type="text"
                :placeholder="$t('databaseConfig.placeholders.username')"
                :disabled="isLoading"
              >
            </div>

            <div class="form-group">
              <label for="password">{{ $t('databaseConfig.password') }}:</label>
              <input
                id="password"
                v-model="config.password"
                type="password"
                :placeholder="$t('databaseConfig.placeholders.password')"
                :disabled="isLoading"
              >
            </div>

            <div class="form-group">
              <label for="sslMode">{{ $t('databaseConfig.sslMode') }}:</label>
              <select
                id="sslMode"
                v-model="config.sslMode"
                :disabled="isLoading"
              >
                <option value="require">{{ $t('databaseConfig.sslModes.require') }}</option>
                <option value="disable">{{ $t('databaseConfig.sslModes.disable') }}</option>
              </select>
            </div>

          </div>
        </div>

        <div class="modal-footer">
          <button
            @click="testConnection"
            :class="['btn', 'secondary', { loading: isTesting }]"
            :disabled="!canTest || isTesting"
          >
            <svg v-if="!isTesting" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
            {{ $t('databaseConfig.testConnection') }}
          </button>

          <button
            @click="saveConfig"
            :class="['btn', 'primary', { loading: isSaving }]"
            :disabled="!canSave || isSaving"
          >
            <svg v-if="!isSaving" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"/>
            </svg>
            {{ $t('databaseConfig.saveConfig') }}
          </button>

          <button
            v-if="hasExistingConfig"
            @click="deleteConfig"
            :class="['btn', 'danger', { loading: isDeleting }]"
            :disabled="isDeleting"
          >
            <svg v-if="!isDeleting" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
            {{ $t('databaseConfig.deleteConfig') }}
          </button>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <div v-if="showConfirmDelete" class="confirm-dialog-overlay">
      <div class="confirm-dialog">
        <div class="confirm-dialog-header">
          <h3>{{ $t('databaseConfig.deleteConfig') }}</h3>
        </div>
        <div class="confirm-dialog-body">
          <p>{{ $t('databaseConfig.messages.confirmDelete') }}</p>
        </div>
        <div class="confirm-dialog-footer">
          <button @click="cancelDelete" class="btn secondary">
            {{ $t('databaseConfig.cancel') }}
          </button>
          <button @click="confirmDeleteConfig" class="btn danger" :disabled="isDeleting">
            {{ isDeleting ? $t('loading.deleting') : $t('databaseConfig.deleteConfig') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

// Props
const props = defineProps({
  initialConfig: {
    type: Object,
    default: () => ({})
  }
})

// Emits
const emit = defineEmits(['close', 'config-saved', 'config-deleted'])

// i18n
const { t } = useI18n()

// Reactive data
const config = ref({
  host: 'localhost',
  port: 5432,
  database: 'augment_tokens',
  username: 'postgres',
  password: '',
  sslMode: 'require',
  enabled: true
})

const isLoading = ref(false)
const isTesting = ref(false)
const isSaving = ref(false)
const isDeleting = ref(false)
const hasExistingConfig = ref(false)
const isConnectionTested = ref(false)
const showConfirmDelete = ref(false)

// Computed properties
const canTest = computed(() => {
  return config.value.host && 
         config.value.port && 
         config.value.database && 
         config.value.username && 
         config.value.password
})

const canSave = computed(() => {
  return canTest.value && isConnectionTested.value
})

// Methods
const loadConfig = async () => {
  isLoading.value = true
  try {
    const loadedConfig = await invoke('load_database_config')
    if (loadedConfig && loadedConfig.enabled) {
      config.value = {
        host: loadedConfig.host || 'localhost',
        port: loadedConfig.port || 5432,
        database: loadedConfig.database || 'augment_tokens',
        username: loadedConfig.username || 'postgres',
        password: '', // 不显示已保存的密码
        sslMode: loadedConfig.ssl_mode || 'require',
        enabled: loadedConfig.enabled || false
      }
      hasExistingConfig.value = true
    }
  } catch (error) {
    console.error('Failed to load database config:', error)
  } finally {
    isLoading.value = false
  }
}

const testConnection = async () => {
  isTesting.value = true
  isConnectionTested.value = false

  try {
    await invoke('test_database_connection', {
      host: config.value.host,
      port: config.value.port,
      database: config.value.database,
      username: config.value.username,
      password: config.value.password,
      ssl_mode: config.value.sslMode
    })

    // 连接成功时发送toast通知
    window.$notify.success(t('databaseConfig.messages.testSuccess'))
    isConnectionTested.value = true
  } catch (error) {
    // 连接失败时发送toast通知
    window.$notify.error(`${t('databaseConfig.messages.testFailed')}: ${error}`)
    isConnectionTested.value = false
  } finally {
    isTesting.value = false
  }
}

const saveConfig = async () => {
  isSaving.value = true
  
  try {
    await invoke('save_database_config', {
      host: config.value.host,
      port: config.value.port,
      database: config.value.database,
      username: config.value.username,
      password: config.value.password,
      ssl_mode: config.value.sslMode
    })
    
    window.$notify.success(t('databaseConfig.messages.saveSuccess'))
    emit('config-saved')
    emit('close')
  } catch (error) {
    window.$notify.error(`${t('databaseConfig.messages.saveFailed')}: ${error}`)
  } finally {
    isSaving.value = false
  }
}

const deleteConfig = () => {
  showConfirmDelete.value = true
}

const confirmDeleteConfig = async () => {
  showConfirmDelete.value = false
  isDeleting.value = true
  
  try {
    await invoke('delete_database_config')
    
    window.$notify.success(t('databaseConfig.messages.deleteSuccess'))
    emit('config-deleted')
    emit('close')
  } catch (error) {
    window.$notify.error(`${t('databaseConfig.messages.deleteFailed')}: ${error}`)
  } finally {
    isDeleting.value = false
  }
}

const cancelDelete = () => {
  showConfirmDelete.value = false
}

// Watch for config changes to reset connection test status
watch(config, () => {
  isConnectionTested.value = false
}, { deep: true })

// Lifecycle
onMounted(() => {
  loadConfig()
})
</script>



<style scoped>
.database-config-modal {
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

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
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

.modal-header h2 {
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
  flex: 1;
  overflow-y: auto;
  min-height: 0;
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

.form-group input {
  padding: 10px 12px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s ease;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
}

.form-group input:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-group input:disabled {
  background-color: var(--color-surface-alt, #f9fafb);
  color: var(--color-text-muted, #6b7280);
  cursor: not-allowed;
}

.form-group select {
  padding: 10px 12px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s ease;
  background-color: var(--color-surface, #ffffff);
}

.form-group select:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.form-group select:disabled {
  background-color: var(--color-surface-alt, #f9fafb);
  color: var(--color-text-muted, #6b7280);
  cursor: not-allowed;
}



.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 16px;
  padding: 20px 24px;
  border-top: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
  border-radius: 0 0 12px 12px;
  flex-shrink: 0;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.btn.primary {
  background: var(--color-accent, #3b82f6);
  color: var(--color-text-inverse, #ffffff);
}

.btn.primary:hover:not(:disabled) {
  background: var(--color-accent-hover, #2563eb);
}

.btn.secondary {
  background: var(--color-text-muted, #6b7280);
  color: var(--color-text-inverse, #ffffff);
}

.btn.secondary:hover:not(:disabled) {
  background: var(--color-text-secondary, #4b5563);
}

.btn.danger {
  background: var(--color-danger-bg, #dc2626);
  color: var(--color-text-inverse, #ffffff);
}

.btn.danger:hover:not(:disabled) {
  background: var(--color-danger-bg-hover, #b91c1c);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn.loading {
  opacity: 0.7;
  pointer-events: none;
}

@media (max-width: 900px) {
  .modal-content {
    width: 95%;
    margin: 20px;
  }

  .modal-footer {
    flex-direction: column;
    gap: 12px;
  }

  .btn {
    width: 100%;
    justify-content: center;
  }
}

/* 确认对话框样式 */
.confirm-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3100;
}

.confirm-dialog {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  width: 90%;
  max-width: 400px;
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.confirm-dialog-header {
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
}

.confirm-dialog-header h3 {
  margin: 0;
  color: var(--color-text-primary, #374151);
  font-size: 16px;
  font-weight: 600;
}

.confirm-dialog-body {
  padding: 16px 24px 20px;
}

.confirm-dialog-body p {
  margin: 0;
  color: var(--color-text-secondary, #6b7280);
  line-height: 1.5;
}

.confirm-dialog-footer {
  padding: 16px 24px 20px;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}
</style>
