<template>
  <div :class="['token-card', { 'menu-open': showCopyMenu || showCheckMenu, 'highlighted': isHighlighted }]" @click="handleClickOutside">
    <!-- 状态指示器 -->
    <div v-if="showStatusIndicator" class="status-indicator">
      <span
        v-if="hasTag"
        class="status-badge tag-badge"
        :style="tagBadgeStyle"
        :title="tagDisplayName"
      >
        {{ tagDisplayName }}
      </span>
      <span
        v-if="hasStatusBadge"
        :class="['status-badge', getStatusClass(token.ban_status), { clickable: isBannedWithSuspensions }]"
        @click="handleStatusClick"
        :title="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
      >
        {{ getStatusText(token.ban_status) }}
      </span>
    </div>

    <div class="card-main">
      <div class="token-info">
        <h3 class="tenant-name">{{ displayUrl }}</h3>
        <div class="token-meta">
          <!-- 第一行：创建日期 -->
          <div class="meta-row">
            <span class="created-date">{{ formatDate(token.created_at) }}</span>
          </div>
          <!-- 第二行：邮箱备注（如果有） -->
          <div v-if="token.email_note" class="meta-row email-row">
            <div class="email-note-container">
              <span
                class="email-note"
                @mouseenter="isEmailHovered = true"
                @mouseleave="isEmailHovered = false"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="email-icon">
                  <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.89 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
                </svg>
                {{ isEmailHovered ? token.email_note : maskedEmail }}
              </span>
              <button @click="copyEmailNote" class="copy-email-btn" :title="$t('tokenCard.copyEmailNote')">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
              </button>
            </div>
          </div>
          <!-- 第二行：Portal信息 -->
          <template v-if="token.portal_url">
            <div class="meta-row portal-row">
              <template v-if="portalInfo.data">
                <span v-if="portalInfo.data.expiry_date" class="portal-meta expiry">{{ $t('tokenCard.expiry') }}: {{ formatExpiryDate(portalInfo.data.expiry_date) }}</span>
              </template>
              <!-- 余额显示：无论是否有数据都显示 -->
              <span
                :class="balanceClasses"
                @click="toggleBalanceColor"
                :style="{ cursor: isBalanceClickable ? 'pointer' : 'default' }"
              >
                {{ balanceDisplay }}
              </span>
              <template v-if="portalInfo.data">
                <!-- Credit 统计按钮 -->
                <button
                  v-if="token.auth_session"
                  @click="showCreditUsageModal = true"
                  class="credit-stats-btn"
                  :title="$t('credit.viewUsage')"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
                  </svg>
                </button>
              </template>
            </div>
          </template>
        </div>
      </div>

      <div class="actions">
        <button @click="openEditorModal" class="btn-action vscode" :title="$t('tokenCard.selectEditor')">
          <img :src="editorIcons.vscode" :alt="$t('tokenCard.selectEditor')" width="18" height="18" />
        </button>
        <button @click="exportTokenAsJson" class="btn-action export" :title="$t('tokenCard.exportJson')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>
        <div class="copy-menu-wrapper" @click.stop>
          <button @click.stop="toggleCopyMenu" class="btn-action copy" :title="$t('tokenCard.copyMenu')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
          <Transition name="dropdown">
            <div v-if="showCopyMenu" class="copy-dropdown" @click.stop>
              <button @click="handleCopyMenuClick('token')" class="copy-menu-item">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
                <span>{{ $t('tokenCard.copyToken') }}</span>
              </button>
              <button @click="handleCopyMenuClick('url')" class="copy-menu-item">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
                </svg>
                <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
              </button>
              <button v-if="token.portal_url" @click="handleCopyMenuClick('portal')" class="copy-menu-item">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
                </svg>
                <span>{{ $t('tokenCard.copyPortalUrl') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('session')" class="copy-menu-item">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                </svg>
                <span>{{ $t('tokenCard.copyAuthSession') }}</span>
              </button>
            </div>
          </Transition>
        </div>
        <div class="check-menu-wrapper">
          <button
            @click="checkAccountStatus"
            @contextmenu.prevent="showCheckMenu = !showCheckMenu"
            :class="['btn-action', 'status-check', {
              loading: isCheckingStatus || (isBatchChecking && !token.skip_check),
              disabled: token.skip_check
            }]"
            :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check)"
            :title="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
          >
            <svg v-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check) && !token.skip_check" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <!-- 禁用检测时显示暂停图标 -->
            <svg v-else-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check) && token.skip_check" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
            <div v-else-if="isCheckingStatus || (isBatchChecking && !token.skip_check)" class="loading-spinner"></div>
          </button>
          <Transition name="dropdown">
            <div v-if="showCheckMenu" class="check-dropdown" @click.stop>
              <button @click="toggleSkipCheck" class="check-menu-item">
                <!-- 禁用检测图标 - 暂停 -->
                <svg v-if="!token.skip_check" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
                </svg>
                <!-- 启用检测图标 - 播放 -->
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
                <span>{{ token.skip_check ? $t('tokenCard.enableCheck') : $t('tokenCard.disableCheck') }}</span>
              </button>
            </div>
          </Transition>
        </div>
        <button v-if="token.portal_url" @click="showPortalDialog = true" class="btn-action portal" :title="$t('tokenCard.openPortal')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>
        <button @click="$emit('edit', token)" class="btn-action edit" :title="$t('tokenCard.editToken')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>
        <button @click="deleteToken" class="btn-action delete" :title="$t('tokenCard.deleteToken')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="showEditorModal" class="editor-modal-overlay">
        <div class="editor-modal" @click.stop>
          <div class="modal-header">
            <h3>{{ $t('tokenCard.selectEditor') }}</h3>
            <button @click.stop="showEditorModal = false" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="modal-content">
            <!-- VSCode 系编辑器区域 -->
            <div class="editor-section">
              <div class="editor-options jetbrains-grid">
                <button @click="handleEditorClick('vscode')" class="editor-option vscode-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.vscode" alt="VS Code" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">VS Code</span>
                  </div>
                </button>
                <button @click="handleEditorClick('cursor')" class="editor-option cursor-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.cursor" alt="Cursor" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Cursor</span>
                  </div>
                </button>
                <button @click="handleEditorClick('kiro')" class="editor-option kiro-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.kiro" alt="Kiro" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Kiro</span>
                  </div>
                </button>
                <button @click="handleEditorClick('trae')" class="editor-option trae-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.trae" alt="Trae" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Trae</span>
                  </div>
                </button>
                <button @click="handleEditorClick('windsurf')" class="editor-option windsurf-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.windsurf" alt="Windsurf" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Windsurf</span>
                  </div>
                </button>
                <button @click="handleEditorClick('qoder')" class="editor-option qoder-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.qoder" alt="Qoder" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Qoder</span>
                  </div>
                </button>
                <button @click="handleEditorClick('vscodium')" class="editor-option vscodium-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.vscodium" alt="VSCodium" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">VSCodium</span>
                  </div>
                </button>
                <button @click="handleEditorClick('codebuddy')" class="editor-option codebuddy-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.codebuddy" alt="CodeBuddy" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">CodeBuddy</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- JetBrains 系编辑器区域 -->
            <div class="editor-section">
              <div class="editor-options jetbrains-grid">
                <button @click="handleEditorClick('idea')" class="editor-option idea-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.idea" alt="IntelliJ IDEA" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">IntelliJ IDEA</span>
                  </div>
                </button>
                <button @click="handleEditorClick('pycharm')" class="editor-option pycharm-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.pycharm" alt="PyCharm" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">PyCharm</span>
                  </div>
                </button>
                <button @click="handleEditorClick('goland')" class="editor-option goland-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.goland" alt="GoLand" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">GoLand</span>
                  </div>
                </button>
                <button @click="handleEditorClick('rustrover')" class="editor-option rustrover-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.rustrover" alt="RustRover" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">RustRover</span>
                  </div>
                </button>
                <button @click="handleEditorClick('webstorm')" class="editor-option webstorm-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.webstorm" alt="WebStorm" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">WebStorm</span>
                  </div>
                </button>
                <button @click="handleEditorClick('phpstorm')" class="editor-option phpstorm-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.phpstorm" alt="PhpStorm" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">PhpStorm</span>
                  </div>
                </button>
                <button @click="handleEditorClick('androidstudio')" class="editor-option androidstudio-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.androidstudio" alt="Android Studio" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Android Studio</span>
                  </div>
                </button>
                <button @click="handleEditorClick('clion')" class="editor-option clion-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.clion" alt="CLion" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">CLion</span>
                  </div>
                </button>
                <button @click="handleEditorClick('datagrip')" class="editor-option datagrip-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.datagrip" alt="DataGrip" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">DataGrip</span>
                  </div>
                </button>
                <button @click="handleEditorClick('rider')" class="editor-option rider-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.rider" alt="Rider" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Rider</span>
                  </div>
                </button>
                <button @click="handleEditorClick('rubymine')" class="editor-option rubymine-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.rubymine" alt="RubyMine" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">RubyMine</span>
                  </div>
                </button>
                <button @click="handleEditorClick('aqua')" class="editor-option aqua-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.aqua" alt="Aqua" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Aqua</span>
                  </div>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <ExternalLinkDialog
    :show="showPortalDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="token.portal_url || ''"
    :browser-title="getPortalBrowserTitle()"
    @close="showPortalDialog = false"
  />

  <!-- Suspensions 详情模态框 -->
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="showSuspensionsModal" class="suspensions-modal-overlay" @click="showSuspensionsModal = false">
        <div class="suspensions-modal" @click.stop>
          <div class="modal-header">
            <h3>{{ $t('tokenCard.suspensionDetails') }}</h3>
            <button @click="showSuspensionsModal = false" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="formattedSuspensions.length > 0" class="suspensions-list">
              <div v-for="(suspension, index) in formattedSuspensions" :key="index" class="suspension-item">
                <div class="suspension-field">
                  <strong>{{ $t('tokenCard.suspensionType') }}:</strong>
                  <span class="suspension-value">{{ suspension.type }}</span>
                </div>
                <div v-if="suspension.reason" class="suspension-field">
                  <strong>{{ $t('tokenCard.reason') }}:</strong>
                  <span class="suspension-value">{{ suspension.reason }}</span>
                </div>
                <div v-if="suspension.date" class="suspension-field">
                  <strong>{{ $t('tokenCard.date') }}:</strong>
                  <span class="suspension-value">{{ suspension.date }}</span>
                </div>
              </div>
            </div>
            <div v-else class="no-suspensions">
              <p>{{ $t('tokenCard.noSuspensionData') }}</p>
            </div>
            <!-- 原始 JSON 数据 -->
            <details class="raw-json" open>
              <summary>{{ $t('tokenCard.rawData') }}</summary>
              <pre>{{ JSON.stringify(token.suspensions, null, 2) }}</pre>
            </details>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- Trae 版本选择对话框 -->
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="showTraeVersionDialog" class="trae-version-modal-overlay" @click="showTraeVersionDialog = false">
        <div class="trae-version-modal" @click.stop>
          <div class="modal-header">
            <h3>选择 Trae 版本</h3>
            <button @click="showTraeVersionDialog = false" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="version-options">
              <button @click="handleTraeVersionSelect('global')" class="version-option">
                <div class="version-icon">🌍</div>
                <div class="version-name">Trae 国际版</div>
              </button>
              <button @click="handleTraeVersionSelect('cn')" class="version-option">
                <div class="version-icon">🇨🇳</div>
                <div class="version-name">Trae 国内版</div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Credit Usage Modal -->
    <Transition name="modal" appear>
      <CreditUsageModal
        v-if="showCreditUsageModal && token.auth_session"
        :auth-session="token.auth_session"
        :credits-balance="remainingCredits"
        @close="showCreditUsageModal = false"
        @refresh-balance="handleCreditUsageRefresh"
      />
    </Transition>
  </Teleport>
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ExternalLinkDialog from './ExternalLinkDialog.vue'
import CreditUsageModal from './CreditUsageModal.vue'

const { t } = useI18n()


// Props
const props = defineProps({
  token: {
    type: Object,
    required: true
  },
  isBatchChecking: {
    type: Boolean,
    default: false
  },
  isHighlighted: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['delete', 'edit', 'token-updated'])

// Reactive data
const isLoadingPortalInfo = ref(false)
const portalInfo = ref({ data: null, error: null })
const isCheckingStatus = ref(false)
const isEmailHovered = ref(false)
const showEditorModal = ref(false)
const isModalClosing = ref(false)
const showPortalDialog = ref(false)
const showCheckMenu = ref(false)
const showSuspensionsModal = ref(false)
const showTraeVersionDialog = ref(false)
const showCopyMenu = ref(false)
const showCreditUsageModal = ref(false)

const DEFAULT_TAG_COLOR = '#f97316'

const tagDisplayName = computed(() => (props.token.tag_name ?? '').trim())
const hasTag = computed(() => tagDisplayName.value.length > 0)

const normalizeHexColor = (color) => {
  if (typeof color !== 'string') {
    return DEFAULT_TAG_COLOR
  }
  const value = color.trim()
  return /^#[0-9a-fA-F]{6}$/.test(value) ? value.toLowerCase() : DEFAULT_TAG_COLOR
}

const getContrastingTextColor = (hex) => {
  if (!/^#[0-9a-fA-F]{6}$/.test(hex)) {
    return '#ffffff'
  }
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  return luminance > 0.6 ? '#1f2937' : '#ffffff'
}

const tagBadgeStyle = computed(() => {
  if (!hasTag.value) {
    return {}
  }
  const color = normalizeHexColor(props.token.tag_color || DEFAULT_TAG_COLOR)
  return {
    backgroundColor: color,
    borderColor: color,
    color: getContrastingTextColor(color)
  }
})

const hasStatusBadge = computed(() => {
  const hasPortalStatus = props.token.portal_url && portalInfo.value.data
  return Boolean(hasPortalStatus || props.token.ban_status)
})

const showStatusIndicator = computed(() => hasTag.value || hasStatusBadge.value)

// 图标映射
const editorIcons = {
  vscode: '/icons/vscode.svg',
  cursor: '/icons/cursor.svg',
  kiro: '/icons/kiro.svg',
  trae: '/icons/trae.svg',
  windsurf: '/icons/windsurf.svg',
  qoder: '/icons/qoder.svg',
  vscodium: '/icons/vscodium.svg',
  codebuddy: '/icons/codebuddy.svg',
  idea: '/icons/idea.svg',
  pycharm: '/icons/pycharm.svg',
  goland: '/icons/goland.svg',
  rustrover: '/icons/rustrover.svg',
  webstorm: '/icons/webstorm.svg',
  phpstorm: '/icons/phpstorm.svg',
  clion: '/icons/clion.svg',
  datagrip: '/icons/datagrip.svg',
  rider: '/icons/rider.svg',
  rubymine: '/icons/rubymine.svg',
  aqua: '/icons/aqua.svg',
  androidstudio: '/icons/androidstudio.svg'
}

// 判断是否为封禁状态且有 suspensions 数据
const isBannedWithSuspensions = computed(() => {
  return (
    props.token.ban_status === 'SUSPENDED' &&
    props.token.suspensions &&
    (Array.isArray(props.token.suspensions) ? props.token.suspensions.length > 0 : true)
  )
})

// 格式化 suspensions 数据
const formattedSuspensions = computed(() => {
  if (!props.token.suspensions) return []

  if (Array.isArray(props.token.suspensions)) {
    return props.token.suspensions.map(s => ({
      type: s.suspensionType || 'Unknown',
      reason: s.reason || '',
      date: s.date || s.createdAt || ''
    }))
  }

  // 如果不是数组,尝试作为单个对象处理
  return [{
    type: props.token.suspensions.suspensionType || 'Unknown',
    reason: props.token.suspensions.reason || '',
    date: props.token.suspensions.date || props.token.suspensions.createdAt || ''
  }]
})

// Computed properties
const displayUrl = computed(() => {
  try {
    const url = new URL(props.token.tenant_url)
    return url.hostname
  } catch {
    return props.token.tenant_url
  }
})


const maskedEmail = computed(() => {
  const email = props.token.email_note
  if (!email || !email.includes('@')) return email

  const [username, domain] = email.split('@')

  // 如果用户名太短，直接返回原邮箱
  if (username.length <= 3) {
    return email
  }

  let maskedUsername
  if (username.length <= 6) {
    // 短邮箱：保留前1-2个字符，其余用星号替换
    const keepChars = username.length <= 4 ? 1 : 2
    const hiddenCount = username.length - keepChars
    maskedUsername = username.substring(0, keepChars) + '*'.repeat(hiddenCount)
  } else {
    // 长邮箱：保留前后各2-3个字符，中间用4个星号替换
    const frontKeep = username.length >= 8 ? 3 : 2
    const backKeep = 2
    maskedUsername = username.substring(0, frontKeep) + '****' + username.substring(username.length - backKeep)
  }

  return maskedUsername + '@' + domain
})

// Portal余额显示相关计算属性
const balanceClasses = computed(() => {
  // 网络错误或账号异常状态显示红色
  const hasError = portalInfo.value?.error
  const exhausted = (
    props.token.ban_status === 'EXPIRED' ||
    props.token.ban_status === 'SUSPENDED'
  )

  // 如果是异常状态或网络错误（红色），不应用颜色模式
  if (hasError || exhausted) {
    return ['portal-meta', 'balance', 'exhausted']
  }

  // 没有数据时返回默认样式
  if (!portalInfo.value || !portalInfo.value.data) {
    return ['portal-meta', 'balance']
  }

  // 正常状态下应用颜色模式
  const colorMode = props.token.balance_color_mode || 'green'
  return ['portal-meta', 'balance', `color-${colorMode}`]
})

// 判断余额是否可点击（非异常状态才可点击）
const isBalanceClickable = computed(() => {
  // 网络错误或没有数据时不可点击
  if (!portalInfo.value || !portalInfo.value.data || portalInfo.value.error) {
    return false
  }
  const exhausted = (
    props.token.ban_status === 'EXPIRED' ||
    props.token.ban_status === 'SUSPENDED'
  )
  return !exhausted
})

const balanceDisplay = computed(() => {
  if (!portalInfo.value) return ''

  // 显示错误信息
  if (portalInfo.value.error) {
    return t('tokenCard.networkError')
  }

  if (!portalInfo.value.data) return ''

  const status = props.token.ban_status
  if (status === 'EXPIRED') return t('tokenCard.expired')
  if (status === 'SUSPENDED') return t('tokenCard.banned')
  const credits = portalInfo.value.data.credits_balance
  return `${t('tokenCard.balance')}: ${credits}`
})

const remainingCredits = computed(() => {
  const currentCredits = portalInfo.value?.data?.credits_balance
  if (currentCredits !== undefined && currentCredits !== null) {
    return currentCredits
  }
  const fallbackCredits = props.token.portal_info?.credits_balance
  if (fallbackCredits !== undefined && fallbackCredits !== null) {
    return fallbackCredits
  }
  return null
})

// 获取状态样式类
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

// 获取状态显示文本
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


// Methods
const formatDate = (dateString) => {
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}





const deleteToken = () => {
  // 直接发出删除事件，让父组件处理确认逻辑
  emit('delete', props.token.id)
}

// 复制到剪贴板的通用方法
const copyToClipboard = async (text) => {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch (error) {
    // 备用方案
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    return true
  }
}

// 通用复制方法
const copyWithNotification = async (text, successMessage, errorMessage) => {
  const success = await copyToClipboard(text)
  if (success) {
    window.$notify.success(t(successMessage))
  } else {
    window.$notify.error(t(errorMessage))
  }
}

// 复制Token
const copyToken = () => copyWithNotification(
  props.token.access_token,
  'messages.tokenCopied',
  'messages.copyTokenFailed'
)

// 复制租户URL
const copyTenantUrl = () => copyWithNotification(
  props.token.tenant_url,
  'messages.tenantUrlCopied',
  'messages.copyTenantUrlFailed'
)

// 复制邮箱备注
const copyEmailNote = () => copyWithNotification(
  props.token.email_note,
  'messages.emailNoteCopied',
  'messages.copyEmailNoteFailed'
)

// 复制Portal URL
const copyPortalUrl = () => {
  copyWithNotification(
    props.token.portal_url,
    'messages.portalUrlCopied',
    'messages.copyPortalUrlFailed'
  )
}

// 复制Auth Session
const copyAuthSession = () => {
  if (!props.token.auth_session) {
    window.$notify.warning(t('messages.noAuthSession'))
    return
  }
  copyWithNotification(
    props.token.auth_session,
    'messages.authSessionCopied',
    'messages.copyAuthSessionFailed'
  )
}

// 导出Token为JSON
const exportTokenAsJson = () => {
  const exportData = {
    access_token: props.token.access_token,
    tenant_url: props.token.tenant_url
  }

  if (props.token.portal_url) {
    exportData.portal_url = props.token.portal_url
  }

  if (props.token.email_note) {
    exportData.email_note = props.token.email_note
  }

  if (props.token.tag_name) {
    exportData.tag_name = props.token.tag_name
    if (props.token.tag_color) {
      exportData.tag_color = props.token.tag_color
    }
  }

  if (props.token.auth_session) {
    exportData.auth_session = props.token.auth_session
  }

  const jsonString = JSON.stringify(exportData, null, 2)
  copyWithNotification(
    jsonString,
    'messages.tokenJsonExported',
    'messages.exportTokenJsonFailed'
  )
}

// 切换复制菜单
const toggleCopyMenu = () => {
  showCopyMenu.value = !showCopyMenu.value
}

// 处理复制菜单项点击
const handleCopyMenuClick = (type) => {
  showCopyMenu.value = false
  switch (type) {
    case 'token':
      copyToken()
      break
    case 'url':
      copyTenantUrl()
      break
    case 'portal':
      copyPortalUrl()
      break
    case 'session':
      copyAuthSession()
      break
  }
}

// 处理状态标签点击
const handleStatusClick = () => {
  if (isBannedWithSuspensions.value) {
    showSuspensionsModal.value = true
  }
}

// 键盘事件处理
const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    if (showEditorModal.value) {
      showEditorModal.value = false
    }
    if (showCopyMenu.value) {
      showCopyMenu.value = false
    }
  }
}

// 点击外部关闭复制菜单和检测菜单
const handleClickOutside = () => {
  if (showCopyMenu.value) {
    showCopyMenu.value = false
  }
  if (showCheckMenu.value) {
    showCheckMenu.value = false
  }
}

// 打开编辑器模态框
const openEditorModal = () => {
  if (showEditorModal.value || isModalClosing.value) return
  showEditorModal.value = true
}


const editorNames = {
  'cursor': 'Cursor',
  'vscode': 'VS Code',
  'kiro': 'Kiro',
  'trae': 'Trae',
  'windsurf': 'Windsurf',
  'qoder': 'Qoder',
  'vscodium': 'VSCodium',
  'codebuddy': 'CodeBuddy',
  'idea': 'IntelliJ IDEA',
  'pycharm': 'PyCharm',
  'goland': 'GoLand',
  'rustrover': 'RustRover',
  'webstorm': 'WebStorm',
  'phpstorm': 'PhpStorm',
  'androidstudio': 'Android Studio',
  'clion': 'CLion',
  'datagrip': 'DataGrip',
  'rider': 'Rider',
  'rubymine': 'RubyMine',
  'aqua': 'Aqua'
}

const vscodeSchemes = {
  'cursor': 'cursor',
  'vscode': 'vscode',
  'kiro': 'kiro',
  'trae': 'trae',
  'trae-cn': 'trae-cn',
  'windsurf': 'windsurf',
  'qoder': 'qoder',
  'vscodium': 'vscodium',
  'codebuddy': 'codebuddy'
}

const createVSCodeProtocolUrl = (scheme, label) => {
  try {
    const token = encodeURIComponent(props.token.access_token)
    const url = encodeURIComponent(props.token.tenant_url)
    const portalUrl = encodeURIComponent(props.token.portal_url || "")
    return `${scheme}://Augment.vscode-augment/autoAuth?token=${token}&url=${url}&portal=${portalUrl}`
  } catch (error) {
    return '#'
  }
}


const jetbrainsEditors = new Set([
  'idea', 'pycharm', 'goland', 'rustrover', 'webstorm',
  'phpstorm', 'androidstudio', 'clion', 'datagrip', 'rider', 'rubymine', 'aqua'
])

const vscodeProtocolResolvers = Object.fromEntries(
  Object.entries(vscodeSchemes).map(([type, scheme]) => [
    type,
    () => createVSCodeProtocolUrl(scheme, editorNames[type] || type)
  ])
)

// 为 JetBrains 编辑器创建 JSON 文件
const createJetBrainsTokenFile = async (editorType) => {
  try {
    // 创建 JSON 数据
    const tokenData = {
      url: props.token.tenant_url,
      token: props.token.access_token,
      timestamp: Date.now(),
      ide: editorType
    }

    // 调用 Tauri 后端命令创建文件
    const result = await invoke('create_jetbrains_token_file', {
      editorType,
      tokenData: JSON.stringify(tokenData, null, 2)
    })

    return { success: true, filePath: result }
  } catch (error) {
    return { success: false, error: error.toString() }
  }
}

// 处理编辑器链接点击事件
const handleEditorClick = async (editorType) => {
  // 如果是 Trae，显示版本选择对话框
  if (editorType === 'trae') {
    showTraeVersionDialog.value = true
    return
  }

  try {
    const editorName = editorNames[editorType] || editorType

    if (jetbrainsEditors.has(editorType)) {
      const result = await createJetBrainsTokenFile(editorType)

      if (result.success) {
        emit('copy-success', t('messages.editorTokenFileCreated', { editor: editorName }), 'success')
      } else {
        emit('copy-success', t('messages.createEditorTokenFileFailed', { editor: editorName, error: result.error }), 'error')
      }
    } else {
      const resolver = vscodeProtocolResolvers[editorType]

      if (!resolver) {
        throw new Error(`Unknown VSCode editor type: ${editorType}`)
      }

      const protocolUrl = resolver()

      await invoke('open_editor_with_protocol', { protocolUrl })
      window.$notify.success(t('messages.openingEditor', { editor: editorName }))
    }
  } catch (error) {
    window.$notify.error(t('messages.operationFailed'))
  } finally {
    showEditorModal.value = false
    isModalClosing.value = false
  }
}

// 处理 Trae 版本选择
const handleTraeVersionSelect = async (version) => {
  showTraeVersionDialog.value = false

  try {
    const editorType = version === 'global' ? 'trae' : 'trae-cn'
    const resolver = vscodeProtocolResolvers[editorType]

    if (!resolver) {
      throw new Error(`Unknown Trae version: ${version}`)
    }

    const protocolUrl = resolver()
    await invoke('open_editor_with_protocol', { protocolUrl })
    window.$notify.success(t('messages.openingEditor', { editor: 'Trae' }))
  } catch (error) {
    window.$notify.error(t('messages.operationFailed'))
  } finally {
    showEditorModal.value = false
    isModalClosing.value = false
  }
}

// Portal相关方法
const getPortalBrowserTitle = () => {
  if (!props.token) return 'Portal'
  const displayUrl = props.token.tenant_url.replace(/^https?:\/\//, '').replace(/\/$/, '')
  return `Portal - ${displayUrl}`
}




const formatExpiryDate = (dateString) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return dateString
  }
}



// 切换余额颜色模式
const toggleBalanceColor = () => {
  // 只有在非异常状态下才允许切换
  if (!isBalanceClickable.value) {
    return
  }

  // 切换颜色模式：green <-> blue
  const currentMode = props.token.balance_color_mode || 'green'
  props.token.balance_color_mode = currentMode === 'green' ? 'blue' : 'green'

  // 更新时间戳，确保双向同步时使用最新版本
  props.token.updated_at = new Date().toISOString()

  // 通知父组件有更新，触发保存
  emit('token-updated')
}

// 切换跳过检测状态
const toggleSkipCheck = () => {
  // 切换 skip_check 状态
  props.token.skip_check = !props.token.skip_check

  // 更新时间戳，确保双向同步时使用最新版本
  props.token.updated_at = new Date().toISOString()

  // 关闭菜单
  showCheckMenu.value = false

  // 通知父组件有更新
  emit('token-updated')

  // 显示提示
  const message = props.token.skip_check
    ? t('messages.checkDisabled')
    : t('messages.checkEnabled')
  window.$notify.info(message)
}

// 深度比对两个对象是否相等
const isEqual = (obj1, obj2) => {
  if (obj1 === obj2) return true
  if (obj1 == null || obj2 == null) return false
  if (typeof obj1 !== 'object' || typeof obj2 !== 'object') return obj1 === obj2

  const keys1 = Object.keys(obj1)
  const keys2 = Object.keys(obj2)

  if (keys1.length !== keys2.length) return false

  for (const key of keys1) {
    if (!keys2.includes(key)) return false
    if (!isEqual(obj1[key], obj2[key])) return false
  }

  return true
}

// 检测账号状态
const checkAccountStatus = async (showNotification = true) => {
  // 如果禁用了检测，静默返回
  if (props.token.skip_check) {
    return
  }

  // 如果正在检测中，或者批量检测中（且未禁用），则返回
  if (isCheckingStatus.value || (props.isBatchChecking && !props.token.skip_check)) return

  isCheckingStatus.value = true

  try {
    // 单次API调用同时获取账号状态和Portal信息
    const batchResults = await invoke('batch_check_tokens_status', {
      tokens: [{
        id: props.token.id,
        access_token: props.token.access_token,
        tenant_url: props.token.tenant_url,
        portal_url: props.token.portal_url || null,
        auth_session: props.token.auth_session || null
      }]
    })

    // 处理结果
    let statusMessage = ''
    let statusType = 'info'
    let hasChanges = false

    if (batchResults && batchResults.length > 0) {
      const result = batchResults[0] // 取第一个结果对象
      const statusResult = result.status_result // 账号状态结果

      // 使用后端返回的具体状态
      const banStatus = statusResult.status

      // 比对并更新 access_token
      if (props.token.access_token !== result.access_token) {
        props.token.access_token = result.access_token
        hasChanges = true
      }

      // 比对并更新 tenant_url
      if (props.token.tenant_url !== result.tenant_url) {
        props.token.tenant_url = result.tenant_url
        hasChanges = true
      }

      // 比对并更新 ban_status
      if (props.token.ban_status !== banStatus) {
        props.token.ban_status = banStatus
        hasChanges = true
      }

      // 自动禁用封禁或过期的账号检测
      if ((banStatus === 'SUSPENDED' || banStatus === 'EXPIRED') && !props.token.skip_check) {
        props.token.skip_check = true
        hasChanges = true
        // 通知父组件有更新，触发保存
        emit('token-updated')
        // 显示通知
        const autoDisableMsg = banStatus === 'SUSPENDED'
          ? t('messages.autoDisabledBanned')
          : t('messages.autoDisabledExpired')
        window.$notify.info(autoDisableMsg)
      }

      // 比对并更新 suspensions 信息（如果有）
      if (result.suspensions) {
        if (!isEqual(props.token.suspensions, result.suspensions)) {
          props.token.suspensions = result.suspensions
          hasChanges = true
          console.log(`Updated suspensions for token ${props.token.id}:`, result.suspensions)
        }
      }

      // 比对并更新 Portal 信息（如果有）
      if (result.portal_info) {
        const newPortalInfo = {
          credits_balance: result.portal_info.credits_balance,
          expiry_date: result.portal_info.expiry_date
        }

        if (!isEqual(props.token.portal_info, newPortalInfo)) {
          props.token.portal_info = newPortalInfo
          hasChanges = true

          // 更新UI显示
          portalInfo.value = {
            data: props.token.portal_info,
            error: null
          }
        }
      } else if (result.portal_error) {
        // 保存错误信息到 token
        if (props.token.portal_info !== null) {
          props.token.portal_info = null
          hasChanges = true
        }

        portalInfo.value = {
          data: null,
          error: result.portal_error
        }
      }

      // 只有在有实际变化时才更新时间戳
      if (hasChanges) {
        props.token.updated_at = new Date().toISOString()
        console.log(`Updated token ${props.token.id} with changes`)
      } else {
        console.log(`No changes for token ${props.token.id}, skipping update`)
      }

      // 根据具体状态设置消息
      switch (banStatus) {
        case 'SUSPENDED':
          statusMessage = t('messages.accountBanned')
          statusType = 'error'
          break
        case 'EXPIRED':
          statusMessage = t('tokenCard.expired')
          statusType = 'warning'
          break
        case 'INVALID_TOKEN':
          statusMessage = t('messages.tokenInvalid')
          statusType = 'warning'
          break
        case 'ACTIVE':
          statusMessage = t('messages.accountStatusNormal')
          statusType = 'success'
          break
        case 'ERROR':
          statusMessage = `${t('messages.statusCheckFailed')}: ${statusResult.error_message || 'Unknown error'}`
          statusType = 'error'
          break
        default:
          statusMessage = `${t('messages.accountStatus')}: ${banStatus}`
          statusType = 'info'
      }
    } else {
      statusMessage = t('messages.statusCheckFailed') + ': No results returned'
      statusType = 'error'
    }


    // 发送账号状态消息（不包含次数信息）
    if (showNotification) {
      const finalMessage = `${t('messages.checkComplete')}: ${statusMessage}`
      window.$notify[statusType](finalMessage)
    }

  } catch (error) {
    // 设置错误状态，让UI显示网络错误
    portalInfo.value = {
      data: null,
      error: String(error)
    }

    if (showNotification) {
      window.$notify.error(`${t('messages.checkFailed')}: ${error}`)
    }
  } finally {
    isCheckingStatus.value = false
    isLoadingPortalInfo.value = false
  }
}




// 监听token变化，同步更新Portal信息显示
watch(() => props.token.portal_info, (newPortalInfo) => {
  if (newPortalInfo && props.token.portal_url) {
    portalInfo.value = {
      data: {
        credits_balance: newPortalInfo.credits_balance,
        expiry_date: newPortalInfo.expiry_date
      },
      error: null
    }
  }
}, { deep: true })

// 组件挂载时加载Portal信息
onMounted(async () => {
  if (props.token.portal_url) {
    // 如果有本地数据，立即显示
    if (props.token.portal_info) {
      portalInfo.value = {
        data: {
          credits_balance: props.token.portal_info.credits_balance,
          expiry_date: props.token.portal_info.expiry_date
        },
        error: null
      }
    }
    // 禁用自动刷新，避免清空搜索时大量重新挂载并发起请求
    // checkAccountStatus(false)
  }

  // 添加事件监听器
  document.addEventListener('keydown', handleKeydown)
})

// 组件卸载时清理事件监听器
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// 暴露检查账号状态的方法
const refreshAccountStatus = async () => {
  return await checkAccountStatus()
}

const handleCreditUsageRefresh = () => {
  checkAccountStatus(false)
}

// 暴露方法给父组件
defineExpose({
  refreshAccountStatus
})
</script>

<style scoped>
.token-card {
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  transition: all 0.2s ease;
  height: fit-content;
  min-height: 120px;
  position: relative; /* 为状态指示器定位 */
  z-index: 1;
}

.token-card.menu-open {
  z-index: 1000;
}

/* 高亮动画 */
.token-card.highlighted {
  animation: highlight-pulse 1s ease-in-out 3;
  z-index: 100;
}

@keyframes highlight-pulse {
  0% {
    box-shadow: 0 0 0 3px #fbbf24, 0 2px 8px rgba(0, 0, 0, 0.08);
  }
  50% {
    box-shadow: 0 0 0 6px #fbbf24, 0 2px 8px rgba(0, 0, 0, 0.08);
  }
  100% {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }
}

.status-indicator {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: flex-end;
}

.status-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.status-badge.clickable {
  cursor: pointer;
}

.status-badge.clickable:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.status-badge.active {
  background: var(--color-success-surface, #d4edda);
  color: var(--color-success-text, #155724);
  border: 1px solid var(--color-success-border, #c3e6cb);
}

.status-badge.inactive {
  background: var(--color-danger-surface, #f8d7da);
  color: var(--color-danger-text, #721c24);
  border: 1px solid var(--color-danger-border, #f5c6cb);
}

.status-badge.banned {
  background: var(--color-danger-surface, #f8d7da);
  color: var(--color-danger-text, #721c24);
  border: 1px solid var(--color-danger-border, #f5c6cb);
}

.status-badge.invalid {
  background: var(--color-warning-surface, #fff3cd);
  color: var(--color-warning-text, #856404);
  border: 1px solid var(--color-warning-border, #ffeaa7);
}

.tag-badge {
  max-width: 160px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.token-card:hover {
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
  border-color: var(--color-accent, #3b82f6);
  transform: translateY(-2px);
}

.card-main {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

.token-info {
  flex: 1;
  min-width: 0;
}

.tenant-name {
  margin: 0 0 6px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
  word-break: break-all;
  line-height: 1.3;
}

.token-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.meta-row {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.portal-row {
  margin-top: 2px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.credit-stats-btn {
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 4px 6px;
  cursor: pointer;
  color: var(--color-btn-primary-bg);
  transition: background-color 0.2s, border-color 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 4px;
  flex-shrink: 0;
}

.credit-stats-btn:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-btn-primary-bg);
}

.credit-stats-btn svg {
  display: block;
  flex-shrink: 0;
}

.created-date {
  font-size: 12px;
  color: var(--color-text-muted, #666);
}

/* 邮箱行样式 */
.email-row {
  width: 100%;
}

.email-note-container {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
}

.email-note {
  font-size: 12px;
  color: var(--color-link-visited, #4f46e5);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--color-info-surface, #f0f9ff);
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid var(--color-info-surface, #e0f2fe);
  cursor: pointer;
  user-select: none;
  /* 固定高度避免悬浮时抖动 */
  min-height: 22px;
  /* 限制最大宽度,超出显示省略号 */
  max-width: calc(100% - 30px);
  overflow: hidden;
  /* 不使用 transition,避免尺寸变化时的动画导致抖动 */
}

.email-note:hover {
  background: var(--color-info-surface, #e0f2fe);
  border-color: var(--color-info-border, #bae6fd);
  /* 移除 transform 避免抖动 */
}

/* 邮箱图标固定尺寸,避免抖动 */
.email-icon {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
}

/* 黑暗模式下的邮箱样式优化 */
[data-theme='dark'] .email-note {
  background: rgba(56, 189, 248, 0.2);
  color: #93c5fd;
  border-color: rgba(56, 189, 248, 0.4);
}

[data-theme='dark'] .email-note:hover {
  background: rgba(56, 189, 248, 0.3);
  border-color: rgba(56, 189, 248, 0.6);
  color: #bfdbfe;
}

/* 黑暗模式下的按钮样式优化 */
[data-theme='dark'] .btn-action {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(71, 85, 105, 0.6);
  color: #cbd5e1;
}

[data-theme='dark'] .btn-action:hover {
  background: rgba(71, 85, 105, 0.6);
  border-color: rgba(100, 116, 139, 0.7);
}

[data-theme='dark'] .btn-action.delete {
  color: #fca5a5;
}

[data-theme='dark'] .btn-action.delete:hover {
  background: rgba(220, 38, 38, 0.2);
  border-color: rgba(220, 38, 38, 0.4);
}

[data-theme='dark'] .btn-action.portal {
  color: #93c5fd;
}

[data-theme='dark'] .btn-action.portal:hover {
  background: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.4);
}

[data-theme='dark'] .btn-action.edit {
  color: #86efac;
}

[data-theme='dark'] .btn-action.edit:hover {
  background: rgba(34, 197, 94, 0.2);
  border-color: rgba(34, 197, 94, 0.4);
}

[data-theme='dark'] .btn-action.vscode {
  color: #7dd3fc;
}

[data-theme='dark'] .btn-action.vscode:hover {
  background: rgba(14, 165, 233, 0.2);
  border-color: rgba(14, 165, 233, 0.4);
}

[data-theme='dark'] .btn-action.status-check {
  color: #fcd34d;
}

[data-theme='dark'] .btn-action.status-check:hover {
  background: rgba(245, 158, 11, 0.2);
  border-color: rgba(245, 158, 11, 0.4);
}

[data-theme='dark'] .btn-action.copy {
  color: #93c5fd;
}

[data-theme='dark'] .btn-action.copy:hover {
  background: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.4);
}

[data-theme='dark'] .btn-action.export {
  color: #c4b5fd;
}

[data-theme='dark'] .btn-action.export:hover {
  background: rgba(124, 58, 237, 0.2);
  border-color: rgba(124, 58, 237, 0.4);
}

[data-theme='dark'] .copy-dropdown {
  background: var(--color-surface, #1f2937);
  border-color: rgba(75, 85, 99, 0.6);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .copy-menu-item {
  color: var(--color-text-primary, #e5e7eb);
}

[data-theme='dark'] .copy-menu-item:hover {
  background: rgba(55, 65, 81, 0.6);
}

/* 暗黑模式下的余额颜色 */
[data-theme='dark'] .portal-meta.balance.color-green {
  color: #86efac;
  background: rgba(34, 197, 94, 0.2);
}

[data-theme='dark'] .portal-meta.balance.color-green:hover {
  background: rgba(34, 197, 94, 0.3);
}

[data-theme='dark'] .portal-meta.balance.color-blue {
  color: #f9a8d4;
  background: rgba(236, 72, 153, 0.2);
}

[data-theme='dark'] .portal-meta.balance.color-blue:hover {
  background: rgba(236, 72, 153, 0.3);
}

[data-theme='dark'] .portal-meta.balance.exhausted {
  color: #fca5a5;
  background: rgba(220, 38, 38, 0.2);
}

[data-theme='dark'] .credit-stats-btn {
  border-color: rgba(148, 163, 184, 0.35);
  color: #a78bfa;
}

[data-theme='dark'] .credit-stats-btn:hover {
  background: rgba(124, 58, 237, 0.2);
  border-color: rgba(124, 58, 237, 0.4);
}

.email-icon {
  flex-shrink: 0;
  opacity: 0.7;
}

.copy-email-btn {
  background: none;
  border: none;
  padding: 2px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  border-radius: 3px;
  transition: background 0.2s ease, color 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  /* 固定尺寸避免抖动 */
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.copy-email-btn:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-link-visited, #4f46e5);
}

.copy-email-btn:active {
  transform: scale(0.95);
}

.copy-email-btn svg {
  flex-shrink: 0;
}

.portal-meta {
  font-size: 12px;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 4px;
}

.portal-meta.loading {
  color: var(--color-text-muted, #6c757d);
  font-style: italic;
}

.portal-meta.error {
  color: var(--color-danger-bg, #dc3545);
  background: var(--color-danger-surface, #f8d7da);
}

.portal-meta.expiry {
  color: var(--color-warning-text, #856404);
  background: var(--color-warning-surface, #fff3cd);
}

.portal-meta.balance {
  font-weight: 600;
  transition: all 0.2s ease;
}

/* 绿色模式（默认） */
.portal-meta.balance.color-green {
  color: var(--color-success-text, #155724);
  background: var(--color-success-surface, #d4edda);
}

.portal-meta.balance.color-green:hover {
  background: #c3e6cb;
}

/* 粉色模式 */
.portal-meta.balance.color-blue {
  color: #be185d;
  background: #fce7f3;
}

.portal-meta.balance.color-blue:hover {
  background: #fbcfe8;
}

/* 异常状态（红色，不可切换） */
.portal-meta.balance.exhausted {
  color: var(--color-danger-text, #721c24);
  background: var(--color-danger-surface, #f8d7da);
  cursor: default !important;
}





.actions {
  display: flex;
  flex-direction: row;
  gap: 6px;
  justify-content: flex-end;
  margin-top: auto;
  flex-wrap: wrap;
}

.btn-action {
  background: rgba(148, 163, 184, 0.15);
  border: 1px solid rgba(148, 163, 184, 0.3);
  border-radius: 8px;
  padding: 8px;
  cursor: pointer;
  color: #64748b;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 36px;
  min-height: 36px;
  flex-shrink: 0;
}

/* 防止按钮内的 SVG 图标在 hover 时抖动 */
.btn-action svg,
.btn-action img {
  will-change: transform;
  backface-visibility: hidden;
  -webkit-font-smoothing: subpixel-antialiased;
}

.btn-action:hover {
  background: rgba(148, 163, 184, 0.25);
  border-color: rgba(148, 163, 184, 0.5);
  transform: translateY(-1px);
}

.btn-action.delete {
  color: #dc2626;
}

.btn-action.delete:hover {
  background: rgba(220, 38, 38, 0.15);
  border-color: rgba(220, 38, 38, 0.3);
}

.btn-action.portal {
  color: #2563eb;
}

.btn-action.portal:hover {
  background: rgba(37, 99, 235, 0.15);
  border-color: rgba(37, 99, 235, 0.3);
}

.btn-action.edit {
  color: #16a34a;
}

.btn-action.edit:hover {
  background: rgba(22, 163, 74, 0.15);
  border-color: rgba(22, 163, 74, 0.3);
}

.btn-action.vscode {
  color: #0284c7;
}

.btn-action.vscode:hover {
  background: rgba(2, 132, 199, 0.15);
  border-color: rgba(2, 132, 199, 0.3);
}

.btn-action.status-check {
  color: #ca8a04;
}

.btn-action.status-check:hover {
  background: rgba(202, 138, 4, 0.15);
  border-color: rgba(202, 138, 4, 0.3);
}

.btn-action.status-check.loading {
  opacity: 0.7;
  cursor: not-allowed;
}

.btn-action.export {
  color: #7c3aed;
}

.btn-action.export:hover {
  background: rgba(124, 58, 237, 0.15);
  border-color: rgba(124, 58, 237, 0.3);
}

.btn-action.copy {
  color: #2563eb;
}

.btn-action.copy:hover {
  background: rgba(37, 99, 235, 0.15);
  border-color: rgba(37, 99, 235, 0.3);
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 复制菜单样式 */
.copy-menu-wrapper {
  position: relative;
}

.copy-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  overflow: hidden;
  z-index: 1001; /* 比 token-card.menu-open 的 z-index: 1000 更高 */
}

.copy-menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  color: var(--color-text-primary, #374151);
  transition: background 0.2s ease;
  text-align: left;
  font-family: inherit;
}

.copy-menu-item:hover {
  background: var(--color-surface-hover, #f3f4f6);
}

.copy-menu-item svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.copy-menu-item span {
  flex: 1;
}

/* 检测菜单样式 - 复用复制菜单样式 */
.check-menu-wrapper {
  position: relative;
}

.check-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  overflow: hidden;
  z-index: 1001; /* 比 token-card.menu-open 的 z-index: 1000 更高 */
}

.check-menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  color: var(--color-text-primary, #374151);
  transition: background 0.2s ease;
  text-align: left;
  font-family: inherit;
}

.check-menu-item:hover {
  background: var(--color-surface-hover, #f3f4f6);
}

.check-menu-item svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.check-menu-item span {
  flex: 1;
}

/* 禁用检测时的按钮样式 */
.btn-action.status-check.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-action.status-check.disabled:hover {
  background: rgba(148, 163, 184, 0.15);
  border-color: rgba(148, 163, 184, 0.3);
  transform: none;
}

/* 下拉菜单动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.dropdown-enter-to,
.dropdown-leave-from {
  opacity: 1;
  transform: translateY(0);
}

/* Vue 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .editor-modal,
.modal-leave-to .editor-modal {
  transform: translateY(-20px) scale(0.95);
}

.modal-enter-to .editor-modal,
.modal-leave-from .editor-modal {
  transform: translateY(0) scale(1);
}

/* 编辑器选择模态框样式 */
.editor-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2100;
  backdrop-filter: blur(2px);
  pointer-events: auto;
}

.editor-modal {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 700px;
  width: 90%;
  max-height: 95vh;
  overflow: hidden;
  transition: transform 0.3s ease;
  position: relative;
  pointer-events: auto;
  margin: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

.modal-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted, #666);
  border-radius: 4px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-close:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-heading, #333);
}

.modal-content {
  padding: 20px 24px 24px;
  max-height: calc(95vh - 80px);
  overflow-y: auto;
}



.editor-section {
  margin-bottom: 24px;
  padding-bottom: 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.editor-section:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.editor-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.jetbrains-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 12px;
}

.editor-option {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border: 2px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  background: var(--color-surface, #ffffff);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
  width: 100%;
  position: relative;
  /* 移除按钮默认样式 */
  font-family: inherit;
  font-size: inherit;
  text-decoration: none;
  color: inherit;
  box-sizing: border-box;
}

.editor-option:hover {
  border-color: var(--color-accent, #3b82f6);
  background: var(--color-surface-soft, #f8fafc);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.12);
}

.editor-option:active {
  background: var(--color-surface-soft, #f1f5f9);
  box-shadow: 0 1px 4px rgba(59, 130, 246, 0.08);
}

/* 确保链接在所有状态下都保持正确的样式 */
.editor-option:visited {
  color: inherit;
  text-decoration: none;
}

.editor-option:focus {
  outline: 2px solid var(--color-accent, #3b82f6);
  outline-offset: 2px;
}

.editor-icon {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--color-surface-muted, #f8f9fa);
  border: 1px solid var(--color-surface-muted, #e9ecef);
}

.editor-icon img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.cursor-option .editor-icon {
  background: var(--color-info-surface, #f0f9ff);
  border-color: var(--color-info-surface, #e0f2fe);
}

.vscode-option .editor-icon {
  background: var(--color-info-surface, #f0f9ff);
  border-color: var(--color-info-surface, #e0f2fe);
}

.kiro-option .editor-icon,
.trae-option .editor-icon,
.windsurf-option .editor-icon,
.qoder-option .editor-icon,
.vscodium-option .editor-icon,
.codebuddy-option .editor-icon {
  background: var(--color-info-surface, #f0f9ff);
  border-color: var(--color-info-surface, #e0f2fe);
}

.idea-option .editor-icon,
.pycharm-option .editor-icon,
.goland-option .editor-icon,
.rustrover-option .editor-icon,
.webstorm-option .editor-icon,
.phpstorm-option .editor-icon,
.androidstudio-option .editor-icon,
.clion-option .editor-icon,
.datagrip-option .editor-icon,
.rider-option .editor-icon,
.rubymine-option .editor-icon,
.aqua-option .editor-icon {
  background: var(--color-info-surface, #f0f9ff);
  border-color: var(--color-info-surface, #e0f2fe);
}

.editor-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.editor-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}



/* 响应式处理 */
@media (max-width: 768px) {
  .token-card {
    padding: 16px;
    min-height: 100px;
  }

  .card-main {
    gap: 12px;
  }

  .tenant-name {
    font-size: 14px;
  }

  .actions {
    gap: 4px;
  }

  .btn-action {
    padding: 6px;
    min-width: 32px;
    min-height: 32px;
  }

  .btn-action svg,
  .btn-action img {
    width: 16px;
    height: 16px;
  }
}

@media (max-width: 480px) {
  .token-card {
    padding: 12px;
  }

  .actions {
    gap: 3px;
    justify-content: center;
  }

  .btn-action {
    padding: 6px;
    min-width: 30px;
    min-height: 30px;
  }

  .btn-action svg,
  .btn-action img {
    width: 14px;
    height: 14px;
  }

  .token-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  /* 模态框响应式样式 */
  .editor-modal {
    width: 95%;
    margin: 16px;
    max-height: 90vh;
  }

  .modal-header {
    padding: 16px 20px 12px;
  }

  .modal-header h3 {
    font-size: 16px;
  }

  .modal-content {
    padding: 16px 20px 20px;
  }

  .editor-section {
    margin-bottom: 20px;
    padding-bottom: 20px;
  }

  .jetbrains-grid {
    grid-template-columns: 1fr;
  }

  .editor-option {
    padding: 12px;
    gap: 12px;
  }

  .editor-icon {
    width: 40px;
    height: 40px;
  }

  .editor-icon img {
    width: 28px;
    height: 28px;
  }

  .editor-name {
    font-size: 15px;
  }
}

/* Suspensions 模态框样式 */
.suspensions-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
}

.suspensions-modal {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 600px;
  width: 100%;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.suspensions-modal .modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.suspensions-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.suspensions-modal .modal-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.suspensions-modal .modal-close:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

.suspensions-modal .modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.suspensions-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.suspension-item {
  background: var(--color-surface-secondary, #f9fafb);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  padding: 16px;
}

.suspension-field {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.suspension-field:last-child {
  margin-bottom: 0;
}

.suspension-field strong {
  color: var(--color-text-secondary, #6b7280);
  font-size: 14px;
  min-width: 80px;
}

.suspension-value {
  color: var(--color-text-primary, #374151);
  font-size: 14px;
  word-break: break-word;
}

.no-suspensions {
  text-align: center;
  padding: 40px 20px;
  color: var(--color-text-muted, #9ca3af);
}

.raw-json {
  margin-top: 20px;
  border-top: 1px solid var(--color-divider, #e1e5e9);
  padding-top: 16px;
}

.raw-json summary {
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary, #6b7280);
  padding: 8px 0;
  user-select: none;
}

.raw-json summary:hover {
  color: var(--color-text-primary, #374151);
}

.raw-json pre {
  background: var(--color-surface-secondary, #f9fafb);
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 6px;
  padding: 12px;
  margin: 8px 0 0 0;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-text-primary, #374151);
}

/* 黑暗模式 */
[data-theme='dark'] .suspensions-modal {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .suspension-item {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .raw-json pre {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

/* Trae 版本选择对话框样式 */
.trae-version-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
}

.trae-version-modal {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 500px;
  width: 100%;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.trae-version-modal .modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.trae-version-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.trae-version-modal .modal-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.trae-version-modal .modal-close:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

.trae-version-modal .modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.version-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.version-option {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  border: 2px solid var(--color-divider, #e1e5e9);
  border-radius: 12px;
  background: var(--color-surface, #ffffff);
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  text-align: left;
  font-family: inherit;
}

.version-option:hover {
  border-color: var(--color-accent, #3b82f6);
  background: var(--color-surface-soft, #f8fafc);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
  transform: translateY(-2px);
}

.version-option:active {
  transform: translateY(0);
}

.version-icon {
  font-size: 32px;
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 8px;
}

.version-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

/* 黑暗模式 */
[data-theme='dark'] .trae-version-modal {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .version-option {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .version-option:hover {
  background: rgba(55, 65, 81, 0.7);
  border-color: rgba(59, 130, 246, 0.6);
}

[data-theme='dark'] .version-icon {
  background: rgba(55, 65, 81, 0.8);
}

</style>
