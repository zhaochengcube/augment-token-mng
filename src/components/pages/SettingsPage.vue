<template>
  <div class="settings-page">
    <div class="page-header">
      <h2>{{ $t('app.settings') }}</h2>
      <p>{{ $t('settings.description') }}</p>
    </div>

    <div class="page-body">
      <!-- Configuration Cards Grid -->
      <div class="settings-grid">
        <!-- API Server Card -->
        <div class="settings-card clickable" @click="showApiServerModal = true">
          <div class="card-title-row">
            <h3>{{ $t('apiServer.title') }}</h3>
            <span class="status-badge" :class="{ enabled: serverStatus.running }">
              {{ serverStatus.running ? $t('apiServer.running') : $t('apiServer.stopped') }}
            </span>
          </div>
        </div>

        <!-- Proxy Settings Card -->
        <div class="settings-card clickable" @click="showProxyModal = true">
          <div class="card-title-row">
            <h3>{{ $t('proxyConfig.title') }}</h3>
            <span class="status-badge" :class="{ enabled: proxyEnabled }">
              {{ proxyEnabled ? $t('proxyConfig.enabled') : $t('proxyConfig.disabled') }}
            </span>
          </div>
        </div>

        <!-- Database Settings Card -->
        <div class="settings-card clickable" @click="showDatabaseModal = true">
          <div class="card-title-row">
            <h3>{{ $t('databaseConfig.title') }}</h3>
            <span class="status-badge" :class="{ enabled: databaseConnected }">
              {{ databaseConnected ? $t('databaseConfig.connected') : $t('databaseConfig.notConfigured') }}
            </span>
          </div>
        </div>
      </div>

      <!-- About Section -->
      <div class="settings-section about-section">
        <div class="section-header">
          <h3>{{ $t('settings.about') }}</h3>
        </div>
        <div class="section-content">
          <div class="info-row">
            <span class="label">{{ $t('settings.version') }}</span>
            <div class="value-with-action">
              <span class="value">v{{ appVersion }}</span>
              <button @click="checkForUpdates" class="btn-check-update" :disabled="checkingUpdate">
                <svg v-if="!checkingUpdate" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                </svg>
                {{ checkingUpdate ? $t('update.checkingForUpdates') : $t('app.checkForUpdates') }}
              </button>
            </div>
          </div>
          <div class="info-row">
            <span class="label">{{ $t('settings.author') }}</span>
            <span class="value">Cube</span>
          </div>
          <div class="info-row">
            <span class="label">{{ $t('settings.links') }}</span>
            <div class="external-links-row">
              <a href="https://github.com/zhaochengcube/augment-token-mng" target="_blank" class="external-link app-home">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
                </svg>
                <span>{{ $t('app.appHome') }}</span>
              </a>
              <a href="https://github.com/zhaochengcube/augment-code-auto" target="_blank" class="external-link plugin-home">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-2 .9-2 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z" />
                </svg>
                <span>{{ $t('app.pluginHome') }}</span>
              </a>
              <a href="https://t.me/+kwZP2akb9es1Yjg9" target="_blank" class="external-link telegram-group">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M9.78 18.65l.28-4.23 7.68-6.92c.34-.31-.07-.46-.52-.19L7.74 13.3 3.64 12c-.88-.25-.89-.86.2-1.3l15.97-6.16c.73-.33 1.43.18 1.15 1.3l-2.72 12.81c-.19.91-.74 1.13-1.5.71L12.6 16.3l-1.99 1.93c-.23.23-.42.42-.83.42z"/>
                </svg>
                <span>{{ $t('app.telegramGroup') }}</span>
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <ApiServerStatus v-if="showApiServerModal" @close="showApiServerModal = false" />
    <ProxyConfig v-if="showProxyModal" @close="showProxyModal = false" />
    <DatabaseConfig v-if="showDatabaseModal" @close="showDatabaseModal = false" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'
import ApiServerStatus from '../status/ApiServerStatus.vue'
import ProxyConfig from '../settings/ProxyConfig.vue'
import DatabaseConfig from '../settings/DatabaseConfig.vue'

// i18n
const { t } = useI18n()

// Settings store
const settingsStore = useSettingsStore()

// Modal states
const showApiServerModal = ref(false)
const showProxyModal = ref(false)
const showDatabaseModal = ref(false)

// Update check
const checkingUpdate = ref(false)

// Computed properties from store
const appVersion = computed(() => settingsStore.appVersion)
const serverStatus = computed(() => settingsStore.serverStatus)
const proxyEnabled = computed(() => settingsStore.proxyConfig.enabled)
const databaseConnected = computed(() => settingsStore.databaseConfig.enabled)

// Methods

const checkForUpdates = async () => {
  checkingUpdate.value = true
  try {
    const updateInfo = await invoke('check_for_updates')
    if (updateInfo && updateInfo.has_update) {
      // Show update notification
      alert(`${t('update.newVersionAvailable')}: ${updateInfo.latest_version}`)
    } else {
      alert(t('update.upToDate'))
    }
  } catch (error) {
    console.error('Failed to check for updates:', error)
    alert(t('update.checkFailed'))
  } finally {
    checkingUpdate.value = false
  }
}

// Initialize - 从store加载所有设置
onMounted(() => {
  // 使用store的loadAllSettings方法,它会自动处理缓存
  settingsStore.loadAllSettings()
})
</script>

<style scoped>
.settings-page {
  max-width: 900px;
  margin: 0 auto;
  padding: 24px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-header {
  margin-bottom: 20px;
  flex-shrink: 0;
}

.page-header h2 {
  font-size: 22px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0 0 4px 0;
}

.page-header p {
  font-size: 14px;
  color: var(--text-muted);
  margin: 0;
}


.page-body {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* Settings Grid */
.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

/* Settings Card */
.settings-card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 16px;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-card.clickable {
  cursor: pointer;
}

.settings-card.clickable:hover {
  border-color: var(--primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.card-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.card-info {
  font-size: 13px;
  color: var(--text-muted);
  font-family: 'Consolas', 'Monaco', monospace;
}

.card-title-row h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
  flex: 1;
  min-width: 0;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  background: var(--bg-muted);
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
}

.status-badge.running,
.status-badge.enabled {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.card-content {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.info-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.info-label {
  font-size: 13px;
  color: var(--text-muted);
}

.info-value {
  font-size: 13px;
  color: var(--text-strong);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* About Section */
.about-section {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.section-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
}

.section-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-muted);
  min-width: 80px;
  flex-shrink: 0;
}

.value {
  font-size: 14px;
  color: var(--text-strong);
  font-family: 'Consolas', 'Monaco', monospace;
}

.value-with-action {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.btn-check-update {
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--border);
  background: var(--bg-surface);
  color: var(--text);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.btn-check-update:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--primary);
  color: var(--primary);
}

.btn-check-update:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.external-links-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
  background: var(--bg-base);
  color: var(--text-muted);
}

.status-indicator.running {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.link {
  color: var(--primary);
  text-decoration: none;
  font-size: 14px;
  transition: all 0.2s;
}

.link:hover {
  color: var(--primary-hover);
  text-decoration: underline;
}

.external-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 4px;
  text-decoration: none;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s;
  border: 1px solid transparent;
  white-space: nowrap;
}

.external-link.app-home {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  border-color: rgba(59, 130, 246, 0.2);
}

.external-link.app-home:hover {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
}

.external-link.plugin-home {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border-color: rgba(34, 197, 94, 0.2);
}

.external-link.plugin-home:hover {
  background: rgba(34, 197, 94, 0.15);
  border-color: rgba(34, 197, 94, 0.3);
}

.external-link.telegram-group {
  background: rgba(34, 167, 240, 0.1);
  color: #22a7f0;
  border-color: rgba(34, 167, 240, 0.2);
}

.external-link.telegram-group:hover {
  background: rgba(34, 167, 240, 0.15);
  border-color: rgba(34, 167, 240, 0.3);
}
</style>



