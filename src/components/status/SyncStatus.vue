<template>
  <button
    class="sync-status-component"
    @click="handleSync"
    :disabled="isSyncing"
    :title="syncTooltip"
  >
    <div class="status-header">
      <div class="header-content">
        <h3>{{ $t('storage.status') }}</h3>
      </div>
    </div>

    <div class="status-content">
      <!-- 存储类型显示 -->
      <div class="storage-info">
        <div class="storage-mode">
          <span :class="['storage-badge', storageTypeClass]">{{ simpleStorageText }}</span>
        </div>
        <div class="sync-hint">
          {{ syncHintText }}
        </div>
      </div>
    </div>
  </button>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Emits
const emit = defineEmits(['storage-status-changed'])

// Reactive data
const storageStatus = ref(null)
const lastSyncStatus = ref(null)
const isRefreshing = ref(false)
const isSyncing = ref(false)

// Computed properties
const simpleStorageText = computed(() => {
  if (!storageStatus.value) return t('loading.loading')

  switch (storageStatus.value.storage_type) {
    case 'dual_storage':
      return t('storage.dual')
    case 'local_only':
      return t('storage.local')
    case 'postgresql':
      return t('storage.database')
    default:
      return t('storage.unknown')
  }
})

const storageTypeClass = computed(() => {
  if (!storageStatus.value) return ''

  switch (storageStatus.value.storage_type) {
    case 'dual_storage':
      return 'dual'
    case 'local_only':
      return 'local'
    case 'postgresql':
      return 'database'
    default:
      return ''
  }
})

const syncHintText = computed(() => {
  if (!storageStatus.value) return t('loading.loading')

  if (storageStatus.value.is_database_available) {
    return t('storage.syncData')
  } else {
    return t('storage.detectDatabase')
  }
})

const syncTooltip = computed(() => {
  if (!storageStatus.value?.is_database_available) {
    return t('storage.clickToDetect')
  }
  return t('storage.clickToSync')
})

const canSync = computed(() => {
  return storageStatus.value?.is_database_available && !isSyncing.value
})



// Methods
const refreshStatus = async () => {
  isRefreshing.value = true
  try {
    const status = await invoke('get_storage_status')
    storageStatus.value = status

    // 发出存储状态变化事件
    emit('storage-status-changed', status?.is_database_available || false)

    // 同时获取同步状态
    try {
      const syncStatus = await invoke('get_sync_status')
      if (syncStatus) {
        lastSyncStatus.value = syncStatus
      }
    } catch (syncError) {
      console.error('Failed to get sync status:', syncError)
    }
  } catch (error) {
    console.error('Failed to get storage status:', error)
    window.$notify.error(`${t('messages.getStorageStatusFailed')}: ${error}`)
  } finally {
    isRefreshing.value = false
  }
}





const handleSync = async () => {
  if (storageStatus.value?.is_database_available) {
    // 双重存储模式：执行双向同步
    if (!canSync.value) return

    isSyncing.value = true
    try {
      const result = await invoke('bidirectional_sync_tokens')
      lastSyncStatus.value = result
      window.$notify.success(t('messages.bidirectionalSyncComplete'))
    } catch (error) {
      window.$notify.error(`${t('messages.syncFailed')}: ${error}`)
    } finally {
      isSyncing.value = false
    }
  } else {
    // 本地存储模式：刷新存储状态
    await refreshStatus()
    if (storageStatus.value?.is_database_available) {
      window.$notify.success(t('messages.databaseDetected'))
    } else {
      window.$notify.info(t('messages.databaseNotDetected'))
    }
  }
}

// Lifecycle
onMounted(() => {
  refreshStatus()
})
</script>

<style scoped>
.sync-status-component {
  background: var(--color-surface, #ffffff);
  border-radius: 6px;
  padding: 10px 5px;
  border: 1px solid var(--color-border, #e5e7eb);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: fit-content;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
  width: 100%;
}

.sync-status-component:hover:not(:disabled) {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  border-color: var(--color-border-strong, #d1d5db);
}

.sync-status-component:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.status-header {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-shrink: 0;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}





.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.storage-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.storage-mode {
  display: flex;
  justify-content: center;
}

.storage-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 12px;
  white-space: nowrap;
}

.sync-hint {
  font-size: 10px;
  color: var(--color-text-muted, #6b7280);
  text-align: center;
  font-weight: 500;
}

.storage-badge.dual {
  background: var(--state-success);
  color: #ffffff;
}

.storage-badge.local {
  background: var(--state-warning);
  color: #ffffff;
}

.storage-badge.database {
  background: var(--accent);
  color: #ffffff;
}



@media (max-width: 768px) {
  .sync-status-component {
    padding: 8px 10px;
    gap: 6px;
  }

  .status-header h3 {
    font-size: 12px;
  }

  .storage-badge {
    font-size: 10px;
    padding: 2px 6px;
  }

  .sync-hint {
    font-size: 9px;
  }
}
</style>
