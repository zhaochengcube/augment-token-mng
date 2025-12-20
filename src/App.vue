<template>
  <div class="app">
    <!-- App Body: Sidebar + Main Content -->
    <div class="app-body">
      <!-- Sidebar -->
      <aside :class="['app-sidebar', { collapsed: isSidebarCollapsed }]">
        <div class="sidebar-content">
          <!-- Sidebar Top: Logo/Brand -->
          <div class="sidebar-top">
            <div class="sidebar-brand" @click="toggleSidebar" :title="isSidebarCollapsed ? '展开侧边栏' : '收起侧边栏'">
              <!-- 收缩时显示展开图标，展开时显示 ATM 文本 -->
              <svg v-if="isSidebarCollapsed" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" class="expand-icon">
                <path d="M3 18h13v-2H3v2zm0-5h10v-2H3v2zm0-7v2h13V6H3zm18 9.59L17.42 12 21 8.41 19.59 7l-5 5 5 5L21 15.59z"/>
              </svg>
              <span v-else class="brand-text">ATM</span>
            </div>
          </div>

          <!-- Sidebar Navigation -->
          <nav class="sidebar-nav">
            <button
              :class="['nav-item', { active: activeView === 'platforms' || activeView === 'platformDetail' }]"
              @click="navigateToView('platforms')"
              :title="$t('platforms.title')"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z"/>
              </svg>
              <span v-if="!isSidebarCollapsed">{{ $t('platforms.title') }}</span>
            </button>

            <button
              :class="['nav-item', { active: activeView === 'bookmarks' }]"
              @click="navigateToView('bookmarks')"
              :title="$t('app.bookmarkManager')"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z" />
              </svg>
              <span v-if="!isSidebarCollapsed">{{ $t('app.bookmarkManager') }}</span>
            </button>

            <button
              :class="['nav-item', { active: activeView === 'emails' }]"
              @click="navigateToView('emails')"
              :title="$t('emails.title')"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
              </svg>
              <span v-if="!isSidebarCollapsed">{{ $t('emails.title') }}</span>
            </button>
          </nav>

          <!-- Sidebar Bottom: Controls -->
          <div class="sidebar-bottom">
            <!-- Theme Toggle -->
            <button type="button" class="sidebar-control-btn" @click="toggleTheme"
              :aria-pressed="isDarkTheme" :aria-label="themeToggleLabel" :title="themeToggleLabel">
              <svg v-if="isDarkTheme" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="5" />
                <path d="m12 1 0 2" />
                <path d="m12 21 0 2" />
                <path d="m4.22 4.22 1.42 1.42" />
                <path d="m18.36 18.36 1.42 1.42" />
                <path d="m1 12 2 0" />
                <path d="m21 12 2 0" />
                <path d="m4.22 19.78 1.42-1.42" />
                <path d="m18.36 5.64 1.42-1.42" />
              </svg>
              <span v-if="!isSidebarCollapsed" class="control-label">{{ isDarkTheme ? $t('app.lightMode') : $t('app.darkMode') }}</span>
            </button>

            <!-- Language Toggle -->
            <button type="button" class="sidebar-control-btn" @click="toggleLanguage"
              :aria-label="languageToggleLabel" :title="languageToggleLabel">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <path d="M2 12h20"/>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
              </svg>
              <span v-if="!isSidebarCollapsed" class="control-label">{{ currentLocale === 'zh-CN' ? 'English' : '中文' }}</span>
            </button>

            <!-- Settings Button -->
            <button type="button" class="sidebar-control-btn"
              @click="navigateToView('settings')"
              :aria-label="$t('app.settings')"
              :title="$t('app.settings')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
              <span v-if="!isSidebarCollapsed" class="control-label">{{ $t('app.settings') }}</span>
            </button>
          </div>
        </div>
      </aside>

      <!-- Main Content Area -->
      <main class="main-content">
        <!-- Platform Selector View -->
        <PlatformSelector
          v-if="activeView === 'platforms'"
          :platforms="platforms"
          @select="navigateToPlatform"
        />

        <!-- Platform Detail View -->
        <div v-else-if="activeView === 'platformDetail'" class="platform-detail-view">
          <!-- Augment Token Manager -->
          <AugmentTokenManager v-if="activePlatformId === 'augment'" />

          <!-- Windsurf Token Manager (Coming Soon) -->
          <div v-else-if="activePlatformId === 'windsurf'" class="coming-soon">
            <h2>{{ $t('platform.windsurf.title') }}</h2>
            <p>{{ $t('messages.comingSoon') }}</p>
          </div>
        </div>

        <!-- Bookmarks View -->
        <BookmarkPage v-else-if="activeView === 'bookmarks'" :key="'bookmarks-' + viewRefreshKey" />

        <!-- Emails View -->
        <EmailPage v-else-if="activeView === 'emails'" :key="'emails-' + viewRefreshKey" />

        <!-- Settings View -->
        <SettingsPage v-else-if="activeView === 'settings'" :key="'settings-' + viewRefreshKey" />
      </main>
    </div><!-- End of app-body -->

    <!-- Session Help Modal -->
    <div v-if="showSessionHelp" class="help-modal" @click.self="showSessionHelp = false">
      <div class="help-content">
        <div class="help-header">
          <h2>{{ $t('sessionHelp.title') }}</h2>
          <button @click="showSessionHelp = false" class="close-button">×</button>
        </div>

        <div class="help-body">
          <div class="help-step">
            <h4>{{ $t('sessionHelp.step1Title') }}</h4>
            <p class="help-inline">
              {{ $t('sessionHelp.step1Content') }}
              <a :href="$t('sessionHelp.step1LoginLink')" target="_blank" class="help-link">
                {{ $t('sessionHelp.step1LoginLink') }} ↗
              </a>
            </p>
            <p class="help-inline">
              {{ $t('sessionHelp.step1LinkPrefix') }}
              <a :href="$t('sessionHelp.step1Link')" target="_blank" class="help-link">
                {{ $t('sessionHelp.step1Link') }} ↗
              </a>
            </p>
          </div>

          <div class="help-step">
            <h4>{{ $t('sessionHelp.step2Title') }}</h4>
            <p>{{ $t('sessionHelp.step2Content') }}</p>
          </div>

          <div class="help-step">
            <h4>{{ $t('sessionHelp.step3Title') }}</h4>
            <p>{{ $t('sessionHelp.step3Content') }}</p>
          </div>

          <div class="help-step">
            <h4>{{ $t('sessionHelp.step4Title') }}</h4>
            <p>{{ $t('sessionHelp.step4Content') }}</p>
          </div>

          <div class="help-step">
            <h4>{{ $t('sessionHelp.step5Title') }}</h4>
            <p>{{ $t('sessionHelp.step5Content') }}</p>
          </div>
        </div>
      </div>
    </div>


    <!-- 删除确认对话框 -->
    <div v-if="showDeleteConfirm" class="portal-dialog-overlay" @click="cancelDelete">
      <div class="portal-dialog delete-confirm" @click.stop>
        <h3>{{ $t('dialogs.confirmDelete') }}</h3>
        <p>{{ $t('dialogs.confirmDeleteMessage') }}</p>
        <div class="dialog-buttons">
          <button @click="cancelDelete" class="dialog-btn cancel">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
            </svg>
            {{ $t('dialogs.cancel') }}
          </button>
          <button @click="confirmDelete" class="dialog-btn delete">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
            </svg>
            {{ $t('dialogs.delete') }}
          </button>
        </div>
      </div>
    </div>


    <!-- Bookmark Manager Modal -->
    <BookmarkManager v-if="showBookmarkManager" @close="showBookmarkManager = false" />

    <!-- Outlook Manager Modal -->
    <OutlookManager v-if="showOutlookManager" @close="showOutlookManager = false" />

    <!-- GPTMail Manager Modal -->
    <GPTMailManager v-if="showGPTMailManager" @close="showGPTMailManager = false" />

    <!-- Proxy Config Modal -->
    <ProxyConfig v-if="showProxyConfig" @close="showProxyConfig = false" @config-saved="handleProxyConfigSaved" />

    <!-- API Server Status Modal -->
    <ApiServerStatus v-if="showApiServerStatus" @close="showApiServerStatus = false" />



    <!-- Notification Manager -->
    <NotificationManager ref="notificationManager" />

    <!-- 授权URL链接对话框 - removed, now handled in TokenForm -->
    <!-- App主页和插件主页链接已移至设置页面 -->

    <!-- 更新横幅 -->
    <UpdateBanner v-if="updateInfo && updateInfo.has_update" :update-info="updateInfo" @close="closeUpdateBanner" />
  </div>
</template>

<script setup>
import { ref, onMounted, computed, inject, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import BookmarkManager from './components/common/BookmarkManager.vue'
import OutlookManager from './components/email/OutlookManager.vue'
import GPTMailManager from './components/email/GPTMailManager.vue'
import ProxyConfig from './components/settings/ProxyConfig.vue'
import ExternalLinkDialog from './components/common/ExternalLinkDialog.vue'
import NotificationManager from './components/common/NotificationManager.vue'
import UpdateBanner from './components/common/UpdateBanner.vue'
import ApiServerStatus from './components/status/ApiServerStatus.vue'
import PlatformSelector from './components/platforms/PlatformSelector.vue'
import BookmarkPage from './components/pages/BookmarkPage.vue'
import EmailPage from './components/pages/EmailPage.vue'
import SettingsPage from './components/pages/SettingsPage.vue'
import AugmentTokenManager from './components/platform/AugmentTokenManager.vue'

const { t, locale } = useI18n()

// ========== 新增：主视图状态管理 ==========
// 主视图类型定义：'platforms' | 'platformDetail' | 'bookmarks' | 'emails' | 'settings'
const activeView = ref('platforms')
// 当前选中的平台：'augment' | 'windsurf' | null
const activePlatformId = ref(null)
// 侧边栏折叠状态
const isSidebarCollapsed = ref(false)

// 视图刷新计数器,用于强制重新渲染
const viewRefreshKey = ref(0)

// 导航函数
const navigateToView = (view) => {
  // 如果点击的是当前已激活的视图,增加刷新计数器强制重新渲染
  if (activeView.value === view) {
    viewRefreshKey.value++
  } else {
    activeView.value = view
  }

  if (view !== 'platformDetail') {
    activePlatformId.value = null
  }
}

const navigateToPlatform = (platformId) => {
  activePlatformId.value = platformId
  activeView.value = 'platformDetail'
}

// 侧边栏收缩/展开
const toggleSidebar = () => {
  isSidebarCollapsed.value = !isSidebarCollapsed.value
  // 保存状态到localStorage
  try {
    localStorage.setItem('sidebar-collapsed', isSidebarCollapsed.value.toString())
  } catch (error) {
    console.warn('Failed to save sidebar state', error)
  }
}

// 平台配置
const platforms = computed(() => [
  {
    id: 'augment',
    name: 'Augment',
    description: 'Augment Code AI 平台',
    icon: isDarkTheme.value ? '/icons/auggie_dark.svg' : '/icons/auggie.svg',
    status: 'ready'
  },
  {
    id: 'antigravity',
    name: 'Antigravity',
    description: 'Antigravity AI 平台',
    icon: '/icons/antigravity.png',
    status: 'coming_soon'
  },
  {
    id: 'windsurf',
    name: 'Windsurf',
    description: 'Windsurf AI 平台',
    icon: '/icons/windsurf.svg',
    status: 'coming_soon'
  }
])
// ========== 主视图状态管理结束 ==========

// 当前语言
const currentLocale = ref(locale.value)

// 切换语言
const changeLanguage = () => {
  locale.value = currentLocale.value
  // 可以在这里添加保存语言偏好到本地存储的逻辑
  localStorage.setItem('preferred-language', currentLocale.value)
}

// 语言切换按钮
const toggleLanguage = () => {
  currentLocale.value = currentLocale.value === 'zh-CN' ? 'en-US' : 'zh-CN'
  changeLanguage()
}

const languageToggleLabel = computed(() => (currentLocale.value === 'zh-CN' ? t('app.switchToEnglish') : t('app.switchToChinese')))

// 点击外部区域关闭设置菜单
const handleClickOutside = (event) => {
  // 保留此函数以防其他地方使用
}

// GPT邮箱管理器解锁/锁定功能
const unlockGPTMail = () => {
  isGPTMailUnlocked.value = true
  localStorage.setItem('gpt-mail-unlocked', 'true')
  showStatus(t('messages.gptMailUnlocked'), 'success')
}

const lockGPTMail = () => {
  isGPTMailUnlocked.value = false
  localStorage.removeItem('gpt-mail-unlocked')
  showStatus(t('messages.gptMailLocked'), 'info')
}

// 处理Logo点击事件(连续点击5次解锁)
const handleLogoClick = () => {
  logoClickCount.value++

  // 清除之前的定时器
  if (logoClickTimer) {
    clearTimeout(logoClickTimer)
  }

  // 如果已解锁,不处理
  if (isGPTMailUnlocked.value) {
    return
  }

  // 检查是否达到5次点击
  if (logoClickCount.value >= 5) {
    unlockGPTMail()
    logoClickCount.value = 0
    return
  }

  // 设置2秒超时,重置计数
  logoClickTimer = setTimeout(() => {
    logoClickCount.value = 0
  }, 2000)
}

// 处理快捷键(Ctrl+Shift+G)
const handleKeyboardShortcut = (event) => {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  const modifierKey = isMac ? event.metaKey : event.ctrlKey

  if (modifierKey && event.shiftKey && event.key.toLowerCase() === 'g') {
    event.preventDefault()
    if (isGPTMailUnlocked.value) {
      lockGPTMail()
    } else {
      unlockGPTMail()
    }
  }
}

const showBookmarkManager = ref(false)
const showOutlookManager = ref(false)
const showGPTMailManager = ref(false)
const showProxyConfig = ref(false)
const showApiServerStatus = ref(false)

// GPT邮箱管理器解锁状态
const isGPTMailUnlocked = ref(false)
const logoClickCount = ref(0)
let logoClickTimer = null

// 代理配置保存处理
const handleProxyConfigSaved = () => {
  // 通知已在 ProxyConfig 组件中显示,这里不需要重复显示
}

// 组件引用
const notificationManager = ref(null)


// Session import data (kept for backward compatibility with modals)
const sessionInput = ref('')
const sessionTokenResult = ref(null)
const isImportingSession = ref(false)
const sessionImportProgress = ref('')
const showSessionHelp = ref(false)
const isOpeningBrowser = ref(false)

// Template refs (removed old OAuth refs)




const themeManager = inject('themeManager', null)
const themeStorageKey = themeManager?.storageKey ?? 'atm-theme'

let storedThemePreference = null
try {
  storedThemePreference = themeManager?.getStoredTheme?.() ?? localStorage.getItem(themeStorageKey) ?? null
} catch (error) {
  console.warn('Failed to read stored theme preference inside App.vue', error)
}

const hasManualThemePreference = ref(storedThemePreference === 'dark' || storedThemePreference === 'light')
const currentTheme = ref(themeManager?.initialTheme ?? document.documentElement.dataset.theme ?? 'light')
const isDarkTheme = computed(() => currentTheme.value === 'dark')

const fallbackApplyTheme = (theme) => {
  const normalized = theme === 'dark' ? 'dark' : 'light'
  const root = document.documentElement
  root.dataset.theme = normalized
  root.style.colorScheme = normalized
}

const setTheme = (nextTheme, options = {}) => {
  const normalized = nextTheme === 'dark' ? 'dark' : 'light'
  currentTheme.value = normalized

  if (themeManager?.applyTheme) {
    themeManager.applyTheme(normalized)
  } else {
    fallbackApplyTheme(normalized)
  }

  if (options.persist === false) {
    return
  }

  try {
    localStorage.setItem(themeStorageKey, normalized)
    hasManualThemePreference.value = true
  } catch (error) {
    console.warn('Failed to persist theme preference', error)
  }
}

const toggleTheme = () => {
  setTheme(isDarkTheme.value ? 'light' : 'dark')
}

const themeToggleLabel = computed(() => (isDarkTheme.value ? t('app.switchToLight') : t('app.switchToDark')))

let cleanupSystemThemeListener

if (themeManager?.mediaQuery) {
  const mediaQuery = themeManager.mediaQuery
  const handleSystemThemeChange = (event) => {
    if (hasManualThemePreference.value) {
      return
    }
    setTheme(event.matches ? 'dark' : 'light', { persist: false })
  }

  if (typeof mediaQuery.addEventListener === 'function') {
    mediaQuery.addEventListener('change', handleSystemThemeChange)
    cleanupSystemThemeListener = () => mediaQuery.removeEventListener('change', handleSystemThemeChange)
  } else if (typeof mediaQuery.addListener === 'function') {
    mediaQuery.addListener(handleSystemThemeChange)
    cleanupSystemThemeListener = () => mediaQuery.removeListener(handleSystemThemeChange)
  }
}

onMounted(() => {
  setTheme(currentTheme.value, { persist: hasManualThemePreference.value })
})



// Delete confirmation dialog
const showDeleteConfirm = ref(false)
const tokenToDelete = ref(null)

// External links dialogs - removed, now in settings page

// Update check
const updateInfo = ref(null)

const checkForUpdates = async (silent = false) => {
  try {
    const result = await invoke('check_for_updates')
    if (result && result.has_update) {
      updateInfo.value = result
      if (!silent) {
        showStatus(t('update.newVersionAvailable'), 'info')
      }
    } else if (!silent) {
      showStatus(t('update.upToDate'), 'success')
    }
  } catch (error) {
    console.error('Failed to check for updates:', error)
    if (!silent) {
      showStatus(t('update.checkFailed'), 'error')
    }
  }
}

const closeUpdateBanner = () => {
  updateInfo.value = null
}

// Computed properties (OAuth-related removed)

// Methods
const showStatus = (message, type = 'info') => {
  // 优先使用全局$notify
  window.$notify[type](message)
}


const cancelDelete = () => {
  showDeleteConfirm.value = false
  tokenToDelete.value = null
}




// Token generator methods
const copyToClipboard = async (text) => {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
    return false
  }
}

// OAuth methods removed - now handled in TokenForm component


// Session import methods
const importFromSession = async () => {
  if (!sessionInput.value.trim()) {
    showStatus(t('messages.sessionRequired'), 'warning')
    return
  }

  isImportingSession.value = true
  sessionImportProgress.value = t('messages.sessionImportStarting')
  showStatus(t('messages.importingSession'), 'info')

  try {
    const authSession = sessionInput.value.trim()
    const result = await invoke('add_token_from_session', { session: authSession })

    // Session导入成功，跳转到Augment平台页
    sessionImportProgress.value = t('messages.sessionImportSuccess')
    showStatus(t('messages.sessionImportSuccess'), 'success')

    // 清空输入
    sessionInput.value = ''
    sessionTokenResult.value = null

    // 跳转到Augment平台页
    navigateToPlatform('augment')

  } catch (error) {
    sessionImportProgress.value = t('messages.sessionImportFailed')
    // 映射后端错误标识符到 i18n key
    let errorMessage = error
    if (error.includes('SESSION_ERROR_OR_ACCOUNT_BANNED')) {
      errorMessage = t('messages.sessionErrorOrAccountBanned')
    }
    showStatus(`${t('messages.error')}: ${errorMessage}`, 'error')
  } finally {
    isImportingSession.value = false
  }
}

// 打开内置浏览器进行自动导入
const openInternalBrowserForAutoImport = async () => {
  if (isOpeningBrowser.value) {
    return // 防止重复点击
  }

  isOpeningBrowser.value = true
  try {
    // 打开登录页面,登录后会跳转到 auth.augmentcode.com
    await invoke('open_internal_browser', {
      url: 'https://app.augmentcode.com/',
      title: t('tokenGenerator.autoImportBrowserTitle')
    })
  } catch (error) {
    showStatus(`${t('messages.error')}: ${error}`, 'error')
  } finally {
    // 延迟重置状态，避免窗口创建过程中再次点击
    setTimeout(() => {
      isOpeningBrowser.value = false
    }, 1000)
  }
}



// saveToken method removed - now handled in TokenForm component










onMounted(async () => {
  // 读取保存的语言偏好
  const savedLanguage = localStorage.getItem('preferred-language')
  if (savedLanguage && (savedLanguage === 'zh-CN' || savedLanguage === 'en-US')) {
    currentLocale.value = savedLanguage
    locale.value = savedLanguage
  }

  // 读取侧边栏折叠状态
  const sidebarCollapsed = localStorage.getItem('sidebar-collapsed')
  if (sidebarCollapsed === 'true') {
    isSidebarCollapsed.value = true
  }

  // 读取GPT邮箱管理器解锁状态
  const gptMailUnlocked = localStorage.getItem('gpt-mail-unlocked')
  if (gptMailUnlocked === 'true') {
    isGPTMailUnlocked.value = true
  }

  // 注册快捷键监听器
  document.addEventListener('keydown', handleKeyboardShortcut)

  // 启动时检查更新（静默模式）
  checkForUpdates(true)

  // 监听 Session 导入进度事件
  await listen('session-import-progress', (event) => {
    console.log('Progress event received:', event.payload)
    // 后端发送的是 i18n key,需要转换为翻译文本
    sessionImportProgress.value = t('messages.' + event.payload)
  })

  // 监听 Session 自动导入成功事件
  await listen('session-auto-imported', async (event) => {
    console.log('Session auto-imported:', event.payload)

    // Session导入成功，显示提示并跳转到Augment平台页
    showStatus(t('messages.sessionAutoImported'), 'success')
    navigateToPlatform('augment')
  })

  // 监听 Session 自动导入失败事件
  await listen('session-auto-import-failed', (event) => {
    console.error('Session auto-import failed:', event.payload)
    // 映射后端错误标识符到 i18n key
    let errorMessage = event.payload.error
    if (errorMessage.includes('SESSION_ERROR_OR_ACCOUNT_BANNED')) {
      errorMessage = t('messages.sessionErrorOrAccountBanned')
    }
    showStatus(t('messages.sessionAutoImportFailed') + ': ' + errorMessage, 'error')
  })

  // 监听 Deep Link Session 接收事件（由前端处理导入）
  await listen('deep-link-session-received', async (event) => {
    // 调用前端的导入方法（会显示进度和结果提示，并跳转到Augment平台页）
    if (event.payload.session) {
      sessionInput.value = event.payload.session
      await importFromSession()
    }
  })

  // 添加点击外部区域关闭设置菜单的事件监听器
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  if (typeof cleanupSystemThemeListener === 'function') {
    cleanupSystemThemeListener()
  }
  // 移除事件监听器
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeyboardShortcut)

  // 清除定时器
  if (logoClickTimer) {
    clearTimeout(logoClickTimer)
  }
})


</script>

<style scoped>
.app {
  height: 100vh;
  background: var(--color-surface-muted, #f8f9fa);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 隐藏所有滚动条 */
* {
  scrollbar-width: none;
  /* Firefox */
  -ms-overflow-style: none;
  /* IE and Edge */
}

*::-webkit-scrollbar {
  width: 0px;
  background: transparent;
}

/* 确保body和html不产生滚动条 */
html,
body {
  overflow: hidden;
  height: 100%;
  margin: 0;
  padding: 0;
}

/* App Body: Sidebar + Main Content */
.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Sidebar Styles */
.app-sidebar {
  width: 200px;
  background: var(--bg-surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width 0.3s ease;
}

.app-sidebar.collapsed {
  width: 64px;
}

.sidebar-content {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.sidebar-top {
  padding: 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-strong);
  font-weight: 600;
  font-size: 18px;
  cursor: pointer;
  transition: all 0.2s;
  user-select: none;
  width: 100%;
  padding: 4px;
  border-radius: 6px;
}

.sidebar-brand:hover {
  background: var(--bg-hover);
  color: var(--accent);
}

.app-sidebar.collapsed .sidebar-brand {
  font-size: 14px;
}

.brand-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: center;
  font-weight: 700;
  letter-spacing: 0.5px;
}

.expand-icon {
  flex-shrink: 0;
  transition: transform 0.2s;
}

.sidebar-brand:hover .expand-icon {
  transform: translateX(2px);
}

.sidebar-nav {
  flex: 1;
  padding: 16px 8px;
  overflow-y: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  margin-bottom: 4px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s ease;
  font-size: 14px;
  width: 100%;
  text-align: left;
}

.app-sidebar.collapsed .nav-item {
  justify-content: center;
  padding: 10px;
}

.nav-item:hover {
  background: var(--bg-muted);
  color: var(--text-strong);
}

.nav-item.active {
  background: var(--accent);
  color: var(--text-contrast);
}

.nav-item svg {
  flex-shrink: 0;
}

.nav-item span {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-divider {
  height: 1px;
  background: var(--border);
  margin: 8px 12px;
}

.sidebar-bottom {
  padding: 12px 8px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
}

.sidebar-control-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s ease;
  font-size: 14px;
  width: 100%;
  text-align: left;
}

.app-sidebar.collapsed .sidebar-control-btn {
  justify-content: center;
  padding: 10px;
}

.sidebar-control-btn:hover {
  background: var(--bg-muted);
  color: var(--text-strong);
}

.sidebar-control-btn svg {
  flex-shrink: 0;
}

.control-label {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Settings Menu Popup removed - now in SettingsPage */

.control-btn:hover svg {
  transform: scale(1.1);
}

/* 黑暗模式下的固定控制按钮样式 */
[data-theme='dark'] .control-btn {
  background: var(--color-surface, #1e293b);
  color: var(--color-text-primary, #cbd5e1);
  border-color: rgba(148, 163, 184, 0.35);
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.3);
}

[data-theme='dark'] .control-btn:hover {
  background: rgba(148, 163, 184, 0.16);
  border-color: rgba(148, 163, 184, 0.55);
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .control-btn.settings-toggle {
  background: var(--color-text-muted, #9ca3af);
  color: var(--color-text-inverse, #ffffff);
  border-color: var(--color-text-muted, #9ca3af);
}

[data-theme='dark'] .control-btn.settings-toggle:hover {
  background: var(--color-text-secondary, #d1d5db);
  border-color: var(--color-text-secondary, #d1d5db);
}







/* Main Content Area */
.main-content {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-page);
}



.generator-header {
  margin-bottom: 32px;
}

.header-content {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  gap: 20px;
}

.title-section {
  text-align: center;
}

.title-section h2 {
  margin: 0 0 8px 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--color-text-strong, #1f2937);
  line-height: 1.2;
}

.title-section p {
  margin: 0;
  font-size: 16px;
  color: var(--color-text-muted, #6b7280);
  line-height: 1.5;
}

/* Tab Navigation Styles */
.tab-navigation {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-top: 24px;
}

.tab-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
  border: 1px solid var(--color-border-strong, #d1d5db);
}

.tab-btn:hover {
  background: var(--color-border, #e5e7eb);
  border-color: var(--color-border-hover, #9ca3af);
}

.tab-btn.active {
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border: 1px solid var(--color-blue-soft-border, #90caf9);
}

.tab-btn.active:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
}

.tab-btn svg {
  flex-shrink: 0;
}

/* Tab Content */
.tab-content {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Session Loading State - 小巧版本 */
.session-loading {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  margin-top: 12px;
  background: var(--color-surface-hover, #f8f9fa);
  border-radius: 8px;
  border: 1px solid var(--color-border, #e5e7eb);
}

.session-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--color-border, #e5e7eb);
  border-top-color: var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.session-loading span {
  font-size: 14px;
  color: var(--color-text-secondary, #6b7280);
}

/* Session Header with Help Button */
.session-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.session-header h3 {
  margin: 0;
}

.help-button {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--color-text-muted, #6b7280);
  color: white;
  border: none;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.help-button:hover {
  background: var(--color-text-secondary, #4b5563);
}

/* Help Modal Styles */
.help-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  animation: fadeIn 0.2s;
  backdrop-filter: blur(6px);
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

.help-content {
  background: #ffffff;
  border-radius: 12px;
  max-width: 700px;
  max-height: 85vh;
  width: 90%;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s;
}

/* 深色主题下的帮助弹窗 */
:root[data-theme="dark"] .help-content {
  background: #1e293b;
}

@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }

  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.help-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 30px;
  border-bottom: 1px solid #e5e7eb;
}

:root[data-theme="dark"] .help-header {
  border-bottom-color: #374151;
}

.help-header h2 {
  margin: 0;
  color: #1f2937;
  font-size: 24px;
}

:root[data-theme="dark"] .help-header h2 {
  color: #f3f4f6;
}

.close-button {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: transparent;
  border: none;
  font-size: 28px;
  color: #6b7280;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  line-height: 1;
}

.close-button:hover {
  background: #f3f4f6;
  color: #1f2937;
}

:root[data-theme="dark"] .close-button {
  color: #9ca3af;
}

:root[data-theme="dark"] .close-button:hover {
  background: #374151;
  color: #f3f4f6;
}

.help-body {
  padding: 30px;
  overflow-y: auto;
  flex: 1;
}

.help-step {
  margin-bottom: 28px;
}

.help-step:last-child {
  margin-bottom: 0;
}

.help-step h4 {
  color: #4CAF50;
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
}

.help-step p {
  margin: 0 0 8px 0;
  color: #4b5563;
  line-height: 1.6;
  font-size: 14px;
}

:root[data-theme="dark"] .help-step p {
  color: #d1d5db;
}

.help-inline {
  margin: 4px 0;
  color: #4b5563;
  font-size: 14px;
  line-height: 1.6;
}

:root[data-theme="dark"] .help-inline {
  color: #d1d5db;
}

.help-link {
  color: #4CAF50;
  text-decoration: none;
  font-size: 14px;
}

.help-link:hover {
  text-decoration: underline;
}

.help-footer {
  padding: 20px 30px;
  border-top: 1px solid #e5e7eb;
  display: flex;
  justify-content: flex-end;
}

:root[data-theme="dark"] .help-footer {
  border-top-color: #374151;
}

.help-footer .btn {
  min-width: 100px;
}

.section-description {
  margin: 8px 0 16px 0;
  font-size: 14px;
  color: var(--color-text-muted, #6b7280);
  line-height: 1.5;
}

.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.btn.primary {
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border: 1px solid var(--color-blue-soft-border, #90caf9);
}

.btn.primary:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
}

.btn.secondary {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
  border: 1px solid var(--color-border-strong, #d1d5db);
}

.btn.secondary:hover {
  background: var(--color-border, #e5e7eb);
  border-color: var(--color-border-hover, #9ca3af);
}

.btn.warning {
  background: #faf5ff !important;
  color: #7c3aed !important;
  border: 1px solid #c4b5fd !important;
}

.btn.warning:hover {
  background: #ede9fe !important;
  border-color: #a78bfa !important;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(124, 58, 237, 0.3);
}

.btn.info {
  background: var(--color-info-surface, #dbeafe);
  color: var(--color-info-text, #1e40af);
  border: 1px solid var(--color-info-border, #93c5fd);
}

.btn.info:hover {
  background: var(--color-info-surface, #bfdbfe);
  border-color: var(--color-info-border, #60a5fa);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(30, 64, 175, 0.3);
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
}

.btn.disabled,
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.button-container {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.login-buttons {
  display: flex;
  gap: 6px;
  align-items: center;
}

/* 输入框样式 */
input[type="text"] {
  padding: 10px 12px;
  border: 1px solid var(--color-btn-secondary-border, #ddd);
  border-radius: 4px;
  font-size: 14px;
  transition: border-color 0.2s;
  width: 100%;
  box-sizing: border-box;
}

input[type="text"]:focus {
  outline: none;
  border-color: var(--color-blue-soft-text, #1976d2);
  box-shadow: 0 0 0 2px rgba(25, 118, 210, 0.1);
}

input[type="text"]:read-only {
  background-color: var(--color-surface-muted, #f8f9fa);
  color: var(--color-text-muted, #6c757d);
}

/* 带复制图标的输入框 */
.input-with-copy {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
}

.input-with-copy input {
  padding-right: 45px;
  flex: 1;
}

.copy-icon-btn {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  color: var(--color-text-muted, #6c757d);
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.copy-icon-btn:hover {
  background: var(--color-surface-muted, #e9ecef);
  color: var(--color-text-secondary, #495057);
}

.copy-icon-btn:active {
  transform: scale(0.95);
}



/* 响应式设计 */
@media (max-width: 768px) {
  .app-header {
    padding: 8px 12px;
    min-height: 56px;
  }

  .app-header h1 {
    font-size: 18px;
  }

  .header-buttons {
    gap: 6px;
  }

  .btn {
    padding: 8px 12px;
    font-size: 13px;
  }

  .btn.small {
    padding: 6px 10px;
    font-size: 12px;
  }

  .sidebar-control-btn {
    width: 36px;
    height: 36px;
  }
}



/* Modal styles */
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
  overflow-y: auto;
  position: relative;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
  border-radius: 12px 12px 0 0;
}

.modal-header h3 {
  margin: 0;
  color: var(--color-text-primary, #374151);
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--color-border, #e5e7eb);
  color: var(--color-text-primary, #374151);
}

@media (max-width: 480px) {
  .app-header {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 8px;
  }

  .header-left {
    justify-content: space-between;
    width: 100%;
  }

  .header-buttons {
    justify-content: space-between;
    width: 100%;
  }

  .app-sidebar {
    width: 200px;
  }

  .sidebar-control-btn {
    font-size: 13px;
    padding: 8px 10px;
  }

  .sidebar-brand {
    font-size: 14px;
  }

  .main-content {
    padding: 16px;
  }
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--color-text-muted, #666);
}

.empty-icon {
  margin-bottom: 24px;
  color: var(--color-btn-primary-disabled-bg, #ccc);
}

.empty-state h2 {
  margin: 0 0 12px 0;
  color: var(--color-text-heading, #333);
  font-size: 24px;
  font-weight: 600;
}

.empty-state p {
  margin: 0;
  font-size: 16px;
  line-height: 1.5;
}

.loading-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--color-text-muted, #666);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--color-surface-hover, #f3f3f3);
  border-top: 4px solid var(--color-blue-soft-text, #1976d2);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.token-list {
  width: 100%;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.list-header h2 {
  margin: 0;
  color: var(--color-text-heading, #333);
  font-size: 20px;
  font-weight: 600;
}


/* 黑暗主题下的按钮样式 */
[data-theme='dark'] .btn.warning {
  background: rgba(139, 92, 246, 0.2) !important;
  color: #c4b5fd !important;
  border: 1px solid rgba(196, 181, 253, 0.4) !important;
}

[data-theme='dark'] .btn.warning:hover {
  background: rgba(139, 92, 246, 0.3) !important;
  border-color: rgba(168, 139, 250, 0.6) !important;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(139, 92, 246, 0.4);
}

[data-theme='dark'] .btn.primary {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
  border: 1px solid rgba(147, 197, 253, 0.4);
}

[data-theme='dark'] .btn.primary:hover {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(96, 165, 250, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(59, 130, 246, 0.4);
}

[data-theme='dark'] .btn.app-home-btn {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
  border: 1px solid rgba(147, 197, 253, 0.4);
}

[data-theme='dark'] .btn.app-home-btn:hover {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(96, 165, 250, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(59, 130, 246, 0.4);
}

[data-theme='dark'] .btn.plugin-home-btn {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
  border: 1px solid rgba(134, 239, 172, 0.4);
}

[data-theme='dark'] .btn.plugin-home-btn:hover {
  background: rgba(34, 197, 94, 0.3);
  border-color: rgba(110, 231, 183, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(34, 197, 94, 0.4);
}

[data-theme='dark'] .btn.info {
  background: rgba(14, 165, 233, 0.2);
  color: #7dd3fc;
  border: 1px solid rgba(125, 211, 252, 0.4);
}

[data-theme='dark'] .btn.info:hover {
  background: rgba(14, 165, 233, 0.3);
  border-color: rgba(56, 189, 248, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(14, 165, 233, 0.4);
}


/* Portal对话框样式 */
.portal-dialog-overlay {
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
  /* 确保在所有其他元素之上 */
}

.portal-dialog {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  min-width: 300px;
  max-width: 400px;
}

.portal-dialog h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
  text-align: center;
}

.dialog-buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border: 2px solid transparent;
  border-radius: 8px;
  background: var(--color-surface-muted, #f8f9fa);
  color: var(--color-text-secondary, #495057);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  justify-content: center;
}

.dialog-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.dialog-btn.external {
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border-color: var(--color-blue-soft-border, #90caf9);
}

.dialog-btn.external:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
}

.dialog-btn.internal {
  background: var(--color-success-surface, #e8f5e8);
  color: var(--color-success-text, #2e7d32);
  border-color: var(--color-success-border, #a5d6a7);
}

.dialog-btn.internal:hover {
  background: var(--color-success-surface, #c8e6c9);
  border-color: var(--color-success-border, #81c784);
}

.dialog-btn.cancel {
  background: var(--color-rose-surface, #fce4ec);
  color: var(--color-rose-text, #c2185b);
  border-color: var(--color-rose-border, #f8bbd9);
}

.dialog-btn.cancel:hover {
  background: var(--color-rose-border, #f8bbd9);
  border-color: var(--color-rose-hover, #f48fb1);
}

.dialog-btn.delete {
  background: var(--color-danger-soft-surface, #ffebee);
  color: var(--color-danger-text, #d32f2f);
  border-color: var(--color-danger-soft-border, #ffcdd2);
}

.dialog-btn.delete:hover {
  background: var(--color-danger-soft-border, #ffcdd2);
  border-color: var(--color-danger-soft-border, #ef9a9a);
}

/* 删除确认对话框特定样式 */
.portal-dialog.delete-confirm p {
  margin: 0 0 20px 0;
  color: var(--color-text-muted, #666);
  text-align: center;
  line-height: 1.5;
}

.delete-confirm .dialog-buttons {
  flex-direction: row;
  gap: 12px;
}

.delete-confirm .dialog-btn {
  flex: 1;
}

.additional-fields {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--color-divider, #e1e5e9);
}

.field-container {
  margin-bottom: 15px;
}

.field-container label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
  color: var(--color-text-primary, #374151);
  font-size: 14px;
}

.field-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border-strong, #d1d5db);
  border-radius: 6px;
  font-size: 14px;
  background: var(--color-input-bg, #ffffff);
  color: var(--color-text-primary, #374151);
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.field-input:focus {
  outline: none;
  border-color: var(--color-accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.field-input::placeholder {
  color: var(--color-text-soft, #9ca3af);
}

/* 移除了重复的状态指示器样式，现在在 TokenList.vue 中 */

/* Platform Detail View */
.platform-detail-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.coming-soon {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 40px;
  text-align: center;
}

.coming-soon h2 {
  margin: 0 0 16px 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--text-strong);
}

.coming-soon p {
  margin: 0;
  font-size: 16px;
  color: var(--text-muted);
}
</style>
