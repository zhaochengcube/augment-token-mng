import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  // State
  const appVersion = ref('1.0.0')
  const serverStatus = ref({
    running: false,
    address: '',
    port: 0
  })
  const proxyConfig = ref({
    enabled: false,
    host: '',
    port: 0
  })
  const databaseConfig = ref({
    enabled: false,
    host: 'localhost',
    port: 5432,
    database: 'augment_tokens',
    username: 'postgres',
    ssl_mode: 'require'
  })
  
  // Tray state - read from localStorage
  const trayEnabled = ref(localStorage.getItem('tray_enabled') === 'true')

  // Telegram config state
  const telegramConfig = ref({
    enabled: false,
    bot_token: '',
    chat_id: '',
    notify_days: [15, 7, 3],
    check_interval_hours: 6
  })

  // Loading states
  const isLoadingVersion = ref(false)
  const isLoadingServer = ref(false)
  const isLoadingProxy = ref(false)
  const isLoadingDatabase = ref(false)
  const isLoadingTray = ref(false)
  const isLoadingTelegram = ref(false)

  // 数据是否已加载标记 (应用生命周期内只加载一次)
  const versionLoaded = ref(false)
  const serverLoaded = ref(false)
  const proxyLoaded = ref(false)
  const databaseLoaded = ref(false)
  const telegramLoaded = ref(false)

  // Actions
  const loadAppVersion = async (force = false) => {
    // 如果已加载且不强制刷新,直接返回缓存数据
    if (!force && versionLoaded.value) {
      return appVersion.value
    }

    isLoadingVersion.value = true
    try {
      const updateInfo = await invoke('check_for_updates')
      appVersion.value = updateInfo.current_version || '1.0.0'
      versionLoaded.value = true
      return appVersion.value
    } catch (error) {
      console.error('Failed to load app version:', error)
      appVersion.value = '1.0.0'
      throw error
    } finally {
      isLoadingVersion.value = false
    }
  }

  const loadServerStatus = async (force = false) => {
    // 如果已加载且不强制刷新,直接返回缓存数据
    if (!force && serverLoaded.value) {
      return serverStatus.value
    }

    isLoadingServer.value = true
    try {
      const status = await invoke('get_api_server_status')
      serverStatus.value = {
        running: status.running || false,
        address: status.address || '',
        port: status.port || 0
      }
      serverLoaded.value = true
      return serverStatus.value
    } catch (error) {
      console.error('Failed to load server status:', error)
      serverStatus.value = {
        running: false,
        address: '',
        port: 0
      }
      throw error
    } finally {
      isLoadingServer.value = false
    }
  }

  const loadProxyConfig = async (force = false) => {
    // 如果已加载且不强制刷新,直接返回缓存数据
    if (!force && proxyLoaded.value) {
      return proxyConfig.value
    }

    isLoadingProxy.value = true
    try {
      const config = await invoke('load_proxy_config')
      proxyConfig.value = {
        enabled: config?.enabled || false,
        host: config?.host || '',
        port: config?.port || 0
      }
      proxyLoaded.value = true
      return proxyConfig.value
    } catch (error) {
      console.error('Failed to load proxy config:', error)
      proxyConfig.value = {
        enabled: false,
        host: '',
        port: 0
      }
      throw error
    } finally {
      isLoadingProxy.value = false
    }
  }

  const loadDatabaseConfig = async (force = false) => {
    // 如果已加载且不强制刷新,直接返回缓存数据
    if (!force && databaseLoaded.value) {
      return databaseConfig.value
    }

    isLoadingDatabase.value = true
    try {
      const config = await invoke('load_database_config')
      databaseConfig.value = {
        enabled: config?.enabled || false,
        host: config?.host || 'localhost',
        port: config?.port || 5432,
        database: config?.database || 'augment_tokens',
        username: config?.username || 'postgres',
        ssl_mode: config?.ssl_mode || 'require'
      }
      databaseLoaded.value = true
      return databaseConfig.value
    } catch (error) {
      console.error('Failed to load database config:', error)
      databaseConfig.value = {
        enabled: false,
        host: 'localhost',
        port: 5432,
        database: 'augment_tokens',
        username: 'postgres',
        ssl_mode: 'require'
      }
      throw error
    } finally {
      isLoadingDatabase.value = false
    }
  }

  // Toggle tray
  const toggleTray = async (enabled) => {
    isLoadingTray.value = true
    try {
      await invoke('toggle_tray', { enabled })
      trayEnabled.value = enabled
      localStorage.setItem('tray_enabled', String(enabled))
      return true
    } catch (error) {
      console.error('Failed to toggle tray:', error)
      throw error
    } finally {
      isLoadingTray.value = false
    }
  }

  // Initialize tray based on stored preference
  const initializeTray = async () => {
    if (trayEnabled.value) {
      try {
        await invoke('create_tray')
      } catch (error) {
        console.error('Failed to initialize tray:', error)
      }
    }
  }

  // Load telegram config
  const loadTelegramConfig = async (force = false) => {
    if (!force && telegramLoaded.value) {
      return telegramConfig.value
    }

    isLoadingTelegram.value = true
    try {
      const config = await invoke('load_telegram_config')
      telegramConfig.value = {
        enabled: config?.enabled || false,
        bot_token: config?.bot_token || '',
        chat_id: config?.chat_id || '',
        notify_days: config?.notify_days || [15, 7, 3],
        check_interval_hours: config?.check_interval_hours || 6
      }
      telegramLoaded.value = true
      return telegramConfig.value
    } catch (error) {
      console.error('Failed to load telegram config:', error)
      telegramConfig.value = {
        enabled: false,
        bot_token: '',
        chat_id: '',
        notify_days: [15, 7, 3],
        check_interval_hours: 6
      }
      throw error
    } finally {
      isLoadingTelegram.value = false
    }
  }

  // Load all settings
  const loadAllSettings = async (force = false) => {
    await Promise.all([
      loadAppVersion(force),
      loadServerStatus(force),
      loadProxyConfig(force),
      loadDatabaseConfig(force),
      loadTelegramConfig(force)
    ])
  }

  return {
    // State
    appVersion,
    serverStatus,
    proxyConfig,
    databaseConfig,
    trayEnabled,
    telegramConfig,
    
    // Loading states
    isLoadingVersion,
    isLoadingServer,
    isLoadingProxy,
    isLoadingDatabase,
    isLoadingTray,
    isLoadingTelegram,
    
    // Actions
    loadAppVersion,
    loadServerStatus,
    loadProxyConfig,
    loadDatabaseConfig,
    loadTelegramConfig,
    loadAllSettings,
    toggleTray,
    initializeTray
  }
})

