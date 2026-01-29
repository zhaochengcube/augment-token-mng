<template>
  <tr
    :class="[
      'transition-colors duration-200 group',
      isSelected ? 'bg-accent/6' : 'bg-surface',
      'hover:bg-accent/6',
      { 'ring-2 ring-warning ring-offset-1 bg-warning/10': isHighlighted }
    ]"
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
      <!-- 添加标签按钮（无标签时显示） -->
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
      <!-- 标签（有标签时显示，可点击编辑） -->
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

    <!-- 状态 -->
    <td class="w-[75px] py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <span
        :class="['badge', getStatusClass(token.ban_status), { editable: isBannedWithSuspensions }]"
        v-tooltip="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
        @click.stop="handleStatusClick"
      >
        <span :class="['status-dot', getStatusClass(token.ban_status)]"></span>
        {{ getStatusText(token.ban_status) }}
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

    <!-- 剩余次数 -->
    <td class="w-[85px] text-center py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <span class="badge" :class="balanceClasses">
        {{ balanceDisplay }}
      </span>
    </td>

    <!-- 创建/重置/Session更新时间 -->
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
        <span
          v-if="token.session_updated_at"
          class="text-meta"
          v-tooltip="$t('tokenCard.sessionUpdatedAt') + ': ' + formatDate(token.session_updated_at)"
        >
          S: {{ formatDate(token.session_updated_at) }}
        </span>
      </div>
    </td>

    <!-- 操作 -->
    <td class="w-[230px] min-w-[230px] py-3.5 px-2.5 border-b border-border/50 align-middle whitespace-nowrap bg-inherit">
      <div class="flex items-center gap-1.5">
        <!-- 编辑器选择 -->
        <button @click.stop="showEditorModal = true" class="btn btn--icon btn--ghost" v-tooltip="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" alt="Editor" width="16" height="16" />
        </button>

        <!-- 导出JSON -->
        <button @click.stop="exportTokenAsJson" class="btn btn--icon btn--ghost" v-tooltip="$t('tokenCard.exportJson')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>

        <!-- 复制菜单 -->
        <div class="relative" @click.stop>
          <button @click.stop="toggleCopyMenu" class="btn btn--icon btn--ghost" v-tooltip="$t('tokenCard.copyMenu')">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
          <Transition name="dropdown">
            <div v-if="showCopyMenu" class="dropdown-menu" @click.stop>
              <button @click="handleCopyMenuClick('token')" class="dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
                <span>{{ $t('tokenCard.copyToken') }}</span>
              </button>
              <button @click="handleCopyMenuClick('url')" class="dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
                </svg>
                <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
              </button>
              <button v-if="token.portal_url" @click="handleCopyMenuClick('portal')" class="dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
                </svg>
                <span>{{ $t('tokenCard.copyPortalUrl') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('session')" class="dropdown-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                </svg>
                <span>{{ $t('tokenCard.copyAuthSession') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('payment')" class="dropdown-item" :disabled="isFetchingPaymentLink">
                <span v-if="isFetchingPaymentLink" class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
                </svg>
                <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
              </button>
            </div>
          </Transition>
        </div>

        <!-- 刷新状态 -->
        <div class="dropdown" @click.stop>
          <button
            @click.stop="handleCheckAccountStatus"
            @contextmenu.prevent.stop="showCheckMenu = !showCheckMenu"
            :class="['btn btn--icon btn--ghost', { disabled: token.skip_check }]"
            :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check) || token.skip_check"
            v-tooltip="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
          >
            <svg v-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check) && !token.skip_check" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <svg v-else-if="token.skip_check" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
            <span v-else class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
          </button>
          <Transition name="dropdown">
            <div v-if="showCheckMenu" class="dropdown-menu" @click.stop>
              <button @click="handleToggleSkipCheck" class="dropdown-item">
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
                @click="handleRefreshSession"
                :class="['dropdown-item', { disabled: isRefreshingSession }]"
                :disabled="isRefreshingSession"
              >
                <svg v-if="!isRefreshingSession" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                </svg>
                <span v-else class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
                <span>{{ $t('tokenCard.refreshSession') }}</span>
              </button>
            </div>
          </Transition>
        </div>

        <!-- Portal -->
        <button
          v-if="token.portal_url"
          @click.stop="showPortalDialog = true"
          class="btn btn--icon btn--ghost"
          v-tooltip="$t('tokenCard.openPortal')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>

        <!-- 编辑 -->
        <button @click.stop="editToken" class="btn btn--icon btn--ghost" v-tooltip="$t('tokenCard.editToken')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>

        <!-- 删除 -->
        <button @click.stop="deleteToken" class="btn btn--icon btn--ghost text-danger hover:bg-danger-muted" v-tooltip="$t('tokenCard.deleteToken')">
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
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useTokenActions } from '@/composables/useTokenActions'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'
import SuspensionsModal from './SuspensionsModal.vue'

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
  toggleSkipCheck,
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
const showCheckMenu = ref(false)
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const showTagEditor = ref(false)
const showSuspensionsModal = ref(false)
const isRefreshingSession = ref(false)

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

const balanceClasses = computed(() => {
  const hasError = !props.token.portal_info
  const exhausted = (
    props.token.ban_status === 'EXPIRED'
    // 移除 SUSPENDED 的判断，让封禁账号也能正常显示颜色
    // || props.token.ban_status === 'SUSPENDED'
  )

  if (hasError || exhausted) {
    return 'badge--danger'
  }

  return 'badge--success'
})

const balanceDisplay = computed(() => {
  if (!props.token.portal_info) return '-'

  const status = props.token.ban_status
  if (status === 'EXPIRED') return t('tokenCard.expired')
  // 封禁状态也显示额度信息（如果有的话）
  // if (status === 'SUSPENDED') return t('tokenCard.banned')

  const credits = props.token.portal_info.credits_balance
  return credits !== undefined ? credits : '-'
})

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

// 处理状态标签点击
const handleStatusClick = () => {
  if (isBannedWithSuspensions.value) {
    showSuspensionsModal.value = true
  }
}

// 全局点击监听，点击菜单外部关闭
const handleGlobalClick = (event) => {
  // 检查是否点击了复制按钮或菜单
  const clickedCopyButton = event.target.closest('.copy-menu-wrapper .btn-icon')
  const clickedCopyDropdown = event.target.closest('.copy-dropdown')

  // 检查是否点击了检测按钮或菜单
  const clickedCheckButton = event.target.closest('.check-menu-wrapper .btn-icon')
  const clickedCheckDropdown = event.target.closest('.check-dropdown')

  // 如果没有点击复制相关元素，关闭复制菜单
  if (showCopyMenu.value && !clickedCopyButton && !clickedCopyDropdown) {
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
    // 使用 setTimeout 确保在当前点击事件处理完后再添加监听器
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

// 监听复制菜单打开，添加全局点击监听
watch(showCopyMenu, (isOpen) => {
  if (isOpen) {
    // 添加全局点击监听
    addGlobalClickListener()
  } else {
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

onMounted(async () => {
  // 不再需要在这里添加监听器，由 watch 处理
})

onUnmounted(() => {
  // 清理全局监听器
  if (globalClickListenerAdded.value) {
    document.removeEventListener('click', handleGlobalClick, true)
  }
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
