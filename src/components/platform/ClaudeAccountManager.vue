<template>
  <div class="flex flex-col h-full w-full min-h-0" @click.stop="handlePageContentClick">
    <FixedPaginationLayout
      :show-pagination="shouldShowPagination"
      :scroll-key="currentPage"
      :body-class="viewMode === 'table' ? 'px-0 py-0' : 'px-3 py-1 pb-4'"
      :pagination-class="[
        'fixed bottom-0 right-0 border-t border-border bg-surface z-10',
        isSidebarCollapsed ? 'left-16' : 'left-38'
      ]"
    >
      <template #header>
        <!-- Page Header -->
        <div class="flex items-center justify-between gap-4 px-5 py-4 border-b border-border bg-surface shrink-0">
          <!-- 左侧：存储状态 + 功能性操作 -->
          <div class="flex items-center gap-3 shrink-0">
            <!-- 存储状态徽章 -->
            <div
              :class="['badge', storageStatusClass, { clickable: isDatabaseAvailable }]"
              v-tooltip="isDatabaseAvailable ? $t('platform.claude.viewSyncQueueTooltip') : ''"
              @click="isDatabaseAvailable && openSyncQueue()"
            >
              <span :class="['status-dot', storageStatusClass]"></span>
              <span class="text-[11px] font-semibold tracking-[0.3px]">{{ storageStatusText }}</span>
            </div>

            <!-- 功能性操作按钮 -->
            <div class="flex items-center gap-2" @click.stop>
              <button class="btn btn--icon btn--ghost" @click="setToolbarMode('search')" v-tooltip="$t('common.search')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5z" />
                </svg>
              </button>
              <button class="btn btn--icon btn--ghost" @click="setToolbarMode('filter')" v-tooltip="$t('common.filter')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3 4h18l-7 8v6l-4 2v-8L3 4z"/>
                </svg>
              </button>
              <button class="btn btn--icon btn--ghost" @click="setToolbarMode('sort')" v-tooltip="$t('common.sort')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <path d="M7 16V6M4 9l3-3 3 3" />
                  <path d="M17 8v10M14 15l3 3 3-3" />
                </svg>
              </button>
              <button
                class="btn btn--icon btn--ghost"
                @click="toggleViewMode"
                v-tooltip="viewMode === 'card' ? $t('common.switchToTable') : $t('common.switchToCard')"
              >
                <svg v-if="viewMode === 'table'" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M4 11h5V5H4v6zm0 7h5v-6H4v6zm6 0h5v-6h-5v6zm6 0h5v-6h-5v6zm-6-7h5V5h-5v6zm6-6v6h5V5h-5z"/>
                </svg>
                <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3 14h4v-4H3v4zm0 5h4v-4H3v4zM3 9h4V5H3v4zm5 5h13v-4H8v4zm0 5h13v-4H8v4zM8 5v4h13V5H8z"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- 右侧：主要操作按钮 -->
          <div class="flex items-center gap-2 shrink-0" @click.stop>
            <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('platform.claude.addAccount')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
            </button>
            <button
              v-if="isDatabaseAvailable"
              class="btn btn--icon btn--ghost"
              @click="handleSync"
              :disabled="isSyncing"
              v-tooltip="$t('tokenList.syncTooltip')"
            >
              <svg v-if="!isSyncing" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z" />
              </svg>
              <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
            </button>
            <button
              class="btn btn--icon btn--ghost"
              @click="handleRefresh"
              :disabled="isRefreshing"
              v-tooltip="$t('common.refresh')"
            >
              <svg v-if="!isRefreshing" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
              </svg>
              <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
            </button>
          </div>
        </div>
      </template>

      <!-- Loading State -->
      <div v-if="isLoading" class="flex flex-col items-center justify-center h-full gap-5 text-text-muted py-12 px-6">
        <div class="spinner"></div>
        <p class="m-0">{{ $t('common.loading') }}</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="showEmptyState" class="flex flex-col items-center justify-center py-[60px] px-5 text-text-secondary">
        <div class="text-text-muted mb-[18px] opacity-60">
          <img src="/icons/claude.svg" alt="Claude" width="64" height="64" />
        </div>
        <p class="mt-4 text-sm">{{ searchQuery ? $t('common.noSearchResults') : $t('platform.claude.emptyState') }}</p>
        <button v-if="!searchQuery" class="btn btn--primary mt-4" @click="showAddDialog = true">
          {{ $t('platform.claude.addFirst') }}
        </button>
      </div>

      <!-- Account List -->
      <template v-else>
        <!-- 卡片布局 -->
        <div v-if="viewMode === 'card'" class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3 p-1">
          <AccountCard
            v-for="account in paginatedAccounts"
            :key="account.id"
            :account="account"
            :is-current="account.id === currentAccountId"
            :is-switching="isSwitching"
            :is-selected="selectedAccountIds.has(account.id)"
            :selection-mode="isSelectionMode"
            :all-accounts="accounts"
            @edit="handleEdit"
            @delete="handleDelete"
            @switch="handleSwitch"
            @select="toggleAccountSelection"
            @account-updated="handleAccountUpdated"
          />
        </div>

        <!-- 列表布局 -->
        <div v-else class="table-container">
          <table class="table table-fixed">
            <thead>
              <tr>
                <th class="th w-11 text-center">
                  <div class="inline-flex items-center justify-center h-5 cursor-pointer align-middle leading-none" @click="toggleSelectAll">
                    <div class="checkbox-inner" :class="{ 'checked': isAllSelected || isPartialSelected }">
                      <svg v-if="isAllSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                      </svg>
                      <svg v-else-if="isPartialSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M19 13H5v-2h14v2z" />
                      </svg>
                    </div>
                  </div>
                </th>
                <th class="th w-[120px]">{{ $t('platform.claude.table.serviceName') }}</th>
                <th class="th">{{ $t('platform.claude.table.websiteUrl') }}</th>
                <th class="th w-[100px]">{{ $t('platform.claude.table.expiryDate') }}</th>
                <th class="th w-[60px]">{{ $t('subscriptions.fields.tag') }}</th>
                <th class="th w-[80px] text-center">{{ $t('common.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <AccountTableRow
                v-for="account in paginatedAccounts"
                :key="account.id"
                :account="account"
                :is-current="account.id === currentAccountId"
                :is-switching="isSwitching"
                :is-selected="selectedAccountIds.has(account.id)"
                :selection-mode="isSelectionMode"
                :all-accounts="accounts"
                @edit="handleEdit"
                @delete="handleDelete"
                @switch="handleSwitch"
                @select="toggleAccountSelection"
                @account-updated="handleAccountUpdated"
              />
            </tbody>
          </table>
        </div>
      </template>

      <template #pagination>
        <Pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total-pages="totalPages"
          :total-items="filteredAccounts.length"
          :page-size-options="[10, 20, 50, 100]"
        />
      </template>
    </FixedPaginationLayout>

    <!-- Search Toolbar -->
    <ActionToolbar
      :visible="toolbarMode === 'search'"
      :title="$t('common.search')"
      @close="setToolbarMode('hidden')">
      <div class="flex items-center gap-3 w-full px-4 h-10 border border-border rounded-lg bg-surface transition-all duration-200 focus-within:border-accent">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="shrink-0 text-text-muted">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5z" />
        </svg>
        <input
          ref="searchInputRef"
          type="text"
          v-model="searchQuery"
          :placeholder="$t('platform.claude.searchPlaceholder')"
          class="flex-1 bg-transparent border-none outline-none text-sm text-text placeholder:text-text-muted"
          @keydown.escape="setToolbarMode('hidden')"
        />
        <button v-if="searchQuery" class="btn btn--icon-sm btn--ghost" @click="searchQuery = ''">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>
    </ActionToolbar>

    <!-- Filter Toolbar -->
    <ActionToolbar
      :visible="toolbarMode === 'filter'"
      :title="$t('common.filter')"
      max-width="720px"
      @close="setToolbarMode('hidden')">
      <div class="flex flex-col gap-4">
        <!-- Expiry Filter -->
        <div class="flex flex-col gap-2">
          <span class="label">{{ $t('subscriptions.filters.expiry') }}</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="['btn btn--sm', selectedExpiryFilter === 'all' ? 'btn--primary' : 'btn--secondary']"
              @click="setExpiryFilter('all')"
            >
              <span>{{ $t('subscriptions.filters.all') }}</span>
              <span class="ml-1 opacity-70">({{ expiryStatistics.all }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedExpiryFilter === 'expired' ? 'btn--primary' : 'btn--secondary']"
              @click="setExpiryFilter('expired')"
            >
              <span>{{ $t('subscriptions.filters.expired') }}</span>
              <span class="ml-1 opacity-70">({{ expiryStatistics.expired }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedExpiryFilter === '7d' ? 'btn--primary' : 'btn--secondary']"
              @click="setExpiryFilter('7d')"
            >
              <span>{{ $t('subscriptions.filters.within7Days') }}</span>
              <span class="ml-1 opacity-70">({{ expiryStatistics['7d'] }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedExpiryFilter === '30d' ? 'btn--primary' : 'btn--secondary']"
              @click="setExpiryFilter('30d')"
            >
              <span>{{ $t('subscriptions.filters.within30Days') }}</span>
              <span class="ml-1 opacity-70">({{ expiryStatistics['30d'] }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedExpiryFilter === 'gt30' ? 'btn--primary' : 'btn--secondary']"
              @click="setExpiryFilter('gt30')"
            >
              <span>{{ $t('subscriptions.filters.after30Days') }}</span>
              <span class="ml-1 opacity-70">({{ expiryStatistics.gt30 }})</span>
            </button>
          </div>
        </div>

        <!-- Tag Filter -->
        <div class="flex flex-col gap-2">
          <span class="label">{{ $t('subscriptions.filters.tag') }}</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="['btn btn--sm', selectedTagFilter === null ? 'btn--primary' : 'btn--secondary']"
              @click="selectedTagFilter = null"
            >
              <span>{{ $t('subscriptions.filters.all') }}</span>
              <span class="ml-1 opacity-70">({{ tagStatistics.all }})</span>
            </button>
            <button
              v-for="tag in tagOptions"
              :key="tag.name"
              :class="['btn btn--sm', selectedTagFilter === tag.name ? 'btn--primary' : 'btn--secondary']"
              @click="selectedTagFilter = tag.name"
            >
              <span>{{ tag.name }}</span>
              <span class="ml-1 opacity-70">({{ tagStatistics[tag.name] || 0 }})</span>
            </button>
          </div>
        </div>
      </div>
    </ActionToolbar>

    <!-- Sort Toolbar -->
    <ActionToolbar
      :visible="toolbarMode === 'sort'"
      :title="$t('common.sort')"
      @close="setToolbarMode('hidden')">
      <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <!-- 排序方向 -->
        <div class="flex items-center gap-3">
          <span class="label">{{ $t('common.sortDirection') }}</span>
          <div class="flex items-center gap-2 flex-wrap">
            <button
              :class="[
                'btn btn--sm',
                sortOrder === 'asc' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType(sortType, 'asc')"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 14l4-4 4 4H8z" />
              </svg>
              <span class="ml-1">{{ $t('common.ascending') }}</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                sortOrder === 'desc' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType(sortType, 'desc')"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 10l-4 4-4-4h8z" />
              </svg>
              <span class="ml-1">{{ $t('common.descending') }}</span>
            </button>
          </div>
        </div>

        <!-- 排序字段 -->
        <div class="flex items-center gap-3">
          <span class="label">{{ $t('common.sortBy') }}</span>
          <div class="flex items-center gap-2 flex-wrap">
            <button
              :class="[
                'btn btn--sm',
                sortType === 'created' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('created', sortOrder)"
            >
              {{ $t('subscriptions.sortByCreated') }}
            </button>
            <button
              :class="[
                'btn btn--sm',
                sortType === 'expiry' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('expiry', sortOrder)"
            >
              {{ $t('subscriptions.sortByExpiry') }}
            </button>
          </div>
        </div>
      </div>
    </ActionToolbar>

    <!-- Add/Edit Dialog -->
    <AccountDialog
      v-if="showAddDialog || editingAccount"
      :account="editingAccount"
      :all-accounts="accounts"
      @save="handleSave"
      @close="closeDialog"
    />

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="accounts.length"
      :title="$t('platform.claude.syncQueueTitle')"
      :upserts-title="$t('platform.claude.syncQueueUpsertsTitle')"
      :deletions-title="$t('platform.claude.syncQueueDeletionsTitle')"
      :empty-text="$t('platform.claude.syncQueueEmpty')"
      :full-sync-text="$t('platform.claude.fullSync')"
      :sync-text="$t('platform.claude.sync')"
      label-field="service_name"
      fallback-field="id"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />

    <!-- 批量操作工具栏 -->
    <BatchToolbar
      :visible="isSelectionMode"
      :selected-count="selectedAccountIds.size"
      @select-all="selectAllOnPage"
      @clear="clearSelection"
    >
      <template #actions>
        <!-- 批量编辑标签 -->
        <button
          @click="showBatchTagEditor = true"
          class="btn btn--icon btn--ghost"
          v-tooltip="$t('tokenList.batchEditTag')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58.55 0 1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41 0-.55-.23-1.06-.59-1.42zM5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7z"/>
          </svg>
        </button>

        <!-- 批量删除 -->
        <button
          @click="showBatchDeleteConfirm"
          class="btn btn--icon btn--ghost text-danger hover:bg-danger-muted"
          v-tooltip="$t('common.batchDelete')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
          </svg>
        </button>
      </template>
    </BatchToolbar>

    <!-- 批量编辑标签模态框 -->
    <TagEditorModal
      v-model:visible="showBatchTagEditor"
      :tokens="selectedAccounts"
      :all-tokens="allAccountsAsTokens"
      @save="handleBatchTagSave"
      @clear="handleBatchTagClear"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, nextTick, toRefs, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useStorageSync } from '@/composables/useStorageSync'
import ActionToolbar from '../common/ActionToolbar.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'
import Pagination from '../common/Pagination.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import TagEditorModal from '../token/TagEditorModal.vue'
import AccountCard from '../claude/AccountCard.vue'
import AccountTableRow from '../claude/AccountTableRow.vue'
import AccountDialog from '../claude/AccountDialog.vue'

const { t: $t } = useI18n()

const props = defineProps({
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})

const { isSidebarCollapsed } = toRefs(props)

// 本地存储键（用于视图模式）
const VIEW_MODE_KEY = 'atm-claude-accounts-view-mode'

// 状态
const accounts = ref([])
const isLoading = ref(false)
const isRefreshing = ref(false)
const isSwitching = ref(false)
const showAddDialog = ref(false)
const editingAccount = ref(null)
const currentAccountId = ref(null)
const viewMode = ref('card')
const searchQuery = ref('')
const toolbarMode = ref('hidden')
const searchInputRef = ref(null)
const selectedExpiryFilter = ref('all')
const selectedTagFilter = ref(null)

// 排序状态
const sortType = ref('created')
const sortOrder = ref('desc')

// 分页状态
const currentPage = ref(1)
const pageSize = ref(20)

// 批量操作状态
const selectedAccountIds = ref(new Set())
const showBatchTagEditor = ref(false)

// 使用双向存储同步
const {
  isDatabaseAvailable,
  isSyncing,
  storageStatusText,
  storageStatusClass,
  pendingUpsertsList,
  pendingDeletionsList,
  showSyncQueueModal,
  initSync,
  markItemUpsert,
  markItemDeletion,
  markAllForSync,
  openSyncQueue,
  handleSync
} = useStorageSync({
  platform: 'claude',
  syncCommand: 'claude_sync_accounts',
  items: accounts,
  itemKey: 'account',
  labelField: 'service_name',
  onSyncComplete: async () => {
    try {
      const response = await invoke('claude_list')
      accounts.value = response.accounts || []
    } catch (error) {
      console.error('Failed to reload accounts after sync:', error)
    }
  }
})

// 标记全部同步
const handleMarkAllForSync = () => {
  markAllForSync()
}

// 辅助函数：计算账户的剩余天数
const getDaysLeft = (account) => {
  if (!account.expiry_date) return null
  const expiryTime = account.expiry_date * 1000
  return Math.ceil((expiryTime - Date.now()) / (1000 * 60 * 60 * 24))
}

// 辅助函数：应用到期时间过滤
const applyExpiryFilter = (accs, filter) => {
  if (filter === 'all') return accs
  return accs.filter((account) => {
    const daysLeft = getDaysLeft(account)
    if (daysLeft === null) return false
    if (filter === 'expired') return daysLeft < 0
    if (filter === '7d') return daysLeft >= 0 && daysLeft <= 7
    if (filter === '30d') return daysLeft >= 0 && daysLeft <= 30
    if (filter === 'gt30') return daysLeft > 30
    return true
  })
}

// 只应用搜索的账户列表
const searchFilteredAccounts = computed(() => {
  let result = accounts.value
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(acc =>
      acc.service_name?.toLowerCase().includes(query) ||
      acc.website_url?.toLowerCase().includes(query) ||
      acc.tag?.toLowerCase().includes(query)
    )
  }
  return result
})

// 应用搜索 + 标签筛选（用于计算到期时间统计）
const tagFilteredAccounts = computed(() => {
  let result = searchFilteredAccounts.value
  if (selectedTagFilter.value) {
    result = result.filter(acc => acc.tag === selectedTagFilter.value)
  }
  return result
})

// 应用搜索 + 到期时间筛选（用于计算标签统计）
const expiryFilteredAccounts = computed(() => {
  return applyExpiryFilter(searchFilteredAccounts.value, selectedExpiryFilter.value)
})

// 到期时间统计 - 基于搜索+标签筛选
const expiryStatistics = computed(() => {
  const accs = tagFilteredAccounts.value
  const stats = {
    all: accs.length,
    expired: 0,
    '7d': 0,
    '30d': 0,
    gt30: 0
  }
  accs.forEach(acc => {
    const daysLeft = getDaysLeft(acc)
    if (daysLeft === null) return
    if (daysLeft < 0) stats.expired++
    else if (daysLeft <= 7) stats['7d']++
    else if (daysLeft <= 30) stats['30d']++
    else stats.gt30++
  })
  return stats
})

// 标签统计 - 基于搜索+到期时间筛选
const tagStatistics = computed(() => {
  const accs = expiryFilteredAccounts.value
  let taggedCount = 0
  const stats = {}
  accs.forEach(acc => {
    if (acc.tag) {
      stats[acc.tag] = (stats[acc.tag] || 0) + 1
      taggedCount++
    }
  })
  stats.all = taggedCount
  return stats
})

// 计算属性：过滤并排序后的账户列表
const filteredAccounts = computed(() => {
  // 应用到期时间筛选
  let result = applyExpiryFilter(searchFilteredAccounts.value, selectedExpiryFilter.value)

  // 过滤：标签
  if (selectedTagFilter.value) {
    result = result.filter(acc => acc.tag === selectedTagFilter.value)
  }

  // 排序：当前账号始终在第一位，其他账号按排序方式排列
  result = [...result].sort((a, b) => {
    // 当前账号优先
    const aIsCurrent = a.id === currentAccountId.value
    const bIsCurrent = b.id === currentAccountId.value
    if (aIsCurrent && !bIsCurrent) return -1
    if (!aIsCurrent && bIsCurrent) return 1

    // 其他排序逻辑
    if (sortType.value === 'created') {
      const timeA = a.created_at || 0
      const timeB = b.created_at || 0
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'expiry') {
      const getExpiryTime = (acc) => {
        if (!acc.expiry_date) return sortOrder.value === 'desc' ? -Infinity : Infinity
        return acc.expiry_date * 1000
      }
      const timeA = getExpiryTime(a)
      const timeB = getExpiryTime(b)
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    }
    return 0
  })

  return result
})

const tagOptions = computed(() => {
  const tagMap = new Map()
  accounts.value.forEach((acc) => {
    if (acc.tag && !tagMap.has(acc.tag)) {
      tagMap.set(acc.tag, { name: acc.tag })
    }
  })
  return Array.from(tagMap.values()).sort((a, b) => a.name.localeCompare(b.name))
})

// 分页后的账户列表
const paginatedAccounts = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredAccounts.value.slice(start, end)
})

// 是否显示分页
const totalPages = computed(() => Math.ceil(filteredAccounts.value.length / pageSize.value))

const shouldShowPagination = computed(() => !isLoading.value && filteredAccounts.value.length > 0)

// 空状态判断
const showEmptyState = computed(() => filteredAccounts.value.length === 0)

// 批量操作计算属性
const isSelectionMode = computed(() => selectedAccountIds.value.size > 0)

const selectedAccounts = computed(() => {
  return accounts.value
    .filter(a => selectedAccountIds.value.has(a.id))
    .map(acc => ({
      id: acc.id,
      tag_name: acc.tag || '',
      tag_color: acc.tag_color || ''
    }))
})

const allAccountsAsTokens = computed(() =>
  accounts.value.map(acc => ({
    tag_name: acc.tag || '',
    tag_color: acc.tag_color || ''
  }))
)

const isAllSelected = computed(() => {
  return paginatedAccounts.value.length > 0 && paginatedAccounts.value.every(a => selectedAccountIds.value.has(a.id))
})

const isPartialSelected = computed(() => {
  return selectedAccountIds.value.size > 0 && !isAllSelected.value
})

// 工具栏模式切换
const setToolbarMode = (mode) => {
  toolbarMode.value = toolbarMode.value === mode ? 'hidden' : mode
  if (toolbarMode.value === 'search') {
    nextTick(() => searchInputRef.value?.focus())
  }
}

const setExpiryFilter = (value) => {
  selectedExpiryFilter.value = value
}

// 视图模式切换
const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'card' ? 'table' : 'card'
  saveViewMode()
}

// 排序设置
const setSortType = (type, order) => {
  sortType.value = type
  sortOrder.value = order
}

// 加载账户数据
const loadAccounts = async () => {
  isLoading.value = true
  try {
    await initSync()
    const response = await invoke('claude_list')
    accounts.value = response.accounts || []
    // 加载当前账户ID
    await loadCurrentAccountId()
  } catch (error) {
    console.error('Failed to load accounts:', error)
    accounts.value = []
  } finally {
    isLoading.value = false
  }
}

// 加载当前账户ID
const loadCurrentAccountId = async () => {
  try {
    const currentId = await invoke('claude_get_current_account_id')
    currentAccountId.value = currentId
  } catch (error) {
    console.error('Failed to load current account ID:', error)
    currentAccountId.value = null
  }
}

// 刷新数据
const handleRefresh = async () => {
  isRefreshing.value = true
  try {
    await initSync()
    const response = await invoke('claude_list')
    accounts.value = response.accounts || []
    await loadCurrentAccountId()
    window.$notify?.success($t('common.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh accounts:', error)
    window.$notify?.error($t('common.refreshFailed'))
  } finally {
    isRefreshing.value = false
  }
}

// 切换账户
const handleSwitch = async (accountId) => {
  const account = accounts.value.find(a => a.id === accountId)
  if (!account) return

  isSwitching.value = true
  try {
    await invoke('claude_switch_account', { accountId })
    currentAccountId.value = accountId
    window.$notify?.success($t('platform.claude.messages.switchSuccess'))
  } catch (error) {
    console.error('Failed to switch account:', error)
    window.$notify?.error($t('platform.claude.messages.switchFailed'))
  } finally {
    isSwitching.value = false
  }
}

// 保存视图模式
const saveViewMode = () => {
  localStorage.setItem(VIEW_MODE_KEY, viewMode.value)
}

// 加载视图模式
const loadViewMode = () => {
  const saved = localStorage.getItem(VIEW_MODE_KEY)
  if (saved === 'card' || saved === 'table') {
    viewMode.value = saved
  }
}

// 生成唯一ID
const generateId = () => `claude_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

// 保存账户（新增或编辑）
const handleSave = async (formData) => {
  const now = Math.floor(Date.now() / 1000)

  try {
    if (editingAccount.value) {
      // 编辑模式
      const updated = {
        ...editingAccount.value,
        ...formData,
        updated_at: now
      }

      await invoke('claude_update', { account: updated })

      const index = accounts.value.findIndex(a => a.id === editingAccount.value.id)
      if (index !== -1) {
        accounts.value[index] = updated
      }
      markItemUpsert(updated)
      window.$notify?.success($t('platform.claude.messages.updateSuccess'))
    } else {
      // 新增模式
      const newAccount = {
        id: generateId(),
        ...formData,
        created_at: now,
        updated_at: now,
        version: 0,
        deleted: false
      }

      await invoke('claude_add', { account: newAccount })

      accounts.value.unshift(newAccount)
      markItemUpsert(newAccount)
      window.$notify?.success($t('platform.claude.messages.addSuccess'))
    }
    closeDialog()
  } catch (error) {
    console.error('Failed to save account:', error)
    window.$notify?.error($t('platform.claude.messages.saveFailed'))
  }
}

// 编辑账户
const handleEdit = (account) => {
  editingAccount.value = account
}

// 删除账户
const handleDelete = async (account) => {
  const confirmed = await window.$confirm?.({
    title: $t('platform.claude.deleteConfirm.title'),
    message: $t('platform.claude.deleteConfirm.message', { name: account.service_name }),
    confirmText: $t('common.delete'),
    cancelText: $t('common.cancel'),
    type: 'danger'
  })
  if (confirmed) {
    try {
      await invoke('claude_delete', { id: account.id })

      accounts.value = accounts.value.filter(a => a.id !== account.id)
      markItemDeletion(account)
      window.$notify?.success($t('platform.claude.messages.deleteSuccess'))
    } catch (error) {
      console.error('Failed to delete account:', error)
      window.$notify?.error($t('platform.claude.messages.deleteFailed'))
    }
  }
}

// 关闭弹窗
const closeDialog = () => {
  showAddDialog.value = false
  editingAccount.value = null
}

// 批量操作方法
const toggleAccountSelection = (accountId) => {
  const newSet = new Set(selectedAccountIds.value)
  if (newSet.has(accountId)) {
    newSet.delete(accountId)
  } else {
    newSet.add(accountId)
  }
  selectedAccountIds.value = newSet
}

const toggleSelectAll = () => {
  const currentIds = paginatedAccounts.value.map(a => a.id)
  const allSelected = currentIds.every(id => selectedAccountIds.value.has(id))

  const newSet = new Set(selectedAccountIds.value)
  if (allSelected) {
    currentIds.forEach(id => newSet.delete(id))
  } else {
    currentIds.forEach(id => newSet.add(id))
  }
  selectedAccountIds.value = newSet
}

const selectAllOnPage = () => {
  const newSet = new Set(selectedAccountIds.value)
  paginatedAccounts.value.forEach(a => newSet.add(a.id))
  selectedAccountIds.value = newSet
}

const clearSelection = () => {
  selectedAccountIds.value = new Set()
}

const showBatchDeleteConfirm = async () => {
  const confirmed = await window.$confirm?.({
    title: $t('platform.claude.batchDeleteConfirm.title'),
    message: $t('platform.claude.batchDeleteConfirm.message', { count: selectedAccountIds.value.size }),
    confirmText: $t('common.delete'),
    cancelText: $t('common.cancel'),
    type: 'danger'
  })
  if (confirmed) {
    try {
      for (const accountId of selectedAccountIds.value) {
        const account = accounts.value.find(a => a.id === accountId)
        await invoke('claude_delete', { id: accountId })
        if (account) {
          markItemDeletion(account)
        }
      }
      selectedAccountIds.value = new Set()
      await loadAccounts()
      window.$notify?.success($t('platform.claude.messages.batchDeleteSuccess'))
    } catch (error) {
      console.error('Failed to batch delete accounts:', error)
      window.$notify?.error($t('platform.claude.messages.deleteFailed'))
    }
  }
}

const handleBatchTagSave = async ({ tagName, tagColor }) => {
  if (selectedAccountIds.value.size === 0) return

  const selectedIds = Array.from(selectedAccountIds.value)
  let updatedCount = 0

  for (const accountId of selectedIds) {
    const account = accounts.value.find(a => a.id === accountId)
    if (account) {
      account.tag = tagName
      account.tag_color = tagColor
      account.updated_at = Math.floor(Date.now() / 1000)
      updatedCount++
      markItemUpsert(account)
    }
  }

  try {
    for (const accountId of selectedIds) {
      const account = accounts.value.find(a => a.id === accountId)
      if (account) {
        await invoke('claude_update', { account })
      }
    }
  } catch (error) {
    console.error('Failed to save accounts:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagUpdated', { count: updatedCount }))
}

const handleBatchTagClear = async () => {
  if (selectedAccountIds.value.size === 0) return

  const selectedIds = Array.from(selectedAccountIds.value)
  let clearedCount = 0

  for (const accountId of selectedIds) {
    const account = accounts.value.find(a => a.id === accountId)
    if (account) {
      account.tag = ''
      account.tag_color = ''
      account.updated_at = Math.floor(Date.now() / 1000)
      clearedCount++
      markItemUpsert(account)
    }
  }

  try {
    for (const accountId of selectedIds) {
      const account = accounts.value.find(a => a.id === accountId)
      if (account) {
        await invoke('claude_update', { account })
      }
    }
  } catch (error) {
    console.error('Failed to save accounts:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagCleared', { count: clearedCount }))
}

const handleAccountUpdated = async (updatedAccount) => {
  if (!updatedAccount?.id) return
  try {
    const index = accounts.value.findIndex(a => a.id === updatedAccount.id)
    if (index !== -1) {
      accounts.value[index] = { ...accounts.value[index], ...updatedAccount }
    }
    await invoke('claude_update', { account: updatedAccount })
    markItemUpsert(updatedAccount)
  } catch (error) {
    console.error('Failed to update account:', error)
    window.$notify?.error($t('messages.updateFailed'))
  }
}

const handlePageContentClick = () => {
  // 工具栏会自动处理点击外部关闭
}

// 监听搜索和筛选变化，重置分页
watch([searchQuery, selectedExpiryFilter], () => {
  currentPage.value = 1
})

onMounted(async () => {
  loadViewMode()
  await loadAccounts()
})
</script>
