<template>
  <BaseModal
    :visible="true"
    :title="$t('apiServer.title')"
    :body-scroll="true"
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-5">
      <!-- 服务器状态 -->
      <div class="status-section">
        <div class="flex items-center gap-3 mb-3">
          <span class="text-[13px] font-semibold text-text">{{ $t('apiServer.status') }}:</span>
          <span :class="['badge', serverStatus.running ? 'badge--success' : '']">
            <span class="status-dot"></span>
            {{ serverStatus.running ? $t('apiServer.running') : $t('apiServer.stopped') }}
          </span>
        </div>

        <div v-if="serverStatus.running" class="flex flex-col gap-2 text-[13px]">
          <div class="flex gap-2">
            <span class="text-text-secondary">{{ $t('apiServer.address') }}:</span>
            <span class="font-mono text-text">{{ serverStatus.address }}</span>
          </div>
          <div class="flex gap-2">
            <span class="text-text-secondary">{{ $t('apiServer.port') }}:</span>
            <span class="font-mono text-text">{{ serverStatus.port }}</span>
          </div>
        </div>
      </div>

      <!-- 可用端点 -->
      <div v-if="serverStatus.running">
        <h4 class="text-[13px] font-semibold text-text mb-3">{{ $t('apiServer.endpoints') }}</h4>
        <div class="flex flex-col gap-2">
          <div
            v-for="endpoint in endpoints"
            :key="endpoint.path"
            class="flex items-center gap-3.5 px-3.5 py-2.5 bg-muted border border-border rounded-lg"
          >
            <span :class="['badge font-bold uppercase w-14 justify-center', endpoint.method === 'GET' ? 'badge--accent-tech' : 'badge--success-tech']">
              {{ endpoint.method }}
            </span>
            <span class="font-mono text-[13px] text-text">{{ endpoint.path }}</span>
          </div>
        </div>
      </div>

      <!-- API 使用示例 -->
      <div v-if="serverStatus.running">
        <h4 class="text-[13px] font-semibold text-text mb-3">{{ $t('apiServer.examples') }}</h4>
        <div class="relative bg-muted border border-border rounded-lg p-4">
          <pre class="m-0 overflow-x-auto"><code class="text-[12px] leading-relaxed text-text">{{ apiExamples }}</code></pre>
          <button @click="copyExamples" class="btn btn--sm btn--secondary absolute top-3 right-3">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            {{ $t('apiServer.copyExample') }}
          </button>
        </div>
      </div>

      <!-- 自动启动选项 -->
      <label class="flex items-center gap-2.5 cursor-pointer text-[13px] text-text">
        <input
          type="checkbox"
          v-model="autoStart"
          @change="saveAutoStartPreference"
          class="w-4 h-4 cursor-pointer accent-accent"
        />
        <span>{{ $t('apiServer.autoStart') }}</span>
      </label>
    </div>

    <template #footer>
      <button
        v-if="!serverStatus.running"
        @click="startServer"
        class="btn btn--primary"
        :disabled="isLoading"
      >
        <span v-if="isLoading" class="btn-spinner" aria-hidden="true"></span>
        {{ $t('apiServer.startServer') }}
      </button>
      <button
        v-else
        @click="stopServer"
        class="btn btn--danger"
        :disabled="isLoading"
      >
        <span v-if="isLoading" class="btn-spinner" aria-hidden="true"></span>
        {{ $t('apiServer.stopServer') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'
import BaseModal from '../common/BaseModal.vue'

const { t } = useI18n()

const emit = defineEmits(['close'])

// Use settings store
const settingsStore = useSettingsStore()
const serverStatus = computed(() => settingsStore.serverStatus)

const isLoading = ref(false)
const autoStart = ref(true)

// API 端点列表
const endpoints = [
  { method: 'GET', path: '/api/health' },
  { method: 'POST', path: '/api/import/session' },
  { method: 'POST', path: '/api/import/sessions' }
]

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
    window.$notify.error(t('apiServer.loadStatusFailed'))
  }
}

// 启动服务器
const startServer = async () => {
  isLoading.value = true

  try {
    await invoke('start_api_server_cmd')
    await loadStatus()
    window.$notify.success(t('apiServer.startSuccess'))
  } catch (error) {
    console.error('Failed to start API server:', error)
    window.$notify.error(`${t('apiServer.startFailed')}: ${error}`)
  } finally {
    isLoading.value = false
  }
}

// 停止服务器
const stopServer = async () => {
  isLoading.value = true

  try {
    await invoke('stop_api_server')
    await loadStatus()
    window.$notify.success(t('apiServer.stopSuccess'))
  } catch (error) {
    console.error('Failed to stop API server:', error)
    window.$notify.error(`${t('apiServer.stopFailed')}: ${error}`)
  } finally {
    isLoading.value = false
  }
}

// 复制示例
const copyExamples = async () => {
  try {
    await navigator.clipboard.writeText(apiExamples.value)
    window.$notify.success(t('apiServer.exampleCopied'))
  } catch (error) {
    console.error('Failed to copy examples:', error)
    window.$notify.error(t('apiServer.copyFailed'))
  }
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
