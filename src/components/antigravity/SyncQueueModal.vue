<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="visible" class="sync-queue-overlay" @click="close">
        <div class="sync-queue-dialog" @click.stop>
          <div class="sync-queue-header">
            <div class="sync-queue-title">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
              </svg>
              <h3>{{ $t('platform.antigravity.syncQueueTitle') }}</h3>
            </div>
            <button @click="close" class="modal-close">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
          </div>

          <div class="sync-queue-tabs">
            <button
              :class="['tab-btn', { active: activeTab === 'upserts' }]"
              @click="activeTab = 'upserts'"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
              <span>{{ $t('platform.antigravity.syncQueueUpsertsTitle') }}</span>
              <span class="tab-count upserts">{{ pendingUpserts.length }}</span>
            </button>
            <button
              :class="['tab-btn', { active: activeTab === 'deletions' }]"
              @click="activeTab = 'deletions'"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
              <span>{{ $t('platform.antigravity.syncQueueDeletionsTitle') }}</span>
              <span class="tab-count deletions">{{ pendingDeletions.length }}</span>
            </button>
          </div>

          <div class="sync-queue-body">
            <div v-show="activeTab === 'upserts'" class="tab-content">
              <div v-if="pendingUpserts.length === 0" class="sync-queue-empty">
                <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.2">
                  <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
                <span>{{ $t('platform.antigravity.syncQueueEmpty') }}</span>
              </div>
              <div v-else class="sync-queue-list">
                <div v-for="account in pendingUpserts" :key="account.id" class="sync-queue-item">
                  <div class="item-icon upsert">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                    </svg>
                  </div>
                  <div class="item-content">
                    <span class="item-email">{{ account.email || account.id }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div v-show="activeTab === 'deletions'" class="tab-content">
              <div v-if="pendingDeletions.length === 0" class="sync-queue-empty">
                <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.2">
                  <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
                <span>{{ $t('platform.antigravity.syncQueueEmpty') }}</span>
              </div>
              <div v-else class="sync-queue-list">
                <div v-for="item in pendingDeletions" :key="item.id" class="sync-queue-item deletion">
                  <div class="item-icon delete">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M19 13H5v-2h14v2z"/>
                    </svg>
                  </div>
                  <div class="item-content">
                    <span class="item-email">{{ item.email || item.id }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="sync-queue-footer">
            <button
              @click="handleMarkAllForSync"
              class="btn primary"
              :disabled="syncing || totalAccountsCount === 0"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
              </svg>
              {{ $t('platform.antigravity.fullSync') }}
            </button>
            <div class="footer-right">
              <button @click="close" class="btn secondary">
                {{ $t('common.close') }}
              </button>
              <button
                v-if="pendingUpserts.length > 0 || pendingDeletions.length > 0"
                @click="handleSync"
                class="btn primary"
                :disabled="syncing"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
                </svg>
                {{ $t('platform.antigravity.sync') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  pendingUpserts: {
    type: Array,
    default: () => []
  },
  pendingDeletions: {
    type: Array,
    default: () => []
  },
  syncing: {
    type: Boolean,
    default: false
  },
  totalAccountsCount: {
    type: Number,
    default: 0
  }
})

const emit = defineEmits(['update:visible', 'sync', 'mark-all-for-sync'])

const activeTab = ref('upserts')

const close = () => {
  emit('update:visible', false)
}

const handleSync = () => {
  close()
  emit('sync')
}

const handleMarkAllForSync = () => {
  emit('mark-all-for-sync')
}
</script>

<style scoped>
.sync-queue-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  padding: 20px;
  backdrop-filter: blur(4px);
}

.sync-queue-dialog {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 18px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
  width: 100%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sync-queue-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 22px 26px;
  border-bottom: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.sync-queue-title {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-strong);
}

.sync-queue-title h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.modal-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  color: var(--text);
}

.sync-queue-tabs {
  display: flex;
  padding: 12px 16px;
  gap: 10px;
  border-bottom: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 20%, transparent);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-muted);
  border-radius: 10px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.2s ease;
}

.tab-btn.active {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  color: var(--accent);
}

.tab-count {
  padding: 2px 6px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  background: color-mix(in srgb, var(--bg-muted) 40%, transparent);
  color: var(--text-muted);
}

.sync-queue-body {
  padding: 16px;
  overflow: auto;
  flex: 1;
}

.sync-queue-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  color: var(--text-muted);
  padding: 30px 0;
}

.sync-queue-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.sync-queue-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border: 1px solid color-mix(in srgb, var(--tech-glass-border) 80%, transparent);
}

.sync-queue-item.deletion {
  opacity: 0.85;
}

.item-icon {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
}

.item-icon.delete {
  background: color-mix(in srgb, var(--state-danger) 12%, transparent);
  color: var(--state-danger);
}

.item-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.item-email {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
}

.sync-queue-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-top: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 20%, transparent);
  gap: 12px;
  flex-wrap: wrap;
}

.footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
