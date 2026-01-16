<template>
  <BaseModal
    :visible="true"
    :title="$t('proxyConfig.title')"
    :body-scroll="false"
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-5">
      <!-- 启用代理开关 -->
      <div class="flex flex-col gap-2">
        <label class="flex items-center cursor-pointer font-medium text-text">
          <input
            type="checkbox"
            v-model="config.enabled"
            :disabled="isLoading"
            class="mr-2.5 w-[18px] h-[18px] cursor-pointer accent-accent disabled:cursor-not-allowed"
          >
          <span>{{ $t('proxyConfig.enableProxy') }}</span>
        </label>
      </div>

      <!-- 代理类型选择 -->
      <div class="form-group !mb-0">
        <label class="label">
          {{ $t('proxyConfig.proxyType') }}:
        </label>
        <div ref="proxyTypeDropdownRef" class="dropdown w-full">
          <button
            id="proxyType"
            type="button"
            class="btn btn--secondary w-full justify-between"
            :disabled="proxyTypeDisabled"
            :aria-expanded="showProxyTypeMenu ? 'true' : 'false'"
            aria-haspopup="listbox"
            @click="toggleProxyTypeMenu"
          >
            <span>{{ proxyTypeLabel }}</span>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </button>
          <div v-if="showProxyTypeMenu" class="dropdown-menu left-0 w-full z-50" role="listbox">
            <button
              v-for="option in proxyTypeOptions"
              :key="option.value"
              type="button"
              class="dropdown-item"
              role="option"
              @click="selectProxyType(option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </div>
        <small class="text-[12px] text-text-muted mt-1 block">{{ getProxyTypeHelp }}</small>
      </div>

      <!-- 代理服务器配置 (仅当选择 http/https/socks5 时显示) -->
      <template v-if="needsServerConfig">
        <div class="form-group !mb-0">
          <label for="host" class="label">
            {{ $t('proxyConfig.host') }}:
          </label>
          <input
            id="host"
            v-model="config.host"
            type="text"
            class="input"
            :placeholder="$t('proxyConfig.placeholders.host')"
            :disabled="!config.enabled || isLoading"
          >
        </div>

        <div class="form-group !mb-0">
          <label for="port" class="label">
            {{ $t('proxyConfig.port') }}:
          </label>
          <input
            id="port"
            v-model.number="config.port"
            type="number"
            min="1"
            max="65535"
            class="input"
            :placeholder="$t('proxyConfig.placeholders.port')"
            :disabled="!config.enabled || isLoading"
          >
        </div>

        <!-- 认证信息 (可选) -->
        <div class="form-group !mb-0">
          <label for="username" class="label">
            {{ $t('proxyConfig.username') }}
            <span class="text-text-muted">({{ $t('proxyConfig.optional') }})</span>
          </label>
          <input
            id="username"
            v-model="config.username"
            type="text"
            class="input"
            :placeholder="$t('proxyConfig.placeholders.username')"
            :disabled="!config.enabled || isLoading"
          >
        </div>

        <div class="form-group !mb-0">
          <label for="password" class="label">
            {{ $t('proxyConfig.password') }}
            <span class="text-text-muted">({{ $t('proxyConfig.optional') }})</span>
          </label>
          <input
            id="password"
            v-model="config.password"
            type="password"
            class="input"
            :placeholder="$t('proxyConfig.placeholders.password')"
            :disabled="!config.enabled || isLoading"
          >
        </div>
      </template>

      <!-- 自定义 URL 配置 (仅当选择 custom_url 时显示) -->
      <template v-if="needsCustomUrl">
        <div class="form-group !mb-0">
          <label for="customUrl" class="label">
            {{ $t('proxyConfig.customUrl') }}:
          </label>
          <input
            id="customUrl"
            v-model="config.customUrl"
            type="text"
            class="input"
            :placeholder="$t('proxyConfig.placeholders.customUrl')"
            :disabled="!config.enabled || isLoading"
          >
          <small class="text-[12px] text-text-muted mt-1 block">{{ $t('proxyConfig.detailedHelp.customUrl') }}</small>
        </div>
      </template>
    </div>

    <template #footer>
      <div class="flex gap-2.5 items-center mr-auto">
        <button
          @click="testConnection"
          :class="['btn', 'btn--secondary', { loading: isTesting }]"
          :disabled="!canTest || isTesting || isLoading"
        >
          <svg v-if="!isTesting" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
          </svg>
          {{ $t('proxyConfig.testConnection') }}
        </button>

        <span
          v-show="lastTestResult"
          :class="['badge font-mono', testResultBadgeClass]"
        >
          <span class="status-dot"></span>
          {{ testResultText }}
        </span>
      </div>

      <button
        @click="saveConfig"
        :class="['btn', 'btn--primary', { loading: isSaving }]"
        :disabled="!canSave || isSaving || isLoading"
      >
        <svg v-if="!isSaving" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"/>
        </svg>
        {{ $t('proxyConfig.save') }}
      </button>

      <button
        v-if="hasExistingConfig"
        @click="handleDelete"
        class="btn btn--danger"
        :disabled="isLoading || isSaving"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
        {{ $t('proxyConfig.delete') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onBeforeUnmount, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'
import BaseModal from '../common/BaseModal.vue'

// Emits
const emit = defineEmits(['close', 'config-deleted'])

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
const lastTestResult = ref(null) // { success: boolean, latency: number }
const showProxyTypeMenu = ref(false)
const proxyTypeDropdownRef = ref(null)

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

const proxyTypeOptions = computed(() => ([
  { value: 'system', label: t('proxyConfig.types.system') },
  { value: 'http', label: t('proxyConfig.types.http') },
  { value: 'https', label: t('proxyConfig.types.https') },
  { value: 'socks5', label: t('proxyConfig.types.socks5') },
  { value: 'custom_url', label: t('proxyConfig.types.customUrl') }
]))

const proxyTypeLabel = computed(() => {
  const match = proxyTypeOptions.value.find((option) => option.value === config.value.proxyType)
  return match ? match.label : config.value.proxyType
})

const proxyTypeDisabled = computed(() => !config.value.enabled || isLoading.value)

const toggleProxyTypeMenu = () => {
  if (proxyTypeDisabled.value) return
  showProxyTypeMenu.value = !showProxyTypeMenu.value
}

const selectProxyType = (value) => {
  config.value.proxyType = value
  showProxyTypeMenu.value = false
}

const handleDocumentClick = (event) => {
  if (!showProxyTypeMenu.value) return
  const dropdownEl = proxyTypeDropdownRef.value
  if (dropdownEl && !dropdownEl.contains(event.target)) {
    showProxyTypeMenu.value = false
  }
}

// 根据测试结果返回对应的徽章样式类
const testResultBadgeClass = computed(() => {
  if (!lastTestResult.value) return ''
  if (!lastTestResult.value.success) return '' // 默认 badge 样式
  const latency = lastTestResult.value.latency
  if (latency < 100) return 'badge--success'
  if (latency < 300) return 'badge--accent'
  if (latency < 500) return 'badge--warning'
  return 'badge--danger'
})

// 根据测试结果返回显示文本
const testResultText = computed(() => {
  if (!lastTestResult.value) return ''
  if (!lastTestResult.value.success) return t('proxyConfig.messages.testFailedStatus')
  return `${lastTestResult.value.latency}ms`
})

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
    emit('close')
  } catch (error) {
    window.$notify.error(`${t('proxyConfig.messages.saveFailed')}: ${error}`)
  } finally {
    isSaving.value = false
  }
}

const handleDelete = async () => {
  const confirmed = await window.$confirm({
    title: t('proxyConfig.confirmDelete'),
    message: t('proxyConfig.confirmDeleteMessage'),
    confirmText: t('common.confirm'),
    cancelText: t('common.cancel'),
    variant: 'danger'
  })

  if (!confirmed) return

  isLoading.value = true

  try {
    await invoke('delete_proxy_config')

    // 删除成功后刷新store中的配置
    await settingsStore.loadProxyConfig(true)

    hasExistingConfig.value = false
    window.$notify.success(t('proxyConfig.messages.deleteSuccess'))
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
  document.addEventListener('click', handleDocumentClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleDocumentClick)
})

watch(proxyTypeDisabled, (disabled) => {
  if (disabled) {
    showProxyTypeMenu.value = false
  }
})
</script>
