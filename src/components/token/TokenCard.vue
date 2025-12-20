<template>
  <div :class="['token-card', { 'menu-open': showCopyMenu || showCheckMenu, 'highlighted': isHighlighted, 'selected': isSelected }]" @click="handleClickOutside">
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
        :title="$t('tokenList.clickToAddTag')"
        @click.stop="openTagEditor"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </span>
      <!-- 标签（有标签时显示，可点击编辑） -->
      <span
        v-if="hasTag"
        class="status-badge tag-badge editable"
        :style="tagBadgeStyle"
        :title="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ tagDisplayName }}
      </span>
      <!-- 状态徽章 -->
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
        <!-- 第一行：邮箱备注 -->
        <div class="email-note-header">
          <div
            v-if="token.email_note"
            class="email-note-container"
            @click.stop="copyEmailNote"
            :title="token.email_note"
          >
            <span class="email-note">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="email-icon">
                <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.89 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
              </svg>
              {{ showRealEmail ? token.email_note : maskedEmail }}
            </span>
            <span class="copy-hint">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
            </span>
          </div>
          <span v-else class="no-email-note">{{ $t('tokenCard.noEmailNote') }}</span>
        </div>

        <div class="token-meta">
          <!-- 第二行：创建日期 -->
          <div class="meta-row">
            <span class="created-date">{{ formatDate(token.created_at) }}</span>
          </div>

          <!-- 第三行：重置时间和余额 -->
          <template v-if="token.portal_url || portalInfo.data">
            <div class="meta-row portal-row">
              <template v-if="portalInfo.data">
                <span v-if="portalInfo.data.expiry_date" class="portal-meta expiry">{{ $t('tokenCard.resetTime') }}: {{ formatExpiryDate(portalInfo.data.expiry_date) }}</span>
              </template>
              <!-- 余额显示：无论是否有数据都显示 -->
              <span
                :class="balanceClasses"
                @click="handleToggleBalanceColor"
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
          <img src="/icons/vscode.svg" :alt="$t('tokenCard.selectEditor')" width="18" height="18" />
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
              <button v-if="token.auth_session" @click="handleCopyMenuClick('payment')" class="copy-menu-item" :disabled="isFetchingPaymentLink">
                <div v-if="isFetchingPaymentLink" class="loading-spinner-small"></div>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
                </svg>
                <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
              </button>
            </div>
          </Transition>
        </div>
        <div class="check-menu-wrapper">
          <button
            @click="handleCheckAccountStatus"
            @contextmenu.prevent="showCheckMenu = !showCheckMenu"
            :class="['btn-action', 'status-check', {
              loading: isCheckingStatus || (isBatchChecking && !token.skip_check) || (isSelectedRefreshing && isSelected && !token.skip_check),
              disabled: token.skip_check
            }]"
            :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check) || (isSelectedRefreshing && isSelected && !token.skip_check)"
            :title="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
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

  <!-- 编辑器选择模态框 -->
  <EditorSelectModal
    :show="showEditorModal"
    :token="token"
    @close="showEditorModal = false"
    @success="(msg) => window.$notify.success(msg)"
    @error="(msg) => window.$notify.error(msg)"
  />

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
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import CreditUsageModal from '../credit/CreditUsageModal.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'
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
  toggleBalanceColor,
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

// 标签编辑相关
const showTagEditor = ref(false)

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
  if (showEditorModal.value) return
  showEditorModal.value = true
}


// 本地包装的 toggleSkipCheck，需要关闭菜单
const handleToggleSkipCheck = () => {
  toggleSkipCheck()
  showCheckMenu.value = false
}

// 本地包装的 toggleBalanceColor，需要检查 isBalanceClickable
const handleToggleBalanceColor = () => {
  if (!isBalanceClickable.value) {
    return
  }
  toggleBalanceColor()
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
        expiry_date: props.token.portal_info.expiry_date
      },
      error: null
    }
  }

  // 只对有 auth_session 但没有 portal_url 的 token 自动检查状态
  // 排除已封禁(SUSPENDED)和已过期(EXPIRED)的 token
  if (props.token.auth_session &&
      !props.token.portal_url &&
      props.token.ban_status !== 'SUSPENDED' &&
      props.token.ban_status !== 'EXPIRED') {
    console.log('TokenCard: Auto-checking status for token with auth_session but no portal_url:', props.token.id)
    await handleCheckAccountStatus(false)  // 静默检查
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
.token-card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  transition: all 0.2s ease;
  height: fit-content;
  min-height: 150px;
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

/* 选中状态样式 */
.token-card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent), 0 2px 8px rgba(0, 0, 0, 0.08);
}

/* 选择框样式 */
.selection-checkbox {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 15;
  opacity: 0;
  transform: scale(0.8);
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.selection-checkbox.visible,
.token-card:hover .selection-checkbox {
  opacity: 1;
  transform: scale(1);
}

.checkbox-inner {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: 2px solid var(--border);
  background: var(--bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.checkbox-inner:hover {
  border-color: var(--accent);
  background: var(--bg-muted);
}

.checkbox-inner.checked {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--text-contrast);
}

.checkbox-inner.checked svg {
  color: var(--text-contrast);
}

.status-indicator {
  position: absolute;
  top: 12px;
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
  background: var(--state-success);
  color: #ffffff;
  border: 1px solid var(--state-success);
}

.status-badge.inactive {
  background: var(--state-danger);
  color: #ffffff;
  border: 1px solid var(--state-danger);
}

.status-badge.banned {
  background: var(--state-danger);
  color: #ffffff;
  border: 1px solid var(--state-danger);
}

.status-badge.invalid {
  background: var(--state-warning);
  color: #ffffff;
  border: 1px solid var(--state-warning);
}

.tag-badge {
  max-width: 160px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.token-card:hover {
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
  border-color: var(--accent);
  transform: translateY(-2px);
}

.card-main {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
  margin-top: 16px;
}

.token-info {
  flex: 1;
  min-width: 0;
}

.email-note-header {
  margin: 0 0 8px 0;
  min-height: 24px;
  display: flex;
  align-items: center;
}

.email-note-header .email-note-container {
  width: 100%;
}

.email-note-header .email-note {
  font-size: 14px;
  font-weight: 600;
}

.no-email-note {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  font-style: italic;
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
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 4px 6px;
  cursor: pointer;
  color: var(--accent);
  transition: background-color 0.2s, border-color 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 4px;
  flex-shrink: 0;
}

.credit-stats-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
}

.credit-stats-btn svg {
  display: block;
  flex-shrink: 0;
}

.created-date {
  font-size: 12px;
  color: var(--text-muted);
}

/* 邮箱行样式 */
.email-row {
  width: 100%;
}

.email-note-container {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  max-width: 100%;
  position: relative;
}

.email-note {
  font-size: 12px;
  color: var(--accent);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid color-mix(in srgb, var(--accent) 20%, transparent);
  user-select: none;
  /* 固定高度避免悬浮时抖动 */
  min-height: 22px;
  /* 限制最大宽度,超出显示省略号 */
  max-width: fit-content;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.email-note-container:hover .email-note {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent) 30%, transparent);
}

/* 邮箱图标固定尺寸,避免抖动 */
.email-icon {
  flex-shrink: 0;
  width: 12px;
  height: 12px;
  opacity: 0.7;
}

.copy-hint {
  color: var(--text-muted);
  opacity: 0;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  transition: opacity 0.2s ease;
}

.email-note-container:hover .copy-hint {
  opacity: 1;
}

.portal-meta {
  font-size: 12px;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 4px;
}

.portal-meta.loading {
  color: var(--text-muted);
  font-style: italic;
}

.portal-meta.error {
  color: #dc2626;
  background: color-mix(in srgb, #dc2626 15%, transparent);
}

.portal-meta.expiry {
  color: #ffffff;
  background: var(--state-warning);
}

.portal-meta.balance {
  font-weight: 600;
  transition: all 0.2s ease;
}

/* 绿色模式（默认） */
.portal-meta.balance.color-green {
  color: var(--state-success);
  background: color-mix(in srgb, var(--state-success) 15%, transparent);
}

.portal-meta.balance.color-green:hover {
  background: color-mix(in srgb, var(--state-success) 20%, transparent);
}

/* 粉色模式 */
.portal-meta.balance.color-blue {
  color: #ec4899;
  background: color-mix(in srgb, #ec4899 15%, transparent);
}

.portal-meta.balance.color-blue:hover {
  background: color-mix(in srgb, #ec4899 20%, transparent);
}

/* 异常状态（红色，不可切换） */
.portal-meta.balance.exhausted {
  color: #ffffff;
  background: var(--state-danger);
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
  background: var(--bg-muted);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px;
  cursor: pointer;
  color: var(--text-muted);
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
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text);
  transform: translateY(-1px);
}

.btn-action.delete:hover {
  background: color-mix(in srgb, #dc2626 15%, transparent);
  border-color: #dc2626;
  color: #dc2626;
}

.btn-action.status-check.loading {
  opacity: 0.7;
  cursor: not-allowed;
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-spinner-small {
  width: 12px;
  height: 12px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  display: inline-block;
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
  background: var(--bg-surface);
  border: 1px solid var(--border);
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
  color: var(--text);
  transition: background 0.2s ease;
  text-align: left;
  font-family: inherit;
}

.copy-menu-item:hover {
  background: var(--bg-hover);
}

.copy-menu-item svg {
  flex-shrink: 0;
  opacity: 0.7;
}

.copy-menu-item span {
  flex: 1;
}

.copy-menu-item:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.copy-menu-item:disabled:hover {
  background: transparent;
}

/* 检测菜单样式 - 复用复制菜单样式 */
.check-menu-wrapper {
  position: relative;
}

.check-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background: var(--bg-surface);
  border: 1px solid var(--border);
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
  color: var(--text);
  transition: background 0.2s ease;
  text-align: left;
  font-family: inherit;
}

.check-menu-item:hover {
  background: var(--bg-hover);
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

  .email-note-header .email-note {
    font-size: 13px;
  }

  .no-email-note {
    font-size: 13px;
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

.suspensions-modal .modal-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--text-muted);
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.suspensions-modal .modal-close:hover {
  background: var(--bg-hover);
  color: var(--text);
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

.status-badge.tag-badge.editable {
  cursor: pointer;
}

.status-badge.tag-badge.editable:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

</style>
