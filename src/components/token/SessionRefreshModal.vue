<template>
  <BaseModal
    :visible="visible"
    :show-close="true"
    :close-on-overlay="true"
    :close-on-esc="true"
    modal-class="max-w-[500px]"
    @close="close"
  >
    <template #header>
      <div class="flex items-center gap-3 text-text">
        <svg class="h-5 w-5 text-warning drop-shadow-[0_0_8px_rgba(245,158,11,0.3)]" viewBox="0 0 24 24" fill="currentColor">
          <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
        </svg>
        <h3 class="m-0 text-lg font-semibold">{{ $t('tokenList.sessionRefreshTitle') }}</h3>
      </div>
    </template>

    <!-- Empty State -->
    <div v-if="expiringTokens.length === 0" class="flex flex-col items-center justify-center gap-3.5 py-12 text-center text-text-muted">
      <svg class="h-10 w-10 opacity-20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
      </svg>
      <span class="text-sm font-medium">{{ $t('tokenList.noExpiringSession') }}</span>
    </div>

    <!-- Token List -->
    <div v-else class="flex flex-col gap-2.5">
      <div
        v-for="token in expiringTokens"
        :key="token.id"
        class="flex items-center gap-3.5 rounded-xl border border-border bg-muted/50 p-3.5 transition-all duration-200 hover:border-warning/30 hover:bg-muted/70 hover:shadow-[0_0_12px_rgba(245,158,11,0.15)]"
      >
        <!-- Warning Icon -->
        <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-warning/25 bg-warning/15 text-warning">
          <svg class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
          </svg>
        </div>

        <!-- Content -->
        <div class="flex min-w-0 flex-1 flex-col gap-1.5">
          <span class="truncate text-sm font-semibold text-text">{{ token.email_note || $t('tokenList.noEmailNote') }}</span>
          <div class="flex flex-wrap items-center gap-2">
            <span :class="['badge', getBadgeClass(token.ban_status)]">
              <span class="status-dot"></span>
              {{ getStatusText(token.ban_status) }}
            </span>
            <span class="whitespace-nowrap text-xs font-medium text-warning/80">
              {{ getDaysRemaining(token) }} {{ $t('tokenList.daysRemaining') }}
            </span>
          </div>
        </div>

        <!-- Refresh Button -->
        <button
          @click="handleRefreshSingle(token)"
          :disabled="refreshingTokenId === token.id"
          :title="$t('tokenList.refreshSession')"
          class="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-lg border border-border bg-accent/10 text-accent transition-all duration-200 hover:scale-105 hover:border-accent hover:bg-accent/20 hover:shadow-accent disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:scale-100"
        >
          <svg
            v-if="refreshingTokenId !== token.id"
            class="h-3.5 w-3.5"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
          <span v-else class="btn-spinner btn-spinner--sm" aria-hidden="true"></span>
        </button>
      </div>
    </div>

    <template #footer>
      <button
        v-if="!refreshing"
        @click="handleRefreshAll"
        :disabled="expiringTokens.length === 0"
        class="btn btn--primary"
      >
        <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        {{ $t('tokenList.refreshAllSessions') }}
      </button>
      <button v-else class="btn btn--primary" disabled>
        <span class="btn-spinner btn-spinner--sm" aria-hidden="true"></span>
        {{ $t('tokenList.refreshing') }}
      </button>
      <button @click="close" class="btn btn--secondary">
        {{ $t('common.close') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'

const { t } = useI18n()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  expiringTokens: {
    type: Array,
    default: () => []
  },
  refreshing: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:visible', 'refresh', 'refresh-single'])

// 当前正在刷新的 token ID
const refreshingTokenId = ref(null)

const close = () => {
  emit('update:visible', false)
}

const handleRefreshAll = () => {
  emit('refresh')
}

// 单个刷新 session
const handleRefreshSingle = async (token) => {
  if (!token.auth_session) {
    window.$notify?.warning(t('messages.noAuthSession'))
    return
  }

  refreshingTokenId.value = token.id

  try {
    // 调用后端刷新接口
    const result = await invoke('batch_refresh_sessions', {
      requests: [{
        id: token.id,
        session: token.auth_session
      }]
    })

    if (result && result.length > 0 && result[0].success && result[0].new_session) {
      const now = new Date().toISOString()

      // 通知父组件更新 token
      emit('refresh-single', {
        tokenId: token.id,
        newSession: result[0].new_session,
        updatedAt: now
      })

      window.$notify?.success(t('messages.sessionRefreshSuccess', { count: 1 }))
    } else {
      const errorMsg = result[0]?.error || 'Unknown error'
      console.error(`Failed to refresh session for token ${token.id}:`, errorMsg)
      window.$notify?.error(t('messages.sessionRefreshFailed') + ': ' + errorMsg)
    }
  } catch (error) {
    console.error('Failed to refresh session:', error)
    window.$notify?.error(t('messages.sessionRefreshFailed') + ': ' + error.toString())
  } finally {
    refreshingTokenId.value = null
  }
}

const getDaysRemaining = (token) => {
  if (!token.created_at) return 0
  // 使用 session_updated_at（如果存在）或 created_at 来判断
  const referenceTime = token.session_updated_at || token.created_at
  const referenceDate = new Date(referenceTime)
  const now = new Date()
  const daysSinceReference = Math.floor((now - referenceDate) / (1000 * 60 * 60 * 24))
  return Math.max(0, 30 - daysSinceReference)
}

// 获取 badge 样式类
const getBadgeClass = (status) => {
  switch (status) {
    case 'SUSPENDED':
      return 'badge--danger-tech'
    case 'EXPIRED':
      return 'badge--warning-tech'
    case 'INVALID_TOKEN':
      return 'badge--danger-tech'
    case 'ACTIVE':
      return 'badge--success-tech'
    default:
      return 'badge--success-tech'
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
</script>
