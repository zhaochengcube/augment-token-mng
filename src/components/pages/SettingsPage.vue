<template>
  <div class="mx-auto flex h-full max-w-[920px] flex-col p-4">
    <!-- Page Header -->
    <div class="mb-6 shrink-0">
      <h2 class="mb-1.5 text-2xl font-bold text-text">{{ $t('app.settings') }}</h2>
      <p class="text-sm text-text-muted">{{ $t('settings.description') }}</p>
    </div>

    <!-- Page Body -->
    <div class="flex flex-1 flex-col gap-[26px] overflow-visiable">
      <!-- Configuration Cards Grid -->
      <div class="grid auto-rows-auto grid-cols-[repeat(auto-fit,minmax(290px,1fr))] gap-[18px] pt-1">
        <div
          v-for="card in configCards"
          :key="card.id"
          class="card card--lift p-4"
          @click="openModal(card.id)"
        >
          <div class="flex items-center justify-between gap-3">
            <h3 class="min-w-0 flex-1 text-base font-semibold text-text">{{ $t(card.titleKey) }}</h3>
            <span class="badge" :class="card.isActive ? 'badge--success-tech' : ''">
              <span v-if="card.showDot" class="status-dot"></span>
              {{ $t(card.isActive ? card.activeTextKey : card.inactiveTextKey) }}
            </span>
          </div>
        </div>
      </div>

      <!-- App Behavior Section -->
      <div class="card rounded-lg p-[22px] backdrop-blur-[12px]">
        <!-- Section Header -->
        <div class="mb-[18px] flex items-center justify-between border-b border-border pb-3.5">
          <h3 class="text-base font-semibold text-text">{{ $t('settings.appBehavior') }}</h3>
        </div>

        <!-- Section Content -->
        <div class="flex flex-col gap-[18px]">
          <!-- System Tray Row -->
          <div class="flex items-center justify-between gap-[18px]">
            <div class="flex flex-col gap-1">
              <span class="text-sm font-semibold text-text">{{ $t('tray.title') }}</span>
              <span class="text-xs text-text-muted">{{ $t('tray.description') }}</span>
            </div>
            <button
              @click="handleTrayToggle"
              :disabled="isTogglingTray"
              class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2"
              :class="trayEnabled ? 'bg-accent' : 'bg-border'"
              role="switch"
              :aria-checked="trayEnabled"
            >
              <span
                class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out"
                :class="trayEnabled ? 'translate-x-6' : 'translate-x-1'"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- About Section -->
      <div class="card rounded-lg p-[22px] backdrop-blur-[12px]">
        <!-- Section Header -->
        <div class="mb-[18px] flex items-center justify-between border-b border-border pb-3.5">
          <h3 class="text-base font-semibold text-text">{{ $t('settings.about') }}</h3>
        </div>

        <!-- Section Content -->
        <div class="flex flex-col gap-[18px]">
          <!-- Version Row -->
          <div class="flex items-center gap-[18px]">
            <span class="min-w-[85px] shrink-0 text-sm font-semibold text-text-muted">{{ $t('settings.version') }}</span>
            <div class="flex flex-1 items-center gap-3.5">
              <span class="font-mono text-sm text-text">v{{ appVersion }}</span>
              <button
                @click="checkForUpdates"
                class="btn btn--sm btn--secondary"
                :disabled="checkingUpdate"
              >
                <span v-if="checkingUpdate" class="btn-spinner" aria-hidden="true"></span>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                </svg>
                {{ $t('app.checkForUpdates') }}
              </button>
            </div>
          </div>

          <!-- Author Row -->
          <div class="flex items-center gap-[18px]">
            <span class="min-w-[85px] shrink-0 text-sm font-semibold text-text-muted">{{ $t('settings.author') }}</span>
            <span class="font-mono text-sm text-text">Cube</span>
          </div>

          <!-- Links Row -->
          <div class="flex items-center gap-[18px]">
            <span class="min-w-[85px] shrink-0 text-sm font-semibold text-text-muted">{{ $t('settings.links') }}</span>
            <div class="flex flex-1 flex-wrap items-center gap-2.5">
              <a href="https://github.com/cubezhao/augment-token-mng" target="_blank" class="btn btn--sm btn-tech-accent">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
                </svg>
                <span>{{ $t('app.appHome') }}</span>
              </a>
              <a href="https://github.com/cubezhao/augment-code-auto" target="_blank" class="btn btn--sm btn-tech-accent">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-2 .9-2 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z" />
                </svg>
                <span>{{ $t('app.pluginHome') }}</span>
              </a>
              <a href="https://t.me/+kwZP2akb9es1Yjg9" target="_blank" class="btn btn--sm btn-tech-accent">
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
    <FontConfig v-if="showFontModal" @close="showFontModal = false" />
    <TelegramConfig v-if="showTelegramModal" @close="showTelegramModal = false" @saved="handleTelegramSaved" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../../stores/settings'
import ApiServerStatus from '../settings/ApiServerStatus.vue'
import ProxyConfig from '../settings/ProxyConfig.vue'
import DatabaseConfig from '../settings/DatabaseConfig.vue'
import FontConfig from '../settings/FontConfig.vue'
import TelegramConfig from '../settings/TelegramConfig.vue'

// i18n
const { t } = useI18n()

// Settings store
const settingsStore = useSettingsStore()

// Modal states
const showApiServerModal = ref(false)
const showProxyModal = ref(false)
const showDatabaseModal = ref(false)
const showFontModal = ref(false)
const showTelegramModal = ref(false)

// Update check
const checkingUpdate = ref(false)

// Computed properties from store
const appVersion = computed(() => settingsStore.appVersion)
const serverStatus = computed(() => settingsStore.serverStatus)
const proxyEnabled = computed(() => settingsStore.proxyConfig.enabled)
const databaseConnected = computed(() => settingsStore.databaseConfig.enabled)
const hasCustomFont = computed(() => !!localStorage.getItem('user-font-sans'))
const trayEnabled = computed(() => settingsStore.trayEnabled)
const telegramEnabled = computed(() => settingsStore.telegramConfig.enabled)

// Tray toggle state
const isTogglingTray = ref(false)

// Configuration cards data
const configCards = computed(() => [
  {
    id: 'apiServer',
    titleKey: 'apiServer.title',
    isActive: serverStatus.value.running,
    activeTextKey: 'apiServer.running',
    inactiveTextKey: 'apiServer.stopped',
    showDot: true
  },
  {
    id: 'proxy',
    titleKey: 'proxyConfig.title',
    isActive: proxyEnabled.value,
    activeTextKey: 'proxyConfig.enabled',
    inactiveTextKey: 'proxyConfig.disabled',
    showDot: true
  },
  {
    id: 'database',
    titleKey: 'databaseConfig.title',
    isActive: databaseConnected.value,
    activeTextKey: 'databaseConfig.connected',
    inactiveTextKey: 'databaseConfig.notConfigured',
    showDot: true
  },
  {
    id: 'font',
    titleKey: 'fontConfig.title',
    isActive: hasCustomFont.value,
    activeTextKey: 'fontConfig.customized',
    inactiveTextKey: 'fontConfig.default',
    showDot: true
  },
  {
    id: 'telegram',
    titleKey: 'telegram.title',
    isActive: telegramEnabled.value,
    activeTextKey: 'telegram.enabled',
    inactiveTextKey: 'telegram.disabled',
    showDot: true
  }
])

// Open modal by card id
const openModal = (cardId) => {
  switch (cardId) {
    case 'apiServer':
      showApiServerModal.value = true
      break
    case 'proxy':
      showProxyModal.value = true
      break
    case 'database':
      showDatabaseModal.value = true
      break
    case 'font':
      showFontModal.value = true
      break
    case 'telegram':
      showTelegramModal.value = true
      break
  }
}

// Methods
const handleTrayToggle = async () => {
  isTogglingTray.value = true
  try {
    await settingsStore.toggleTray(!trayEnabled.value)
  } catch (error) {
    console.error('Failed to toggle tray:', error)
    window.$notify?.error(t('tray.toggleFailed'))
  } finally {
    isTogglingTray.value = false
  }
}

const handleTelegramSaved = () => {
  // Reload telegram config after saving
  settingsStore.loadTelegramConfig(true)
}

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
