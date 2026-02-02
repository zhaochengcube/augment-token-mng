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
      <div class="flex items-center gap-1.5">
        <div class="text-copyable" @click.stop="copyEmail" v-tooltip="account.email">
          <span class="text-copyable__content">{{ showRealEmail ? account.email : maskedEmail }}</span>
        </div>
        <span :class="planBadgeClasses">{{ planLabel }}</span>
      </div>
    </td>

    <!-- 配额 -->
    <td class="px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text">
      <div class="flex items-center gap-2 min-w-[140px]">
        <div class="flex-1 h-1.5 bg-muted rounded overflow-hidden">
          <div
            class="h-full rounded transition-all"
            :class="getQuotaBarClass(remainingPercentage)"
            :style="{ width: remainingPercentage + '%' }"
          ></div>
        </div>
        <span class="text-[11px] font-medium tabular-nums text-text-muted shrink-0">
          {{ remainingCredits }}/{{ totalCredits }}
        </span>
      </div>
    </td>

    <!-- 过期时间 -->
    <td class="w-[100px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text-muted">
      <span v-if="expiresText">{{ expiresText }}</span>
      <span v-else>-</span>
    </td>

    <!-- 操作 -->
    <td class="w-[110px] px-2.5 py-3.5 border-b border-border/50 align-top whitespace-nowrap text-[13px] text-text text-center">
      <div class="flex items-center justify-center gap-1">
        <button
          v-if="!isCurrent"
          @click.stop="$emit('switch', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isSwitching"
          v-tooltip="$t('platform.windsurf.switch')"
        >
          <svg v-if="!isSwitching" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z" />
          </svg>
          <span v-else class="btn-spinner btn-spinner--xs text-accent" aria-hidden="true"></span>
        </button>

        <button
          @click.stop="$emit('refresh', account.id)"
          class="btn btn--ghost btn--icon-sm"
          :disabled="isRefreshing"
          v-tooltip="$t('platform.windsurf.refresh')"
        >
          <svg v-if="!isRefreshing" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
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
            <button @click="handleMenuClick('copyRefreshToken', close)" class="dropdown-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
              </svg>
              <span>{{ $t('accountCard.copyRefreshToken') }}</span>
            </button>
            <button @click="handleMenuClick('delete', close)" class="dropdown-item text-danger hover:bg-danger/10">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
              <span>{{ $t('platform.windsurf.delete') }}</span>
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
</template>

<script setup>
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import FloatingDropdown from '../common/FloatingDropdown.vue'
import TagEditorModal from '../token/TagEditorModal.vue'

const { t: $t } = useI18n()

const DEFAULT_TAG_COLOR = '#f97316'

const props = defineProps({
  account: { type: Object, required: true },
  isCurrent: { type: Boolean, default: false },
  isSwitching: { type: Boolean, default: false },
  isRefreshing: { type: Boolean, default: false },
  isSelected: { type: Boolean, default: false },
  selectionMode: { type: Boolean, default: false },
  showRealEmail: { type: Boolean, default: true },
  allAccounts: { type: Array, default: () => [] }
})

const emit = defineEmits(['switch', 'refresh', 'delete', 'select', 'account-updated'])

const menuRef = ref(null)
const showTagEditor = ref(false)

const maskedEmail = computed(() => {
  const email = props.account.email
  if (!email || !email.includes('@')) return email
  return 'hello@windsurf.com'
})

// 配额计算
const totalCredits = computed(() => props.account.quota?.total_credits || 0)
const usedCredits = computed(() => props.account.quota?.used_credits || 0)
const remainingCredits = computed(() => Math.max(0, totalCredits.value - usedCredits.value))
const remainingPercentage = computed(() => {
  if (!totalCredits.value) return 0
  return Math.round((remainingCredits.value / totalCredits.value) * 100)
})

const getQuotaBarClass = (percent) => {
  if (percent < 10) return 'bg-danger'
  if (percent < 30) return 'bg-warning'
  return 'bg-success'
}

const isAvailable = computed(() => remainingPercentage.value >= 20)

const statusClass = computed(() => {
  if (props.isCurrent) return 'current'
  if (props.account.disabled) return 'disabled'
  if (isAvailable.value) return 'available'
  if (remainingPercentage.value > 0) return 'low'
  return 'expired'
})

const statusBadgeClass = computed(() => {
  switch (statusClass.value) {
    case 'current': return 'badge--accent-tech'
    case 'disabled': return 'badge--danger-tech'
    case 'available': return 'badge--success-tech'
    case 'low': return 'badge--warning-tech'
    case 'expired': return 'badge--danger-tech'
    default: return ''
  }
})

const statusDotClass = computed(() => {
  switch (statusClass.value) {
    case 'current': return 'text-accent'
    case 'disabled': return 'text-danger'
    case 'available': return 'text-success'
    case 'low': return 'text-warning'
    case 'expired': return 'text-danger'
    default: return ''
  }
})

const statusLabel = computed(() => {
  if (props.isCurrent) return $t('platform.windsurf.status.current')
  if (props.account.disabled) return $t('platform.windsurf.status.disabled')
  if (isAvailable.value) return $t('platform.windsurf.status.available')
  if (remainingPercentage.value > 0) return $t('platform.windsurf.status.low')
  return $t('platform.windsurf.status.expired')
})

const planLabel = computed(() => props.account.quota?.plan_name || 'Free')

const planBadgeClasses = computed(() => {
  const base = 'inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wide leading-none border'
  const plan = props.account.quota?.plan_name?.toLowerCase() || 'free'
  if (plan.includes('enterprise')) return `${base} text-amber-400 border-amber-400/50 bg-amber-400/12`
  if (plan.includes('pro')) return `${base} text-sky-400 border-sky-400/50 bg-sky-400/12`
  return `${base} text-slate-400 border-slate-400/45 bg-slate-400/12`
})

const expiresText = computed(() => {
  if (!props.account.quota?.expires_at) return ''
  try {
    const date = new Date(props.account.quota.expires_at)
    if (isNaN(date.getTime())) return ''
    return date.toLocaleDateString()
  } catch {
    return ''
  }
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
    case 'copyRefreshToken':
      await copyRefreshToken()
      break
    case 'delete':
      emit('delete', props.account.id)
      break
  }
}

const copyRefreshToken = async () => {
  try {
    const refreshToken = props.account.token?.refresh_token
    if (!refreshToken) {
      window.$notify?.error($t('messages.noRefreshToken'))
      return
    }
    await navigator.clipboard.writeText(refreshToken)
    window.$notify?.success($t('messages.refreshTokenCopied'))
  } catch (err) {
    window.$notify?.error($t('messages.copyFailed'))
  }
}
</script>
