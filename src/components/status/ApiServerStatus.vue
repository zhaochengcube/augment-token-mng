<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content api-server-modal">
      <div class="modal-header">
        <h2>{{ $t('apiServer.title') }}</h2>
        <button @click="$emit('close')" class="modal-close" :aria-label="$t('common.close')">
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
            class="btn primary"
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
/* ============================================
   ApiServerStatus - Modern Tech Style
   ============================================ */

.api-server-modal {
  width: 620px;
  max-width: 90vw;
}

.api-server-modal.modal-content {
  display: flex;
  flex-direction: column;
}

.api-server-modal .modal-body {
  padding: 26px;
  overflow-y: auto;
  flex: 1;
}

.status-section {
  margin-bottom: 26px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 18px;
}

.label {
  font-weight: 600;
  color: var(--text);
}

/* 状态指示器 - 科技风 */
.status-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  color: var(--text-muted);
  font-size: 14px;
}

.status-indicator.running {
  background: color-mix(in srgb, #10b981 12%, transparent);
  border-color: color-mix(in srgb, #10b981 35%, transparent);
  color: #10b981;
  box-shadow: 0 0 12px rgba(16, 185, 129, 0.3);
}

.info-rows {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-row {
  display: flex;
  gap: 10px;
  font-size: 14px;
}

.value {
  font-family: var(--tech-mono-font);
  color: var(--text);
}

.endpoints-section,
.examples-section {
  margin-bottom: 26px;
}

.endpoints-section h3,
.examples-section h3 {
  margin: 0 0 14px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-strong);
}

.endpoint-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* 端点项 - 科技风 */
.endpoint-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 10px;
  font-size: 14px;
}

/* 方法标签 - 科技风 */
.method {
  padding: 4px 10px;
  border-radius: 6px;
  font-weight: 700;
  font-size: 11px;
  text-transform: uppercase;
}

.method.get {
  background: color-mix(in srgb, #3b82f6 15%, transparent);
  color: #3b82f6;
  border: 1px solid color-mix(in srgb, #3b82f6 30%, transparent);
}

.method.post {
  background: color-mix(in srgb, #10b981 15%, transparent);
  color: #10b981;
  border: 1px solid color-mix(in srgb, #10b981 30%, transparent);
}

.path {
  font-family: var(--tech-mono-font);
  color: var(--text);
}

/* 代码示例框 - 科技风 */
.example-box {
  position: relative;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  padding: 18px;
}

.example-box pre {
  margin: 0;
  overflow-x: auto;
}

.example-box code {
  font-family: var(--tech-mono-font);
  font-size: 13px;
  line-height: 1.6;
  color: var(--text);
}

/* 复制按钮 - 科技风 */
.copy-btn {
  position: absolute;
  top: 14px;
  right: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text);
  transition: all 0.2s ease;
}

.copy-btn:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}

/* 自动启动区域 - 科技风 */
.auto-start-section {
  margin-bottom: 26px;
  padding: 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  user-select: none;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: var(--accent);
}

.control-buttons {
  display: flex;
  gap: 14px;
  margin-bottom: 18px;
}

/* 状态消息 - 科技风 */
.status-message {
  padding: 14px;
  border-radius: 10px;
  font-size: 14px;
  text-align: center;
}

.status-message.success {
  background: color-mix(in srgb, #10b981 12%, transparent);
  color: #10b981;
  border: 1px solid color-mix(in srgb, #10b981 35%, transparent);
}

.status-message.error {
  background: color-mix(in srgb, #ef4444 12%, transparent);
  color: #ef4444;
  border: 1px solid color-mix(in srgb, #ef4444 35%, transparent);
}

.status-message.info {
  background: color-mix(in srgb, #3b82f6 12%, transparent);
  color: #3b82f6;
  border: 1px solid color-mix(in srgb, #3b82f6 35%, transparent);
}
</style>
