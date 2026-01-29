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
              v-tooltip="isDatabaseAvailable ? $t('platform.openai.viewSyncQueueTooltip') : ''"
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
                :class="{ 'active': viewMode === 'table' }"
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
            <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('platform.openai.addAccount')">
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
              @click="handleRefreshCurrentPageQuota"
              :disabled="isRefreshingAll"
              v-tooltip="$t('platform.openai.refreshAllQuota')"
            >
              <svg v-if="!isRefreshingAll" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
            </button>
            <button class="btn btn--icon btn--ghost" @click="setToolbarMode('more')" v-tooltip="'更多'">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 7a2 2 0 110-4 2 2 0 010 4zm0 7a2 2 0 110-4 2 2 0 010 4zm0 7a2 2 0 110-4 2 2 0 010 4z"/>
              </svg>
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
          <img src="/icons/openai.svg" alt="OpenAI" width="64" height="64" />
        </div>
        <p class="mt-4 text-sm">{{ $t('common.noSearchResults') }}</p>
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
            :is-switching="switchingAccountId === account.id"
            :is-refreshing="refreshingIds.has(account.id)"
            :is-deleting="deletingIds.has(account.id)"
            :is-selected="selectedAccountIds.has(account.id)"
            :selection-mode="isSelectionMode"
            :show-real-email="showRealEmail"
            :all-accounts="accounts"
            @switch="handleSwitch"
            @refresh="handleRefreshToken"
            @refresh-quota="handleRefreshQuota"
            @delete="handleDelete"
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
                  <div class="inline-flex items-center justify-center h-5 cursor-pointer" @click="toggleSelectAll">
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
                <th class="th w-[60px]">{{ $t('platform.openai.table.tag') }}</th>
                <th class="th w-[60px]">{{ $t('platform.openai.table.status') }}</th>
                <th class="th">{{ $t('platform.openai.table.email') }}</th>
                <th class="th w-[140px]">{{ $t('platform.openai.table.time') }}</th>
                <th class="th">{{ $t('platform.openai.table.quota') }}</th>
                <th class="th w-[110px] text-center">{{ $t('platform.openai.table.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <AccountTableRow
                v-for="account in paginatedAccounts"
                :key="account.id"
                :account="account"
                :is-current="account.id === currentAccountId"
                :is-switching="switchingAccountId === account.id"
                :is-refreshing="refreshingIds.has(account.id)"
                :is-deleting="deletingIds.has(account.id)"
                :is-selected="selectedAccountIds.has(account.id)"
                :selection-mode="isSelectionMode"
                :show-real-email="showRealEmail"
                @switch="handleSwitch"
                @refresh="handleRefreshToken"
                @refresh-quota="handleRefreshQuota"
                @delete="handleDelete"
                @select="toggleAccountSelection"
                @account-updated="handleAccountUpdated"
              />
            </tbody>
          </table>
        </div>
      </template>
      <template #pagination>
        <Pagination
          :current-page="currentPage"
          :total-pages="totalPages"
          :total-items="filteredAccounts.length"
          :page-size="pageSize"
          :page-size-options="pageSizeOptions"
          @update:current-page="handlePageChange"
          @update:page-size="changePageSize"
        />
      </template>
    </FixedPaginationLayout>

    <!-- 搜索工具栏 -->
    <ActionToolbar
      :visible="toolbarMode === 'search'"
      :title="$t('common.search')"
      @close="setToolbarMode('hidden')">
      <div class="flex items-center gap-3 w-full px-4 h-10 border border-border rounded-lg bg-surface transition-all duration-200 focus-within:border-accent">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="shrink-0 text-text transition-colors duration-200">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5z" />
        </svg>
        <input
          ref="toolbarSearchInputRef"
          type="text"
          v-model="searchQuery"
          :placeholder="$t('platform.openai.searchPlaceholder')"
          class="flex-1 border-0 outline-none bg-transparent text-text text-sm placeholder:text-text-muted placeholder:opacity-60 p-0 m-0 h-full"
        />
        <button v-if="searchQuery.trim()" @click="searchQuery = ''" class="btn btn--icon-sm btn--ghost shrink-0">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
          </svg>
        </button>
      </div>
    </ActionToolbar>

    <!-- 筛选工具栏 -->
    <ActionToolbar
      :visible="toolbarMode === 'filter'"
      :title="$t('common.filter')"
      max-width="700px"
      @close="setToolbarMode('hidden')">
      <div class="flex flex-col gap-4">
        <!-- 状态筛选 -->
        <div class="flex flex-col gap-2">
          <span class="label">{{ $t('tokenList.filterByStatus') }}</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="[
                'btn btn--sm',
                selectedStatusFilter === null ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter(null)"
            >
              <span>{{ $t('platform.openai.filter.all') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.total }})</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                selectedStatusFilter === 'active' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter('active')"
            >
              <span>{{ $t('platform.openai.filter.active') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.active }})</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                selectedStatusFilter === 'expired' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter('expired')"
            >
              <span>{{ $t('platform.openai.filter.expired') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.expired }})</span>
            </button>
          </div>
        </div>
      </div>
    </ActionToolbar>

    <!-- 排序工具栏 -->
    <ActionToolbar
      :visible="toolbarMode === 'sort'"
      :title="$t('common.sort')"
      @close="setToolbarMode('hidden')">
      <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <!-- 排序方向 -->
        <div class="flex flex-col gap-2">
          <span class="label">排序方向</span>
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
              <span class="ml-1">升序</span>
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
              <span class="ml-1">降序</span>
            </button>
          </div>
        </div>

        <!-- 排序字段 -->
        <div class="flex flex-col gap-2">
          <span class="label">排序字段</span>
          <div class="flex items-center gap-2 flex-wrap">
            <button
              :class="[
                'btn btn--sm',
                sortType === 'time' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('time', sortOrder)"
            >
              {{ $t('common.sortByTime') }}
            </button>
            <button
              :class="[
                'btn btn--sm',
                sortType === 'email' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('email', sortOrder)"
            >
              按邮箱
            </button>
          </div>
        </div>
      </div>
    </ActionToolbar>

    <!-- 更多操作工具栏 -->
    <ActionToolbar
      :visible="toolbarMode === 'more'"
      title="更多"
      max-width="800px"
      @close="setToolbarMode('hidden')">
      <div class="flex flex-wrap items-center gap-2">
        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="showRealEmail = !showRealEmail"
          :class="{ 'active': showRealEmail }"
          v-tooltip="showRealEmail ? $t('tokenList.hideRealEmail') : $t('tokenList.showRealEmail')"
        >
          <svg v-if="showRealEmail" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z"/>
          </svg>
          <span>{{ showRealEmail ? $t('tokenList.hideRealEmail') : $t('tokenList.showRealEmail') }}</span>
        </button>

        <!-- 导出账号 -->
        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="exportAccounts"
          v-tooltip="$t('platform.openai.export')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
          </svg>
          <span>{{ $t('platform.openai.export') }}</span>
        </button>
      </div>
    </ActionToolbar>

    <!-- 批量操作工具栏 -->
    <BatchToolbar
      :visible="isSelectionMode"
      :selected-count="selectedAccountIds.size"
      @select-all="selectAllOnPage"
      @clear="clearSelection"
    >
      <template #actions>
        <!-- 批量刷新 Token -->
        <button
          @click="batchRefreshTokensSelected"
          class="btn btn--icon btn--ghost"
          :disabled="isBatchRefreshing"
          v-tooltip="$t('platform.openai.batchRefreshToken')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
          </svg>
        </button>

        <!-- 批量刷新配额 -->
        <button
          @click="batchRefreshSelected"
          class="btn btn--icon btn--ghost"
          :disabled="isBatchRefreshing"
          v-tooltip="$t('platform.openai.batchRefresh')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
          </svg>
        </button>

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
          @click="showBatchDeleteSelectedConfirm"
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

    <!-- Add Account Dialog -->
    <AddAccountDialog v-if="showAddDialog" @close="showAddDialog = false" @added="handleAccountAdded" />

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="accounts.length"
      :title="$t('platform.openai.syncQueueTitle')"
      :upserts-title="$t('platform.openai.syncQueueUpsertsTitle')"
      :deletions-title="$t('platform.openai.syncQueueDeletionsTitle')"
      :empty-text="$t('platform.openai.syncQueueEmpty')"
      :full-sync-text="$t('platform.openai.fullSync')"
      :sync-text="$t('platform.openai.sync')"
      label-field="email"
      fallback-field="id"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { downloadDir } from '@tauri-apps/api/path'
import { useI18n } from 'vue-i18n'
import AccountCard from '../openai/AccountCard.vue'
import AccountTableRow from '../openai/AccountTableRow.vue'
import AddAccountDialog from '../openai/AddAccountDialog.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import Pagination from '../common/Pagination.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import ActionToolbar from '../common/ActionToolbar.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'
import TagEditorModal from '../token/TagEditorModal.vue'
import { useStorageSync } from '@/composables/useStorageSync'

const { t: $t } = useI18n()

// Props
const props = defineProps({
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})

// 基础状态
const accounts = ref([])
const currentAccountId = ref(null)
const showAddDialog = ref(false)
const isLoading = ref(false)
const refreshingIds = ref(new Set())
const deletingIds = ref(new Set())
const switchingAccountId = ref(null)

// 使用存储同步 composable
const {
  isDatabaseAvailable,
  isStorageInitializing,
  isSyncing,
  isSyncNeeded,
  isLoadingFromSync,
  hasPendingChanges,
  pendingUpsertsList,
  pendingDeletionsList,
  storageStatusText,
  storageStatusClass,
  showSyncQueueModal,
  initSync,
  markItemUpsert,
  markItemDeletion,
  markItemUpsertById,
  markAllForSync,
  openSyncQueue,
  closeSyncQueue,
  handleSync
} = useStorageSync({
  platform: 'openai',
  syncCommand: 'openai_sync_accounts',
  items: accounts,
  itemKey: 'account',
  labelField: 'email',
  onSyncComplete: async () => {
    await loadAccounts()
  }
})

// 搜索和筛选
const searchQuery = ref('')
const selectedStatusFilter = ref(null)
const toolbarMode = ref('hidden')
const toolbarSearchInputRef = ref(null)

// 排序
const sortType = ref('time')
const sortOrder = ref('desc')

// 视图模式
const viewMode = ref('card')

// 邮箱显示模式
const showRealEmail = ref(true)

// 分页
const currentPage = ref(1)
const pageSize = ref(20)
const pageSizeOptions = [10, 20, 50, 100, 200]

// 批量操作
const selectedAccountIds = ref(new Set())
const isBatchRefreshing = ref(false)
const isRefreshingAll = ref(false)
const showBatchTagEditor = ref(false)

// 计算属性
const isSelectionMode = computed(() => selectedAccountIds.value.size > 0)

// 选中的账户列表（用于批量编辑标签）
const selectedAccounts = computed(() => {
  return accounts.value
    .filter(a => selectedAccountIds.value.has(a.id))
    .map(acc => ({
      id: acc.id,
      tag_name: acc.tag || '',
      tag_color: acc.tag_color || ''
    }))
})

// 所有账户转换为 TagEditorModal 需要的格式
const allAccountsAsTokens = computed(() =>
  accounts.value.map(acc => ({
    tag_name: acc.tag || '',
    tag_color: acc.tag_color || ''
  }))
)

// 状态判断
const isAccountActive = (account) => {
  if (!account.token) return false
  if (account.token.expires_at) {
    const now = Math.floor(Date.now() / 1000)
    return account.token.expires_at > now
  }
  return true
}

const statusStatistics = computed(() => {
  const stats = {
    total: accounts.value.length,
    active: 0,
    expired: 0
  }

  accounts.value.forEach(account => {
    if (isAccountActive(account)) {
      stats.active++
    } else {
      stats.expired++
    }
  })

  return stats
})

const filteredAccounts = computed(() => {
  let result = accounts.value

  // 搜索过滤
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(a => a.email.toLowerCase().includes(query))
  }

  // 状态筛选
  if (selectedStatusFilter.value) {
    result = result.filter(account => {
      if (selectedStatusFilter.value === 'active') {
        return isAccountActive(account)
      } else if (selectedStatusFilter.value === 'expired') {
        return !isAccountActive(account)
      }
      return true
    })
  }

  // 排序（当前账号置顶，其余按设置排序）
  result = [...result].sort((a, b) => {
    const currentId = currentAccountId.value
    if (currentId) {
      const aIsCurrent = a.id === currentId
      const bIsCurrent = b.id === currentId
      if (aIsCurrent !== bIsCurrent) {
        return aIsCurrent ? -1 : 1
      }
    }

    if (sortType.value === 'time') {
      const timeA = a.created_at
      const timeB = b.created_at
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'email') {
      return sortOrder.value === 'desc'
        ? b.email.localeCompare(a.email)
        : a.email.localeCompare(b.email)
    }
    return 0
  })

  return result
})

const totalPages = computed(() => Math.ceil(filteredAccounts.value.length / pageSize.value))

const paginatedAccounts = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredAccounts.value.slice(start, start + pageSize.value)
})

const isAllSelected = computed(() => {
  return paginatedAccounts.value.length > 0 && paginatedAccounts.value.every(a => selectedAccountIds.value.has(a.id))
})

const isPartialSelected = computed(() => {
  return selectedAccountIds.value.size > 0 && !isAllSelected.value
})

const showEmptyState = computed(() => !isLoading.value && filteredAccounts.value.length === 0)
const shouldShowPagination = computed(() => !isLoading.value && accounts.value.length > 0 && filteredAccounts.value.length > 0)

// 方法
const loadAccounts = async () => {
  isLoading.value = true
  try {
    const jsonString = await invoke('openai_load_accounts_json')
    const data = JSON.parse(jsonString)
    accounts.value = data.accounts || []
    currentAccountId.value = data.current_account_id || null
  } catch (error) {
    console.error('Failed to load accounts:', error)
    accounts.value = []
    currentAccountId.value = null
  } finally {
    isLoading.value = false
  }
}

const handleRefresh = async (accountId) => {
  refreshingIds.value.add(accountId)
  try {
    await invoke('openai_refresh_account', { accountId })
    await loadAccounts()
    markItemUpsertById(accountId)
    window.$notify?.success($t('platform.openai.messages.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh account:', error)
    window.$notify?.error($t('platform.openai.messages.refreshFailed', { error: error?.message || error }))
  } finally {
    refreshingIds.value.delete(accountId)
  }
}

// 刷新配额
const handleRefreshQuota = async (accountId) => {
  refreshingIds.value.add(accountId)
  try {
    const updatedAccount = await invoke('openai_fetch_quota', { accountId })
    // 直接更新本地数组中的账号数据
    const index = accounts.value.findIndex(a => a.id === accountId)
    if (index !== -1) {
      accounts.value[index] = updatedAccount
    }
    markItemUpsertById(accountId)
    window.$notify?.success($t('platform.openai.messages.quotaRefreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh quota:', error)
    window.$notify?.error($t('platform.openai.messages.quotaRefreshFailed', { error: error?.message || error }))
  } finally {
    refreshingIds.value.delete(accountId)
  }
}

// 刷新当前页所有配额
const handleRefreshCurrentPageQuota = async () => {
  isRefreshingAll.value = true
  try {
    for (const account of paginatedAccounts.value) {
      await handleRefreshQuota(account.id)
    }
  } finally {
    isRefreshingAll.value = false
  }
}

// 刷新 Token
const handleRefreshToken = async (accountId) => {
  refreshingIds.value.add(accountId)
  try {
    const updatedAccount = await invoke('openai_refresh_account', { accountId })
    // 直接更新本地数组中的账号数据
    const index = accounts.value.findIndex(a => a.id === accountId)
    if (index !== -1) {
      accounts.value[index] = updatedAccount
    }
    markItemUpsertById(accountId)
    window.$notify?.success($t('platform.openai.messages.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh account:', error)
    window.$notify?.error($t('platform.openai.messages.refreshFailed', { error: error?.message || error }))
  } finally {
    refreshingIds.value.delete(accountId)
  }
}

const handleSwitch = async (accountId) => {
  switchingAccountId.value = accountId
  try {
    await invoke('openai_switch_account', { accountId })
    // 直接更新当前账号 ID
    currentAccountId.value = accountId
    markItemUpsertById(accountId)
    window.$notify?.success($t('platform.openai.messages.switchSuccess'))
  } catch (error) {
    console.error('Failed to switch account:', error)
    window.$notify?.error($t('platform.openai.messages.switchFailed', { error: error?.message || error }))
  } finally {
    switchingAccountId.value = null
  }
}

const handleAccountAdded = async (account) => {
  showAddDialog.value = false
  await loadAccounts()
  if (account?.id) {
    markItemUpsertById(account.id)
  }
  window.$notify?.success($t('platform.openai.messages.addSuccess'))
}

const handleDelete = async (accountId) => {
  const account = accounts.value.find(a => a.id === accountId)
  deletingIds.value.add(accountId)
  try {
    await invoke('openai_delete_account', { accountId })
    if (account) {
      markItemDeletion(account)
    }
    await loadAccounts()
    window.$notify?.success($t('platform.openai.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to delete account:', error)
    window.$notify?.error($t('platform.openai.messages.deleteFailed', { error: error?.message || error }))
  } finally {
    deletingIds.value.delete(accountId)
  }
}

// 工具栏模式切换
const setToolbarMode = (mode) => {
  toolbarMode.value = toolbarMode.value === mode ? 'hidden' : mode
  if (toolbarMode.value === 'search') {
    nextTick(() => toolbarSearchInputRef.value?.focus())
  }
}

// 筛选和排序
const selectStatusFilter = (filter) => {
  selectedStatusFilter.value = filter
  currentPage.value = 1
}

const setSortType = (type, order) => {
  sortType.value = type
  sortOrder.value = order
}

const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'card' ? 'table' : 'card'
  currentPage.value = 1
}

// 导出账号
const exportAccounts = async () => {
  try {
    const dataStr = JSON.stringify(accounts.value, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)
    const link = document.createElement('a')
    link.href = url
    link.download = `openai-accounts-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)

    // 获取下载文件夹路径
    const downloadsPath = await downloadDir()
    window.$notify.success($t('platform.openai.exportSuccess') + '\n' + $t('platform.openai.fileSavedIn', { path: downloadsPath }))
  } catch (error) {
    console.error('Export failed:', error)
    window.$notify.error($t('platform.openai.exportFailed'))
  }
}

// 处理页面内容点击
const handlePageContentClick = () => {
  // 工具栏会自动处理点击外部关闭
}

// 分页
const handlePageChange = (page) => {
  currentPage.value = page
}

const changePageSize = (size) => {
  pageSize.value = size
  currentPage.value = 1
}

// 批量操作
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

const batchRefreshSelected = async () => {
  isBatchRefreshing.value = true
  try {
    for (const accountId of selectedAccountIds.value) {
      await handleRefreshQuota(accountId)
    }
  } finally {
    isBatchRefreshing.value = false
  }
}

const batchRefreshTokensSelected = async () => {
  isBatchRefreshing.value = true
  try {
    for (const accountId of selectedAccountIds.value) {
      await handleRefreshToken(accountId)
    }
    window.$notify?.success($t('platform.openai.batchRefreshSuccess', { count: selectedAccountIds.value.size }))
  } finally {
    isBatchRefreshing.value = false
  }
}

const showBatchDeleteSelectedConfirm = () => {
  handleBatchDeleteSelected()
}

const handleBatchDeleteSelected = async () => {
  try {
    for (const accountId of selectedAccountIds.value) {
      const account = accounts.value.find(a => a.id === accountId)
      await invoke('openai_delete_account', { accountId })
      if (account) {
        markItemDeletion(account)
      }
    }
    selectedAccountIds.value = new Set()
    await loadAccounts()
    window.$notify?.success($t('platform.openai.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to batch delete accounts:', error)
    window.$notify?.error($t('platform.openai.messages.deleteFailed', { error: error?.message || error }))
  }
}

const handleMarkAllForSync = () => {
  markAllForSync()
  window.$notify.success($t('platform.openai.allAccountsMarkedForSync', { count: accounts.value.length }))
}

// 批量编辑标签 - 保存
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

  // 保存更改到本地
  try {
    await invoke('openai_save_accounts', { accounts: accounts.value })
  } catch (error) {
    console.error('Failed to save accounts:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagUpdated', { count: updatedCount }))
}

// 批量编辑标签 - 清除
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

  // 保存更改到本地
  try {
    await invoke('openai_save_accounts', { accounts: accounts.value })
  } catch (error) {
    console.error('Failed to save accounts:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagCleared', { count: clearedCount }))
}

// 账户更新处理（标签等属性）
const handleAccountUpdated = async (updatedAccount) => {
  if (!updatedAccount?.id) return
  try {
    const index = accounts.value.findIndex(a => a.id === updatedAccount.id)
    if (index !== -1) {
      accounts.value[index] = { ...accounts.value[index], ...updatedAccount }
    }
    await invoke('openai_update_account', { account: updatedAccount })
    markItemUpsert(updatedAccount)
  } catch (error) {
    console.error('Failed to update account:', error)
    window.$notify?.error($t('messages.updateFailed'))
  }
}

// 监听搜索和筛选变化，重置分页
watch([searchQuery, selectedStatusFilter], () => {
  currentPage.value = 1
})

onMounted(async () => {
  await initSync()
  await loadAccounts()
})
</script>
