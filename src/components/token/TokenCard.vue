<template>
  <div
    class="bg-surface border border-border rounded-lg p-3 cursor-pointer relative group transition-all duration-150 hover:bg-hover hover:border-border-strong"
    :class="{
      'border-accent bg-accent/5': isSelected,
      'ring-2 ring-warning ring-offset-1 bg-warning/10': isHighlighted
    }"
    @click="handleCardClick"
  >
    <!-- 头部：选择框 + 邮箱 -->
    <div class="flex items-center gap-2 mb-3 pr-0">
      <div
        class="selection-checkbox"
        :class="{ 'visible': selectionMode || isSelected }"
        @click.stop="$emit('select', token.id)"
      >
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
        </div>
      </div>
      <div
        class="inline-flex items-center gap-1 text-[15px] font-semibold text-text truncate flex-1"
        v-tooltip="token.email_note"
        @click.stop="copyEmailNote"
      >
        <span v-if="token.email_note">{{ showRealEmail ? token.email_note : maskedEmail }}</span>
        <span v-else class="text-text-muted">{{ $t('tokenCard.noEmailNote') }}</span>
      </div>
    </div>

    <!-- 右上角按钮组（悬停显示） -->
    <div
      class="absolute right-3 top-3 z-20 flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity"
      :class="{ 'opacity-100': isMenuOpen }"
      @click.stop
    >
      <button
        @click="openEditorModal"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors"
        v-tooltip="$t('tokenCard.selectEditor')"
      >
        <img src="/icons/vscode.svg" class="w-3.5 h-3.5" :alt="$t('tokenCard.selectEditor')" />
      </button>
      <button
        @click="handleCheckAccountStatus"
        class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-accent transition-colors"
        :disabled="isRefreshingOrLoading"
        v-tooltip="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
      >
        <svg v-if="!isRefreshingOrLoading && !token.skip_check" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <svg v-else-if="!isRefreshingOrLoading && token.skip_check" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
        </svg>
        <span v-else class="btn-spinner btn-spinner--sm text-accent" aria-hidden="true"></span>
      </button>
      <FloatingDropdown
        ref="menuRef"
        placement="bottom-end"
        :close-on-select="false"
        @open="isMenuOpen = true"
        @close="isMenuOpen = false"
      >
        <template #trigger>
          <button
            class="w-7 h-7 rounded border-none bg-surface text-text-secondary cursor-pointer flex items-center justify-center shadow-sm hover:bg-hover hover:text-text transition-colors"
            v-tooltip="$t('app.moreOptions')"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
            </svg>
          </button>
        </template>
        <template #default="{ close }">
          <button @click="exportTokenAsJson(); close()" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
            </svg>
            <span>{{ $t('tokenCard.exportJson') }}</span>
          </button>
          <button @click="copyToken(); close()" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
            <span>{{ $t('tokenCard.copyToken') }}</span>
          </button>
          <button @click="copyTenantUrl(); close()" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
            </svg>
            <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
          </button>
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
          <!-- 团队 / 查看使用详情 已注释
          <button v-if="token.auth_session" @click="showTeamManageModal = true; close()" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
            </svg>
            <span>{{ $t('tokenCard.team') }}</span>
          </button>
          <button v-if="token.auth_session" @click="showCreditUsageModal = true; close()" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
            </svg>
            <span>{{ $t('credit.viewUsage') }}</span>
          </button>
          -->
          <button v-if="token.portal_url" @click="showPortalDialog = true; close()" class="dropdown-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
            </svg>
            <span>{{ $t('tokenCard.openPortal') }}</span>
          </button>
          <button @click="deleteToken(); close()" class="dropdown-item text-danger hover:bg-danger/10">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
            <span>{{ $t('tokenCard.deleteToken') }}</span>
          </button>
        </template>
      </FloatingDropdown>
    </div>

    <!-- 属性区 -->
    <div class="flex flex-col gap-1.5">
      <!-- 创建时间 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM9 10H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2z"/>
          </svg>
          <span>{{ $t('tokenCard.createdAt') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text-muted">{{ formatDate(token.created_at) }}</div>
      </div>

      <!-- 重置时间 -->
      <div v-if="portalInfo.data && portalInfo.data.expiry_date" class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>
          </svg>
          <span>{{ $t('tokenCard.resetAt') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text-muted">{{ formatDate(portalInfo.data.expiry_date) }}</div>
      </div>

      <!-- Session 更新时间（session 相关已注释）
      <div v-if="token.session_updated_at" class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
          </svg>
          <span>{{ $t('tokenCard.sessionUpdatedAt') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text-muted">{{ formatDate(token.session_updated_at) }}</div>
      </div>
      -->

      <!-- 额度 -->
      <div v-if="showBalanceRow" class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V5h14v14zM7 12h2v5H7zm4-3h2v8h-2zm4-3h2v11h-2z"/>
          </svg>
          <span>{{ $t('tokenCard.balance') }}</span>
        </div>
        <div v-if="showProgressBar" class="flex-1 flex items-center gap-2">
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
        <div v-else class="flex-1 text-[13px]">
          <span class="badge" :class="balanceClasses">{{ balanceDisplay }}</span>
        </div>
      </div>

      <!-- 封禁状态 -->
      <div v-if="token.ban_status === 'SUSPENDED'" class="flex items-center min-h-6" @click.stop>
        <div
          :class="[
            'w-full inline-flex items-center gap-1.5 rounded bg-danger/10 px-2 py-1.5 text-xs text-danger',
            { 'cursor-pointer': isBannedWithSuspensions }
          ]"
          v-tooltip="isBannedWithSuspensions ? $t('tokenCard.clickToViewDetails') : ''"
          @click="handleStatusClick"
        >
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
          </svg>
          <span>{{ $t('tokenCard.banned') }}</span>
          <span v-if="isBannedWithSuspensions" class="w-1.5 h-1.5 rounded-full bg-danger shrink-0"></span>
        </div>
      </div>

      <!-- Token 失效状态 -->
      <div v-if="token.ban_status === 'INVALID_TOKEN'" class="flex items-center min-h-6" @click.stop>
        <div class="w-full inline-flex items-center gap-1.5 rounded bg-warning/10 px-2 py-1.5 text-xs text-warning">
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
          </svg>
          <span>{{ $t('tokenCard.tokenInvalid') }}</span>
        </div>
      </div>

      <!-- 标签 -->
      <div class="flex items-center gap-1 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.63 5.84C17.27 5.33 16.67 5 16 5L5 5.01C3.9 5.01 3 5.9 3 7v10c0 1.1.9 1.99 2 1.99L16 19c.67 0 1.27-.33 1.63-.84L22 12l-4.37-6.16z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.tag') }}</span>
        </div>
        <div class="flex-1 text-[13px]">
          <span
            v-if="!hasTag"
            class="inline-flex items-center gap-0.5 px-1.5 py-0.5 border border-dashed border-border rounded text-text-muted text-xs cursor-pointer hover:border-accent hover:text-accent transition-colors"
            @click.stop="openTagEditor"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
            </svg>
            {{ $t('tokenList.clickToAddTag') }}
          </span>
          <span
            v-else
            class="badge editable badge--sm"
            :style="tagBadgeStyle"
            @click.stop="openTagEditor"
          >
            {{ tagDisplayName }}
          </span>
        </div>
      </div>
    </div>
  </div>

  <!-- 编辑器选择模态框 -->
  <EditorSelectModal
    :show="showEditorModal"
    :token="token"
    @close="showEditorModal = false"
    @success="(msg) => window.$notify.success(msg)"
    @error="(msg) => window.$notify.error(msg)"
  />

  <!-- Portal 对话框 -->
  <ExternalLinkDialog
    :show="showPortalDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="token.portal_url || ''"
    :browser-title="getPortalBrowserTitle()"
    @close="showPortalDialog = false"
  />

  <!-- Suspensions 详情模态框 -->
  <SuspensionsModal
    v-if="showSuspensionsModal"
    :suspensions="token.suspensions"
    @close="showSuspensionsModal = false"
  />

  <!-- Credit Usage Modal（团队/查看使用详情 已注释）
  <CreditUsageModal
    v-if="showCreditUsageModal && token.auth_session"
    :auth-session="token.auth_session"
    :credits-balance="remainingCredits"
    :has-portal-url="!!token.portal_url"
    @close="showCreditUsageModal = false"
    @refresh-balance="handleCreditUsageRefresh"
    @update-portal-url="handleUpdatePortalUrl"
  />
  -->
  <!-- 标签编辑模态框 -->
  <TagEditorModal
    v-model:visible="showTagEditor"
    :token="token"
    :all-tokens="allTokens"
    @save="handleTagSave"
    @clear="handleTagClear"
  />

  <!-- Team 管理模态框（团队/查看使用详情 已注释）
  <TeamManageModal
    v-if="showTeamManageModal && token.auth_session"
    :auth-session="token.auth_session"
    @close="showTeamManageModal = false"
  />
  -->
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import ExternalLinkDialog from '../common/ExternalLinkDialog.vue'
import CreditUsageModal from '../credit/CreditUsageModal.vue'
import EditorSelectModal from './EditorSelectModal.vue'
import TagEditorModal from './TagEditorModal.vue'
import TeamManageModal from '../team/TeamManageModal.vue'
import SuspensionsModal from './SuspensionsModal.vue'
import FloatingDropdown from '../common/FloatingDropdown.vue'
import { useTokenActions } from '@/composables/useTokenActions'

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

const emit = defineEmits(['delete', 'edit', 'token-updated', 'select', 'payment-link-fetched'])

const {
  DEFAULT_TAG_COLOR,
  isCheckingStatus,
  isFetchingPaymentLink,
  portalInfo,
  tagDisplayName,
  hasTag,
  tagBadgeStyle,
  displayUrl,
  maskedEmail,
  formatExpiryDate,
  getStatusClass,
  getStatusText,
  normalizeHexColor,
  getContrastingTextColor,
  isEqual,
  copyToClipboard,
  copyWithNotification,
  copyToken,
  copyTenantUrl,
  copyEmailNote,
  copyPortalUrl,
  copyAuthSession,
  exportTokenAsJson,
  copyPaymentMethodLink,
  deleteToken,
  editToken,
  toggleSelection,
  handleTagSave,
  handleTagClear,
  toggleSkipCheck,
  checkAccountStatus,
  handleUpdatePortalUrl,
  getPortalBrowserTitle,
  initPortalInfo,
  formatDate
} = useTokenActions(props, emit)

// 本地 UI 状态
const isLoadingPortalInfo = ref(false)
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const menuRef = ref(null)
const isMenuOpen = ref(false)
const showSuspensionsModal = ref(false)
// 团队/查看使用详情 已注释
// const showCreditUsageModal = ref(false)
// const showTeamManageModal = ref(false)
// session 相关已注释
// const isRefreshingSession = ref(false)
const showTagEditor = ref(false)

const isRefreshingOrLoading = computed(() => {
  return isCheckingStatus.value ||
    (props.isBatchChecking && !props.token.skip_check) ||
    (props.isSelectedRefreshing && props.isSelected && !props.token.skip_check)
})

const isBannedWithSuspensions = computed(() => {
  return (
    props.token.ban_status === 'SUSPENDED' &&
    props.token.suspensions &&
    (Array.isArray(props.token.suspensions) ? props.token.suspensions.length > 0 : true)
  )
})

const balanceClasses = computed(() => {
  const hasError = portalInfo.value?.error
  const exhausted = props.token.ban_status === 'EXPIRED'
  if (hasError || exhausted) return 'badge--danger'
  return 'badge--success'
})

const isBannedOrExpired = computed(() => {
  const status = props.token.ban_status
  return status === 'EXPIRED' || status === 'SUSPENDED'
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

const remainingCredits = computed(() => {
  const currentCredits = portalInfo.value?.data?.credits_balance
  if (currentCredits !== undefined && currentCredits !== null) return currentCredits
  const fallbackCredits = props.token.portal_info?.credits_balance
  if (fallbackCredits !== undefined && fallbackCredits !== null) return fallbackCredits
  return null
})

const handleStatusClick = () => {
  if (isBannedWithSuspensions.value) {
    showSuspensionsModal.value = true
  }
}

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    if (showEditorModal.value) showEditorModal.value = false
    if (menuRef.value?.isOpen) menuRef.value.close()
  }
}

const openEditorModal = () => {
  if (showEditorModal.value) return
  showEditorModal.value = true
}

const handleToggleSkipCheck = (close) => {
  toggleSkipCheck()
  close?.()
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

const handleCheckAccountStatus = async (showNotification = true) => {
  await checkAccountStatus({
    showNotification,
    isBatchChecking: props.isBatchChecking,
    isSelectedRefreshing: props.isSelectedRefreshing,
    isSelected: props.isSelected
  })
}

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

onMounted(async () => {
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

const refreshAccountStatus = async () => {
  return await handleCheckAccountStatus()
}

// 团队/查看使用详情 已注释
// const handleCreditUsageRefresh = () => {
//   handleCheckAccountStatus(false)
// }

const handleCardClick = (event) => {
  if (event.target.closest('button, a, [role="button"], .selection-checkbox, [data-dropdown-trigger], input, label, .badge')) return
  emit('edit', props.token)
}

const openTagEditor = () => {
  showTagEditor.value = true
}

defineExpose({
  refreshAccountStatus
})
</script>
