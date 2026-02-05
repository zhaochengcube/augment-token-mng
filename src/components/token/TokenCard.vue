<template>
  <div :class="['card rounded-xl group', { 'selected': isSelected }, { 'ring-2 ring-warning ring-offset-1 bg-warning/10': isHighlighted }]">
    <!-- 选择框 - 左上角 -->
    <div 
      class="selection-checkbox" 
      :class="{ 'visible': selectionMode || isSelected }"
      @click.stop="$emit('select', token.id)" >
      <div class="checkbox-inner" :class="{ 'checked': isSelected }">
        <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
        </svg>
      </div>
    </div>
    <!-- 状态指示器 -->
    <div class="absolute top-3 right-3 z-10 flex items-center gap-1.5 justify-end">
      <!-- 添加标签按钮（无标签时显示） -->
      <span
        v-if="!hasTag"
        class="btn btn--icon-sm btn--dashed"
        v-tooltip="$t('tokenList.clickToAddTag')"
        @click.stop="openTagEditor">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </span>
      <!-- 标签（有标签时显示，可点击编辑） -->
      <span
        v-if="hasTag"
        class="badge editable"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor">
        {{ tagDisplayName }}
      </span>
      <!-- 状态徽章 -->
      <span
        :class="['badge', getStatusClass(token.ban_status), { editable: isBannedWithSuspensions }]"
        v-tooltip="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
        @click="handleStatusClick">
        <span :class="['status-dot', getStatusClass(token.ban_status)]"></span>
        {{ getStatusText(token.ban_status) }}
      </span>
    </div>

    <div class="flex flex-col gap-1 h-full mt-3.5">
      <div class="flex-1 min-w-0">
        <!-- 第一行：邮箱备注 -->
        <div class="mb-2.5 min-h-[26px] flex items-center">
          <div
            v-if="token.email_note"
            class="text-copyable"
            v-tooltip="token.email_note"
            @click.stop="copyEmailNote"
          >
            <span class="text-copyable__content">
              {{ showRealEmail ? token.email_note : maskedEmail }}
            </span>
          </div>
          <span v-else class="text-copyable--muted">{{ $t('tokenCard.noEmailNote') }}</span>
        </div>

        <div class="flex flex-col gap-2">
          <!-- 第二行：创建日期、重置时间和 Session 更新时间 -->
          <div class="flex justify-start items-center gap-2.5 flex-wrap">
            <span class="text-meta" v-tooltip="$t('tokenCard.createdAt') + ': ' + formatDate(token.created_at)">C: {{ formatDate(token.created_at) }}</span>
            <span v-if="portalInfo.data && portalInfo.data.expiry_date" class="text-meta" v-tooltip="$t('tokenCard.resetAt') + ': ' + formatDate(portalInfo.data.expiry_date)">R: {{ formatDate(portalInfo.data.expiry_date) }}</span>
            <span v-if="token.session_updated_at" class="text-meta" v-tooltip="$t('tokenCard.sessionUpdatedAt') + ': ' + formatDate(token.session_updated_at)">S: {{ formatDate(token.session_updated_at) }}</span>
          </div>

          <!-- 第三行：余额进度条 -->
          <template v-if="token.portal_url || portalInfo.data">
            <div class="flex items-center gap-2.5 mt-1">
              <div v-if="showProgressBar" class="w-full mb-1">
                <QuotaBar
                  :label="balanceDisplay"
                  :value="portalInfo.data.credits_balance"
                  :max="portalInfo.data.credit_total"
                  :low-threshold="20"
                  :medium-threshold="50"
                />
              </div>
              <!-- 无进度条时的简单显示 -->
              <span
                v-else
                class="badge"
                :class="balanceClasses"
              >
                {{ balanceDisplay }}
              </span>
            </div>
          </template>
        </div>

        <!-- 第四行：Team 管理入口 (仅当有 auth_session 时显示) -->
        <template v-if="token.auth_session">
          <div class="flex justify-start items-center gap-2.5 flex-wrap mt-1.5">
            <button
              class="btn btn--secondary btn--sm"
              v-tooltip="$t('tokenCard.manageTeam')"
              @click="showTeamManageModal = true"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
              </svg>
              <span>{{ $t('tokenCard.team') }}</span>
            </button>
            <button
              class="btn btn--secondary btn--sm"
              v-tooltip="$t('credit.viewUsage')"
              @click="showCreditUsageModal = true"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3 3h2v18H3V3zm16 8h2v10h-2V11zm-8 4h2v6h-2v-6zm4-8h2v14h-2V7zm-8 6h2v8H7v-8z"/>
              </svg>
              <span>{{ $t('credit.viewUsage') }}</span>
            </button>
          </div>
        </template>
      </div>

      <div class="flex flex-row gap-1.5 justify-end mt-auto flex-wrap relative md:gap-1.5 max-[480px]:gap-1 max-[480px]:justify-center">
        <button @click="openEditorModal" class="btn btn--icon btn--secondary" v-tooltip="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" :alt="$t('tokenCard.selectEditor')" width="18" height="18" />
        </button>
        <button @click="exportTokenAsJson" class="btn btn--secondary btn--icon" v-tooltip="$t('tokenCard.exportJson')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>
        <FloatingDropdown ref="copyMenuRef" placement="bottom-end" :close-on-select="false" @click.stop>
          <template #trigger>
            <button class="btn btn--icon btn--secondary" v-tooltip="$t('tokenCard.copyMenu')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
            </button>
          </template>
          <template #default="{ close }">
            <button @click="handleCopyMenuClick('token', close)" class="dropdown-item">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('tokenCard.copyToken') }}</span>
            </button>
            <button @click="handleCopyMenuClick('url', close)" class="dropdown-item">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
              </svg>
              <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
            </button>
            <button v-if="token.portal_url" @click="handleCopyMenuClick('portal', close)" class="dropdown-item">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
              </svg>
              <span>{{ $t('tokenCard.copyPortalUrl') }}</span>
            </button>
            <button v-if="token.auth_session" @click="handleCopyMenuClick('session', close)" class="dropdown-item">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
              </svg>
              <span>{{ $t('tokenCard.copyAuthSession') }}</span>
            </button>
            <button v-if="token.auth_session" @click="handleCopyMenuClick('payment', close)" class="dropdown-item" :disabled="isFetchingPaymentLink">
              <svg v-if="!isFetchingPaymentLink" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.9-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
              </svg>
              <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
            </button>
          </template>
        </FloatingDropdown>
        <!-- 刷新按钮（左键刷新，右键菜单） -->
        <FloatingDropdown ref="checkMenuRef" placement="bottom-end" :close-on-select="true" @click.stop>
          <template #trigger>
            <button
              @click.stop="handleCheckAccountStatus"
              @contextmenu.prevent.stop="checkMenuRef?.toggle()"
              :class="['btn btn--icon btn--secondary', { disabled: token.skip_check }]"
              :disabled="isRefreshingOrLoading"
              v-tooltip="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
            >
              <svg v-if="!isRefreshingOrLoading && !token.skip_check" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              <svg v-else-if="!isRefreshingOrLoading && token.skip_check" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
              </svg>
              <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
            </button>
          </template>
          <template #default="{ close }">
            <button @click="handleToggleSkipCheck(close)" class="dropdown-item">
              <svg v-if="!token.skip_check" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
              </svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
              <span>{{ token.skip_check ? $t('tokenCard.enableCheck') : $t('tokenCard.disableCheck') }}</span>
            </button>
            <button
              v-if="token.auth_session"
              @click="handleRefreshSession(close)"
              :class="['dropdown-item', { disabled: isRefreshingSession }]"
              :disabled="isRefreshingSession"
            >
              <svg v-if="!isRefreshingSession" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
              </svg>
              <span v-else class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
              <span>{{ $t('tokenCard.refreshSession') }}</span>
            </button>
          </template>
        </FloatingDropdown>
        <button v-if="token.portal_url" @click="showPortalDialog = true" class="btn btn--icon btn--secondary" v-tooltip="$t('tokenCard.openPortal')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>
        <button @click="$emit('edit', token)" class="btn btn--icon btn--secondary" v-tooltip="$t('tokenCard.editToken')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>
        <button @click="deleteToken" class="btn btn--icon btn--secondary text-danger hover:bg-danger-muted" v-tooltip="$t('tokenCard.deleteToken')">
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

  <!-- Portal 对话框 -->
  <ExternalLinkDialog
    :show="showPortalDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="token.portal_url || ''"
    :browser-title="getPortalBrowserTitle()"
    @close="showPortalDialog = false"
  />

  <!-- Suspensions 详情模态框 -->
  <SuspensionsModal
    v-if="showSuspensionsModal"
    :suspensions="token.suspensions"
    @close="showSuspensionsModal = false"
  />

  <!-- Credit Usage Modal -->
  <CreditUsageModal
    v-if="showCreditUsageModal && token.auth_session"
    :auth-session="token.auth_session"
    :credits-balance="remainingCredits"
    :has-portal-url="!!token.portal_url"
    @close="showCreditUsageModal = false"
    @refresh-balance="handleCreditUsageRefresh"
    @update-portal-url="handleUpdatePortalUrl"
  />

  <!-- 标签编辑模态框 -->
  <TagEditorModal
    v-model:visible="showTagEditor"
    :token="token"
    :all-tokens="allTokens"
    @save="handleTagSave"
    @clear="handleTagClear"
  />

  <!-- Team 管理模态框 -->
  <TeamManageModal
    v-if="showTeamManageModal && token.auth_session"
    :auth-session="token.auth_session"
    @close="showTeamManageModal = false"
  />
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import QuotaBar from '../common/QuotaBar.vue'
import CreditUsageModal from '../credit/CreditUsageModal.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'
import TeamManageModal from '../team/TeamManageModal.vue'
import SuspensionsModal from './SuspensionsModal.vue'
import FloatingDropdown from '../common/FloatingDropdown.vue'
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
const checkMenuRef = ref(null)
const showSuspensionsModal = ref(false)
const copyMenuRef = ref(null)
const showCreditUsageModal = ref(false)
const showTeamManageModal = ref(false)
const isRefreshingSession = ref(false)

// 标签编辑相关
const showTagEditor = ref(false)

// 是否正在刷新或加载中（提取重复条件）
const isRefreshingOrLoading = computed(() => {
  return isCheckingStatus.value ||
    (props.isBatchChecking && !props.token.skip_check) ||
    (props.isSelectedRefreshing && props.isSelected && !props.token.skip_check)
})

// (theme observer removed; not used in this component)

// 判断是否为封禁状态且有 suspensions 数据
const isBannedWithSuspensions = computed(() => {
  return (
    props.token.ban_status === 'SUSPENDED' &&
    props.token.suspensions &&
    (Array.isArray(props.token.suspensions) ? props.token.suspensions.length > 0 : true)
  )
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
    return 'badge--danger'
  }

  return 'badge--success'
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



// 处理复制菜单项点击
const handleCopyMenuClick = async (type, close) => {
  // payment 类型不立即关闭菜单，等异步操作完成后再关闭
  if (type !== 'payment') {
    close()
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
        onMenuClose: () => { close() }
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
    if (copyMenuRef.value?.isOpen) {
      copyMenuRef.value.close()
    }
  }
}

// 打开编辑器模态框
const openEditorModal = () => {
  if (showEditorModal.value) return
  showEditorModal.value = true
}


// 本地包装的 toggleSkipCheck，需要关闭菜单
const handleToggleSkipCheck = (close) => {
  toggleSkipCheck()
  close?.()
}

// 刷新单个 session
const handleRefreshSession = async (close) => {
  if (!props.token.auth_session) {
    window.$notify.warning(t('messages.noAuthSession'))
    close?.()
    return
  }

  isRefreshingSession.value = true
  close?.()

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
