<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content add-account-modal">
      <div class="modal-header">
        <h3>{{ $t('platform.antigravity.addAccountDialog.title') }}</h3>
        <button @click="$emit('close')" class="modal-close">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <!-- 添加方式选择 -->
        <div class="method-tabs">
          <button
            :class="['method-tab', { active: addMethod === 'oauth' }]"
            @click="addMethod = 'oauth'; oauthStep = 0; authCode = ''"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
            </svg>
            {{ $t('platform.antigravity.addAccountDialog.oauthMethod') }}
          </button>
          <button
            :class="['method-tab', { active: addMethod === 'manual' }]"
            @click="addMethod = 'manual'"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
            </svg>
            {{ $t('platform.antigravity.addAccountDialog.manualMethod') }}
          </button>
        </div>

        <!-- OAuth 授权方式 -->
        <div v-if="addMethod === 'oauth'" class="oauth-section">
          <div class="oauth-start">
            <div class="oauth-info">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
              </svg>
              <p>{{ $t('platform.antigravity.addAccountDialog.oauthInfo') }}</p>
            </div>

            <button @click="handleOAuthLogin" class="oauth-btn" :disabled="isLoading">
              <svg v-if="isLoading" class="spin" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
              </svg>
              <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12.545,10.239v3.821h5.445c-0.712,2.315-2.647,3.972-5.445,3.972c-3.332,0-6.033-2.701-6.033-6.032s2.701-6.032,6.033-6.032c1.498,0,2.866,0.549,3.921,1.453l2.814-2.814C17.503,2.988,15.139,2,12.545,2C7.021,2,2.543,6.477,2.543,12s4.478,10,10.002,10c8.396,0,10.249-7.85,9.426-11.748L12.545,10.239z"/>
              </svg>
              <span v-if="isLoading">{{ $t('platform.antigravity.addAccountDialog.adding') }}</span>
              <span v-else>{{ $t('platform.antigravity.addAccountDialog.googleLogin') }}</span>
            </button>
          </div>
        </div>

        <!-- 手动添加方式 -->
        <div v-else class="manual-section">
          <div class="form-group">
            <label>{{ $t('platform.antigravity.addAccountDialog.email') }}</label>
            <input
              v-model="email"
              type="email"
              :placeholder="$t('platform.antigravity.addAccountDialog.emailPlaceholder')"
              class="form-input"
              :disabled="isLoading"
            />
          </div>

          <div class="form-group">
            <label>{{ $t('platform.antigravity.addAccountDialog.refreshToken') }}</label>
            <textarea
              v-model="refreshToken"
              :placeholder="$t('platform.antigravity.addAccountDialog.refreshTokenPlaceholder')"
              class="form-textarea"
              rows="4"
              :disabled="isLoading"
            ></textarea>
            <p class="form-hint">
              {{ $t('platform.antigravity.addAccountDialog.refreshTokenHint') }}
            </p>
          </div>
        </div>

        <div v-if="error" class="error-message">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
          </svg>
          {{ error }}
        </div>
      </div>

      <div class="modal-footer">
        <button @click="$emit('close')" class="btn secondary" :disabled="isLoading">
          {{ $t('common.cancel') }}
        </button>
        <button
          v-if="addMethod === 'manual'"
          @click="handleAdd"
          class="btn primary"
          :disabled="!canSubmit || isLoading"
        >
          <svg v-if="isLoading" class="spin" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
          </svg>
          <span v-else>{{ $t('platform.antigravity.addAccountDialog.add') }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'add', 'added'])

const addMethod = ref('oauth') // 'oauth' or 'manual'
const email = ref('')
const refreshToken = ref('')
const isLoading = ref(false)
const error = ref('')

const canSubmit = computed(() => {
  if (addMethod.value === 'oauth') return true
  return email.value.trim() && refreshToken.value.trim()
})

const handleOAuthLogin = async () => {
  error.value = ''
  isLoading.value = true

  try {
    // 使用 OAuth 服务器模式，自动完成整个流程
    const account = await invoke('antigravity_start_oauth_login')

    emit('added', account)

  } catch (err) {
    error.value = err.message || 'OAuth 授权失败'
  } finally {
    isLoading.value = false
  }
}

const handleAdd = async () => {
  if (!canSubmit.value) return

  error.value = ''
  isLoading.value = true

  try {
    await emit('add', email.value.trim(), refreshToken.value.trim())
  } catch (err) {
    error.value = err.message || '添加账号失败'
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.add-account-modal {
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.add-account-modal .modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.method-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  padding: 4px;
  background: var(--bg-muted);
  border-radius: 8px;
}

.method-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.method-tab:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.method-tab.active {
  background: var(--bg-surface);
  color: var(--accent);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.oauth-section, .manual-section {
  animation: fadeIn 0.3s;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.oauth-info {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid rgba(102, 126, 234, 0.2);
  border-radius: 8px;
  margin-bottom: 20px;
}

.oauth-info svg {
  flex-shrink: 0;
  color: var(--accent);
  margin-top: 2px;
}

.oauth-info p {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.oauth-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 14px 20px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: white;
  color: #333;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.oauth-btn:hover:not(:disabled) {
  background: #f8f9fa;
  border-color: #dadce0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.oauth-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.oauth-btn svg {
  width: 20px;
  height: 20px;
}

.oauth-code-input {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.oauth-actions {
  display: flex;
  gap: 12px;
}

.oauth-actions button {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary {
  background: var(--bg-muted);
  color: var(--text);
  border: 1px solid var(--border);
}

.btn-secondary:hover {
  background: var(--bg-hover);
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-soft);
}

.error-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  color: #ef4444;
  font-size: 13px;
  margin-top: 16px;
}

.add-account-modal .modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.add-account-modal .modal-footer .btn {
  margin: 0;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
