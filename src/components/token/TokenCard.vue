<template>
  <div :class="['token-card', { 'menu-open': showCopyMenu || showCheckMenu, 'highlighted': isHighlighted, 'selected': isSelected }]">
    <!-- 选择框 - 左上角 -->
    <div 
      class="selection-checkbox" 
      :class="{ 'visible': selectionMode || isSelected }"
      @click.stop="$emit('select', token.id)"
    >
      <div class="checkbox-inner" :class="{ 'checked': isSelected }">
        <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
        </svg>
      </div>
    </div>
    <!-- 状态指示器 -->
    <div class="status-indicator">
      <!-- 添加标签按钮（无标签时显示） -->
      <span
        v-if="!hasTag"
        class="add-tag-btn"
        v-tooltip="$t('tokenList.clickToAddTag')"
        @click.stop="openTagEditor"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </span>
      <!-- 标签（有标签时显示，可点击编辑） -->
      <span
        v-if="hasTag"
        class="status-badge editable"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ tagDisplayName }}
      </span>
      <!-- 状态徽章 -->
      <span
        v-if="hasStatusBadge"
        :class="['status-badge', getStatusClass(token.ban_status), { clickable: isBannedWithSuspensions }]"
        v-tooltip="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
        @click="handleStatusClick"
      >
        <span :class="['status-dot', getStatusClass(token.ban_status)]"></span>
        {{ getStatusText(token.ban_status) }}
      </span>
    </div>

    <div class="card-main">
      <div class="token-info">
        <!-- 第一行：邮箱备注 -->
        <div class="email-note-header">
          <div
            v-if="token.email_note"
            class="email-note-container"
            v-tooltip="token.email_note"
            @click.stop="copyEmailNote"
          >
            <span class="email-note">
              {{ showRealEmail ? token.email_note : maskedEmail }}
            </span>
          </div>
          <span v-else class="no-email-note">{{ $t('tokenCard.noEmailNote') }}</span>
        </div>

        <div class="token-meta">
          <!-- 第二行：创建日期、重置时间和 Session 更新时间 -->
          <div class="meta-row">
            <span class="created-date" v-tooltip="$t('tokenCard.createdAt') + ': ' + formatDate(token.created_at)">C: {{ formatDate(token.created_at) }}</span>
            <span v-if="portalInfo.data && portalInfo.data.expiry_date" class="created-date" v-tooltip="$t('tokenCard.resetAt') + ': ' + formatDate(portalInfo.data.expiry_date)">R: {{ formatDate(portalInfo.data.expiry_date) }}</span>
            <span v-if="token.session_updated_at" class="created-date" v-tooltip="$t('tokenCard.sessionUpdatedAt') + ': ' + formatDate(token.session_updated_at)">S: {{ formatDate(token.session_updated_at) }}</span>
          </div>

          <!-- 第三行：余额进度条 -->
          <template v-if="token.portal_url || portalInfo.data">
            <div class="meta-row portal-row">
              <!-- 余额进度条显示 (可点击查看详情) -->
              <div
                v-if="showProgressBar && token.auth_session"
                class="balance-quota-item clickable"
                @click="showCreditUsageModal = true"
                v-tooltip="$t('credit.viewUsage')"
              >
                <div class="balance-quota-header">
                  <span class="balance-info">{{ balanceDisplay }}</span>
                  <span :class="['balance-percentage', progressBarColorClass]">
                    {{ progressPercentage.toFixed(1) }}%
                  </span>
                </div>
                <div class="balance-quota-bar">
                  <div
                    class="balance-quota-fill"
                    :class="progressBarColorClass"
                    :style="{ width: progressPercentage + '%' }"
                  ></div>
                </div>
              </div>
              <!-- 余额进度条显示 (无 auth_session，不可点击) -->
              <div
                v-else-if="showProgressBar"
                class="balance-quota-item"
              >
                <div class="balance-quota-header">
                  <span class="balance-info">{{ balanceDisplay }}</span>
                  <span :class="['balance-percentage', progressBarColorClass]">
                    {{ progressPercentage.toFixed(1) }}%
                  </span>
                </div>
                <div class="balance-quota-bar">
                  <div
                    class="balance-quota-fill"
                    :class="progressBarColorClass"
                    :style="{ width: progressPercentage + '%' }"
                  ></div>
                </div>
              </div>
              <!-- 无进度条时的简单显示 -->
              <span
                v-else
                :class="balanceClasses"
              >
                {{ balanceDisplay }}
              </span>
            </div>
          </template>
        </div>

        <!-- 第四行：Team 管理入口 (仅当有 auth_session 时显示) -->
        <template v-if="token.auth_session">
          <div class="meta-row team-row">
            <button
              class="team-manage-btn"
              v-tooltip="$t('tokenCard.manageTeam')"
              @click="showTeamManageModal = true"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
              </svg>
              <span>{{ $t('tokenCard.team') }}</span>
            </button>
          </div>
        </template>
      </div>

      <div class="actions">
        <button @click="openEditorModal" class="btn-icon" v-tooltip="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" :alt="$t('tokenCard.selectEditor')" width="18" height="18" />
        </button>
        <button @click="exportTokenAsJson" class="btn-icon" v-tooltip="$t('tokenCard.exportJson')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>
        <div class="copy-menu-wrapper" @click.stop ref="copyMenuButton">
          <button @click.stop="toggleCopyMenu" class="btn-icon" v-tooltip="$t('tokenCard.copyMenu')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
        </div>
        <div class="check-menu-wrapper">
          <button
            @click="handleCheckAccountStatus"
            @contextmenu.prevent="showCheckMenu = !showCheckMenu"
            :class="['btn-icon', 'status-check', {
              loading: isCheckingStatus || (isBatchChecking && !token.skip_check) || (isSelectedRefreshing && isSelected && !token.skip_check),
              disabled: token.skip_check
            }]"
            :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check) || (isSelectedRefreshing && isSelected && !token.skip_check)"
            v-tooltip="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
          >
            <svg v-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check) && !(isSelectedRefreshing && isSelected && !token.skip_check) && !token.skip_check" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <!-- 禁用检测时显示暂停图标 -->
            <svg v-else-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check) && !(isSelectedRefreshing && isSelected && !token.skip_check) && token.skip_check" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
            <div v-else-if="isCheckingStatus || (isBatchChecking && !token.skip_check) || (isSelectedRefreshing && isSelected && !token.skip_check)" class="loading-spinner"></div>
          </button>
          <Transition name="dropdown">
            <div v-if="showCheckMenu" class="check-dropdown" @click.stop>
              <button @click="handleToggleSkipCheck" class="check-menu-item">
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
              <!-- 刷新 Session 选项 - 仅当有 auth_session 时显示 -->
              <button
                v-if="token.auth_session"
                @click="handleRefreshSession"
                class="check-menu-item"
                :disabled="isRefreshingSession"
              >
                <svg v-if="!isRefreshingSession" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                </svg>
                <div v-else class="loading-spinner-small"></div>
                <span>{{ $t('tokenCard.refreshSession') }}</span>
              </button>
            </div>
          </Transition>
        </div>
        <button v-if="token.portal_url" @click="showPortalDialog = true" class="btn-icon" v-tooltip="$t('tokenCard.openPortal')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>
        <button @click="$emit('edit', token)" class="btn-icon" v-tooltip="$t('tokenCard.editToken')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>
        <button @click="deleteToken" class="btn-icon delete" v-tooltip="$t('tokenCard.deleteToken')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <!-- 编辑器选择模态框 -->
  <EditorSelectModal
    :show="showEditorModal"
    :token="token"
    @close="showEditorModal = false"
    @success="(msg) => window.$notify.success(msg)"
    @error="(msg) => window.$notify.error(msg)"
  />

  <!-- Portal 对话框 - 使用 Teleport 确保正确定位 -->
  <Teleport to="body">
    <ExternalLinkDialog
      :show="showPortalDialog"
      :title="$t('dialogs.selectOpenMethod')"
      :url="token.portal_url || ''"
      :browser-title="getPortalBrowserTitle()"
      @close="showPortalDialog = false"
    />
  </Teleport>

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

  <!-- Credit Usage Modal -->
  <Teleport to="body">
    <Transition name="modal" appear>
      <CreditUsageModal
        v-if="showCreditUsageModal && token.auth_session"
        :auth-session="token.auth_session"
        :credits-balance="remainingCredits"
        :has-portal-url="!!token.portal_url"
        @close="showCreditUsageModal = false"
        @refresh-balance="handleCreditUsageRefresh"
        @update-portal-url="handleUpdatePortalUrl"
      />
    </Transition>
  </Teleport>

  <!-- 标签编辑模态框 -->
  <TagEditorModal
    v-model:visible="showTagEditor"
    :token="token"
    :all-tokens="allTokens"
    @save="handleTagSave"
    @clear="handleTagClear"
  />

  <!-- Team 管理模态框 -->
  <Teleport to="body">
    <Transition name="modal" appear>
      <TeamManageModal
        v-if="showTeamManageModal && token.auth_session"
        :auth-session="token.auth_session"
        @close="showTeamManageModal = false"
      />
    </Transition>
  </Teleport>

  <!-- 复制菜单 - Teleport 到 body -->
  <Teleport to="body">
    <Transition name="dropdown">
      <div v-if="showCopyMenu" class="copy-dropdown-teleport" @click.stop :style="copyMenuPosition">
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
        <button v-if="token.auth_session" @click="handleCopyMenuClick('payment')" class="copy-menu-item" :disabled="isFetchingPaymentLink">
          <div v-if="isFetchingPaymentLink" class="loading-spinner-small"></div>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
          </svg>
          <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import CreditUsageModal from '../credit/CreditUsageModal.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'
import TeamManageModal from '../team/TeamManageModal.vue'
import { useTokenActions } from '@/composables/useTokenActions'

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
  },
  isSelected: {
    type: Boolean,
    default: false
  },
  selectionMode: {
    type: Boolean,
    default: false
  },
  isSelectedRefreshing: {
    type: Boolean,
    default: false
  },
  cachedPaymentLink: {
    type: String,
    default: null
  },
  allTokens: {
    type: Array,
    default: () => []
  },
  showRealEmail: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['delete', 'edit', 'token-updated', 'select', 'payment-link-fetched'])

// 使用共享的 token actions
const {
  // 常量
  DEFAULT_TAG_COLOR,
  // 响应式状态
  isCheckingStatus,
  isFetchingPaymentLink,
  portalInfo,
  // 计算属性
  tagDisplayName,
  hasTag,
  tagBadgeStyle,
  displayUrl,
  maskedEmail,
  // 工具方法
  formatExpiryDate,
  getStatusClass,
  getStatusText,
  normalizeHexColor,
  getContrastingTextColor,
  isEqual,
  // 剪贴板操作
  copyToClipboard,
  copyWithNotification,
  copyToken,
  copyTenantUrl,
  copyEmailNote,
  copyPortalUrl,
  copyAuthSession,
  exportTokenAsJson,
  copyPaymentMethodLink,
  // 基本操作方法
  deleteToken,
  editToken,
  toggleSelection,
  // 标签操作
  handleTagSave,
  handleTagClear,
  // 状态切换
  toggleSkipCheck,
  // 账号状态检查
  checkAccountStatus,
  handleUpdatePortalUrl,
  getPortalBrowserTitle,
  initPortalInfo,
  formatDate
} = useTokenActions(props, emit)

// 本地 UI 状态
const isLoadingPortalInfo = ref(false)
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const showCheckMenu = ref(false)
const showSuspensionsModal = ref(false)
const showCopyMenu = ref(false)
const showCreditUsageModal = ref(false)
const showTeamManageModal = ref(false)
const copyMenuButton = ref(null)
const isRefreshingSession = ref(false)

// 标签编辑相关
const showTagEditor = ref(false)

// 计算复制菜单位置
const copyMenuPosition = ref({})

const updateCopyMenuPosition = () => {
  if (!copyMenuButton.value || !showCopyMenu.value) {
    return
  }

  const rect = copyMenuButton.value.getBoundingClientRect()
  copyMenuPosition.value = {
    position: 'fixed',
    top: `${rect.bottom + 6}px`,
    right: `${window.innerWidth - rect.right}px`
  }
}

// 监听滚动，实时更新菜单位置
const handleScroll = () => {
  if (showCopyMenu.value) {
    updateCopyMenuPosition()
  }
}

// 全局点击监听，点击菜单外部关闭
const handleGlobalClick = (event) => {
  // 检查是否点击了当前 TokenCard 的复制按钮或菜单
  const clickedCopyButton = copyMenuButton.value?.contains(event.target)
  const clickedCopyMenu = event.target.closest('.copy-dropdown-teleport')

  // 检查是否点击了检测按钮或菜单（任何 TokenCard 的）
  const clickedCheckButton = event.target.closest('.check-menu-wrapper .btn-icon')
  const clickedCheckDropdown = event.target.closest('.check-dropdown')

  // 如果没有点击当前卡片的复制相关元素，关闭复制菜单
  if (showCopyMenu.value && !clickedCopyButton && !clickedCopyMenu) {
    showCopyMenu.value = false
  }

  // 如果没有点击检测相关元素，关闭检测菜单
  if (showCheckMenu.value && !clickedCheckButton && !clickedCheckDropdown) {
    showCheckMenu.value = false
  }
}

// 跟踪全局监听器是否已添加
const globalClickListenerAdded = ref(false)

// 添加全局点击监听器
const addGlobalClickListener = () => {
  if (!globalClickListenerAdded.value) {
    // 使用 nextTick 确保在当前点击事件处理完后再添加监听器
    setTimeout(() => {
      document.addEventListener('click', handleGlobalClick, true)
      globalClickListenerAdded.value = true
    }, 0)
  }
}

// 移除全局点击监听器
const removeGlobalClickListener = () => {
  if (globalClickListenerAdded.value && !showCopyMenu.value && !showCheckMenu.value) {
    document.removeEventListener('click', handleGlobalClick, true)
    globalClickListenerAdded.value = false
  }
}

// 监听复制菜单打开，更新位置并添加滚动监听
watch(showCopyMenu, (isOpen) => {
  if (isOpen) {
    updateCopyMenuPosition()
    // 添加滚动监听，实时更新位置
    window.addEventListener('scroll', handleScroll, true)
    // 添加全局点击监听
    addGlobalClickListener()
  } else {
    // 移除滚动监听
    window.removeEventListener('scroll', handleScroll, true)
    // 移除全局点击监听（如果两个菜单都关闭）
    removeGlobalClickListener()
  }
})

// 监听检测菜单打开，添加全局点击监听
watch(showCheckMenu, (isOpen) => {
  if (isOpen) {
    // 添加全局点击监听
    addGlobalClickListener()
  } else {
    // 移除全局点击监听（如果两个菜单都关闭）
    removeGlobalClickListener()
  }
})

const hasStatusBadge = computed(() => {
  const hasPortalStatus = portalInfo.value.data  // 只要有 portal_info 数据就显示
  return Boolean(hasPortalStatus || props.token.ban_status)
})

// 当前主题
const currentTheme = ref('light')

// 监听主题变化
const updateTheme = () => {
  currentTheme.value = document.documentElement.dataset.theme || 'light'
}

onMounted(() => {
  updateTheme()
  // 监听主题变化
  const observer = new MutationObserver(() => {
    updateTheme()
  })
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme']
  })
  // 清理
  onUnmounted(() => {
    observer.disconnect()
    // 清理滚动监听
    window.removeEventListener('scroll', handleScroll, true)
    // 清理全局点击监听
    if (globalClickListenerAdded.value) {
      document.removeEventListener('click', handleGlobalClick, true)
      globalClickListenerAdded.value = false
    }
  })
})

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

// Portal余额显示相关计算属性
const balanceClasses = computed(() => {
  // 网络错误或账号异常状态显示红色
  const hasError = portalInfo.value?.error
  const exhausted = (
    props.token.ban_status === 'EXPIRED'
    // 移除 SUSPENDED 的判断，让封禁账号也能正常显示颜色
    // || props.token.ban_status === 'SUSPENDED'
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
// 是否为封禁或过期状态
const isBannedOrExpired = computed(() => {
  const status = props.token.ban_status
  return status === 'EXPIRED' || status === 'SUSPENDED'
})

// 是否显示进度条
const showProgressBar = computed(() => {
  if (!portalInfo.value?.data) return false
  const status = props.token.ban_status
  // 只有过期状态不显示进度条，封禁状态也显示进度条
  if (status === 'EXPIRED') return false

  const balance = portalInfo.value.data.credits_balance
  const total = portalInfo.value.data.credit_total
  return balance !== undefined && total !== undefined && total > 0
})

// 进度百分比
const progressPercentage = computed(() => {
  if (!showProgressBar.value) return 0
  const balance = portalInfo.value.data.credits_balance
  const total = portalInfo.value.data.credit_total
  return Math.min(100, Math.max(0, (balance / total) * 100))
})

// 进度条颜色类
const progressBarColorClass = computed(() => {
  const percentage = progressPercentage.value
  if (percentage > 50) return 'progress-high'
  if (percentage > 20) return 'progress-medium'
  return 'progress-low'
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
  // 封禁状态也显示额度信息（如果有的话）
  // if (status === 'SUSPENDED') return t('tokenCard.banned')

  const balance = portalInfo.value.data.credits_balance
  const total = portalInfo.value.data.credit_total

  // 如果有总额，显示 "余额 / 总额" 格式
  if (total !== undefined && total > 0) {
    return `${balance} / ${total}`
  }

  // 否则只显示余额
  return `${t('tokenCard.balance')}: ${balance}`
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



// 切换复制菜单
const toggleCopyMenu = () => {
  showCopyMenu.value = !showCopyMenu.value
}

// 处理复制菜单项点击
const handleCopyMenuClick = async (type) => {
  // payment 类型不立即关闭菜单，等异步操作完成后再关闭
  if (type !== 'payment') {
    showCopyMenu.value = false
  }
  
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
    case 'payment':
      await copyPaymentMethodLink({
        cachedPaymentLink: props.cachedPaymentLink,
        onMenuClose: () => { showCopyMenu.value = false }
      })
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

// 打开编辑器模态框
const openEditorModal = () => {
  if (showEditorModal.value) return
  showEditorModal.value = true
}


// 本地包装的 toggleSkipCheck，需要关闭菜单
const handleToggleSkipCheck = () => {
  toggleSkipCheck()
  showCheckMenu.value = false
}

// 刷新单个 session
const handleRefreshSession = async () => {
  if (!props.token.auth_session) {
    window.$notify.warning(t('messages.noAuthSession'))
    showCheckMenu.value = false
    return
  }

  isRefreshingSession.value = true
  showCheckMenu.value = false

  try {
    // 调用后端刷新接口
    const results = await invoke('batch_refresh_sessions', {
      requests: [{
        id: props.token.id,
        session: props.token.auth_session
      }]
    })

    if (results && results.length > 0) {
      const result = results[0]
      if (result.success && result.new_session) {
        const now = new Date().toISOString()

        // 更新 token 的 auth_session 和 session_updated_at
        props.token.auth_session = result.new_session
        props.token.session_updated_at = now
        props.token.updated_at = now

        // 触发 token-updated 事件，通知父组件保存
        emit('token-updated', props.token)

        window.$notify.success(t('messages.sessionRefreshSuccess', { count: 1 }))
      } else {
        const errorMsg = result.error || 'Unknown error'
        console.error(`Failed to refresh session for token ${props.token.id}:`, errorMsg)
        window.$notify.error(t('messages.sessionRefreshFailed') + ': ' + errorMsg)
      }
    }
  } catch (error) {
    console.error('Failed to refresh session:', error)
    window.$notify.error(t('messages.sessionRefreshFailed') + ': ' + error.toString())
  } finally {
    isRefreshingSession.value = false
  }
}

// 本地包装的 checkAccountStatus
const handleCheckAccountStatus = async (showNotification = true) => {
  await checkAccountStatus({
    showNotification,
    isBatchChecking: props.isBatchChecking,
    isSelectedRefreshing: props.isSelectedRefreshing,
    isSelected: props.isSelected
  })
}




// 监听token变化，同步更新Portal信息显示
watch(() => props.token.portal_info, (newPortalInfo) => {
  if (newPortalInfo) {
    portalInfo.value = {
      data: {
        credits_balance: newPortalInfo.credits_balance,
        credit_total: newPortalInfo.credit_total,
        expiry_date: newPortalInfo.expiry_date
      },
      error: null
    }
  }
}, { deep: true })

// 组件挂载时加载Portal信息
onMounted(async () => {
  // 如果有本地 portal_info 数据，立即显示（不管是否有 portal_url）
  if (props.token.portal_info) {
    portalInfo.value = {
      data: {
        credits_balance: props.token.portal_info.credits_balance,
        credit_total: props.token.portal_info.credit_total,
        expiry_date: props.token.portal_info.expiry_date
      },
      error: null
    }

    // 如果 credit_total 为空，自动刷新
    if (!props.token.portal_info.credit_total) {
      await handleCheckAccountStatus(false)
    }
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
  return await handleCheckAccountStatus()
}

const handleCreditUsageRefresh = () => {
  handleCheckAccountStatus(false)
}

// 打开标签编辑器
const openTagEditor = () => {
  showTagEditor.value = true
}

// 暴露方法给父组件
defineExpose({
  refreshAccountStatus
})
</script>

<style scoped>
/* ============================================
   TokenCard - Modern Tech Style
   ============================================ */

.token-card {
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  padding: 20px;
  box-shadow: var(--tech-border-glow);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  height: fit-content;
  min-height: 140px;
  position: relative;
  z-index: 1;
  overflow: hidden;
  /* 优化滚动性能,防止样式丢失 */
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-font-smoothing: subpixel-antialiased;
}

.token-card.menu-open {
  z-index: 1200;
  overflow: visible;
}

/* 高亮动画 - 科技风脉冲 */
.token-card.highlighted {
  animation: tech-pulse 1.2s ease-in-out 3;
  z-index: 100;
}

@keyframes tech-pulse {
  0%, 100% {
    box-shadow: var(--tech-border-glow);
    border-color: var(--tech-glass-border);
  }
  50% {
    box-shadow: 0 0 0 2px var(--accent), 0 0 30px -5px var(--tech-glow-primary);
    border-color: var(--accent);
  }
}

/* 选中状态 */
.token-card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent), 0 0 25px -5px var(--tech-glow-primary);
}

/* 选择框样式 */
.selection-checkbox {
  position: absolute;
  top: 12px;
  left: 12px;
  z-index: 15;
  opacity: 0;
  transform: scale(0.85);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.selection-checkbox.visible,
.token-card:hover .selection-checkbox {
  opacity: 1;
  transform: scale(1);
}

.checkbox-inner {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  border: 1.5px solid var(--border);
  background: var(--bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.checkbox-inner:hover {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
}

.checkbox-inner.checked {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

.checkbox-inner.checked svg {
  color: #fff;
}

.status-indicator {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: flex-end;
}

/* 状态徽章 - 现代简约 */
/* Hover 效果 */
.token-card:hover {
  box-shadow: 0 0 0 1px var(--tech-glass-border), 0 8px 32px -8px rgba(0, 0, 0, 0.2), 0 0 20px -5px var(--tech-glow-primary);
  border-color: color-mix(in srgb, var(--accent) 50%, var(--tech-glass-border));
  transform: translateY(-2px);
}

.card-main {
  display: flex;
  flex-direction: column;
  gap: 14px;
  height: 100%;
  margin-top: 14px;
}

/* 信息区域 */
.token-info {
  flex: 1;
  min-width: 0;
}

.email-note-header {
  margin: 0 0 10px 0;
  min-height: 26px;
  display: flex;
  align-items: center;
}

.email-note-header .email-note-container {
  max-width: 100%;
}

.token-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.meta-row {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.portal-row {
  margin-top: 4px;
  display: flex;
  align-items: center;
  gap: 10px;
}

/* 余额配额项 - 参考 AccountCard 样式 */
.balance-quota-item {
  width: 100%;
  margin-bottom: 4px;
  transition: all 0.2s ease;
}

.balance-quota-item.clickable {
  cursor: pointer;
  border-radius: 8px;
  padding: 6px;
  margin: -6px;
  margin-bottom: -2px;
}

.balance-quota-item.clickable:hover {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  transform: translateY(-1px);
}

.balance-quota-item.clickable:hover .balance-quota-bar {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.balance-quota-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
  font-size: 12px;
}

.balance-info {
  color: var(--text-primary);
  font-weight: 500;
  font-family: var(--tech-mono-font);
}

.balance-percentage {
  font-weight: 600;
  font-family: var(--tech-mono-font);
}

.balance-percentage.progress-high {
  color: #4caf50;
}

.balance-percentage.progress-medium {
  color: #ff9800;
}

.balance-percentage.progress-low {
  color: #f44336;
}

.balance-quota-bar {
  width: 100%;
  height: 6px;
  background: color-mix(in srgb, var(--text-muted) 15%, transparent);
  border-radius: 3px;
  overflow: hidden;
}

.balance-quota-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.balance-quota-fill.progress-high {
  background: linear-gradient(90deg, #4caf50, #66bb6a);
}

.balance-quota-fill.progress-medium {
  background: linear-gradient(90deg, #ff9800, #ffb74d);
}

.balance-quota-fill.progress-low {
  background: linear-gradient(90deg, #f44336, #ef5350);
}

.credit-stats-btn {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 25%, transparent);
  border-radius: 8px;
  padding: 5px 7px;
  cursor: pointer;
  color: var(--accent);
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 2px;
  flex-shrink: 0;
}

.credit-stats-btn:hover {
  background: color-mix(in srgb, var(--accent) 18%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 10px var(--tech-glow-primary);
  transform: scale(1.05);
}

.credit-stats-btn svg {
  display: block;
  flex-shrink: 0;
}

.created-date {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--tech-mono-font);
  opacity: 0.7;
}

/* 邮箱行样式 */
.email-row {
  width: 100%;
}

/* Portal 元信息 */
.portal-meta {
  font-size: 11px;
  font-weight: 600;
  font-family: var(--tech-mono-font);
  padding: 4px 10px;
  border-radius: 8px;
  letter-spacing: 0.2px;
}

.portal-meta.loading {
  color: var(--text-muted);
  font-style: italic;
}

.portal-meta.error {
  color: #f87171;
  background: color-mix(in srgb, #ef4444 15%, transparent);
  border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
}

/* 余额显示 */
.portal-meta.balance {
  font-weight: 700;
  transition: all 0.25s ease;
}

/* 绿色模式 */
.portal-meta.balance.color-green {
  color: #34d399;
  background: color-mix(in srgb, #10b981 18%, transparent);
  border: 1px solid color-mix(in srgb, #10b981 35%, transparent);
}

.portal-meta.balance.color-green:hover {
  background: color-mix(in srgb, #10b981 25%, transparent);
  box-shadow: 0 0 12px var(--tech-glow-success);
}

/* 粉色/紫色模式 */
.portal-meta.balance.color-blue {
  color: #c084fc;
  background: color-mix(in srgb, #a855f7 18%, transparent);
  border: 1px solid color-mix(in srgb, #a855f7 35%, transparent);
}

.portal-meta.balance.color-blue:hover {
  background: color-mix(in srgb, #a855f7 25%, transparent);
  box-shadow: 0 0 12px rgba(168, 85, 247, 0.4);
}

/* 异常状态 */
.portal-meta.balance.exhausted {
  color: #fca5a5;
  background: color-mix(in srgb, #ef4444 18%, transparent);
  border: 1px solid color-mix(in srgb, #ef4444 35%, transparent);
  cursor: default !important;
}


/* 操作按钮区 */
.actions {
  display: flex;
  flex-direction: row;
  gap: 6px;
  justify-content: flex-end;
  margin-top: auto;
  flex-wrap: wrap;
  padding-top: 4px;
}

.btn-icon.status-check.loading {
  opacity: 0.7;
  cursor: not-allowed;
}

/* 加载动画 - 科技风 */
.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
  border-top: 2px solid var(--accent);
  border-radius: 50%;
  animation: tech-spin 0.8s linear infinite;
}

.loading-spinner-small {
  width: 12px;
  height: 12px;
  border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
  border-top: 2px solid var(--accent);
  border-radius: 50%;
  animation: tech-spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes tech-spin {
  to {
    transform: rotate(360deg);
  }
}

/* 下拉菜单 - 磨砂玻璃效果 */
.copy-menu-wrapper {
  position: relative;
}

.copy-dropdown-teleport {
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--tech-card-bg));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  min-width: 190px;
  overflow: hidden;
  z-index: 9999;
  padding: 6px;
}

.copy-menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  color: var(--text);
  transition: all 0.2s ease;
  text-align: left;
  font-family: inherit;
  border-radius: 8px;
}

.copy-menu-item:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.copy-menu-item svg {
  flex-shrink: 0;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.copy-menu-item:hover svg {
  opacity: 1;
}

.copy-menu-item span {
  flex: 1;
}

.copy-menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.copy-menu-item:disabled:hover {
  background: transparent;
  color: var(--text);
}

/* 检测菜单 */
.check-menu-wrapper {
  position: relative;
}

.check-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--tech-card-bg));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  min-width: 180px;
  overflow: hidden;
  z-index: 9999;
  padding: 6px;
}

.check-menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  color: var(--text);
  transition: all 0.2s ease;
  text-align: left;
  font-family: inherit;
  border-radius: 8px;
}

.check-menu-item:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.check-menu-item svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.check-menu-item:hover svg {
  opacity: 1;
}

.check-menu-item span {
  flex: 1;
}

/* 禁用检测时的按钮样式 */
.btn-icon.status-check.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon.status-check.disabled:hover {
  background: var(--bg-muted);
  border-color: var(--border);
  color: var(--text-muted);
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

/* 响应式处理 */
@media (max-width: 768px) {
  .token-card {
    padding: 16px;
    min-height: 100px;
  }

  .card-main {
    gap: 12px;
  }

  .actions {
    gap: 4px;
  }

  .btn-icon {
    padding: 6px;
    min-width: 32px;
    min-height: 32px;
  }

  .btn-icon svg,
  .btn-icon img {
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

  .btn-icon {
    padding: 6px;
    min-width: 30px;
    min-height: 30px;
  }

  .btn-icon svg,
  .btn-icon img {
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

  .editor-modal .modal-header {
    padding: 16px 20px 12px;
  }

  .editor-modal .modal-header h3 {
    font-size: 16px;
  }

  .editor-modal .modal-content {
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
  background: var(--bg-surface);
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
  border-bottom: 1px solid var(--border);
}

.suspensions-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
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
  background: var(--bg-muted);
  border: 1px solid var(--border);
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
  color: var(--text-muted);
  font-size: 14px;
  min-width: 80px;
}

.suspension-value {
  color: var(--text);
  font-size: 14px;
  word-break: break-word;
}

.no-suspensions {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
}

.raw-json {
  margin-top: 20px;
  border-top: 1px solid var(--border);
  padding-top: 16px;
}

.raw-json summary {
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-muted);
  padding: 8px 0;
  user-select: none;
}

.raw-json summary:hover {
  color: var(--text);
}

.raw-json pre {
  background: var(--bg-muted);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 12px;
  margin: 8px 0 0 0;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text);
}

/* 添加标签按钮样式 */
.add-tag-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px dashed var(--border);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  opacity: 0;
}

.token-card:hover .add-tag-btn {
  opacity: 1;
}

.add-tag-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

/* Team 管理按钮样式 */
.team-row {
  margin-top: 6px;
}

.team-manage-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--bg-muted);
  border: 1px solid var(--border);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.team-manage-btn:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.team-manage-btn svg {
  opacity: 0.7;
}

.team-manage-btn:hover svg {
  opacity: 1;
}

</style>
