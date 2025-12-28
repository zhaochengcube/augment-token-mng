<template>
  <tr 
    :class="['token-table-row', { 'selected': isSelected, 'highlighted': isHighlighted }]"
    @click="handleRowClick"
  >
    <!-- 多选框 -->
    <td class="cell-checkbox">
      <div 
        class="checkbox-wrapper"
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
    <td class="cell-tag">
      <div class="tag-cell-wrapper" @click.stop="openTagEditor">
        <span
          v-if="hasTag"
          class="status-badge editable"
          :style="tagBadgeStyle"
          v-tooltip="$t('tokenList.clickToEditTag')"
        >
          {{ tagDisplayName }}
        </span>
        <span v-else class="no-tag add-tag" v-tooltip="$t('tokenList.clickToAddTag')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
        </span>
      </div>
    </td>

    <!-- 状态 -->
    <td class="cell-status">
      <span
        :class="['status-badge', getStatusClass(token.ban_status), { clickable: isBannedWithSuspensions }]"
        v-tooltip="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
        @click.stop="handleStatusClick"
      >
        <span :class="['status-dot', getStatusClass(token.ban_status)]"></span>
        {{ getStatusText(token.ban_status) }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="cell-email">
      <div
        v-if="token.email_note"
        class="email-wrapper"
        @click.stop="copyEmailNote"
        v-tooltip="token.email_note"
      >
        <span class="email-text">{{ showRealEmail ? token.email_note : maskedEmail }}</span>
      </div>
      <span v-else class="no-email">-</span>
    </td>

    <!-- 剩余次数 -->
    <td class="cell-balance">
      <span :class="balanceClasses">
        {{ balanceDisplay }}
      </span>
    </td>

    <!-- 创建/重置/Session更新时间 -->
    <td class="cell-dates">
      <div class="dates-wrapper">
        <Tooltip :content="$t('tokenCard.createdAt') + ': ' + formatDate(token.created_at)">
          <span class="date-text created">C: {{ formatDate(token.created_at) }}</span>
        </Tooltip>
        <Tooltip v-if="expiryDate" :content="$t('tokenCard.resetAt') + ': ' + formatDate(expiryDate)">
          <span class="date-text reset">R: {{ formatDate(expiryDate) }}</span>
        </Tooltip>
        <Tooltip v-if="token.session_updated_at" :content="$t('tokenCard.sessionUpdatedAt') + ': ' + formatDate(token.session_updated_at)">
          <span class="date-text session">S: {{ formatDate(token.session_updated_at) }}</span>
        </Tooltip>
      </div>
    </td>

    <!-- 操作 -->
    <td class="cell-actions">
      <div class="actions-wrapper">
        <!-- 编辑器选择 -->
        <button @click.stop="showEditorModal = true" class="btn-icon editor" v-tooltip="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" alt="Editor" width="16" height="16" />
        </button>

        <!-- 导出JSON -->
        <button @click.stop="exportTokenAsJson" class="btn-icon export" v-tooltip="$t('tokenCard.exportJson')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>

        <!-- 复制菜单 -->
        <div class="copy-menu-wrapper" @click.stop>
          <button @click.stop="toggleCopyMenu" class="btn-icon copy" v-tooltip="$t('tokenCard.copyMenu')">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
          <Transition name="dropdown">
            <div v-if="showCopyMenu" class="copy-dropdown" @click.stop>
              <button @click="handleCopyMenuClick('token')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
                <span>{{ $t('tokenCard.copyToken') }}</span>
              </button>
              <button @click="handleCopyMenuClick('url')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
                </svg>
                <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
              </button>
              <button v-if="token.portal_url" @click="handleCopyMenuClick('portal')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
                </svg>
                <span>{{ $t('tokenCard.copyPortalUrl') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('session')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                </svg>
                <span>{{ $t('tokenCard.copyAuthSession') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('payment')" class="copy-menu-item" :disabled="isFetchingPaymentLink">
                <div v-if="isFetchingPaymentLink" class="loading-spinner-small"></div>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
                </svg>
                <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
              </button>
            </div>
          </Transition>
        </div>

        <!-- 刷新状态 -->
        <button
          @click.stop="handleCheckAccountStatus"
          :class="['btn-icon', 'refresh', { 'loading': isCheckingStatus || (isBatchChecking && !token.skip_check) }]"
          :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check) || token.skip_check"
          v-tooltip="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
        >
          <svg v-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
          <div v-else class="loading-spinner"></div>
        </button>

        <!-- Portal -->
        <button
          v-if="token.portal_url"
          @click.stop="showPortalDialog = true"
          class="btn-icon portal"
          v-tooltip="$t('tokenCard.openPortal')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>

        <!-- 编辑 -->
        <button @click.stop="editToken" class="btn-icon edit" v-tooltip="$t('tokenCard.editToken')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>

        <!-- 删除 -->
        <button @click.stop="deleteToken" class="btn-icon delete" v-tooltip="$t('tokenCard.deleteToken')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
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
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useTokenActions } from '@/composables/useTokenActions'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import Tooltip from '../common/Tooltip.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'

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

// 使用共享的 token actions
const {
  tagDisplayName,
  hasTag,
  tagBadgeStyle,
  displayUrl,
  maskedEmail,
  formatDate,
  formatExpiryDate,
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
  // 新增的共享方法
  isCheckingStatus,
  isFetchingPaymentLink,
  handleTagSave,
  handleTagClear,
  checkAccountStatus,
  copyPaymentMethodLink,
  getPortalBrowserTitle
} = useTokenActions(props, emit)

// 本地状态
const showCopyMenu = ref(false)
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const showTagEditor = ref(false)
const showSuspensionsModal = ref(false)

// 计算属性
const expiryDate = computed(() => {
  return props.token.portal_info?.expiry_date || null
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

const balanceClasses = computed(() => {
  const hasError = !props.token.portal_info
  const exhausted = (
    props.token.ban_status === 'EXPIRED' ||
    props.token.ban_status === 'SUSPENDED'
  )

  if (hasError || exhausted) {
    return ['balance-text', 'exhausted']
  }

  const colorMode = props.token.balance_color_mode || 'green'
  return ['balance-text', `color-${colorMode}`]
})

const balanceDisplay = computed(() => {
  if (!props.token.portal_info) return '-'

  const status = props.token.ban_status
  if (status === 'EXPIRED') return t('tokenCard.expired')
  if (status === 'SUSPENDED') return t('tokenCard.banned')
  
  const credits = props.token.portal_info.credits_balance
  return credits !== undefined ? credits : '-'
})

// 方法
const handleRowClick = () => {
  if (showCopyMenu.value) {
    showCopyMenu.value = false
  }
}

const toggleCopyMenu = () => {
  showCopyMenu.value = !showCopyMenu.value
}

const handleCopyMenuClick = async (type) => {
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

const handleCheckAccountStatus = async () => {
  await checkAccountStatus({
    isBatchChecking: props.isBatchChecking
  })
}

// 处理状态标签点击
const handleStatusClick = () => {
  if (isBannedWithSuspensions.value) {
    showSuspensionsModal.value = true
  }
}

// 点击外部关闭菜单
const handleClickOutside = (event) => {
  if (showCopyMenu.value) {
    const copyWrapper = document.querySelector('.copy-menu-wrapper')
    if (copyWrapper && !copyWrapper.contains(event.target)) {
      showCopyMenu.value = false
    }
  }
}

onMounted(async () => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 打开标签编辑器
const openTagEditor = () => {
  showTagEditor.value = true
}

// 暴露方法给父组件
defineExpose({
  refreshAccountStatus: handleCheckAccountStatus
})
</script>

<style scoped>
/* ============================================
   TokenTableRow - Modern Tech Style
   ============================================ */

.token-table-row {
  background: transparent;
  transition: background-color 0.2s ease;
}

.token-table-row:hover {
  background-color: color-mix(in srgb, var(--accent) 6%, transparent);
}

.token-table-row.selected {
  background-color: color-mix(in srgb, var(--accent) 10%, transparent);
}

.token-table-row.highlighted {
  animation: row-tech-highlight 1.2s ease-in-out 2;
}

@keyframes row-tech-highlight {
  0%, 100% {
    background-color: transparent;
  }
  50% {
    background-color: color-mix(in srgb, var(--accent) 15%, transparent);
  }
}

.token-table-row td {
  padding: 14px 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  vertical-align: middle;
  white-space: nowrap;
  background: inherit;
}

/* 第一个单元格添加左侧指示条 */
.token-table-row td:first-child {
  padding-left: 16px;
  position: relative;
}

.token-table-row td:first-child::before {
  content: '';
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: transparent;
  transition: all 0.25s ease;
}

.token-table-row:hover td:first-child::before {
  background: var(--accent);
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

.token-table-row.selected td:first-child::before {
  background: var(--accent);
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.cell-checkbox {
  width: 44px;
  text-align: center;
}

.checkbox-wrapper {
  display: inline-flex;
  cursor: pointer;
}

.checkbox-inner {
  width: 18px;
  height: 18px;
  border-radius: 5px;
  border: 1.5px solid var(--border);
  background: var(--bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.checkbox-inner:hover {
  border-color: var(--accent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
}

.checkbox-inner.checked {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

.cell-tag {
  width: 85px;
  max-width: 85px;
  overflow: hidden;
}

.tag-cell-wrapper {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  min-height: 26px;
  padding: 3px;
  border-radius: 8px;
  transition: all 0.2s ease;
  max-width: 100%;
  overflow: hidden;
}

.tag-cell-wrapper:hover {
  background-color: color-mix(in srgb, var(--accent) 10%, transparent);
}

.tag-cell-wrapper .status-badge {
  max-width: 75px;
}

.no-tag {
  color: var(--text-muted);
}

.no-tag.add-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  border: 1px dashed var(--border);
  transition: all 0.2s ease;
  opacity: 0.4;
}

.token-table-row:hover .no-tag.add-tag {
  opacity: 0.8;
}

.no-tag.add-tag:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
  opacity: 1;
}

.cell-status {
  width: 75px;
}

.cell-email {
  max-width: 200px;
}

.cell-balance {
  width: 85px;
  text-align: center;
}

/* 余额显示 - 等宽字体 */
.balance-text {
  font-size: 12px;
  font-weight: 700;
  font-family: var(--tech-mono-font);
  padding: 4px 10px;
  border-radius: 8px;
  letter-spacing: 0.3px;
}

.balance-text.color-green {
  color: #34d399;
  background: color-mix(in srgb, #10b981 18%, transparent);
  border: 1px solid color-mix(in srgb, #10b981 35%, transparent);
}

.balance-text.color-blue {
  color: #c084fc;
  background: color-mix(in srgb, #a855f7 18%, transparent);
  border: 1px solid color-mix(in srgb, #a855f7 35%, transparent);
}

.balance-text.exhausted {
  color: #fca5a5;
  background: color-mix(in srgb, #ef4444 18%, transparent);
  border: 1px solid color-mix(in srgb, #ef4444 35%, transparent);
}

.cell-dates {
  width: 140px;
  min-width: 140px;
}

.dates-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.date-text {
  font-size: 11px;
  font-family: var(--tech-mono-font);
  color: var(--text-muted);
  opacity: 0.7;
  line-height: 1.2;
}

.date-text.created {
  opacity: 0.8;
}

.date-text.reset {
  opacity: 0.7;
}

.cell-actions {
  width: 230px;
  min-width: 230px;
}

.actions-wrapper {
  display: flex;
  align-items: center;
  gap: 5px;
}

/* 操作按钮 */
.btn-icon {
  width: 30px;
  height: 30px;
  padding: 0;
}

.btn-icon.loading {
  pointer-events: none;
}

.btn-icon img {
  width: 16px;
  height: 16px;
}

/* 下拉菜单 - 磨砂玻璃 */
.copy-menu-wrapper {
  position: relative;
}

.copy-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 6px;
  background: var(--tech-glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  min-width: 190px;
  z-index: 1000;
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
  border-radius: 8px;
}

.copy-menu-item:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.copy-menu-item svg {
  opacity: 0.7;
  transition: opacity 0.2s;
}

.copy-menu-item:hover svg {
  opacity: 1;
}

.copy-menu-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* 加载动画 */
.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: tech-spin 0.8s linear infinite;
}

.loading-spinner-small {
  width: 12px;
  height: 12px;
  border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: tech-spin 0.8s linear infinite;
}

@keyframes tech-spin {
  to { transform: rotate(360deg); }
}

/* 下拉动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.2s ease, transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

/* 状态徽章可点击样式 */
.status-badge.clickable {
  cursor: pointer;
}

.status-badge.clickable:hover {
  filter: brightness(1.1);
}

/* Suspensions 模态框 - 磨砂玻璃效果 */
.suspensions-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 20px;
}

.suspensions-modal {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  max-width: 600px;
  width: 100%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 25px 50px rgba(0, 0, 0, 0.3), var(--tech-border-glow);
}

.suspensions-modal .modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  border-bottom: 1px solid var(--tech-glass-border);
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
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border-radius: 12px;
  padding: 18px;
  border: 1px solid var(--tech-glass-border);
}

.suspension-field {
  margin-bottom: 10px;
}

.suspension-field:last-child {
  margin-bottom: 0;
}

.suspension-field strong {
  color: var(--text-muted);
  font-weight: 500;
  margin-right: 8px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.suspension-value {
  color: var(--text);
  word-break: break-word;
}

.no-suspensions {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
  opacity: 0.7;
}

.raw-json {
  margin-top: 20px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  overflow: hidden;
}

.raw-json summary {
  padding: 14px 18px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  cursor: pointer;
  font-weight: 500;
  color: var(--text-muted);
  transition: all 0.2s ease;
}

.raw-json summary:hover {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent);
}

.raw-json pre {
  margin: 0;
  padding: 18px;
  font-size: 12px;
  font-family: var(--tech-mono-font);
  overflow-x: auto;
  background: color-mix(in srgb, var(--bg-surface) 80%, transparent);
  color: var(--text);
}

/* 模态框动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.25s ease;
}

.modal-enter-active .suspensions-modal,
.modal-leave-active .suspensions-modal {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .suspensions-modal,
.modal-leave-to .suspensions-modal {
  transform: scale(0.92) translateY(10px);
}
</style>
