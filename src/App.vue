<template>
  <div class="app">
    <!-- Header -->
    <header class="app-header">
      <div class="header-left">
        <h1 @click="handleLogoClick" style="cursor: pointer; user-select: none;">Augment Token Manager</h1>
        <!-- External Link buttons -->
        <div class="external-links-group">
          <button @click="showAppHomeDialog = true" class="btn app-home-btn">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
            </svg>
            {{ $t('app.appHome') }}
          </button>
          <button @click="showPluginHomeDialog = true" class="btn plugin-home-btn">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-2 .9-2 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z" />
            </svg>
            {{ $t('app.pluginHome') }}
          </button>
        </div>
      </div>
      <div class="header-buttons">
        <!-- Feature buttons -->
        <button @click="showBookmarkManager = true" class="btn secondary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z" />
          </svg>
          {{ $t('app.bookmarkManager') }}
        </button>
        <button @click="showOutlookManager = true" class="btn warning">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
          </svg>
          {{ $t('app.outlookManager') }}
        </button>
        <button v-if="isGPTMailUnlocked" @click="showGPTMailManager = true" class="btn warning">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
          </svg>
          {{ $t('app.gptMailManager') }}
        </button>

        <button @click="showTokenList = true" class="btn primary">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
          </svg>
          {{ $t('app.viewTokens') }}
        </button>

      </div>
    </header>

    <!-- Main Content -->
    <main class="main-content">
      <div class="token-generator-main">
        <div class="generator-header">
          <div class="header-content">
            <div class="title-section">
              <h2>{{ $t('tokenGenerator.title') }}</h2>
              <p>{{ $t('tokenGenerator.description') }}</p>
            </div>

            <!-- Tab Navigation -->
            <div class="tab-navigation">
              <button :class="['tab-btn', { active: activeTab === 'oauth' }]" @click="activeTab = 'oauth'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
                </svg>
                {{ $t('tokenGenerator.oauthTab') }}
              </button>
              <button :class="['tab-btn', { active: activeTab === 'session' }]" @click="activeTab = 'session'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm0 4c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm6 12H6v-1.4c0-2 4-3.1 6-3.1s6 1.1 6 3.1V19z" />
                </svg>
                {{ $t('tokenGenerator.sessionTab') }}
              </button>
            </div>

          </div>
        </div>

        <div class="generator-body">
          <!-- OAuth Flow Tab -->
          <div v-if="activeTab === 'oauth'" class="tab-content">
            <!-- Step 1: Generate Authorization URL -->
            <div class="section">
              <h3>{{ $t('tokenGenerator.step1') }}</h3>
              <button @click="generateAuthUrl" :class="['btn', 'primary', { loading: isGenerating }]"
                :disabled="isGenerating">
                {{ $t('tokenGenerator.generateUrl') }}
              </button>

              <div v-if="authUrl" class="url-section">
                <label>{{ $t('tokenGenerator.authUrlLabel') }}</label>
                <div class="input-with-copy">
                  <input type="text" :value="authUrl" readonly ref="authUrlInput"
                    :placeholder="$t('tokenGenerator.authUrlPlaceholder')">
                  <button @click="copyAuthUrl" class="copy-icon-btn" :title="$t('tokenGenerator.copyUrl')">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path
                        d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z" />
                    </svg>
                  </button>
                </div>
                <div class="button-container">
                  <button @click="showAuthUrlDialog = true" class="btn secondary">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path
                        d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z" />
                    </svg>
                    {{ $t('tokenGenerator.openAuthUrl') }}
                  </button>
                </div>
              </div>
            </div>

            <!-- Step 2: Enter Authorization Code -->
            <div class="section">
              <h3>{{ $t('tokenGenerator.step2') }}</h3>
              <textarea v-model="authCode" :placeholder="$t('tokenGenerator.authCodePlaceholder')" rows="4"></textarea>
              <div class="button-container">
                <button @click="getAccessToken" :class="['btn', 'primary', { loading: isGettingToken }]"
                  :disabled="!canGetToken || isGettingToken">
                  {{ $t('tokenGenerator.getToken') }}
                </button>
              </div>
            </div>

            <!-- Step 3: Access Token -->
            <div class="section" v-if="tokenResult">
              <h3>{{ $t('tokenGenerator.step3') }}</h3>
              <div class="token-section">
                <div class="result-container">
                  <label>{{ $t('tokenGenerator.accessTokenLabel') }}</label>
                  <div class="token-container">
                    <input type="text" :value="tokenResult.access_token" readonly ref="accessTokenInput">
                    <button @click="copyAccessToken" class="btn secondary">{{ $t('tokenGenerator.copy') }}</button>
                  </div>
                </div>
                <div class="result-container">
                  <label>{{ $t('tokenGenerator.tenantUrlLabel') }}</label>
                  <div class="token-container">
                    <input type="text" :value="tokenResult.tenant_url" readonly ref="tenantUrlInput">
                    <button @click="copyTenantUrl" class="btn secondary">{{ $t('tokenGenerator.copy') }}</button>
                  </div>
                </div>

                <!-- Additional Fields -->
                <div class="additional-fields">
                  <div class="field-container">
                    <label>{{ $t('tokenGenerator.portalUrl') }}:</label>
                    <input type="text" v-model="portalUrl" :placeholder="$t('tokenGenerator.portalUrlPlaceholder')"
                      class="field-input">
                  </div>
                  <div class="field-container">
                    <label>{{ $t('tokenGenerator.emailNote') }}:</label>
                    <input type="text" v-model="emailNote" :placeholder="$t('tokenGenerator.emailNotePlaceholder')"
                      class="field-input">
                  </div>
                </div>

                <div class="button-container">
                  <button @click="saveToken" class="btn success">{{ $t('tokenGenerator.saveToken') }}</button>
                </div>
              </div>
            </div>
          </div>

          <!-- Session Import Tab -->
          <div v-else-if="activeTab === 'session'" class="tab-content">
            <div class="section">
              <div class="session-header">
                <h3>{{ $t('tokenGenerator.sessionImportTitle') }}</h3>
                <button @click="showSessionHelp = true" class="help-button" :title="$t('sessionHelp.title')">
                  ?
                </button>
              </div>
              <p class="section-description">{{ $t('tokenGenerator.sessionImportDescription') }}</p>

              <textarea v-model="sessionInput" :placeholder="$t('tokenGenerator.sessionPlaceholder')" rows="6"
                :disabled="isImportingSession"></textarea>

              <div class="button-container">
                <button @click="importFromSession" class="btn primary"
                  :disabled="!sessionInput.trim() || isImportingSession">
                  {{ $t('tokenGenerator.importSession') }}
                </button>
                <button @click="openInternalBrowserForAutoImport" class="btn secondary"
                  :disabled="isImportingSession || isOpeningBrowser">
                  {{ $t('tokenGenerator.autoImportSession') }}
                </button>
              </div>

              <!-- Loading State -->
              <div v-if="isImportingSession" class="session-loading">
                <div class="session-spinner"></div>
                <span>{{ sessionImportProgress }}</span>
              </div>
            </div>
          </div>
        </div>


      </div>
    </main>

    <!-- Token List Modal -->
    <TokenList v-if="showTokenList" ref="tokenListRef" @close="showTokenList = false" />

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

    <!-- 授权URL链接对话框 -->
    <ExternalLinkDialog :show="showAuthUrlDialog" :title="$t('dialogs.selectOpenMethod')" :url="authUrl"
      :browser-title="$t('messages.oauthTitle')" @close="showAuthUrlDialog = false" />

    <!-- App主页链接对话框 -->
    <ExternalLinkDialog :show="showAppHomeDialog" :title="$t('dialogs.appHomeTitle')"
      url="https://github.com/zhaochengcube/augment-token-mng" :browser-title="$t('messages.appHomeTitle')"
      @close="showAppHomeDialog = false" />

    <!-- 插件主页链接对话框 -->
    <ExternalLinkDialog :show="showPluginHomeDialog" :title="$t('dialogs.pluginHomeTitle')"
      url="https://github.com/zhaochengcube/augment-code-auto" :browser-title="$t('messages.pluginHomeTitle')"
      @close="showPluginHomeDialog = false" />

    <!-- 更新横幅 -->
    <UpdateBanner v-if="updateInfo && updateInfo.has_update" :update-info="updateInfo" @close="updateInfo = null" />

    <!-- 固定在右下角的控制按钮 -->
    <div class="fixed-controls">
      <!-- 弹出的设置选项 -->
      <div v-if="showSettingsMenu" class="settings-menu">
        <!-- GPT邮箱管理器锁定/解锁按钮 -->
        <button v-if="isGPTMailUnlocked" type="button" class="control-btn gpt-mail-lock-toggle" @click="lockGPTMail"
          :aria-label="$t('app.lockGPTMail')" :title="$t('app.lockGPTMail')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
          </svg>
        </button>
        <!-- 检查更新按钮 -->
        <button type="button" class="control-btn update-check-toggle" @click="manualCheckForUpdates"
          :aria-label="$t('app.checkForUpdates')" :title="$t('app.checkForUpdates')" :disabled="checkingUpdate">
          <svg v-if="!checkingUpdate" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <polyline points="16 12 12 8 8 12" />
            <line x1="12" y1="16" x2="12" y2="8" />
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            class="spinning">
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>
        </button>
        <!-- API服务器按钮 -->
        <button type="button" class="control-btn api-server-toggle" @click="showApiServerStatus = true; showSettingsMenu = false"
          :aria-label="$t('apiServer.title')" :title="$t('apiServer.title')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
            <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
            <line x1="6" y1="6" x2="6.01" y2="6" />
            <line x1="6" y1="18" x2="6.01" y2="18" />
          </svg>
        </button>
        <!-- 代理设置按钮 -->
        <button type="button" class="control-btn proxy-toggle" @click="showProxyConfig = true; showSettingsMenu = false"
          :aria-label="$t('app.proxySettings')" :title="$t('app.proxySettings')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <path d="M2 12h20" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
        </button>
        <!-- 语言切换按钮 -->
        <button type="button" class="control-btn language-toggle" @click="toggleLanguage"
          :aria-label="languageToggleLabel" :title="languageToggleLabel">
          {{ currentLocale === 'zh-CN' ? 'EN' : '中' }}
        </button>
        <!-- 主题切换按钮 -->
        <button type="button" class="control-btn theme-toggle" @click="toggleTheme" :aria-pressed="isDarkTheme"
          :aria-label="themeToggleLabel" :title="themeToggleLabel">
          <svg v-if="isDarkTheme" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
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
        </button>
      </div>

      <!-- 设置按钮 -->
      <button type="button" class="control-btn settings-toggle" @click="toggleSettingsMenu"
        :aria-label="$t('app.settings')" :title="$t('app.settings')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed, inject, onBeforeUnmount, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import TokenList from './components/TokenList.vue'
import BookmarkManager from './components/BookmarkManager.vue'
import OutlookManager from './components/OutlookManager.vue'
import GPTMailManager from './components/GPTMailManager.vue'
import ProxyConfig from './components/ProxyConfig.vue'
import ExternalLinkDialog from './components/ExternalLinkDialog.vue'
import NotificationManager from './components/NotificationManager.vue'
import UpdateBanner from './components/UpdateBanner.vue'
import ApiServerStatus from './components/ApiServerStatus.vue'

const { t, locale } = useI18n()

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

// 设置菜单切换
const toggleSettingsMenu = () => {
  showSettingsMenu.value = !showSettingsMenu.value
}

// 点击外部区域关闭设置菜单
const handleClickOutside = (event) => {
  const fixedControls = document.querySelector('.fixed-controls')
  if (fixedControls && !fixedControls.contains(event.target)) {
    showSettingsMenu.value = false
  }
}

// 检查更新
const checkForUpdates = async (silent = true) => {
  try {
    checkingUpdate.value = true
    const result = await invoke('check_for_updates')
    updateInfo.value = result

    if (!silent) {
      if (result.has_update) {
        showStatus(t('update.newVersionAvailable'), 'success')
      } else {
        showStatus(t('update.upToDate'), 'success')
      }
    }
  } catch (error) {
    console.error('Failed to check for updates:', error)
    if (!silent) {
      showStatus(`${t('update.checkFailed')}: ${error}`, 'error')
    }
  } finally {
    checkingUpdate.value = false
  }
}

// 手动检查更新
const manualCheckForUpdates = async () => {
  await checkForUpdates(false)
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

const showTokenList = ref(false)
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
const tokenListRef = ref(null)
const notificationManager = ref(null)


// Token generator data
const authUrl = ref('')
const authCode = ref('')
const tokenResult = ref(null)
const isGenerating = ref(false)
const isGettingToken = ref(false)
const portalUrl = ref('')
const emailNote = ref('')

// Tab state
const activeTab = ref('session')

// Session import data
const sessionInput = ref('')
const sessionTokenResult = ref(null)
const isImportingSession = ref(false)
const sessionImportProgress = ref('')
const showSessionHelp = ref(false)
const isOpeningBrowser = ref(false)

// Template refs
const authUrlInput = ref(null)
const accessTokenInput = ref(null)
const tenantUrlInput = ref(null)




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

// Auth URL dialog
const showAuthUrlDialog = ref(false)

// External links dialogs
const showAppHomeDialog = ref(false)
const showPluginHomeDialog = ref(false)

// Settings menu
const showSettingsMenu = ref(false)

// Update check
const updateInfo = ref(null)
const checkingUpdate = ref(false)

// Computed properties

const canGetToken = computed(() => {
  return authUrl.value && authCode.value.trim().length > 0
})

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

const generateAuthUrl = async () => {
  isGenerating.value = true

  try {
    const url = await invoke('generate_augment_auth_url')
    authUrl.value = url
  } catch (error) {
    showStatus(`${t('messages.error')}: ${error}`, 'error')
  } finally {
    isGenerating.value = false
  }
}

const copyAuthUrl = async () => {
  const success = await copyToClipboard(authUrl.value)
  showStatus(
    success ? t('messages.copySuccess') : t('messages.copyFailed'),
    success ? 'success' : 'error'
  )
}



const getAccessToken = async () => {
  isGettingToken.value = true
  showStatus(t('messages.gettingToken'), 'info')

  try {
    const result = await invoke('get_augment_token', { code: authCode.value })
    tokenResult.value = result
    // 如果返回了 email，自动填充到 emailNote
    if (result.email && !emailNote.value) {
      emailNote.value = result.email
    }
    showStatus(t('messages.tokenGetSuccess'), 'success')
  } catch (error) {
    showStatus(`${t('messages.error')}: ${error}`, 'error')
  } finally {
    isGettingToken.value = false
  }
}

const copyAccessToken = async () => {
  const success = await copyToClipboard(tokenResult.value.access_token)
  showStatus(
    success ? t('messages.accessTokenCopied') : t('messages.copyFailed'),
    success ? 'success' : 'error'
  )
}

const copyTenantUrl = async () => {
  const success = await copyToClipboard(tokenResult.value.tenant_url)
  showStatus(
    success ? t('messages.tenantUrlCopied') : t('messages.copyFailed'),
    success ? 'success' : 'error'
  )
}


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

    // 创建包含 auth_session 的 tokenData
    const tokenData = {
      tenantUrl: result.tenant_url,
      accessToken: result.access_token,
      portalUrl: null,  // Session 导入不再获取 portal_url
      emailNote: result.email || null,  // 从 get-models API 获取的邮箱
      authSession: authSession,  // 保存 auth_session
      suspensions: null,  // Session 导入不再获取 suspensions
      creditsBalance: null,  // Session 导入不再获取余额
      expiryDate: null,  // Session 导入不再获取过期时间
      banStatus: 'ACTIVE'  // Session 导入默认设置为 ACTIVE 状态
    }

    // 先打开 TokenList（如果未打开）
    if (!showTokenList.value) {
      showTokenList.value = true
      await nextTick()
    }

    // 等待 TokenList 初始化完成
    if (tokenListRef.value?.waitUntilReady) {
      await tokenListRef.value.waitUntilReady()
    }

    // 保存 token
    sessionImportProgress.value = t('messages.sessionImportSavingToken')

    // 通过 TokenList 添加 token
    if (tokenListRef.value) {
      const result = tokenListRef.value.addToken(tokenData)
      if (result.success) {
        // 添加成功
        sessionImportProgress.value = t('messages.sessionImportSuccess')
        showStatus(t('messages.sessionImportSuccess'), 'success')
      } else if (result.duplicateId) {
        // 添加失败（重复邮箱），高亮并滚动到重复的 token
        sessionImportProgress.value = ''
        tokenListRef.value.highlightAndScrollTo(result.duplicateId)
      }
    } else {
      showStatus(t('messages.tokenSaveFailed') + ': TokenList not available', 'error')
      return
    }

    // 清空输入
    sessionInput.value = ''
    sessionTokenResult.value = null

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



const saveToken = async () => {
  try {
    // 创建新的 token 数据
    const tokenData = {
      tenantUrl: tokenResult.value.tenant_url,
      accessToken: tokenResult.value.access_token,
      portalUrl: portalUrl.value.trim() || null,
      emailNote: emailNote.value.trim() || tokenResult.value.email || null,
      creditsBalance: tokenResult.value.credits_balance || null,  // 从 get-credit-info 获取的余额
      expiryDate: tokenResult.value.expiry_date || null  // 从 get-credit-info 获取的过期时间
    }

    // 先打开 TokenList（如果未打开）
    if (!showTokenList.value) {
      showTokenList.value = true
      // 等待 TokenList 组件挂载
      await nextTick()
    }

    // 等待 TokenList 初始化完成
    if (tokenListRef.value?.waitUntilReady) {
      await tokenListRef.value.waitUntilReady()
    }

    // 通过TokenList添加token
    if (tokenListRef.value) {
      const result = tokenListRef.value.addToken(tokenData)
      if (result.success) {
        showStatus(t('messages.tokenSaved'), 'success')
      } else if (result.duplicateId) {
        // 添加失败（重复邮箱），高亮并滚动到重复的 token
        tokenListRef.value.highlightAndScrollTo(result.duplicateId)
      }
      // 无论成功还是失败都清空表单
      authUrl.value = ''
      authCode.value = ''
      tokenResult.value = null
      portalUrl.value = ''
      emailNote.value = ''
    } else {
      // 如果仍然无法获取 ref，说明有问题
      showStatus(t('messages.tokenSaveFailed') + ': TokenList not available', 'error')
      return
    }
  } catch (error) {
    showStatus(`${t('messages.tokenSaveFailed')}: ${error}`, 'error')
  }
}










onMounted(async () => {
  // 读取保存的语言偏好
  const savedLanguage = localStorage.getItem('preferred-language')
  if (savedLanguage && (savedLanguage === 'zh-CN' || savedLanguage === 'en-US')) {
    currentLocale.value = savedLanguage
    locale.value = savedLanguage
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

    // 如果 TokenList 已打开,说明是从 TokenForm 触发的自动导入
    // TokenForm 会自己处理,App.vue 不需要重复处理
    if (showTokenList.value) {
      console.log('TokenList is already open, skipping duplicate handling')
      return
    }

    // 打开 TokenList 并添加 token
    showTokenList.value = true

    // 等待 TokenList 准备好
    await nextTick()
    if (tokenListRef.value?.waitUntilReady) {
      await tokenListRef.value.waitUntilReady()
    }

    // 添加 token
    if (tokenListRef.value && event.payload.token) {
      const tokenData = {
        tenantUrl: event.payload.token.tenant_url,
        accessToken: event.payload.token.access_token,
        portalUrl: null,  // Session 导入不再获取 portal_url
        emailNote: event.payload.token.email || null,  // 从 get-models API 获取的邮箱
        authSession: event.payload.session || null,  // 保存 auth_session
        suspensions: null,  // Session 导入不再获取 suspensions
        creditsBalance: null,  // Session 导入不再获取余额
        expiryDate: null  // Session 导入不再获取过期时间
      }
      const result = tokenListRef.value.addToken(tokenData)
      if (result.success) {
        // 添加成功才显示成功提示
        showStatus(t('messages.sessionAutoImported'), 'success')
      } else if (result.duplicateId) {
        // 添加失败（重复邮箱），高亮并滚动到重复的 token
        tokenListRef.value.highlightAndScrollTo(result.duplicateId)
      }
    }
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
    // 打开 TokenList
    showTokenList.value = true

    // 等待 TokenList 准备好
    await nextTick()
    if (tokenListRef.value?.waitUntilReady) {
      await tokenListRef.value.waitUntilReady()
    }

    // 调用前端的导入方法（会显示进度和结果提示）
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

.app-header {
  background: var(--color-surface, #ffffff);
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  flex-shrink: 0;
  min-height: 60px;
  flex-wrap: wrap;
  gap: 8px;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
  min-width: 0;
  align-items: flex-start;
}

.app-header h1 {
  margin: 0;
  color: var(--color-text-heading, #333);
  font-size: 20px;
  font-weight: 600;
  white-space: nowrap;
}

/* 固定控制按钮组 */
.fixed-controls {
  position: fixed;
  bottom: 20px;
  right: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  z-index: 1000;
}

.settings-menu {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 8px;
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.control-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(10px);
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
  border: 1px solid var(--color-border, #e5e7eb);
}

.control-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
}

.control-btn:active {
  transform: translateY(0);
}

/* 语言切换按钮样式 */
.control-btn.language-toggle {
  font-weight: 700;
  font-size: 12px;
}

/* 检查更新按钮样式 - 与其他按钮保持一致 */
.control-btn.update-check-toggle:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

/* 设置按钮样式 */
.control-btn.settings-toggle {
  background: var(--color-text-muted, #6b7280);
  color: var(--color-text-inverse, #ffffff);
  border-color: var(--color-text-muted, #6b7280);
}

.control-btn.settings-toggle:hover {
  background: var(--color-text-secondary, #4b5563);
  border-color: var(--color-text-secondary, #4b5563);
}

.control-btn svg {
  transition: all 0.3s ease;
}

/* 旋转动画 */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.spinning {
  animation: spin 1s linear infinite;
}

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







.header-buttons {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
}



.external-links-group {
  display: flex;
  flex-direction: row;
  gap: 8px;
  align-items: center;
  align-self: flex-start;
}

.external-links-group .btn {
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 500;
  min-width: 80px;
  justify-content: center;
  border: 1px solid transparent;
}

.btn.app-home-btn {
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border: 1px solid var(--color-blue-soft-border, #90caf9);
}

.btn.app-home-btn:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(25, 118, 210, 0.3);
}

.btn.plugin-home-btn {
  background: var(--color-success-surface, #d1fae5);
  color: var(--color-success-text, #065f46);
  border: 1px solid var(--color-success-border, #a7f3d0);
}

.btn.plugin-home-btn:hover {
  background: var(--color-success-surface, #a7f3d0);
  border-color: var(--color-success-border, #6ee7b7);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(6, 95, 70, 0.3);
}



.main-content {
  padding: 20px 16px;
  flex: 1;
  overflow-y: auto;
  min-height: 0;
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

  .fixed-controls {
    bottom: 16px;
    right: 16px;
  }

  .control-btn {
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

  .fixed-controls {
    bottom: 12px;
    right: 12px;
    gap: 6px;
  }

  .control-btn {
    width: 32px;
    height: 32px;
  }

  .control-btn.language-toggle {
    font-size: 10px;
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
</style>
