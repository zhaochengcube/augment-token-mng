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
/* ============================================
   SyncStatus - Modern Tech Style
   ============================================ */

.sync-status-component {
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-radius: 12px;
  padding: 12px 8px;
  border: 1px solid var(--tech-glass-border);
  box-shadow: var(--tech-border-glow);
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: fit-content;
  cursor: pointer;
  transition: all 0.25s ease;
  text-align: left;
  width: 100%;
}

.sync-status-component:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2), var(--tech-border-glow);
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
}

.sync-status-component:disabled {
  opacity: 0.4;
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
  gap: 10px;
}

.status-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-strong);
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
  gap: 6px;
}

.storage-mode {
  display: flex;
  justify-content: center;
}

/* 存储徽章 - 科技风 */
.storage-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 8px;
  white-space: nowrap;
}

.sync-hint {
  font-size: 10px;
  color: var(--text-muted);
  text-align: center;
  font-weight: 500;
}

.storage-badge.dual {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: #fff;
  box-shadow: 0 0 10px rgba(16, 185, 129, 0.4);
}

.storage-badge.local {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  color: #fff;
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.4);
}

.storage-badge.database {
  background: var(--accent);
  color: #fff;
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

@media (max-width: 768px) {
  .sync-status-component {
    padding: 10px 12px;
    gap: 8px;
  }

  .status-header h3 {
    font-size: 12px;
  }

  .storage-badge {
    font-size: 10px;
    padding: 3px 8px;
  }

  .sync-hint {
    font-size: 9px;
  }
}
</style>
