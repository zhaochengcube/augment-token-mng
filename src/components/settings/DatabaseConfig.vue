<template>
  <BaseModal
    :visible="true"
    :title="$t('databaseConfig.title')"
    :body-scroll="false"
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-5">
      <!-- Host -->
      <div class="form-group !mb-0">
        <label for="host" class="label">
          {{ $t('databaseConfig.host') }}:
        </label>
        <input
          id="host"
          v-model="config.host"
          type="text"
          class="input"
          :placeholder="$t('databaseConfig.placeholders.host')"
          :disabled="isLoading"
        >
      </div>

      <!-- Port -->
      <div class="form-group !mb-0">
        <label for="port" class="label">
          {{ $t('databaseConfig.port') }}:
        </label>
        <input
          id="port"
          v-model.number="config.port"
          type="number"
          class="input"
          :placeholder="$t('databaseConfig.placeholders.port')"
          :disabled="isLoading"
        >
      </div>

      <!-- Database -->
      <div class="form-group !mb-0">
        <label for="database" class="label">
          {{ $t('databaseConfig.database') }}:
        </label>
        <input
          id="database"
          v-model="config.database"
          type="text"
          class="input"
          :placeholder="$t('databaseConfig.placeholders.database')"
          :disabled="isLoading"
        >
      </div>

      <!-- Username -->
      <div class="form-group !mb-0">
        <label for="username" class="label">
          {{ $t('databaseConfig.username') }}:
        </label>
        <input
          id="username"
          v-model="config.username"
          type="text"
          class="input"
          :placeholder="$t('databaseConfig.placeholders.username')"
          :disabled="isLoading"
        >
      </div>

      <!-- Password -->
      <div class="form-group !mb-0">
        <label for="password" class="label">
          {{ $t('databaseConfig.password') }}:
        </label>
        <input
          id="password"
          v-model="config.password"
          type="password"
          class="input"
          :placeholder="$t('databaseConfig.placeholders.password')"
          :disabled="isLoading"
        >
      </div>

      <!-- SSL Mode Dropdown -->
      <div class="form-group !mb-0">
        <label class="label">
          {{ $t('databaseConfig.sslMode') }}:
        </label>
        <div ref="sslModeDropdownRef" class="dropdown w-full">
          <button
            type="button"
            class="btn btn--secondary w-full justify-between"
            :disabled="isLoading"
            :aria-expanded="showSslModeMenu ? 'true' : 'false'"
            aria-haspopup="listbox"
            @click="toggleSslModeMenu"
          >
            <span>{{ sslModeLabel }}</span>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </button>
          <div v-if="showSslModeMenu" class="dropdown-menu left-0 w-full z-50" role="listbox">
            <button
              v-for="option in sslModeOptions"
              :key="option.value"
              type="button"
              class="dropdown-item"
              role="option"
              @click="selectSslMode(option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <button
        @click="testConnection"
        :class="['btn', 'btn--secondary', { loading: isTesting }]"
        :disabled="!canTest || isTesting"
      >
        <svg v-if="!isTesting" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
        </svg>
        {{ $t('databaseConfig.testConnection') }}
      </button>

      <button
        @click="saveConfig"
        :class="['btn', 'btn--primary', { loading: isSaving }]"
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
        class="btn btn--danger"
        :disabled="isDeleting"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
        {{ $t('databaseConfig.deleteConfig') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'
import BaseModal from '../common/BaseModal.vue'

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

// SSL Mode dropdown
const showSslModeMenu = ref(false)
const sslModeDropdownRef = ref(null)

const sslModeOptions = computed(() => [
  { value: 'require', label: t('databaseConfig.sslModes.require') },
  { value: 'disable', label: t('databaseConfig.sslModes.disable') }
])

const sslModeLabel = computed(() => {
  const match = sslModeOptions.value.find(option => option.value === config.value.sslMode)
  return match ? match.label : config.value.sslMode
})

const toggleSslModeMenu = () => {
  if (isLoading.value) return
  showSslModeMenu.value = !showSslModeMenu.value
}

const selectSslMode = (value) => {
  config.value.sslMode = value
  showSslModeMenu.value = false
}

const handleDocumentClick = (event) => {
  if (!showSslModeMenu.value) return
  const dropdownEl = sslModeDropdownRef.value
  if (dropdownEl && !dropdownEl.contains(event.target)) {
    showSslModeMenu.value = false
  }
}

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

const deleteConfig = async () => {
  const confirmed = await window.$confirm({
    title: t('databaseConfig.deleteConfig'),
    message: t('databaseConfig.messages.confirmDelete'),
    confirmText: t('databaseConfig.deleteConfig'),
    cancelText: t('databaseConfig.cancel'),
    variant: 'danger'
  })

  if (!confirmed) return

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

// Watch for config changes to reset connection test status
watch(config, () => {
  isConnectionTested.value = false
}, { deep: true })

// Lifecycle
onMounted(() => {
  loadConfig()
  document.addEventListener('click', handleDocumentClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleDocumentClick)
})
</script>
