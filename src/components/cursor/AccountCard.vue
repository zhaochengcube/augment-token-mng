<template>
  <div
    :class="[
      'card group relative flex flex-col gap-4 overflow-hidden rounded-2xl border p-5 transition-all duration-300',
      'min-h-[140px] h-fit z-[1]',
      isSwitching ? 'opacity-60 pointer-events-none' : '',
      isSelected ? 'selected border-accent' : ''
    ]"
    @click="handleCardClick"
  >
    <!-- 选择框 -->
    <div
      class="selection-checkbox"
      :class="{ 'visible': selectionMode || isSelected }"
      @click.stop="toggleSelection"
    >
      <div class="checkbox-inner" :class="{ 'checked': isSelected }">
        <svg v-if="isSelected" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
        </svg>
      </div>
    </div>

    <!-- Status Badge -->
    <div class="absolute right-3 top-3 z-10 flex items-center gap-1.5">
      <!-- 添加标签按钮（无标签时显示） -->
      <span
        v-if="!account.tag"
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
        v-if="account.tag"
        class="badge editable"
        :style="tagBadgeStyle"
        v-tooltip="$t('tokenList.clickToEditTag')"
        @click.stop="openTagEditor"
      >
        {{ account.tag }}
      </span>
      <span :class="['badge', statusBadgeClass]">
        <span class="status-dot" :class="statusDotClass"></span>
        {{ statusLabel }}
      </span>
    </div>

    <!-- Account Info -->
    <div class="flex flex-col gap-1 h-full mt-3.5">
      <div class="flex-1 min-w-0">
        <!-- 第一行：邮箱 -->
        <div class="mb-2.5 min-h-[26px] flex items-center">
          <div class="text-copyable" @click.stop="copyEmail" v-tooltip="account.email">
            <span class="text-copyable__content">{{ showRealEmail ? account.email : maskedEmail }}</span>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <!-- 第二行：日期信息 -->
          <div class="flex flex-wrap items-center justify-start gap-2.5">
            <span class="text-meta" v-tooltip="$t('platform.cursor.createdAt') + ': ' + formatDate(account.created_at)">C: {{ formatDate(account.created_at) }}</span>
            <span v-if="account.token?.expiry_timestamp" class="text-meta" v-tooltip="$t('platform.cursor.accessTokenExpiry') + ': ' + formatDate(account.token.expiry_timestamp)">A: {{ formatDate(account.token.expiry_timestamp) }}</span>
            <span v-if="account.token?.session_expiry_timestamp" class="text-meta" v-tooltip="$t('platform.cursor.sessionExpiry') + ': ' + formatDate(account.token.session_expiry_timestamp)">S: {{ formatDate(account.token.session_expiry_timestamp) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="mt-auto flex gap-1.5 justify-end">
      <button
        v-if="!isCurrent"
        @click.stop="$emit('switch', account.id)"
        class="btn btn--primary btn--sm flex-1"
        :disabled="isSwitching"
      >
        <svg v-if="!isSwitching" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/>
        </svg>
        <span v-else class="btn-spinner" aria-hidden="true"></span>
        {{ isSwitching ? $t('platform.cursor.switching') : $t('platform.cursor.switch') }}
      </button>

      <!-- 使用详情按钮 -->
      <button
        @click.stop="showUsageModal = true"
        class="btn btn--ghost btn--icon shrink-0"
        v-tooltip="$t('cursorUsage.title')"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 3h2v18H3V3zm16 8h2v10h-2V11zm-8 4h2v6h-2v-6zm4-8h2v14h-2V7zm-8 6h2v8H7v-8z"/>
        </svg>
      </button>

      <!-- 复制菜单 -->
      <FloatingDropdown ref="copyMenuRef" placement="bottom-end" :close-on-select="false" @click.stop>
        <template #trigger>
          <button class="btn btn--ghost btn--icon shrink-0" v-tooltip="$t('tokenCard.copyMenu')">
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
        </template>
        <template #default="{ close }">
          <button @click="handleCopyMenuClick('accessToken', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
            <span>{{ $t('accountCard.copyAccessToken') }}</span>
          </button>
          <button @click="handleCopyMenuClick('sessionToken', close)" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
            <span>{{ $t('accountCard.copySessionToken') }}</span>
          </button>
        </template>
      </FloatingDropdown>

      <button
        @click.stop="$emit('delete', account.id)"
        class="btn btn--ghost btn--icon shrink-0 text-danger hover:bg-danger/10 hover:text-danger"
        v-tooltip="$t('common.delete')"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
      </button>
    </div>

  </div>

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

// 复制菜单状态
const copyMenuRef = ref(null)
const showTagEditor = ref(false)
const showUsageModal = ref(false)

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@cursor.com'
})

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

// 格式化日期
const formatDate = (timestamp) => {
  if (!timestamp) return ''
  if (!timestamp) return '-'
  const date = new Date(timestamp * 1000)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const toggleSelection = () => {
  emit('select', props.account.id)
}

const handleCardClick = () => {
  if (props.selectionMode) {
    toggleSelection()
  }
}

const copyEmail = async () => {
  try {
    await navigator.clipboard.writeText(props.account.email)
    window.$notify?.success($t('messages.emailNoteCopied'))
  } catch (err) {
    console.error('Failed to copy email:', err)
    window.$notify?.error($t('messages.copyEmailNoteFailed'))
  }
}

// 复制菜单操作
const handleCopyMenuClick = async (type, close) => {
  close?.()

  switch (type) {
    case 'accessToken':
      await copyAccessToken()
      break
    case 'sessionToken':
      await copySessionToken()
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
    console.error('Failed to copy access token:', err)
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
    console.error('Failed to copy session token:', err)
    window.$notify?.error($t('messages.copyFailed'))
  }
}

// 标签样式
const DEFAULT_TAG_COLOR = '#f97316'

const tagBadgeStyle = computed(() => {
  if (!props.account.tag) return {}
  const color = props.account.tag_color || DEFAULT_TAG_COLOR
  return { '--tag-color': color }
})

// 将 account 转换为 TagEditorModal 需要的 token 格式
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

// 标签操作
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
</script>

