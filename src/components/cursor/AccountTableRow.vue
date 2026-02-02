<template>
  <tr
    :class="[
      'group transition-colors duration-200',
      'hover:bg-accent/6',
      isSelected ? 'bg-accent/10' : ''
    ]"
    @click="handleRowClick"
  >
    <!-- 多选框 -->
    <td class="w-11 text-center px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text relative first-cell">
      <div class="inline-flex items-center justify-center h-5 cursor-pointer align-middle leading-none" @click.stop="toggleSelection">
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
        </div>
      </div>
    </td>

    <!-- 标签 -->
    <td class="w-[60px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
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
        class="badge editable badge--sm max-w-[50px]"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ tagDisplayName }}
      </span>
    </td>

    <!-- 状态 -->
    <td class="w-[60px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <span :class="['badge badge--sm', statusBadgeClass]">
        <span class="status-dot" :class="statusDotClass"></span>
        {{ statusLabel }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div class="text-copyable" @click.stop="copyEmail" v-tooltip="account.email">
        <span class="text-copyable__content">{{ showRealEmail ? account.email : maskedEmail }}</span>
      </div>
    </td>

    <!-- 过期时间（合并显示） -->
    <td class="w-[120px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[12px] text-text-muted">
      <div class="flex flex-col gap-0.5">
        <div class="flex items-center gap-1" v-tooltip="$t('platform.cursor.accessTokenExpiry')">
          <span class="text-text-muted/60">A:</span>
          <span>{{ account.token?.expiry_timestamp ? formatDate(account.token.expiry_timestamp) : '-' }}</span>
        </div>
        <div class="flex items-center gap-1" v-tooltip="$t('platform.cursor.sessionExpiry')">
          <span class="text-text-muted/60">S:</span>
          <span>{{ account.token?.session_expiry_timestamp ? formatDate(account.token.session_expiry_timestamp) : '-' }}</span>
        </div>
      </div>
    </td>

    <!-- 配额 -->
    <td class="w-[80px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <button
        @click.stop="showUsageModal = true"
        class="inline-flex items-center gap-1 px-2 py-0.5 text-xs text-accent bg-accent/10 border border-accent/30 rounded hover:bg-accent/20 transition-colors cursor-pointer"
      >
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 3h2v18H3V3zm16 8h2v10h-2V11zm-8 4h2v6h-2v-6zm4-8h2v14h-2V7zm-8 6h2v8H7v-8z"/>
        </svg>
        {{ $t('cursorUsage.viewUsage') }}
      </button>
    </td>

    <!-- 操作 -->
    <td class="w-[110px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text text-center">
      <div class="flex items-center justify-center gap-1">
        <button
          v-if="!isCurrent"
          @click.stop="$emit('switch', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isSwitching"
          v-tooltip="$t('platform.cursor.switch')"
        >
          <svg v-if="!isSwitching" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z" />
          </svg>
          <span v-else class="btn-spinner btn-spinner--xs text-accent" aria-hidden="true"></span>
        </button>

        <FloatingDropdown ref="menuRef" placement="bottom-end" :close-on-select="true" @click.stop>
          <template #trigger>
            <button class="btn btn--ghost btn--icon-sm" v-tooltip="$t('app.moreOptions')">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
              </svg>
            </button>
          </template>
          <template #default="{ close }">
            <button @click="handleMenuClick('copyAccessToken', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('accountCard.copyAccessToken') }}</span>
            </button>
            <button @click="handleMenuClick('copySessionToken', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('accountCard.copySessionToken') }}</span>
            </button>
            <button @click="handleMenuClick('delete', close)" class="dropdown-item text-danger hover:bg-danger/10">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
              <span>{{ $t('common.delete') }}</span>
            </button>
          </template>
        </FloatingDropdown>
      </div>
    </td>
  </tr>

  <!-- 标签编辑模态框 -->
  <TagEditorModal
    v-model:visible="showTagEditor"
    :token="accountAsToken"
    :all-tokens="allAccountsAsTokens"
    @save="handleTagSave"
    @clear="handleTagClear"
  />

  <!-- 使用详情模态框 -->
  <CursorUsageModal
    v-if="showUsageModal"
    :account="account"
    @close="showUsageModal = false"
  />
</template>

<script setup>
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import FloatingDropdown from '../common/FloatingDropdown.vue'
import TagEditorModal from '../token/TagEditorModal.vue'
import CursorUsageModal from './CursorUsageModal.vue'

const { t: $t } = useI18n()

const DEFAULT_TAG_COLOR = '#f97316'

const props = defineProps({
  account: { type: Object, required: true },
  isCurrent: { type: Boolean, default: false },
  isSwitching: { type: Boolean, default: false },
  isSelected: { type: Boolean, default: false },
  selectionMode: { type: Boolean, default: false },
  showRealEmail: { type: Boolean, default: true },
  allAccounts: { type: Array, default: () => [] }
})

const emit = defineEmits(['switch', 'delete', 'select', 'account-updated'])

const menuRef = ref(null)
const showTagEditor = ref(false)
const showUsageModal = ref(false)

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@cursor.com'
})

// 标签相关
const tagDisplayName = computed(() => (props.account.tag ?? '').trim())
const hasTag = computed(() => tagDisplayName.value.length > 0)

const tagBadgeStyle = computed(() => {
  if (!hasTag.value) return {}
  return { '--tag-color': props.account.tag_color || DEFAULT_TAG_COLOR }
})

const accountAsToken = computed(() => ({
  tag_name: props.account.tag || '',
  tag_color: props.account.tag_color || ''
}))

const allAccountsAsTokens = computed(() =>
  props.allAccounts.map(acc => ({
    tag_name: acc.tag || '',
    tag_color: acc.tag_color || ''
  }))
)

const openTagEditor = () => {
  showTagEditor.value = true
}

const handleTagSave = ({ tagName, tagColor }) => {
  props.account.tag = tagName
  props.account.tag_color = tagColor
  props.account.updated_at = Math.floor(Date.now() / 1000)
  emit('account-updated', props.account)
  window.$notify?.success($t('messages.tagUpdated'))
}

const handleTagClear = () => {
  props.account.tag = ''
  props.account.tag_color = ''
  props.account.updated_at = Math.floor(Date.now() / 1000)
  emit('account-updated', props.account)
  window.$notify?.success($t('messages.tagCleared'))
}

// 状态
const statusClass = computed(() => {
  if (props.isCurrent) return 'current'
  if (props.account.disabled) return 'disabled'
  return 'available'
})

const statusBadgeClass = computed(() => {
  switch (statusClass.value) {
    case 'current': return 'badge--accent-tech'
    case 'disabled': return 'badge--danger-tech'
    case 'available': return 'badge--success-tech'
    default: return ''
  }
})

const statusDotClass = computed(() => {
  switch (statusClass.value) {
    case 'current': return 'text-accent'
    case 'disabled': return 'text-danger'
    case 'available': return 'text-success'
    default: return ''
  }
})

const statusLabel = computed(() => {
  if (props.isCurrent) return $t('platform.cursor.status.current')
  if (props.account.disabled) return $t('platform.cursor.status.disabled')
  return $t('platform.cursor.status.available')
})

const formatDate = (timestamp) => {
  if (!timestamp) return '-'
  const date = new Date(timestamp * 1000)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
}

const toggleSelection = () => emit('select', props.account.id)

const handleRowClick = () => {
  if (props.selectionMode) toggleSelection()
}

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

// 菜单操作
const handleMenuClick = async (type, close) => {
  close?.()
  switch (type) {
    case 'copyAccessToken':
      await copyAccessToken()
      break
    case 'copySessionToken':
      await copySessionToken()
      break
    case 'delete':
      emit('delete', props.account.id)
      break
  }
}

const copyAccessToken = async () => {
  try {
    const accessToken = props.account.token?.access_token
    if (!accessToken) {
      window.$notify?.error($t('messages.noAccessToken'))
      return
    }
    await navigator.clipboard.writeText(accessToken)
    window.$notify?.success($t('messages.accessTokenCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyFailed'))
  }
}

const copySessionToken = async () => {
  try {
    const sessionToken = props.account.token?.workos_cursor_session_token
    if (!sessionToken) {
      window.$notify?.error($t('messages.noSessionToken'))
      return
    }
    await navigator.clipboard.writeText(sessionToken)
    window.$notify?.success($t('messages.sessionTokenCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyFailed'))
  }
}
</script>
