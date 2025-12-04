import { computed } from 'vue'
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

  return {
    // 常量
    DEFAULT_TAG_COLOR,
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
    // 剪贴板操作
    copyToClipboard,
    copyWithNotification,
    copyToken,
    copyTenantUrl,
    copyEmailNote,
    copyPortalUrl,
    copyAuthSession,
    exportTokenAsJson,
    // 操作方法
    deleteToken,
    editToken,
    toggleSelection
  }
}
