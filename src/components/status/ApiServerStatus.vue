<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content api-server-modal">
      <div class="modal-header">
        <h2>{{ $t('apiServer.title') }}</h2>
        <button @click="$emit('close')" class="close-btn" :aria-label="$t('common.close')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <div class="modal-body">
        <!-- 服务器状态 -->
        <div class="status-section">
          <div class="status-row">
            <span class="label">{{ $t('apiServer.status') }}:</span>
            <span class="status-indicator" :class="{ running: serverStatus.running }">
              <span class="status-dot"></span>
              {{ serverStatus.running ? $t('apiServer.running') : $t('apiServer.stopped') }}
            </span>
          </div>

          <div v-if="serverStatus.running" class="info-rows">
            <div class="info-row">
              <span class="label">{{ $t('apiServer.address') }}:</span>
              <span class="value">{{ serverStatus.address }}</span>
            </div>
            <div class="info-row">
              <span class="label">{{ $t('apiServer.port') }}:</span>
              <span class="value">{{ serverStatus.port }}</span>
            </div>
          </div>
        </div>

        <!-- 可用端点 -->
        <div v-if="serverStatus.running" class="endpoints-section">
          <h3>{{ $t('apiServer.endpoints') }}</h3>
          <div class="endpoint-list">
            <div class="endpoint-item">
              <span class="method get">GET</span>
              <span class="path">/api/health</span>
            </div>
            <div class="endpoint-item">
              <span class="method post">POST</span>
              <span class="path">/api/import/session</span>
            </div>
            <div class="endpoint-item">
              <span class="method post">POST</span>
              <span class="path">/api/import/sessions</span>
            </div>
          </div>
        </div>

        <!-- API 使用示例 -->
        <div v-if="serverStatus.running" class="examples-section">
          <h3>{{ $t('apiServer.examples') }}</h3>
          <div class="example-box">
            <pre><code>{{ apiExamples }}</code></pre>
            <button @click="copyExamples" class="copy-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
              {{ $t('apiServer.copyExample') }}
            </button>
          </div>
        </div>

        <!-- 自动启动选项 -->
        <div class="auto-start-section">
          <label class="checkbox-label">
            <input type="checkbox" v-model="autoStart" @change="saveAutoStartPreference" />
            <span>{{ $t('apiServer.autoStart') }}</span>
          </label>
        </div>

        <!-- 控制按钮 -->
        <div class="control-buttons">
          <button
            v-if="!serverStatus.running"
            @click="startServer"
            :disabled="isLoading"
            class="btn primary"
          >
            {{ isLoading ? $t('apiServer.starting') : $t('apiServer.startServer') }}
          </button>
          <button
            v-else
            @click="stopServer"
            :disabled="isLoading"
            class="btn danger"
          >
            {{ isLoading ? $t('apiServer.stopping') : $t('apiServer.stopServer') }}
          </button>
        </div>

        <!-- 状态消息 -->
        <div v-if="statusMessage" class="status-message" :class="statusMessageType">
          {{ statusMessage }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'

const { t } = useI18n()

const emit = defineEmits(['close'])

// Use settings store
const settingsStore = useSettingsStore()
const serverStatus = computed(() => settingsStore.serverStatus)

const isLoading = ref(false)
const statusMessage = ref('')
const statusMessageType = ref('info')
const autoStart = ref(true)

// API 使用示例
const apiExamples = computed(() => {
  const addr = serverStatus.value.address || 'http://127.0.0.1:8766'
  return `# ${t('apiServer.healthCheck')}
curl ${addr}/api/health

# ${t('apiServer.importSingleSession')}
curl -X POST ${addr}/api/import/session \\
  -H "Content-Type: application/json" \\
  -d '{"session":"your_session_here"}'

# ${t('apiServer.importMultipleSessions')}
curl -X POST ${addr}/api/import/sessions \\
  -H "Content-Type: application/json" \\
  -d '{"sessions":["session1","session2"]}'`
})

// 加载服务器状态 - 使用store并强制刷新
const loadStatus = async () => {
  try {
    await settingsStore.loadServerStatus(true) // force refresh
  } catch (error) {
    console.error('Failed to load API server status:', error)
    showMessage(t('apiServer.loadStatusFailed'), 'error')
  }
}

// 启动服务器
const startServer = async () => {
  isLoading.value = true
  statusMessage.value = ''
  
  try {
    await invoke('start_api_server_cmd')
    await loadStatus()
    showMessage(t('apiServer.startSuccess'), 'success')
  } catch (error) {
    console.error('Failed to start API server:', error)
    showMessage(`${t('apiServer.startFailed')}: ${error}`, 'error')
  } finally {
    isLoading.value = false
  }
}

// 停止服务器
const stopServer = async () => {
  isLoading.value = true
  statusMessage.value = ''
  
  try {
    await invoke('stop_api_server')
    await loadStatus()
    showMessage(t('apiServer.stopSuccess'), 'success')
  } catch (error) {
    console.error('Failed to stop API server:', error)
    showMessage(`${t('apiServer.stopFailed')}: ${error}`, 'error')
  } finally {
    isLoading.value = false
  }
}

// 复制示例
const copyExamples = async () => {
  try {
    await navigator.clipboard.writeText(apiExamples.value)
    showMessage(t('apiServer.exampleCopied'), 'success')
  } catch (error) {
    console.error('Failed to copy examples:', error)
    showMessage(t('apiServer.copyFailed'), 'error')
  }
}

// 显示消息
const showMessage = (message, type = 'info') => {
  statusMessage.value = message
  statusMessageType.value = type
  setTimeout(() => {
    statusMessage.value = ''
  }, 3000)
}

// 保存自动启动偏好
const saveAutoStartPreference = () => {
  localStorage.setItem('api-server-auto-start', autoStart.value.toString())
}

// 加载自动启动偏好
const loadAutoStartPreference = () => {
  const saved = localStorage.getItem('api-server-auto-start')
  if (saved !== null) {
    autoStart.value = saved === 'true'
  }
}

onMounted(() => {
  // 不需要重新加载,直接使用store中的数据
  // 如果需要最新数据,可以调用 loadStatus()
  loadAutoStartPreference()
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 90vw;
  max-height: 95vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.api-server-modal {
  width: 600px;
  max-width: 90vw;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
  border-radius: 12px 12px 0 0;
  flex-shrink: 0;
}

.modal-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

.close-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  color: var(--color-text-muted, #666);
  transition: color 0.2s;
}

.close-btn:hover {
  color: var(--color-text-primary, #333);
}

.modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
  background: var(--color-surface, #ffffff);
}

.status-section {
  margin-bottom: 24px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.label {
  font-weight: 500;
  color: var(--color-text-primary, #374151);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 6px;
  background: var(--color-surface-muted, #f3f4f6);
  color: var(--color-text-muted, #6b7280);
  font-size: 14px;
}

.status-indicator.running {
  background: var(--color-success-surface, #d1fae5);
  color: var(--color-success-text, #065f46);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-text-muted, #9ca3af);
}

.status-indicator.running .status-dot {
  background: var(--color-success, #10b981);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.info-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  gap: 8px;
  font-size: 14px;
}

.value {
  font-family: monospace;
  color: var(--color-text-primary, #374151);
}

.endpoints-section,
.examples-section {
  margin-bottom: 24px;
}

.endpoints-section h3,
.examples-section h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

.endpoint-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.endpoint-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 6px;
  font-size: 14px;
}

.method {
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
}

.method.get {
  background: var(--color-blue-soft-bg, #dbeafe);
  color: var(--color-blue-soft-text, #1e40af);
}

.method.post {
  background: var(--color-success-surface, #d1fae5);
  color: var(--color-success-text, #065f46);
}

.path {
  font-family: monospace;
  color: var(--color-text-primary, #374151);
}

.example-box {
  position: relative;
  background: var(--color-surface-muted, #f8f9fa);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  padding: 16px;
}

.example-box pre {
  margin: 0;
  overflow-x: auto;
}

.example-box code {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-primary, #374151);
}

.copy-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-primary, #374151);
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--color-blue-soft-bg, #dbeafe);
  border-color: var(--color-blue-soft-hover, #93c5fd);
}

.auto-start-section {
  margin-bottom: 24px;
  padding: 12px;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 8px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.control-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.btn {
  flex: 1;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn.primary {
  background: var(--color-btn-primary-bg, #3b82f6);
  color: var(--color-btn-primary-text, #ffffff);
}

.btn.primary:hover:not(:disabled) {
  background: var(--color-btn-primary-hover, #2563eb);
}

.btn.danger {
  background: var(--color-danger, #ef4444);
  color: #ffffff;
}

.btn.danger:hover:not(:disabled) {
  background: var(--color-danger-hover, #dc2626);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.status-message {
  padding: 12px;
  border-radius: 6px;
  font-size: 14px;
  text-align: center;
}

.status-message.success {
  background: var(--color-success-surface, #d1fae5);
  color: var(--color-success-text, #065f46);
}

.status-message.error {
  background: var(--color-danger-surface, #fee2e2);
  color: var(--color-danger-text, #991b1b);
}

.status-message.info {
  background: var(--color-blue-soft-bg, #dbeafe);
  color: var(--color-blue-soft-text, #1e40af);
}

/* Dark theme support */
[data-theme='dark'] .modal-content {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .modal-header {
  background: var(--color-surface-alt, #111827);
  border-bottom-color: var(--color-border, #374151);
}

[data-theme='dark'] .modal-header h2 {
  color: var(--color-text-heading, #f9fafb);
}

[data-theme='dark'] .close-btn {
  color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .close-btn:hover {
  background: var(--color-border, #374151);
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .modal-body {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .label {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .value {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .status-indicator {
  background: var(--color-surface-alt, #111827);
  color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .status-indicator.running {
  background: rgba(16, 185, 129, 0.2);
  color: #6ee7b7;
}

[data-theme='dark'] .endpoints-section h3,
[data-theme='dark'] .examples-section h3 {
  color: var(--color-text-heading, #f9fafb);
}

[data-theme='dark'] .endpoint-item {
  background: var(--color-surface-alt, #111827);
}

[data-theme='dark'] .path {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .example-box {
  background: var(--color-surface-alt, #111827);
  border-color: var(--color-border, #374151);
}

[data-theme='dark'] .example-box code {
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .copy-btn {
  background: var(--color-surface, #1f2937);
  border-color: var(--color-border, #374151);
  color: var(--color-text-primary, #f9fafb);
}

[data-theme='dark'] .copy-btn:hover {
  background: var(--color-surface-alt, #111827);
  border-color: var(--color-border, #4b5563);
}

[data-theme='dark'] .auto-start-section {
  background: var(--color-surface-alt, #111827);
}

[data-theme='dark'] .status-message.success {
  background: rgba(16, 185, 129, 0.2);
  color: #6ee7b7;
}

[data-theme='dark'] .status-message.error {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

[data-theme='dark'] .status-message.info {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
}
</style>

