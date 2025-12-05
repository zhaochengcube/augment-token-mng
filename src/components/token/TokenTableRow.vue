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
          class="tag-badge editable"
          :style="tagBadgeStyle"
          :title="$t('tokenList.clickToEditTag')"
        >
          {{ tagDisplayName }}
        </span>
        <span v-else class="no-tag add-tag" :title="$t('tokenList.clickToAddTag')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
        </span>
      </div>
    </td>

    <!-- 状态 -->
    <td class="cell-status">
      <span 
        :class="['status-badge', getStatusClass(token.ban_status)]"
        :title="token.ban_status === 'SUSPENDED' ? $t('tokenCard.clickToViewDetails') : ''"
      >
        {{ getStatusText(token.ban_status) }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="cell-email">
      <div
        v-if="token.email_note"
        class="email-wrapper"
        @click.stop="copyEmailNote"
        :title="token.email_note"
      >
        <span class="email-text">{{ showRealEmail ? token.email_note : maskedEmail }}</span>
        <span class="copy-hint">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
          </svg>
        </span>
      </div>
      <span v-else class="no-email">-</span>
    </td>

    <!-- 剩余次数 -->
    <td class="cell-balance">
      <span :class="balanceClasses">
        {{ balanceDisplay }}
      </span>
    </td>

    <!-- 过期时间 -->
    <td class="cell-expiry">
      <span v-if="expiryDate" class="expiry-text">
        {{ formatExpiryDate(expiryDate) }}
      </span>
      <span v-else class="no-expiry">-</span>
    </td>

    <!-- 操作 -->
    <td class="cell-actions">
      <div class="actions-wrapper">
        <!-- 编辑器选择 -->
        <button @click.stop="showEditorModal = true" class="action-btn editor" :title="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" alt="Editor" width="16" height="16" />
        </button>
        
        <!-- 导出JSON -->
        <button @click.stop="exportTokenAsJson" class="action-btn export" :title="$t('tokenCard.exportJson')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>

        <!-- 复制菜单 -->
        <div class="copy-menu-wrapper" @click.stop>
          <button @click.stop="toggleCopyMenu" class="action-btn copy" :title="$t('tokenCard.copyMenu')">
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
          @click.stop="checkAccountStatus"
          :class="['action-btn', 'refresh', { 'loading': isCheckingStatus || (isBatchChecking && !token.skip_check) }]"
          :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check) || token.skip_check"
          :title="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
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
          class="action-btn portal" 
          :title="$t('tokenCard.openPortal')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>

        <!-- 编辑 -->
        <button @click.stop="editToken" class="action-btn edit" :title="$t('tokenCard.editToken')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>

        <!-- 删除 -->
        <button @click.stop="deleteToken" class="action-btn delete" :title="$t('tokenCard.deleteToken')">
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

  <!-- Portal 对话框 -->
  <ExternalLinkDialog
    :show="showPortalDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="token.portal_url || ''"
    :browser-title="getPortalBrowserTitle()"
    @close="showPortalDialog = false"
  />
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useTokenActions } from '@/composables/useTokenActions'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
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
  toggleSelection
} = useTokenActions(props, emit)

// 本地状态
const showCopyMenu = ref(false)
const isCheckingStatus = ref(false)
const isFetchingPaymentLink = ref(false)
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const showTagEditor = ref(false)

// 计算属性
const expiryDate = computed(() => {
  return props.token.portal_info?.expiry_date || null
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
      await copyPaymentMethodLink()
      break
  }
}

const copyPaymentMethodLink = async () => {
  if (!props.token.auth_session) {
    window.$notify.warning(t('messages.noAuthSession'))
    showCopyMenu.value = false
    return
  }

  if (props.cachedPaymentLink) {
    try {
      await invoke('copy_to_clipboard', { text: props.cachedPaymentLink })
      window.$notify.success(t('messages.paymentLinkCopied'))
    } catch (error) {
      console.error('Failed to copy cached payment link:', error)
      window.$notify.error(t('messages.copyPaymentLinkFailed') + ': ' + error)
    } finally {
      showCopyMenu.value = false
    }
    return
  }

  isFetchingPaymentLink.value = true
  try {
    const result = await invoke('fetch_payment_method_link_command', {
      authSession: props.token.auth_session
    })

    const paymentLink = result.payment_method_link
    if (!paymentLink) {
      window.$notify.error(t('messages.copyPaymentLinkFailed') + ': 链接为空')
      return
    }

    await invoke('copy_to_clipboard', { text: paymentLink })
    window.$notify.success(t('messages.paymentLinkCopied'))
    emit('payment-link-fetched', { tokenId: props.token.id, link: paymentLink })
  } catch (error) {
    console.error('Failed to fetch or copy payment link:', error)
    window.$notify.error(t('messages.copyPaymentLinkFailed') + ': ' + error)
  } finally {
    isFetchingPaymentLink.value = false
    showCopyMenu.value = false
  }
}

const checkAccountStatus = async () => {
  if (props.token.skip_check) return
  if (isCheckingStatus.value || (props.isBatchChecking && !props.token.skip_check)) return

  isCheckingStatus.value = true

  try {
    const batchResults = await invoke('batch_check_tokens_status', {
      tokens: [{
        id: props.token.id,
        access_token: props.token.access_token,
        tenant_url: props.token.tenant_url,
        portal_url: props.token.portal_url || null,
        auth_session: props.token.auth_session || null,
        email_note: props.token.email_note || null
      }]
    })

    if (batchResults && batchResults.length > 0) {
      const result = batchResults[0]
      const statusResult = result.status_result
      const banStatus = statusResult.status
      let hasChanges = false

      if (props.token.ban_status !== banStatus) {
        props.token.ban_status = banStatus
        hasChanges = true
      }

      if (result.portal_info) {
        props.token.portal_info = {
          credits_balance: result.portal_info.credits_balance,
          expiry_date: result.portal_info.expiry_date
        }
        hasChanges = true
      }

      if (hasChanges) {
        props.token.updated_at = new Date().toISOString()
        emit('token-updated')
      }

      // 显示状态消息
      let statusMessage = ''
      let statusType = 'info'
      
      switch (banStatus) {
        case 'SUSPENDED':
          statusMessage = t('messages.accountBanned')
          statusType = 'error'
          break
        case 'EXPIRED':
          statusMessage = t('tokenCard.expired')
          statusType = 'warning'
          break
        case 'ACTIVE':
          statusMessage = t('messages.accountStatusNormal')
          statusType = 'success'
          break
        default:
          statusMessage = `${t('messages.accountStatus')}: ${banStatus}`
      }

      window.$notify[statusType](`${t('messages.checkComplete')}: ${statusMessage}`)
    }
  } catch (error) {
    window.$notify.error(`${t('messages.checkFailed')}: ${error}`)
  } finally {
    isCheckingStatus.value = false
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

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 打开标签编辑器
const openTagEditor = () => {
  showTagEditor.value = true
}

// 处理标签保存
const handleTagSave = ({ tagName, tagColor }) => {
  props.token.tag_name = tagName
  props.token.tag_color = tagColor
  props.token.updated_at = new Date().toISOString()
  emit('token-updated')
  window.$notify.success(t('messages.tagUpdated'))
}

// 处理标签清除
const handleTagClear = () => {
  props.token.tag_name = ''
  props.token.tag_color = ''
  props.token.updated_at = new Date().toISOString()
  emit('token-updated')
  window.$notify.success(t('messages.tagCleared'))
}

// 获取 Portal 浏览器标题
const getPortalBrowserTitle = () => {
  const email = props.token.email_note || ''
  const tag = props.token.tag_name || ''
  if (email && tag) return `${tag} - ${email}`
  if (email) return email
  if (tag) return tag
  return 'Portal'
}

// 暴露方法给父组件
defineExpose({
  refreshAccountStatus: checkAccountStatus
})
</script>

<style scoped>
.token-table-row {
  /* 不添加 transition 避免快速移动鼠标时的闪烁 */
  background: transparent;
}

.token-table-row:hover {
  background-color: var(--color-surface-hover, #f8fafc);
}

.token-table-row.selected {
  background-color: var(--color-accent-surface, rgba(59, 130, 246, 0.08));
}

.token-table-row.highlighted {
  animation: row-highlight 1s ease-in-out 2;
}

@keyframes row-highlight {
  0%, 100% { background-color: transparent; }
  50% { background-color: rgba(251, 191, 36, 0.2); }
}

.token-table-row td {
  padding: 12px 8px;
  /* 使用透明边框占位，避免边框闪烁 */
  border-bottom: 1px solid var(--color-divider, #e5e7eb);
  vertical-align: middle;
  white-space: nowrap;
  /* 确保单元格背景继承行背景 */
  background: inherit;
}

.cell-checkbox {
  width: 40px;
  text-align: center;
}

.checkbox-wrapper {
  display: inline-flex;
  cursor: pointer;
}

.checkbox-inner {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 2px solid var(--color-divider, #d1d5db);
  background: var(--color-surface, #ffffff);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.checkbox-inner:hover {
  border-color: var(--color-accent, #3b82f6);
}

.checkbox-inner.checked {
  background: var(--color-accent, #3b82f6);
  border-color: var(--color-accent, #3b82f6);
  color: white;
}

.cell-tag {
  width: 100px;
  max-width: 120px;
}

.tag-cell-wrapper {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 2px;
  border-radius: 6px;
  transition: background-color 0.15s ease;
}

.tag-cell-wrapper:hover {
  background-color: var(--color-surface-hover, #f1f5f9);
}

.tag-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.tag-badge.editable:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.no-tag {
  color: var(--color-text-muted, #9ca3af);
}

.no-tag.add-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px dashed var(--color-divider, #d1d5db);
  transition: all 0.15s ease;
  opacity: 0.5;
}

.token-table-row:hover .no-tag.add-tag {
  opacity: 1;
}

.no-tag.add-tag:hover {
  border-color: var(--color-accent, #3b82f6);
  color: var(--color-accent, #3b82f6);
  background: var(--color-accent-surface, rgba(59, 130, 246, 0.1));
}

.cell-status {
  width: 70px;
}

.status-badge {
  display: inline-block;
  font-size: 10px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 10px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.status-badge.active {
  background: var(--color-success-surface, #d4edda);
  color: var(--color-success-text, #155724);
  border: 1px solid var(--color-success-border, #c3e6cb);
}

.status-badge.banned {
  background: var(--color-danger-surface, #f8d7da);
  color: var(--color-danger-text, #721c24);
  border: 1px solid var(--color-danger-border, #f5c6cb);
}

.status-badge.inactive {
  background: var(--color-danger-surface, #f8d7da);
  color: var(--color-danger-text, #721c24);
  border: 1px solid var(--color-danger-border, #f5c6cb);
}

.status-badge.invalid {
  background: var(--color-warning-surface, #fff3cd);
  color: var(--color-warning-text, #856404);
  border: 1px solid var(--color-warning-border, #ffeaa7);
}

.cell-email {
  min-width: 200px;
  max-width: 260px;
}

.email-wrapper {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  width: 180px;
  position: relative;
}

.email-text {
  font-size: 12px;
  color: var(--color-link-visited, #4f46e5);
  background: var(--color-info-surface, #f0f9ff);
  padding: 2px 6px;
  border-radius: 4px;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.copy-hint {
  color: var(--color-text-muted, #6b7280);
  opacity: 0;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.email-wrapper:hover .copy-hint {
  opacity: 1;
}

.no-email {
  color: var(--color-text-muted, #9ca3af);
}

.cell-balance {
  width: 80px;
  text-align: center;
}

.balance-text {
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
}

.balance-text.color-green {
  color: var(--color-success-text, #166534);
  background: var(--color-success-surface, #dcfce7);
}

.balance-text.color-blue {
  color: #db2777;
  background: rgba(236, 72, 153, 0.1);
}

.balance-text.exhausted {
  color: var(--color-danger-text, #dc2626);
  background: var(--color-danger-surface, #fee2e2);
}

.cell-expiry {
  width: 110px;
}

.expiry-text {
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
}

.no-expiry {
  color: var(--color-text-muted, #9ca3af);
}

.cell-actions {
  width: 200px;
}

.actions-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 6px;
  background: var(--color-surface, #ffffff);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  color: var(--color-text-secondary, #6b7280);
}

.action-btn:hover:not(:disabled) {
  border-color: var(--color-accent, #3b82f6);
  background: var(--color-surface-hover, #f8fafc);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.loading {
  pointer-events: none;
}

.action-btn.delete:hover {
  border-color: var(--color-danger, #dc2626);
  color: var(--color-danger, #dc2626);
  background: var(--color-danger-surface, #fef2f2);
}

.action-btn.edit:hover {
  border-color: var(--color-success, #16a34a);
  color: var(--color-success, #16a34a);
  background: var(--color-success-surface, #f0fdf4);
}

.action-btn.portal:hover {
  border-color: var(--color-info, #0ea5e9);
  color: var(--color-info, #0ea5e9);
  background: var(--color-info-surface, #f0f9ff);
}

.action-btn img {
  width: 16px;
  height: 16px;
}

/* 复制菜单 */
.copy-menu-wrapper {
  position: relative;
}

.copy-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  z-index: 1000;
  padding: 4px 0;
}

.copy-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-primary, #374151);
  transition: background-color 0.15s ease;
}

.copy-menu-item:hover:not(:disabled) {
  background: var(--color-surface-hover, #f3f4f6);
}

.copy-menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 加载动画 */
.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-divider, #e5e7eb);
  border-top-color: var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.loading-spinner-small {
  width: 12px;
  height: 12px;
  border: 2px solid var(--color-divider, #e5e7eb);
  border-top-color: var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 下拉动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* 暗黑模式 */
[data-theme='dark'] .token-table-row:hover {
  background-color: rgba(55, 65, 81, 0.4);
}

[data-theme='dark'] .token-table-row.selected {
  background-color: rgba(59, 130, 246, 0.15);
}

[data-theme='dark'] .email-text {
  background: rgba(56, 189, 248, 0.2);
  color: #93c5fd;
}

[data-theme='dark'] .action-btn {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(71, 85, 105, 0.6);
  color: #cbd5e1;
}

[data-theme='dark'] .action-btn:hover:not(:disabled) {
  background: rgba(71, 85, 105, 0.6);
  border-color: rgba(100, 116, 139, 0.7);
}

[data-theme='dark'] .copy-dropdown {
  background: var(--color-surface, #1f2937);
  border-color: rgba(75, 85, 99, 0.6);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .copy-menu-item {
  color: var(--color-text-primary, #e5e7eb);
}

[data-theme='dark'] .copy-menu-item:hover:not(:disabled) {
  background: rgba(55, 65, 81, 0.6);
}

[data-theme='dark'] .balance-text.color-green {
  color: #86efac;
  background: rgba(34, 197, 94, 0.2);
}

[data-theme='dark'] .balance-text.color-blue {
  color: #f9a8d4;
  background: rgba(236, 72, 153, 0.2);
}

[data-theme='dark'] .balance-text.exhausted {
  color: #fca5a5;
  background: rgba(220, 38, 38, 0.2);
}

[data-theme='dark'] .tag-cell-wrapper:hover {
  background-color: rgba(55, 65, 81, 0.5);
}

[data-theme='dark'] .no-tag.add-tag {
  border-color: rgba(75, 85, 99, 0.6);
  color: #9ca3af;
}

[data-theme='dark'] .no-tag.add-tag:hover {
  border-color: var(--color-accent, #3b82f6);
  color: var(--color-accent, #3b82f6);
  background: rgba(59, 130, 246, 0.2);
}

[data-theme='dark'] .email-text {
  color: #93c5fd;
  background: rgba(56, 189, 248, 0.15);
}
</style>
