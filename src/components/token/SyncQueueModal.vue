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
              <h3>{{ $t('tokenList.syncQueueTitle') }}</h3>
            </div>
            <button @click="close" class="sync-queue-close">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
          </div>
          
          <!-- Tabs -->
          <div class="sync-queue-tabs">
            <button 
              :class="['tab-btn', { active: activeTab === 'upserts' }]"
              @click="activeTab = 'upserts'"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
              <span>{{ $t('tokenList.syncQueueUpsertsTitle') }}</span>
              <span class="tab-count upserts">{{ pendingUpserts.length }}</span>
            </button>
            <button 
              :class="['tab-btn', { active: activeTab === 'deletions' }]"
              @click="activeTab = 'deletions'"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
              <span>{{ $t('tokenList.syncQueueDeletionsTitle') }}</span>
              <span class="tab-count deletions">{{ pendingDeletions.length }}</span>
            </button>
          </div>
          
          <!-- Tab Content -->
          <div class="sync-queue-body">
            <!-- Upserts Tab -->
            <div v-show="activeTab === 'upserts'" class="tab-content">
              <div v-if="pendingUpserts.length === 0" class="sync-queue-empty">
                <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.2">
                  <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
                <span>{{ $t('tokenList.syncQueueEmpty') }}</span>
              </div>
              <div v-else class="sync-queue-list">
                <div v-for="token in pendingUpserts" :key="token.id" class="sync-queue-item">
                  <div class="item-icon upsert">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                    </svg>
                  </div>
                  <div class="item-content">
                    <span class="item-email">{{ token.email_note || $t('tokenList.noEmailNote') }}</span>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Deletions Tab -->
            <div v-show="activeTab === 'deletions'" class="tab-content">
              <div v-if="pendingDeletions.length === 0" class="sync-queue-empty">
                <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.2">
                  <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
                <span>{{ $t('tokenList.syncQueueEmpty') }}</span>
              </div>
              <div v-else class="sync-queue-list">
                <div v-for="item in pendingDeletions" :key="item.id" class="sync-queue-item deletion">
                  <div class="item-icon delete">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M19 13H5v-2h14v2z"/>
                    </svg>
                  </div>
                  <div class="item-content">
                    <span class="item-email">{{ item.email_note || $t('tokenList.noEmailNote') }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <div class="sync-queue-footer">
            <button @click="handleMarkAllForSync" 
                    class="btn-sync-now"
                    :disabled="syncing || totalTokensCount === 0"
                    >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
              </svg>
              {{ $t('tokenList.fullSync') }}
            </button>
            <div class="footer-right">
              <button @click="close" class="btn-close-queue">
                {{ $t('common.close') }}
              </button>
              <button v-if="pendingUpserts.length > 0 || pendingDeletions.length > 0" 
                      @click="handleSync" 
                      class="btn-sync-now"
                      :disabled="syncing">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
                </svg>
                {{ $t('tokenList.sync') }}
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

const props = defineProps({
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
  totalTokensCount: {
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
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  padding: 20px;
}

.sync-queue-dialog {
  background: var(--color-surface, #ffffff);
  border-radius: 16px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.2);
  width: 100%;
  max-width: 480px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sync-queue-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e5e7eb);
  background: var(--color-surface-muted, #f8f9fa);
}

.sync-queue-title {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--color-text-heading, #333);
}

.sync-queue-title svg {
  color: var(--color-primary, #3b82f6);
}

.sync-queue-title h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.sync-queue-close {
  background: none;
  border: none;
  padding: 6px;
  border-radius: 8px;
  cursor: pointer;
  color: var(--color-text-muted, #666);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.sync-queue-close:hover {
  background: var(--color-surface-hover, #e9ecef);
  color: var(--color-text-heading, #333);
}

/* Tabs */
.sync-queue-tabs {
  display: flex;
  padding: 0 24px 0 0;
  background: var(--color-surface-secondary, #f9fafb);
  border-bottom: 1px solid var(--color-divider, #e5e7eb);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-secondary, #6b7280);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: -1px;
}

.tab-btn:hover {
  color: var(--color-text-primary, #374151);
  background: var(--color-surface-hover, rgba(0, 0, 0, 0.03));
}

.tab-btn.active {
  color: var(--color-primary, #3b82f6);
  border-bottom-color: var(--color-primary, #3b82f6);
  background: var(--color-surface, #ffffff);
}

.tab-btn svg {
  opacity: 0.7;
}

.tab-btn.active svg {
  opacity: 1;
}

.tab-count {
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  min-width: 24px;
  text-align: center;
}

.tab-count.upserts {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.tab-count.deletions {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

/* Body & Tab Content */
.sync-queue-body {
  flex: 1;
  overflow-y: auto;
  min-height: 200px;
  max-height: 400px;
}

.tab-content {
  padding: 20px 24px;
  height: 100%;
}

.sync-queue-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: var(--color-surface-secondary, #f9fafb);
  border-radius: 10px;
  color: var(--color-text-muted, #9ca3af);
  font-size: 13px;
  gap: 8px;
}

.sync-queue-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sync-queue-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--color-surface-secondary, #f9fafb);
  border-radius: 8px;
  border: 1px solid var(--color-divider, #e5e7eb);
  transition: all 0.2s;
}

.sync-queue-item:hover {
  border-color: var(--color-primary, #3b82f6);
  background: var(--color-surface, #ffffff);
}

.sync-queue-item.deletion {
  border-left: 3px solid #ef4444;
}

.item-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.item-icon.upsert {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.item-icon.delete {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-email {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-id {
  font-size: 12px;
  font-family: 'SF Mono', Monaco, monospace;
  background: var(--color-surface, #ffffff);
  padding: 4px 8px;
  border-radius: 4px;
  color: var(--color-text-secondary, #6b7280);
}

.sync-queue-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--color-divider, #e5e7eb);
  background: var(--color-surface, #ffffff);
}

.footer-right {
  display: flex;
  gap: 12px;
}

.btn-close-queue {
  padding: 10px 20px;
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 8px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-secondary, #6b7280);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-close-queue:hover {
  background: var(--color-surface-hover, #f3f4f6);
  border-color: var(--color-border-hover, #9ca3af);
}

.btn-sync-now {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  background: var(--color-primary, #3b82f6);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-sync-now:hover:not(:disabled) {
  background: var(--color-primary-hover, #2563eb);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.btn-sync-now:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Modal 动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .sync-queue-dialog,
.modal-leave-active .sync-queue-dialog {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .sync-queue-dialog,
.modal-leave-to .sync-queue-dialog {
  transform: scale(0.95);
  opacity: 0;
}

/* 暗色模式 */
[data-theme='dark'] .sync-queue-dialog {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .sync-queue-header {
  background: rgba(31, 41, 55, 0.8);
  border-bottom-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .sync-queue-title {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .sync-queue-close {
  color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .sync-queue-close:hover {
  background: rgba(55, 65, 81, 0.6);
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .sync-queue-tabs {
  background: rgba(17, 24, 39, 0.5);
  border-bottom-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .tab-btn {
  color: var(--color-text-secondary, #9ca3af);
}

[data-theme='dark'] .tab-btn:hover {
  color: var(--color-text-primary, #f9fafb);
  background: rgba(55, 65, 81, 0.4);
}

[data-theme='dark'] .tab-btn.active {
  color: var(--color-primary, #60a5fa);
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .sync-queue-empty {
  background: rgba(17, 24, 39, 0.5);
}

[data-theme='dark'] .sync-queue-item {
  background: rgba(17, 24, 39, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .sync-queue-item:hover {
  background: var(--color-surface, #1f2937);
  border-color: var(--color-primary, #3b82f6);
}

[data-theme='dark'] .item-email {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .item-id {
  background: rgba(17, 24, 39, 0.8);
  color: var(--color-text-secondary, #9ca3af);
}

[data-theme='dark'] .sync-queue-footer {
  background: var(--color-surface, #1f2937);
  border-top-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .btn-close-queue {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
  color: var(--color-text-secondary, #9ca3af);
}

[data-theme='dark'] .btn-close-queue:hover {
  background: rgba(55, 65, 81, 0.8);
}
</style>
