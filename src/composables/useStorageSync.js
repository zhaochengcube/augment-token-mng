import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

/**
 * 存储同步 Composable
 * 通用的存储同步逻辑，支持 Antigravity、Windsurf 账号管理和 Token 管理
 *
 * @param {Object} options 配置选项
 * @param {string} options.platform - 平台标识 ('antigravity' | 'windsurf' | 'augment')
 * @param {string} options.syncCommand - 同步命令
 * @param {Ref<Array>} options.items - 数据列表的响应式引用（accounts 或 tokens）
 * @param {Ref<string|null>} [options.currentItemId] - 当前选中项ID的响应式引用（可选）
 * @param {string} [options.itemKey='account'] - 同步请求中的数据键名 ('account' | 'token')
 * @param {string} [options.labelField='email'] - 用于显示的标签字段名 ('email' | 'email_note')
 * @param {Function} [options.onSyncComplete] - 同步完成后的回调函数（可选，用于保存本地文件等）
 */
export function useStorageSync(options) {
  const {
    platform,
    syncCommand,
    items,
    currentItemId = ref(null),
    itemKey = 'account',
    labelField = 'email',
    onSyncComplete
  } = options
  const { t: $t } = useI18n()

  // 存储状态
  const isDatabaseAvailable = ref(false)
  const isStorageInitializing = ref(false)
  const isSyncing = ref(false)
  const isSyncNeeded = ref(false)
  const isLoadingFromSync = ref(false)
  const showSyncQueueModal = ref(false)

  // LocalStorage keys
  const STORAGE_KEY_LAST_VERSION = `atm-${platform}-sync-last-version`
  const STORAGE_KEY_PENDING_UPSERTS = `atm-${platform}-sync-pending-upserts`
  const STORAGE_KEY_PENDING_DELETIONS = `atm-${platform}-sync-pending-deletions`

  // 同步队列状态
  const lastVersion = ref(0)
  const pendingUpserts = ref(new Map())
  const pendingDeletions = ref(new Map())

  // 计算属性
  const hasPendingChanges = computed(() => pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0)
  const pendingUpsertsList = computed(() => Array.from(pendingUpserts.value.values()))
  const pendingDeletionsList = computed(() => Array.from(pendingDeletions.value.values()))

  // 存储状态显示
  const storageStatusText = computed(() => {
    if (isStorageInitializing.value) {
      return $t('storage.initializing')
    }
    if (isDatabaseAvailable.value) {
      return hasPendingChanges.value
        ? `${$t('storage.dualStorage')}-${$t('storage.notSynced')}`
        : $t('storage.dualStorage')
    }
    return $t('storage.localStorage')
  })

  const storageStatusClass = computed(() => {
    if (isStorageInitializing.value) {
      return 'badge--accent-tech'
    }
    if (isDatabaseAvailable.value && hasPendingChanges.value) {
      return 'badge--warning-tech'
    }
    return isDatabaseAvailable.value ? 'badge--success-tech' : 'badge--accent-tech'
  })

  // 版本号管理
  const loadLastVersion = () => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY_LAST_VERSION)
      if (stored) {
        const version = parseInt(stored, 10)
        if (!isNaN(version) && version >= 0) {
          return version
        }
      }
    } catch (error) {
      console.warn(`Failed to load ${platform} lastVersion:`, error)
    }
    return 0
  }

  const saveLastVersion = (version) => {
    try {
      localStorage.setItem(STORAGE_KEY_LAST_VERSION, version.toString())
    } catch (error) {
      console.error(`Failed to save ${platform} lastVersion:`, error)
    }
  }

  // 待同步变更持久化
  const savePendingChanges = () => {
    try {
      const upsertsArr = Array.from(pendingUpserts.value.entries()).map(([id, item]) => ({ id, [itemKey]: item }))
      const deletionsArr = Array.from(pendingDeletions.value.values())

      localStorage.setItem(STORAGE_KEY_PENDING_UPSERTS, JSON.stringify(upsertsArr))
      localStorage.setItem(STORAGE_KEY_PENDING_DELETIONS, JSON.stringify(deletionsArr))
    } catch (error) {
      console.error(`Failed to save ${platform} pending changes:`, error)
    }
  }

  const loadPendingChanges = () => {
    try {
      const upsertsJson = localStorage.getItem(STORAGE_KEY_PENDING_UPSERTS)
      if (upsertsJson) {
        const arr = JSON.parse(upsertsJson)
        if (Array.isArray(arr)) {
          pendingUpserts.value = new Map(
            arr
              .filter(item => item && item.id && item[itemKey])
              .map(item => [item.id, item[itemKey]])
          )
        }
      }

      const deletionsJson = localStorage.getItem(STORAGE_KEY_PENDING_DELETIONS)
      if (deletionsJson) {
        const arr = JSON.parse(deletionsJson)
        if (Array.isArray(arr)) {
          pendingDeletions.value = new Map(
            arr
              .filter(item => item && item.id)
              .map(item => [item.id, { id: item.id, [labelField]: item[labelField] || null }])
          )
        }
      }

      if (pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0) {
        isSyncNeeded.value = true
      }
    } catch (error) {
      console.warn(`Failed to load ${platform} pending changes:`, error)
    }
  }

  // 存储状态检测（带轮询）
  let storageCheckTimer = null

  const getStorageStatus = async () => {
    try {
      // 使用通用的存储状态命令，数据库是共享的
      const status = await invoke('get_storage_status')

      if (status?.is_initializing) {
        isStorageInitializing.value = true
        isDatabaseAvailable.value = false

        if (!storageCheckTimer) {
          storageCheckTimer = setInterval(async () => {
            await getStorageStatus()
          }, 500)
        }
      } else {
        isStorageInitializing.value = false
        isDatabaseAvailable.value = status?.is_database_available || false

        if (storageCheckTimer) {
          clearInterval(storageCheckTimer)
          storageCheckTimer = null
        }
      }
    } catch (error) {
      console.error(`Failed to get ${platform} storage status:`, error)
      isDatabaseAvailable.value = false
      isStorageInitializing.value = false

      if (storageCheckTimer) {
        clearInterval(storageCheckTimer)
        storageCheckTimer = null
      }
    }
  }

  // 标记数据项变更（通用方法，兼容 account 和 token）
  const markItemUpsert = (item) => {
    if (!item?.id) return
    pendingUpserts.value.set(item.id, item)
    pendingDeletions.value.delete(item.id)
    savePendingChanges()
    if (isDatabaseAvailable.value) {
      isSyncNeeded.value = true
    }
  }

  const markItemDeletion = (item) => {
    if (!item?.id) return
    const wasOnlyLocal = pendingUpserts.value.has(item.id)
    pendingUpserts.value.delete(item.id)
    if (!wasOnlyLocal) {
      pendingDeletions.value.set(item.id, { id: item.id, [labelField]: item[labelField] || null })
    } else {
      pendingDeletions.value.delete(item.id)
    }
    savePendingChanges()
    if (isDatabaseAvailable.value) {
      isSyncNeeded.value = pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0
    }
  }

  const markItemUpsertById = (itemId) => {
    const item = items.value.find(a => a.id === itemId)
    if (item) {
      markItemUpsert(item)
    }
  }

  // 标记所有数据项待同步
  const markAllForSync = () => {
    if (items.value.length === 0) {
      return false
    }
    pendingUpserts.value = new Map(items.value.map(item => [item.id, item]))
    pendingDeletions.value.clear()
    savePendingChanges()
    isSyncNeeded.value = true
    return true
  }

  // 打开同步队列
  const openSyncQueue = () => {
    if (!isDatabaseAvailable.value) {
      window.$notify?.info($t('storage.databaseNotAvailable'))
      return
    }
    showSyncQueueModal.value = true
  }

  const closeSyncQueue = () => {
    showSyncQueueModal.value = false
  }

  // 同步操作
  const handleSync = async () => {
    if (isSyncing.value) return
    if (!isDatabaseAvailable.value) {
      window.$notify?.warning($t('messages.databaseNotAvailable'))
      return
    }

    isSyncing.value = true
    try {
      window.$notify?.info($t('messages.syncingData'))

      const req = {
        last_version: lastVersion.value,
        upserts: Array.from(pendingUpserts.value.values()).map(item => ({ [itemKey]: item })),
        deletions: Array.from(pendingDeletions.value.values()).map(item => ({ id: item.id })),
      }

      const res = await invoke(syncCommand, { reqJson: JSON.stringify(req) })

      isLoadingFromSync.value = true

      // 处理服务端返回的 upserts
      for (const serverItem of res.upserts) {
        const idx = items.value.findIndex(a => a.id === serverItem.id)
        if (idx !== -1) {
          items.value[idx] = serverItem
        } else {
          items.value.push(serverItem)
        }
      }

      // 处理服务端返回的 deletions
      for (const id of res.deletions) {
        const idx = items.value.findIndex(a => a.id === id)
        if (idx !== -1) {
          items.value.splice(idx, 1)
        }
        if (currentItemId.value === id) {
          currentItemId.value = null
        }
      }

      lastVersion.value = res.new_version
      saveLastVersion(res.new_version)

      pendingUpserts.value.clear()
      pendingDeletions.value.clear()
      savePendingChanges()

      // 调用同步完成回调（如保存本地文件）
      if (onSyncComplete) {
        await onSyncComplete()
      }

      await new Promise(resolve => setTimeout(resolve, 1200))
      isLoadingFromSync.value = false
      isSyncNeeded.value = false

      window.$notify?.success($t('messages.syncComplete'))
    } catch (error) {
      window.$notify?.error(`${$t('messages.syncFailed')}: ${error}`)
    } finally {
      isSyncing.value = false
    }
  }

  // 初始化
  const initSync = async () => {
    lastVersion.value = loadLastVersion()
    loadPendingChanges()
    await getStorageStatus()
  }

  // 清理定时器
  const cleanup = () => {
    if (storageCheckTimer) {
      clearInterval(storageCheckTimer)
      storageCheckTimer = null
    }
  }

  onUnmounted(cleanup)

  return {
    // 状态
    isDatabaseAvailable,
    isStorageInitializing,
    isSyncing,
    isSyncNeeded,
    isLoadingFromSync,
    showSyncQueueModal,
    hasPendingChanges,
    pendingUpsertsList,
    pendingDeletionsList,
    storageStatusText,
    storageStatusClass,

    // 方法
    initSync,
    getStorageStatus,
    markItemUpsert,
    markItemDeletion,
    markItemUpsertById,
    markAllForSync,
    openSyncQueue,
    closeSyncQueue,
    handleSync,
    cleanup
  }
}

