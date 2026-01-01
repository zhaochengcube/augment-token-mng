<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="visible" class="session-refresh-overlay" @click="close">
        <div class="session-refresh-dialog" @click.stop>
          <div class="session-refresh-header">
            <div class="session-refresh-title">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
              </svg>
              <h3>{{ $t('tokenList.sessionRefreshTitle') }}</h3>
            </div>
            <button @click="close" class="modal-close">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
          </div>

          <div class="session-refresh-body">
            <div v-if="expiringTokens.length === 0" class="session-refresh-empty">
              <svg width="40" height="40" viewBox="0 0 24 24" fill="currentColor" opacity="0.2">
                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
              </svg>
              <span>{{ $t('tokenList.noExpiringSession') }}</span>
            </div>
            <div v-else class="session-refresh-list">
              <div v-for="token in expiringTokens" :key="token.id" class="session-refresh-item">
                <div class="item-icon">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
                  </svg>
                </div>
                <div class="item-content">
                  <span class="item-email">{{ token.email_note || $t('tokenList.noEmailNote') }}</span>
                  <div class="item-meta">
                    <span
                      :class="['status-badge', getStatusClass(token.ban_status)]"
                    >
                      <span :class="['status-dot', getStatusClass(token.ban_status)]"></span>
                      {{ getStatusText(token.ban_status) }}
                    </span>
                    <span class="item-days">{{ getDaysRemaining(token) }} {{ $t('tokenList.daysRemaining') }}</span>
                  </div>
                </div>
                <button
                  @click="handleRefreshSingle(token)"
                  class="btn-refresh-single"
                  :disabled="refreshingTokenId === token.id"
                  :title="$t('tokenList.refreshSession')"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" :class="{ spinning: refreshingTokenId === token.id }">
                    <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>

          <div class="session-refresh-footer">
            <button
              v-if="!refreshing"
              @click="handleRefreshAll"
              class="btn primary"
              :disabled="expiringTokens.length === 0"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              {{ $t('tokenList.refreshAllSessions') }}
            </button>
            <button v-else class="btn primary" disabled>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" class="spinning">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              {{ $t('tokenList.refreshing') }}
            </button>
            <button @click="close" class="btn secondary">
              {{ $t('common.close') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  expiringTokens: {
    type: Array,
    default: () => []
  },
  refreshing: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:visible', 'refresh', 'refresh-single'])

// 当前正在刷新的 token ID
const refreshingTokenId = ref(null)

const close = () => {
  emit('update:visible', false)
}

const handleRefreshAll = () => {
  emit('refresh')
}

// 单个刷新 session
const handleRefreshSingle = async (token) => {
  if (!token.auth_session) {
    window.$notify?.warning(t('messages.noAuthSession'))
    return
  }

  refreshingTokenId.value = token.id

  try {
    // 调用后端刷新接口
    const result = await invoke('batch_refresh_sessions', {
      requests: [{
        id: token.id,
        session: token.auth_session
      }]
    })

    if (result && result.length > 0 && result[0].success && result[0].new_session) {
      const now = new Date().toISOString()

      // 通知父组件更新 token
      emit('refresh-single', {
        tokenId: token.id,
        newSession: result[0].new_session,
        updatedAt: now
      })

      window.$notify?.success(t('messages.sessionRefreshSuccess', { count: 1 }))
    } else {
      const errorMsg = result[0]?.error || 'Unknown error'
      console.error(`Failed to refresh session for token ${token.id}:`, errorMsg)
      window.$notify?.error(t('messages.sessionRefreshFailed') + ': ' + errorMsg)
    }
  } catch (error) {
    console.error('Failed to refresh session:', error)
    window.$notify?.error(t('messages.sessionRefreshFailed') + ': ' + error.toString())
  } finally {
    refreshingTokenId.value = null
  }
}

const getDaysRemaining = (token) => {
  if (!token.created_at) return 0
  // 使用 session_updated_at（如果存在）或 created_at 来判断
  const referenceTime = token.session_updated_at || token.created_at
  const referenceDate = new Date(referenceTime)
  const now = new Date()
  const daysSinceReference = Math.floor((now - referenceDate) / (1000 * 60 * 60 * 24))
  return Math.max(0, 30 - daysSinceReference)
}

// 状态相关函数（与 TokenCard 和 TokenTableRow 保持一致）
const getStatusClass = (status) => {
  switch (status) {
    case 'SUSPENDED':
      return 'banned'
    case 'EXPIRED':
      return 'inactive'
    case 'INVALID_TOKEN':
      return 'invalid'
    case 'ACTIVE':
      return 'active'
    default:
      return 'active'
  }
}

const getStatusText = (status) => {
  switch (status) {
    case 'SUSPENDED':
      return t('tokenCard.banned')
    case 'EXPIRED':
      return t('tokenCard.expired')
    case 'INVALID_TOKEN':
      return t('tokenCard.tokenInvalid')
    case 'ACTIVE':
      return t('tokenCard.active')
    default:
      return t('tokenCard.active')
  }
}
</script>

<style scoped>

.session-refresh-overlay {
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
.session-refresh-dialog {
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

.session-refresh-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 22px 26px;
  border-bottom: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.session-refresh-title {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-strong);
}

.session-refresh-title svg {
  color: var(--warning, #f59e0b);
  filter: drop-shadow(0 0 8px rgba(245, 158, 11, 0.3));
}

.session-refresh-title h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.modal-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--text-muted);
  border-radius: 6px;
  transition: all 0.2s;
}

.modal-close:hover {
  background: var(--bg-muted);
  color: var(--text);
}

/* Body - 科技风 */
.session-refresh-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 26px;
  background: color-mix(in srgb, var(--bg-page) 50%, transparent);
}

.session-refresh-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 50px 20px;
  color: var(--text-muted);
  gap: 14px;
  text-align: center;
}

.session-refresh-empty span {
  font-size: 14px;
  font-weight: 500;
}

.session-refresh-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* List Item - 科技风 */
.session-refresh-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  transition: all 0.2s ease;
}

.session-refresh-item:hover {
  background: color-mix(in srgb, var(--bg-muted) 70%, transparent);
  border-color: color-mix(in srgb, var(--warning, #f59e0b) 30%, transparent);
  box-shadow: 0 0 12px rgba(245, 158, 11, 0.15);
}

.item-icon {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--warning, #f59e0b) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--warning, #f59e0b) 25%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--warning, #f59e0b);
}

.item-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}

.item-email {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-strong);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.item-days {
  font-size: 12px;
  font-weight: 500;
  color: var(--warning, #f59e0b);
  opacity: 0.8;
  white-space: nowrap;
}

/* 单个刷新按钮 */
.btn-refresh-single {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.btn-refresh-single:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--tech-glow-primary);
  transform: scale(1.05);
}

.btn-refresh-single:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.btn-refresh-single svg {
  display: block;
}

/* Footer - 科技风 */
.session-refresh-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 18px 26px;
  border-top: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* Modal 动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .session-refresh-dialog,
.modal-leave-active .session-refresh-dialog {
  transition: transform 0.3s ease;
}

.modal-enter-from .session-refresh-dialog,
.modal-leave-to .session-refresh-dialog {
  transform: scale(0.9);
}
</style>
