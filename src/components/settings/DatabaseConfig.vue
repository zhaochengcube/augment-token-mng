<template>
  <div class="database-config-modal">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>{{ $t('databaseConfig.title') }}</h2>
          <button class="modal-close" @click="$emit('close')">×</button>
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
import { useSettingsStore } from '../../stores/settings'

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

// Settings store
const settingsStore = useSettingsStore()

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
    // 从store加载配置,强制刷新以获取最新数据
    await settingsStore.loadDatabaseConfig(true)
    const loadedConfig = settingsStore.databaseConfig

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

    // 保存成功后刷新store中的配置
    await settingsStore.loadDatabaseConfig(true)

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

    // 删除成功后刷新store中的配置
    await settingsStore.loadDatabaseConfig(true)

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
  // 从store加载配置,如果store中已有数据则不会重复请求
  loadConfig()
})
</script>



<style scoped>
/* ============================================
   DatabaseConfig - Modern Tech Style
   ============================================ */

.database-config-modal {
  position: fixed;
  inset: 0;
  z-index: 3000;
}

.database-config-modal .modal-content {
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.database-config-modal .modal-body {
  padding: 26px;
  flex: 1;
  overflow-y: auto;
  min-height: 0;
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

/* 底部按钮区 */
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 14px;
  padding: 20px 26px;
  border-top: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-radius: 0 0 18px 18px;
  flex-shrink: 0;
}

@media (max-width: 900px) {
  .database-config-modal .modal-content {
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
  z-index: 3100;
  backdrop-filter: blur(4px);
}

.confirm-dialog {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  width: 90%;
  max-width: 400px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
  overflow: hidden;
}

.confirm-dialog-header {
  padding: 22px 26px 18px;
  border-bottom: 1px solid var(--tech-glass-border);
}

.confirm-dialog-header h3 {
  margin: 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

.confirm-dialog-body {
  padding: 18px 26px 22px;
}

.confirm-dialog-body p {
  margin: 0;
  color: var(--text-muted);
  line-height: 1.6;
}

.confirm-dialog-footer {
  padding: 18px 26px 22px;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-top: 1px solid var(--tech-glass-border);
}
</style>
