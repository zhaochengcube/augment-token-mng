import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

/**
 * Token 操作的共享逻辑 composable
 * 可被 TokenCard 和 TokenTableRow 复用
 */
export function useTokenActions(props, emit) {
  const { t } = useI18n()

  // ========== 常量定义 ==========
  const DEFAULT_TAG_COLOR = '#f97316'

  // ========== 响应式状态 ==========
  const isCheckingStatus = ref(false)
  const isFetchingPaymentLink = ref(false)
  const portalInfo = ref({ data: null, error: null })

  // ========== 计算属性 ==========
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
    return 'hello@augmentcode.com'
  })

  // ========== 工具方法 ==========
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

  // ========== 剪贴板操作 ==========
  const copyToClipboard = async (text) => {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      try {
        await navigator.clipboard.writeText(text)
        return true
      } catch (error) {
        console.warn('Clipboard API failed, trying fallback:', error)
      }
    }
    
    try {
      const textArea = document.createElement('textarea')
      textArea.value = text
      textArea.style.position = 'fixed'
      textArea.style.top = '0'
      textArea.style.left = '0'
      textArea.style.width = '2em'
      textArea.style.height = '2em'
      textArea.style.padding = '0'
      textArea.style.border = 'none'
      textArea.style.outline = 'none'
      textArea.style.boxShadow = 'none'
      textArea.style.background = 'transparent'
      textArea.style.opacity = '0'
      
      document.body.appendChild(textArea)
      textArea.focus()
      textArea.select()
      textArea.setSelectionRange(0, text.length)
      
      const success = document.execCommand('copy')
      document.body.removeChild(textArea)
      return success
    } catch (e) {
      console.error('All copy methods failed:', e)
      return false
    }
  }

  const copyWithNotification = async (text, successMessage, errorMessage) => {
    const success = await copyToClipboard(text)
    if (success) {
      window.$notify.success(t(successMessage))
    } else {
      window.$notify.error(t(errorMessage))
    }
  }

  const copyToken = () => copyWithNotification(
    props.token.access_token,
    'messages.tokenCopied',
    'messages.copyTokenFailed'
  )

  const copyTenantUrl = () => copyWithNotification(
    props.token.tenant_url,
    'messages.tenantUrlCopied',
    'messages.copyTenantUrlFailed'
  )

  const copyEmailNote = () => copyWithNotification(
    props.token.email_note,
    'messages.emailNoteCopied',
    'messages.copyEmailNoteFailed'
  )

  const copyPortalUrl = () => copyWithNotification(
    props.token.portal_url,
    'messages.portalUrlCopied',
    'messages.copyPortalUrlFailed'
  )

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

  // ========== 工具方法 ==========
  
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

  // ========== 操作方法 ==========
  const deleteToken = () => {
    emit('delete', props.token.id)
  }

  const editToken = () => {
    emit('edit', props.token)
  }

  const toggleSelection = () => {
    emit('select', props.token.id)
  }

  // ========== 标签操作 ==========
  const handleTagSave = ({ tagName, tagColor }) => {
    props.token.tag_name = tagName
    props.token.tag_color = tagColor
    props.token.updated_at = new Date().toISOString()
    emit('token-updated', props.token)
    window.$notify.success(t('messages.tagUpdated'))
  }

  const handleTagClear = () => {
    props.token.tag_name = ''
    props.token.tag_color = ''
    props.token.updated_at = new Date().toISOString()
    emit('token-updated', props.token)
    window.$notify.success(t('messages.tagCleared'))
  }

  // ========== 余额和检测状态操作 ==========
  const toggleBalanceColor = () => {
    // 切换颜色模式：green <-> blue
    const currentMode = props.token.balance_color_mode || 'green'
    props.token.balance_color_mode = currentMode === 'green' ? 'blue' : 'green'
    props.token.updated_at = new Date().toISOString()
    emit('token-updated', props.token)
  }

  const toggleSkipCheck = () => {
    props.token.skip_check = !props.token.skip_check
    props.token.updated_at = new Date().toISOString()
    emit('token-updated', props.token)
    
    const message = props.token.skip_check
      ? t('messages.checkDisabled')
      : t('messages.checkEnabled')
    window.$notify.info(message)
  }

  // ========== 复制绑卡链接 ==========
  const copyPaymentMethodLink = async (options = {}) => {
    const { cachedPaymentLink, onMenuClose } = options

    if (!props.token.auth_session) {
      window.$notify.warning(t('messages.noAuthSession'))
      onMenuClose?.()
      return
    }

    // 如果有缓存的绑卡链接，直接复制
    if (cachedPaymentLink) {
      try {
        await invoke('copy_to_clipboard', { text: cachedPaymentLink })
        window.$notify.success(t('messages.paymentLinkCopied'))
      } catch (error) {
        console.error('Failed to copy cached payment link:', error)
        window.$notify.error(t('messages.copyPaymentLinkFailed') + ': ' + error)
      } finally {
        onMenuClose?.()
      }
      return
    }

    // 没有缓存，需要获取
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
      
      // 通知父组件缓存链接
      emit('payment-link-fetched', { tokenId: props.token.id, link: paymentLink })
    } catch (error) {
      console.error('Failed to fetch or copy payment link:', error)
      window.$notify.error(t('messages.copyPaymentLinkFailed') + ': ' + error)
    } finally {
      isFetchingPaymentLink.value = false
      onMenuClose?.()
    }
  }

  // ========== 检查账号状态 ==========
  const checkAccountStatus = async (options = {}) => {
    const { 
      showNotification = true, 
      isBatchChecking = false,
      isSelectedRefreshing = false,
      isSelected = false
    } = options

    // 如果禁用了检测，静默返回
    if (props.token.skip_check) {
      return { hasChanges: false }
    }

    // 如果正在检测中，或者批量检测中，返回
    if (isCheckingStatus.value || 
        (isBatchChecking && !props.token.skip_check) ||
        (isSelectedRefreshing && isSelected && !props.token.skip_check)) {
      return { hasChanges: false }
    }

    isCheckingStatus.value = true
    let hasChanges = false

    try {
      // 单次API调用同时获取账号状态和Portal信息
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

      let statusMessage = ''
      let statusType = 'info'

      if (batchResults && batchResults.length > 0) {
        const result = batchResults[0]
        const statusResult = result.status_result
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
          const autoDisableMsg = banStatus === 'SUSPENDED'
            ? t('messages.autoDisabledBanned')
            : t('messages.autoDisabledExpired')
          window.$notify.info(autoDisableMsg)
        }

        // 比对并更新 suspensions 信息
        if (result.suspensions) {
          if (!isEqual(props.token.suspensions, result.suspensions)) {
            props.token.suspensions = result.suspensions
            hasChanges = true
          }
        }

        // 比对并更新 Portal 信息
        if (result.portal_info) {
          const newPortalInfo = {
            credits_balance: result.portal_info.credits_balance,
            expiry_date: result.portal_info.expiry_date
          }

          if (!isEqual(props.token.portal_info, newPortalInfo)) {
            props.token.portal_info = newPortalInfo
            hasChanges = true
            portalInfo.value = { data: newPortalInfo, error: null }
          }
        } else if (result.portal_error) {
          if (props.token.portal_info !== null) {
            props.token.portal_info = null
            hasChanges = true
          }
          portalInfo.value = { data: null, error: result.portal_error }
        }

        // 比对并更新 portal_url
        if (result.portal_url && props.token.portal_url !== result.portal_url) {
          props.token.portal_url = result.portal_url
          hasChanges = true
        }

        // 比对并更新 email_note
        if (result.email_note && props.token.email_note !== result.email_note) {
          props.token.email_note = result.email_note
          hasChanges = true
        }

        // 只有在有实际变化时才更新时间戳并触发更新事件
        if (hasChanges) {
          props.token.updated_at = new Date().toISOString()
          emit('token-updated', props.token)
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

      if (showNotification) {
        const finalMessage = `${t('messages.checkComplete')}: ${statusMessage}`
        window.$notify[statusType](finalMessage)
      }

      return { hasChanges, statusType }
    } catch (error) {
      portalInfo.value = { data: null, error: String(error) }

      if (showNotification) {
        window.$notify.error(`${t('messages.checkFailed')}: ${error}`)
      }

      return { hasChanges: false, error }
    } finally {
      isCheckingStatus.value = false
    }
  }

  // 处理更新 portal_url
  const handleUpdatePortalUrl = (portalUrl) => {
    if (!portalUrl || props.token.portal_url === portalUrl) {
      return
    }
    props.token.portal_url = portalUrl
    emit('token-updated', props.token)
  }

  // 获取 Portal 浏览器标题
  const getPortalBrowserTitle = () => {
    if (!props.token) return 'Portal'
    const email = props.token.email_note || ''
    const tag = props.token.tag_name || ''
    if (email && tag) return `${tag} - ${email}`
    if (email) return email
    if (tag) return tag
    try {
      const url = new URL(props.token.tenant_url)
      return `Portal - ${url.hostname}`
    } catch {
      return 'Portal'
    }
  }

  // 初始化 portalInfo（用于组件挂载时）
  const initPortalInfo = () => {
    if (props.token.portal_info) {
      portalInfo.value = {
        data: {
          credits_balance: props.token.portal_info.credits_balance,
          expiry_date: props.token.portal_info.expiry_date
        },
        error: null
      }
    }
  }

  return {
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
    formatDate,
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
    initPortalInfo
  }
}
