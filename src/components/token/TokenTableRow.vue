<template>
  <tr
    :class="[
      'transition-colors duration-200 group cursor-pointer',
      isSelected ? 'bg-accent/6' : 'bg-surface',
      'hover:bg-accent/6',
      { 'ring-2 ring-warning ring-offset-1 bg-warning/10': isHighlighted }
    ]"
    @click="handleRowClick"
  >
    <!-- 多选框 -->
    <td class="w-11 text-center py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit first:relative">
      <div
        class="inline-flex items-center justify-center h-5 cursor-pointer align-middle leading-none"
        @click.stop="toggleSelection"
      >
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
        </div>
      </div>
    </td>

    <!-- Tag -->
    <td class="w-[85px] max-w-[85px] overflow-hidden py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <span
        v-if="!hasTag"
        class="btn btn--icon-sm btn--dashed"
        v-tooltip="$t('tokenList.clickToAddTag')"
        @click.stop="openTagEditor"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </span>
      <span
        v-else
        class="badge editable max-w-[75px]"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ tagDisplayName }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="max-w-[200px] py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <div
        v-if="token.email_note"
        class="text-copyable"
        @click.stop="copyEmailNote"
        v-tooltip="token.email_note"
      >
        <span class="text-copyable__content">{{ showRealEmail ? token.email_note : maskedEmail }}</span>
      </div>
      <span v-else class="text-copyable--muted">-</span>
    </td>

    <!-- 额度（合并封禁/失效状态） -->
    <td class="min-w-[200px] py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <!-- SUSPENDED -->
      <div
        v-if="token.ban_status === 'SUSPENDED'"
        :class="[
          'inline-flex items-center gap-1.5 rounded bg-danger/10 px-2 py-1.5 text-xs text-danger',
          { 'cursor-pointer': isBannedWithSuspensions }
        ]"
        v-tooltip="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
        @click.stop="handleStatusClick"
      >
        <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
        </svg>
        <span>{{ $t('tokenCard.banned') }}</span>
        <span v-if="isBannedWithSuspensions" class="w-1.5 h-1.5 rounded-full bg-danger shrink-0"></span>
      </div>
      <!-- INVALID_TOKEN -->
      <div
        v-else-if="token.ban_status === 'INVALID_TOKEN'"
        class="inline-flex items-center gap-1.5 rounded bg-warning/10 px-2 py-1.5 text-xs text-warning"
      >
        <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
        </svg>
        <span>{{ $t('tokenCard.tokenInvalid') }}</span>
      </div>
      <!-- 正常额度 -->
      <template v-else-if="showBalanceRow">
        <div v-if="showProgressBar" class="flex items-center gap-2">
          <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
            <div
              class="h-full rounded transition-all"
              :class="getQuotaBarClass(balancePercentage)"
              :style="{ width: balancePercentage + '%' }"
            ></div>
          </div>
          <span class="text-[11px] font-medium tabular-nums text-text-muted shrink-0">
            {{ portalInfo.data.credits_balance }}/{{ portalInfo.data.credit_total }}
          </span>
        </div>
        <span v-else class="badge" :class="balanceClasses">{{ balanceDisplay }}</span>
      </template>
      <span v-else class="text-text-muted">-</span>
    </td>

    <!-- 创建/重置时间 -->
    <td class="w-[140px] min-w-[140px] py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <div class="flex flex-col gap-1">
        <span class="text-meta" v-tooltip="$t('tokenCard.createdAt') + ': ' + formatDate(token.created_at)">
          C: {{ formatDate(token.created_at) }}
        </span>
        <span
          v-if="expiryDate"
          class="text-meta"
          v-tooltip="$t('tokenCard.resetAt') + ': ' + formatDate(expiryDate)"
        >
          R: {{ formatDate(expiryDate) }}
        </span>
        <!-- session_updated_at 已注释
        <span
          v-if="token.session_updated_at"
          class="text-meta"
          v-tooltip="$t('tokenCard.sessionUpdatedAt') + ': ' + formatDate(token.session_updated_at)"
        >
          S: {{ formatDate(token.session_updated_at) }}
        </span>
        -->
      </div>
    </td>

    <!-- 操作 -->
    <td class="w-[140px] min-w-[140px] py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <div class="flex items-center gap-1.5">
        <!-- 编辑器选择 -->
        <button @click.stop="showEditorModal = true" class="btn btn--icon btn--ghost" v-tooltip="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" alt="Editor" width="16" height="16" />
        </button>

        <!-- 刷新状态 -->
        <button
          @click.stop="handleCheckAccountStatus"
          :class="['btn btn--icon btn--ghost', { disabled: token.skip_check }]"
          :disabled="isRefreshingOrLoading"
          v-tooltip="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
        >
          <svg v-if="!isRefreshingOrLoading && !token.skip_check" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
          <svg v-else-if="!isRefreshingOrLoading && token.skip_check" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
          </svg>
          <span v-else class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
        </button>

        <!-- 更多操作 (FloatingDropdown) -->
        <FloatingDropdown
          ref="menuRef"
          placement="bottom-end"
          :close-on-select="false"
          @open="isMenuOpen = true"
          @close="isMenuOpen = false"
        >
          <template #trigger>
            <button
              class="btn btn--icon btn--ghost"
              v-tooltip="$t('app.moreOptions')"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
              </svg>
            </button>
          </template>
          <template #default="{ close }">
            <!-- 导出 JSON -->
            <button @click="exportTokenAsJson(); close()" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
              </svg>
              <span>{{ $t('tokenCard.exportJson') }}</span>
            </button>
            <!-- 复制 Token -->
            <button @click="copyToken(); close()" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('tokenCard.copyToken') }}</span>
            </button>
            <!-- 复制 Tenant URL -->
            <button @click="copyTenantUrl(); close()" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
              </svg>
              <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
            </button>
            <!-- 复制 Portal URL -->
            <button v-if="token.portal_url" @click="copyPortalUrl(); close()" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
              </svg>
              <span>{{ $t('tokenCard.copyPortalUrl') }}</span>
            </button>
            <!-- session 相关菜单项已注释
            <button v-if="token.auth_session" @click="copyAuthSession(); close()" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
              </svg>
              <span>{{ $t('tokenCard.copyAuthSession') }}</span>
            </button>
            <button v-if="token.auth_session" @click="handlePaymentLinkClick(close)" class="dropdown-item" :disabled="isFetchingPaymentLink">
              <svg v-if="!isFetchingPaymentLink" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.9-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
              </svg>
              <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
            </button>
            -->
            <!-- 禁用/启用检测 -->
            <button @click="handleToggleSkipCheck(close)" class="dropdown-item">
              <svg v-if="!token.skip_check" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
              <span>{{ token.skip_check ? $t('tokenCard.enableCheck') : $t('tokenCard.disableCheck') }}</span>
            </button>
            <!-- session 相关：刷新 Session 已注释
            <button
              v-if="token.auth_session"
              @click="handleRefreshSession(close)"
              :class="['dropdown-item', { disabled: isRefreshingSession }]"
              :disabled="isRefreshingSession"
            >
              <svg v-if="!isRefreshingSession" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
              </svg>
              <span v-else class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
              <span>{{ $t('tokenCard.refreshSession') }}</span>
            </button>
            -->
            <!-- 打开 Portal -->
            <button v-if="token.portal_url" @click="showPortalDialog = true; close()" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
              </svg>
              <span>{{ $t('tokenCard.openPortal') }}</span>
            </button>
            <!-- 删除 -->
            <button @click="deleteToken(); close()" class="dropdown-item text-danger hover:bg-danger/10">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
              <span>{{ $t('tokenCard.deleteToken') }}</span>
            </button>
          </template>
        </FloatingDropdown>
      </div>
    </td>
  </tr>

  <!-- 编辑器选择模态框 -->
  <EditorSelectModal
    :show="showEditorModal"
    :token="token"
    @close="showEditorModal = false"
    @success="(msg) => window.$notify.success(msg)"
    @error="(msg) => window.$notify.error(msg)"
  />

  <!-- 标签编辑模态框 -->
  <TagEditorModal
    v-model:visible="showTagEditor"
    :token="token"
    :all-tokens="allTokens"
    @save="handleTagSave"
    @clear="handleTagClear"
  />

  <ExternalLinkDialog
    :show="showPortalDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="token.portal_url || ''"
    :browser-title="getPortalBrowserTitle()"
    @close="showPortalDialog = false"
  />

  <SuspensionsModal
    v-if="showSuspensionsModal"
    :suspensions="token.suspensions"
    @close="showSuspensionsModal = false"
  />
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTokenActions } from '@/composables/useTokenActions'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import FloatingDropdown from '../common/FloatingDropdown.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'
import SuspensionsModal from './SuspensionsModal.vue'

const { t } = useI18n()

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

const emit = defineEmits([
  'delete',
  'edit',
  'token-updated',
  'select',
  'open-editor',
  'open-portal',
  'payment-link-fetched',
  'edit-tag'
])

const {
  tagDisplayName,
  hasTag,
  tagBadgeStyle,
  maskedEmail,
  formatDate,
  getStatusClass,
  getStatusText,
  copyToken,
  copyTenantUrl,
  copyEmailNote,
  copyPortalUrl,
  copyAuthSession,
  exportTokenAsJson,
  deleteToken,
  editToken,
  toggleSelection,
  toggleSkipCheck,
  isCheckingStatus,
  isFetchingPaymentLink,
  handleTagSave,
  handleTagClear,
  checkAccountStatus,
  copyPaymentMethodLink,
  getPortalBrowserTitle,
  portalInfo
} = useTokenActions(props, emit)

// 本地 UI 状态
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const showTagEditor = ref(false)
const showSuspensionsModal = ref(false)
const menuRef = ref(null)
const isMenuOpen = ref(false)
/* session 相关已注释
const isRefreshingSession = ref(false)
*/

// 计算属性 —— 与 Card 一致的额度逻辑
const expiryDate = computed(() => {
  return portalInfo.value?.data?.expiry_date || null
})

const isBannedWithSuspensions = computed(() => {
  return (
    props.token.ban_status === 'SUSPENDED' &&
    props.token.suspensions &&
    (Array.isArray(props.token.suspensions) ? props.token.suspensions.length > 0 : true)
  )
})

const isRefreshingOrLoading = computed(() => {
  return isCheckingStatus.value ||
    (props.isBatchChecking && !props.token.skip_check) ||
    (props.isSelectedRefreshing && props.isSelected && !props.token.skip_check)
})

const showBalanceRow = computed(() => {
  const hasSource = props.token.portal_url || portalInfo.value?.data
  const notDisabled = props.token.ban_status !== 'SUSPENDED' && props.token.ban_status !== 'INVALID_TOKEN'
  const hasContent = showProgressBar.value || (balanceDisplay.value && balanceDisplay.value.length > 0)
  return hasSource && notDisabled && hasContent
})

const showProgressBar = computed(() => {
  if (!portalInfo.value?.data) return false
  if (props.token.ban_status === 'EXPIRED') return false
  const balance = portalInfo.value.data.credits_balance
  const total = portalInfo.value.data.credit_total
  return typeof balance === 'number' && typeof total === 'number'
})

const balancePercentage = computed(() => {
  if (!portalInfo.value?.data) return 0
  const balance = portalInfo.value.data.credits_balance
  const total = portalInfo.value.data.credit_total
  if (!total || total <= 0) return 0
  return Math.round((balance / total) * 100)
})

const getQuotaBarClass = (percent) => {
  if (percent < 20) return 'bg-danger'
  if (percent < 50) return 'bg-warning'
  return 'bg-success'
}

const balanceClasses = computed(() => {
  const hasError = portalInfo.value?.error
  const exhausted = props.token.ban_status === 'EXPIRED'
  if (hasError || exhausted) return 'badge--danger'
  return 'badge--success'
})

const balanceDisplay = computed(() => {
  if (!portalInfo.value) return ''
  if (portalInfo.value.error) return t('tokenCard.networkError')
  if (!portalInfo.value.data) return ''
  const status = props.token.ban_status
  if (status === 'EXPIRED') return t('tokenCard.expired')
  const balance = portalInfo.value.data.credits_balance
  const total = portalInfo.value.data.credit_total
  if (total !== undefined && total > 0) return `${balance} / ${total}`
  return `${t('tokenCard.balance')}: ${balance}`
})

// 事件处理
const handleStatusClick = () => {
  if (isBannedWithSuspensions.value) {
    showSuspensionsModal.value = true
  }
}

const handleToggleSkipCheck = (close) => {
  toggleSkipCheck()
  close?.()
}

const handleCheckAccountStatus = async () => {
  await checkAccountStatus({
    isBatchChecking: props.isBatchChecking,
    isSelectedRefreshing: props.isSelectedRefreshing,
    isSelected: props.isSelected
  })
}

const handleRowClick = (event) => {
  if (event.target.closest('button, a, .btn, .dropdown-item, .badge, .checkbox-inner, [data-dropdown-trigger], input, label')) return
  emit('edit', props.token)
}

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    if (showEditorModal.value) showEditorModal.value = false
    if (menuRef.value?.isOpen) menuRef.value.close()
  }
}

/* session 相关已注释
const handlePaymentLinkClick = async (close) => {
  await copyPaymentMethodLink({
    cachedPaymentLink: props.cachedPaymentLink,
    onMenuClose: () => { close() }
  })
}

const handleRefreshSession = async (close) => {
  if (!props.token.auth_session) {
    window.$notify.warning(t('messages.noAuthSession'))
    close?.()
    return
  }

  isRefreshingSession.value = true
  close?.()

  try {
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
        props.token.auth_session = result.new_session
        props.token.session_updated_at = now
        props.token.updated_at = now
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
*/

const openTagEditor = () => {
  showTagEditor.value = true
}

// 初始化 portalInfo
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

onMounted(() => {
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
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

defineExpose({
  refreshAccountStatus: handleCheckAccountStatus
})
</script>
