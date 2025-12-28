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
            <button @click="close" class="modal-close">
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
                    class="btn primary"
                    :disabled="syncing || totalTokensCount === 0"
                    >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
              </svg>
              {{ $t('tokenList.fullSync') }}
            </button>
            <div class="footer-right">
              <button @click="close" class="btn secondary">
                {{ $t('common.close') }}
              </button>
              <button v-if="pendingUpserts.length > 0 || pendingDeletions.length > 0" 
                      @click="handleSync" 
                      class="btn primary"
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
/* ============================================
   SyncQueueModal - Modern Tech Style
   ============================================ */

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

/* 模态框 - 磨砂玻璃 */
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

.sync-queue-title svg {
  color: var(--accent);
  filter: drop-shadow(0 0 8px var(--tech-glow-primary));
}

.sync-queue-title h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

/* Tabs - 科技风 */
.sync-queue-tabs {
  display: flex;
  padding: 0 26px 0 0;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-bottom: 1px solid var(--tech-glass-border);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 22px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-muted);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: -1px;
}

.tab-btn:hover {
  color: var(--text);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.tab-btn.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
  background: transparent;
  box-shadow: 0 2px 10px var(--tech-glow-primary);
}

.tab-btn svg {
  opacity: 0.7;
}

.tab-btn.active svg {
  opacity: 1;
  filter: drop-shadow(0 0 5px currentColor);
}

.tab-count {
  font-size: 12px;
  font-weight: 700;
  font-family: var(--tech-mono-font);
  padding: 3px 10px;
  border-radius: 12px;
  min-width: 28px;
  text-align: center;
}

.tab-count.upserts {
  background: color-mix(in srgb, #10b981 18%, transparent);
  color: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.3);
}

.tab-count.deletions {
  background: color-mix(in srgb, #ef4444 18%, transparent);
  color: #ef4444;
  box-shadow: 0 0 8px var(--tech-glow-danger);
}

/* Body & Tab Content */
.sync-queue-body {
  flex: 1;
  overflow-y: auto;
  min-height: 200px;
  max-height: 400px;
}

.tab-content {
  padding: 22px 26px;
  height: 100%;
}

.sync-queue-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 28px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px dashed var(--tech-glass-border);
  border-radius: 12px;
  color: var(--text-muted);
  font-size: 13px;
  gap: 10px;
}

.sync-queue-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.sync-queue-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border-radius: 12px;
  border: 1px solid var(--tech-glass-border);
  transition: all 0.2s ease;
}

.sync-queue-item:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.sync-queue-item.deletion {
  border-left: 3px solid #ef4444;
}

.item-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.item-icon.upsert {
  background: color-mix(in srgb, #10b981 18%, transparent);
  color: #10b981;
  box-shadow: 0 0 10px rgba(16, 185, 129, 0.3);
}

.item-icon.delete {
  background: color-mix(in srgb, #ef4444 18%, transparent);
  color: #ef4444;
  box-shadow: 0 0 10px var(--tech-glow-danger);
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-email {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-id {
  font-size: 12px;
  font-family: var(--tech-mono-font);
  background: color-mix(in srgb, var(--bg-muted) 70%, transparent);
  padding: 5px 10px;
  border-radius: 6px;
  color: var(--text-muted);
}

/* Footer - 科技风 */
.sync-queue-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 14px;
  padding: 18px 26px;
  border-top: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.footer-right {
  display: flex;
  gap: 14px;
}

/* Modal 动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.25s ease;
}

.modal-enter-active .sync-queue-dialog,
.modal-leave-active .sync-queue-dialog {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .sync-queue-dialog,
.modal-leave-to .sync-queue-dialog {
  transform: scale(0.92) translateY(10px);
  opacity: 0;
}
</style>
