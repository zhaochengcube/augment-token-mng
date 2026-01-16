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
              v-tooltip="isDatabaseAvailable ? $t('platform.antigravity.viewSyncQueueTooltip') : ''"
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
            <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('platform.antigravity.addAccount')">
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
              <div v-else class="h-4 w-4 border-2 border-accent/30 border-t-accent rounded-full animate-spin"></div>
            </button>
            <button
              class="btn btn--icon btn--ghost"
              @click="handleRefresh"
              :disabled="isRefreshing"
              v-tooltip="$t('platform.antigravity.refreshQuota')"
            >
              <svg v-if="!isRefreshing" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="animate-spin">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
              </svg>
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
          <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
          </svg>
        </div>
        <p class="mt-4 text-sm">{{ $t('common.noSearchResults') }}</p>
      </div>

      <!-- Account List -->
      <template v-else>
        <!-- 卡片布局 -->
        <div v-if="viewMode === 'card'" class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4 p-1">
          <AccountCard
            v-for="account in paginatedAccounts"
            :key="account.id"
            :account="account"
            :is-current="account.id === currentAccountId"
            :is-switching="switchingAccountId === account.id"
            :is-refreshing="refreshingIds.has(account.id)"
            :is-selected="selectedAccountIds.has(account.id)"
            :selection-mode="isSelectionMode"
            :show-real-email="showRealEmail"
            @switch="handleSwitch"
            @refresh="handleRefreshQuota"
            @delete="handleDelete"
            @select="toggleAccountSelection"
            @view-models="openModelsModal"
          />
        </div>

        <!-- 列表布局 -->
        <div v-else class="table-container">
          <table class="table">
            <thead>
              <tr>
                <th class="th w-11 text-center">
                  <div class="inline-flex cursor-pointer" @click="toggleSelectAll">
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
                <th class="th w-[220px]">{{ $t('platform.antigravity.table.info') }}</th>
                <th class="th">{{ $t('platform.antigravity.table.quota') }}</th>
                <th class="th w-[88px] text-center">{{ $t('platform.antigravity.table.actions') }}</th>
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
                :is-selected="selectedAccountIds.has(account.id)"
                :selection-mode="isSelectionMode"
                :show-real-email="showRealEmail"
                @switch="handleSwitch"
                @refresh="handleRefreshQuota"
                @delete="handleDelete"
                @select="toggleAccountSelection"
                @view-models="openModelsModal"
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
          :placeholder="$t('platform.antigravity.searchPlaceholder')"
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
              <span>{{ $t('platform.antigravity.filter.all') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.total }})</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                selectedStatusFilter === 'available' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter('available')"
            >
              <span>{{ $t('platform.antigravity.filter.available') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.available }})</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                selectedStatusFilter === 'low' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter('low')"
            >
              <span>{{ $t('platform.antigravity.filter.low') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.low }})</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                selectedStatusFilter === 'forbidden' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter('forbidden')"
            >
              <span>{{ $t('platform.antigravity.filter.forbidden') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.forbidden }})</span>
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
                sortType === 'quota' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('quota', sortOrder)"
            >
              {{ $t('platform.antigravity.sortByQuota') }}
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

        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="openDataFolder"
          v-tooltip="$t('common.openDataFolder')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z" />
          </svg>
          <span>{{ $t('common.openDataFolder') }}</span>
        </button>

        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="handleBatchDelete"
          v-tooltip="$t('common.batchDelete')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
          </svg>
          <span>{{ $t('common.batchDelete') }}</span>
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
        <!-- 批量刷新 -->
        <button
          @click="batchRefreshSelected"
          class="btn btn--icon btn--ghost"
          :disabled="isBatchRefreshing"
          v-tooltip="$t('platform.antigravity.batchRefresh')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
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

    <!-- Add Account Dialog -->
    <AddAccountDialog v-if="showAddDialog" @close="showAddDialog = false" @add="handleAddAccount" @added="handleAccountAdded" />
    <ModelsModal
      v-if="showModelsModal"
      :visible="showModelsModal"
      :account="activeModelsAccount"
      :refreshing="activeModelsAccount ? refreshingIds.has(activeModelsAccount.id) : false"
      @close="closeModelsModal"
      @refresh="refreshModelsModal"
    />
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="accounts.length"
      :title="$t('platform.antigravity.syncQueueTitle')"
      :upserts-title="$t('platform.antigravity.syncQueueUpsertsTitle')"
      :deletions-title="$t('platform.antigravity.syncQueueDeletionsTitle')"
      :empty-text="$t('platform.antigravity.syncQueueEmpty')"
      :full-sync-text="$t('platform.antigravity.fullSync')"
      :sync-text="$t('platform.antigravity.sync')"
      label-field="email"
      fallback-field="id"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import AccountCard from '../antigravity/AccountCard.vue'
import AccountTableRow from '../antigravity/AccountTableRow.vue'
import AddAccountDialog from '../antigravity/AddAccountDialog.vue'
import ModelsModal from '../antigravity/ModelsModal.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import Pagination from '../common/Pagination.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import ActionToolbar from '../common/ActionToolbar.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'

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
const isRefreshing = ref(false)
const switchingAccountId = ref(null)
const refreshingIds = ref(new Set())
const isDatabaseAvailable = ref(false)
const isStorageInitializing = ref(false)
const isSyncing = ref(false)
const isSyncNeeded = ref(false)
const isLoadingFromSync = ref(false)
const showModelsModal = ref(false)
const activeModelsAccount = ref(null)
const showSyncQueueModal = ref(false)

const STORAGE_KEY_LAST_VERSION = 'atm-antigravity-sync-last-version'
const STORAGE_KEY_PENDING_UPSERTS = 'atm-antigravity-sync-pending-upserts'
const STORAGE_KEY_PENDING_DELETIONS = 'atm-antigravity-sync-pending-deletions'

const lastVersion = ref(0)
const pendingUpserts = ref(new Map())
const pendingDeletions = ref(new Map())

const hasPendingChanges = computed(() => pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0)
const pendingUpsertsList = computed(() => Array.from(pendingUpserts.value.values()))
const pendingDeletionsList = computed(() => Array.from(pendingDeletions.value.values()))

// 搜索和筛选
const searchQuery = ref('')
const selectedStatusFilter = ref(null)
const toolbarMode = ref('hidden') // 'hidden', 'search', 'filter', 'sort', 'more'
const toolbarSearchInputRef = ref(null)
const statusOptions = ['available', 'low', 'forbidden']

// 排序
const sortType = ref('time')
const sortOrder = ref('desc')

// 视图模式
const viewMode = ref('card')

// 邮箱显示模式
const showRealEmail = ref(true) // false = 脱敏显示, true = 真实邮箱

// 分页
const currentPage = ref(1)
const pageSize = ref(20)
const pageSizeOptions = [10, 20, 50, 100, 200]

// 批量操作
const selectedAccountIds = ref(new Set())
const isBatchRefreshing = ref(false)

// 计算属性
const isSelectionMode = computed(() => selectedAccountIds.value.size > 0)

const statusStatistics = computed(() => {
  const stats = {
    total: accounts.value.length,
    available: 0,
    low: 0,
    forbidden: 0
  }

  accounts.value.forEach(account => {
    if (account.quota?.is_forbidden) {
      stats.forbidden++
    } else {
      const gemini = account.quota?.models.find(m => m.name.toLowerCase().includes('gemini'))?.percentage || 0
      const claude = account.quota?.models.find(m => m.name.toLowerCase().includes('claude'))?.percentage || 0

      if (gemini >= 20 && claude >= 20) {
        stats.available++
      } else if (gemini < 20 || claude < 20) {
        stats.low++
      }
    }
  })

  return stats
})

const storageStatusText = computed(() => {
  if (isStorageInitializing.value) {
    return $t('storage.initializing')
  }

  if (isDatabaseAvailable.value) {
    return hasPendingChanges.value
      ? `${$t('storage.dualStorage')}-${$t('storage.notSynced')}`
      : $t('storage.dualStorage')
  }

  return $t('storage.localStorage')
})

const storageStatusClass = computed(() => {
  if (isStorageInitializing.value) {
    return 'badge--accent-tech'
  }

  if (isDatabaseAvailable.value && hasPendingChanges.value) {
    return 'badge--warning-tech'
  }

  return 'badge--success-tech'
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
      if (selectedStatusFilter.value === 'forbidden') {
        return account.quota?.is_forbidden
      }

      const gemini = account.quota?.models.find(m => m.name.toLowerCase().includes('gemini'))?.percentage || 0
      const claude = account.quota?.models.find(m => m.name.toLowerCase().includes('claude'))?.percentage || 0

      if (selectedStatusFilter.value === 'available') {
        return !account.quota?.is_forbidden && gemini >= 20 && claude >= 20
      } else if (selectedStatusFilter.value === 'low') {
        return !account.quota?.is_forbidden && (gemini < 20 || claude < 20)
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
      const timeA = a.last_used || a.created_at
      const timeB = b.last_used || b.created_at
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'quota') {
      const getAvgQuota = (account) => {
        if (!account.quota || account.quota.is_forbidden) return 0
        const gemini = account.quota.models.find(m => m.name.toLowerCase().includes('gemini'))?.percentage || 0
        const claude = account.quota.models.find(m => m.name.toLowerCase().includes('claude'))?.percentage || 0
        return (gemini + claude) / 2
      }
      const quotaA = getAvgQuota(a)
      const quotaB = getAvgQuota(b)
      return sortOrder.value === 'desc' ? quotaB - quotaA : quotaA - quotaB
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
    // 直接从本地文件加载,不触发 storage manager 初始化
    const jsonString = await invoke('antigravity_load_accounts_json')
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

const loadLastVersion = () => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_LAST_VERSION)
    if (stored) {
      const version = parseInt(stored, 10)
      if (!isNaN(version) && version >= 0) {
        return version
      }
    }
  } catch (error) {
    console.warn('Failed to load Antigravity lastVersion:', error)
  }
  return 0
}

const saveLastVersion = (version) => {
  try {
    localStorage.setItem(STORAGE_KEY_LAST_VERSION, version.toString())
  } catch (error) {
    console.error('Failed to save Antigravity lastVersion:', error)
  }
}

const savePendingChanges = () => {
  try {
    const upsertsArr = Array.from(pendingUpserts.value.entries()).map(([id, account]) => ({ id, account }))
    const deletionsArr = Array.from(pendingDeletions.value.values())

    localStorage.setItem(STORAGE_KEY_PENDING_UPSERTS, JSON.stringify(upsertsArr))
    localStorage.setItem(STORAGE_KEY_PENDING_DELETIONS, JSON.stringify(deletionsArr))
  } catch (error) {
    console.error('Failed to save Antigravity pending changes:', error)
  }
}

const loadPendingChanges = () => {
  try {
    const upsertsJson = localStorage.getItem(STORAGE_KEY_PENDING_UPSERTS)
    if (upsertsJson) {
      const arr = JSON.parse(upsertsJson)
      if (Array.isArray(arr)) {
        pendingUpserts.value = new Map(
          arr
            .filter(item => item && item.id && item.account)
            .map(item => [item.id, item.account])
        )
      }
    }

    const deletionsJson = localStorage.getItem(STORAGE_KEY_PENDING_DELETIONS)
    if (deletionsJson) {
      const arr = JSON.parse(deletionsJson)
      if (Array.isArray(arr)) {
        pendingDeletions.value = new Map(
          arr
            .filter(item => item && item.id)
            .map(item => [item.id, { id: item.id, email: item.email || null }])
        )
      }
    }

    if (pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0) {
      isSyncNeeded.value = true
    }
  } catch (error) {
    console.warn('Failed to load Antigravity pending changes:', error)
  }
}

let storageCheckTimer = null

const getStorageStatus = async () => {
  try {
    const status = await invoke('get_antigravity_storage_status')

    // 检查是否正在初始化
    if (status?.is_initializing) {
      isStorageInitializing.value = true
      isDatabaseAvailable.value = false

      // 启动定时器，每 500ms 检查一次
      if (!storageCheckTimer) {
        storageCheckTimer = setInterval(async () => {
          await getStorageStatus()
        }, 500)
      }
    } else {
      // 初始化完成
      isStorageInitializing.value = false
      isDatabaseAvailable.value = status?.is_database_available || false

      // 停止定时器
      if (storageCheckTimer) {
        clearInterval(storageCheckTimer)
        storageCheckTimer = null
      }
    }
  } catch (error) {
    console.error('Failed to get Antigravity storage status:', error)
    isDatabaseAvailable.value = false
    isStorageInitializing.value = false

    // 停止定时器
    if (storageCheckTimer) {
      clearInterval(storageCheckTimer)
      storageCheckTimer = null
    }
  }
}

const markAccountUpsert = (account) => {
  if (!account?.id) return
  pendingUpserts.value.set(account.id, account)
  pendingDeletions.value.delete(account.id)
  savePendingChanges()
  if (isDatabaseAvailable.value) {
    isSyncNeeded.value = true
  }
}

const markAccountDeletion = (account) => {
  if (!account?.id) return
  const wasOnlyLocal = pendingUpserts.value.has(account.id)
  pendingUpserts.value.delete(account.id)
  if (!wasOnlyLocal) {
    pendingDeletions.value.set(account.id, { id: account.id, email: account.email || null })
  } else {
    pendingDeletions.value.delete(account.id)
  }
  savePendingChanges()
  if (isDatabaseAvailable.value) {
    isSyncNeeded.value = pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0
  }
}

const markAccountUpsertById = (accountId) => {
  const account = accounts.value.find(a => a.id === accountId)
  if (account) {
    markAccountUpsert(account)
  }
}

const handleSwitch = async (accountId) => {
  switchingAccountId.value = accountId
  try {
    await invoke('antigravity_switch_account', { accountId })
    await loadAccounts()
    markAccountUpsertById(accountId)
  } catch (error) {
    console.error('Failed to switch account:', error)
    alert(error)
  } finally {
    switchingAccountId.value = null
  }
}

const handleRefreshQuota = async (accountId) => {
  refreshingIds.value.add(accountId)
  try {
    await invoke('antigravity_fetch_quota', { accountId })
    await loadAccounts()
    markAccountUpsertById(accountId)
    window.$notify?.success($t('platform.antigravity.messages.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh quota:', error)
    window.$notify?.error($t('platform.antigravity.messages.refreshFailed', { error: error?.message || error }))
  } finally {
    refreshingIds.value.delete(accountId)
  }
}

const handleRefresh = async () => {
  isRefreshing.value = true
  window.$notify?.info($t('platform.antigravity.refreshing'))
  try {
    await loadAccounts()
    for (const account of accounts.value) {
      refreshingIds.value.add(account.id)
    }
    await invoke('antigravity_refresh_all_quotas')
    await loadAccounts()
    for (const account of accounts.value) {
      markAccountUpsert(account)
    }
    window.$notify?.success($t('platform.antigravity.messages.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh quotas:', error)
    window.$notify?.error($t('platform.antigravity.messages.refreshFailed', { error: error?.message || error }))
  } finally {
    refreshingIds.value.clear()
    isRefreshing.value = false
  }
}

const handleAddAccount = async (email, refreshToken) => {
  try {
    const account = await invoke('antigravity_add_account', { email, refreshToken })
    await handleAccountAdded(account)
  } catch (error) {
    console.error('Failed to add account:', error)
    window.$notify?.error($t('platform.antigravity.messages.addFailed', { error: error?.message || error }))
    throw error
  }
}

const handleAccountAdded = async (account) => {
  showAddDialog.value = false
  await loadAccounts()
  if (account?.id) {
    markAccountUpsertById(account.id)
  }
  window.$notify?.success($t('platform.antigravity.messages.addSuccess'))
}

const handleDelete = async (accountId) => {
  try {
    const account = accounts.value.find(a => a.id === accountId)
    await invoke('antigravity_delete_account', { accountId })
    if (account) {
      markAccountDeletion(account)
    }
    await loadAccounts()
    window.$notify?.success($t('platform.antigravity.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to delete account:', error)
    window.$notify?.error($t('platform.antigravity.messages.deleteFailed', { error: error?.message || error }))
  }
}

const openDataFolder = async () => {
  try {
    await invoke('open_data_folder')
  } catch (error) {
    console.error('Failed to open data folder:', error)
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
const exportAccounts = () => {
  try {
    const dataStr = JSON.stringify(accounts.value, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)
    const link = document.createElement('a')
    link.href = url
    link.download = `antigravity-accounts-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
    window.$notify.success($t('antigravity.exportSuccess'))
  } catch (error) {
    console.error('Export failed:', error)
    window.$notify.error($t('antigravity.exportFailed'))
  }
}

// 处理页面内容点击 (关闭所有工具栏)
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

const showBatchDeleteSelectedConfirm = () => {
  handleBatchDeleteSelected()
}

const handleBatchDeleteSelected = async () => {
  try {
    for (const accountId of selectedAccountIds.value) {
      const account = accounts.value.find(a => a.id === accountId)
      await invoke('antigravity_delete_account', { accountId })
      if (account) {
        markAccountDeletion(account)
      }
    }
    selectedAccountIds.value = new Set()
    await loadAccounts()
    window.$notify?.success($t('platform.antigravity.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to batch delete accounts:', error)
    window.$notify?.error($t('platform.antigravity.messages.deleteFailed', { error: error?.message || error }))
  }
}

const handleBatchDelete = () => {
  if (selectedAccountIds.value.size === 0) {
    alert($t('platform.antigravity.messages.noSelection'))
    return
  }
  showBatchDeleteSelectedConfirm()
}

const openSyncQueue = () => {
  if (!isDatabaseAvailable.value) {
    window.$notify.info($t('storage.databaseNotAvailable'))
    return
  }
  showSyncQueueModal.value = true
}

const closeSyncQueue = () => {
  showSyncQueueModal.value = false
}

const handleMarkAllForSync = () => {
  if (accounts.value.length === 0) {
    window.$notify.warning($t('platform.antigravity.messages.noSelection'))
    return
  }
  pendingUpserts.value = new Map(accounts.value.map(account => [account.id, account]))
  pendingDeletions.value.clear()
  savePendingChanges()
  isSyncNeeded.value = true
  window.$notify.success($t('platform.antigravity.allAccountsMarkedForSync', { count: accounts.value.length }))
}

const openModelsModal = (account) => {
  activeModelsAccount.value = account
  showModelsModal.value = true
}

const closeModelsModal = () => {
  showModelsModal.value = false
  activeModelsAccount.value = null
}

const refreshModelsModal = async () => {
  if (!activeModelsAccount.value) return
  await handleRefreshQuota(activeModelsAccount.value.id)
  activeModelsAccount.value = accounts.value.find(a => a.id === activeModelsAccount.value.id) || activeModelsAccount.value
}

const handleSync = async () => {
  if (isSyncing.value) return
  if (!isDatabaseAvailable.value) {
    window.$notify.warning($t('messages.databaseNotAvailable'))
    return
  }

  isSyncing.value = true
  try {
    window.$notify.info($t('messages.syncingData'))

    const req = {
      last_version: lastVersion.value,
      upserts: Array.from(pendingUpserts.value.values()).map(account => ({ account })),
      deletions: Array.from(pendingDeletions.value.values()).map(item => ({ id: item.id })),
    }

    const res = await invoke('antigravity_sync_accounts', { reqJson: JSON.stringify(req) })

    isLoadingFromSync.value = true

    for (const serverAccount of res.upserts) {
      const idx = accounts.value.findIndex(a => a.id === serverAccount.id)
      if (idx !== -1) {
        accounts.value[idx] = serverAccount
      } else {
        accounts.value.push(serverAccount)
      }
    }

    for (const id of res.deletions) {
      const idx = accounts.value.findIndex(a => a.id === id)
      if (idx !== -1) {
        accounts.value.splice(idx, 1)
      }
      if (currentAccountId.value === id) {
        currentAccountId.value = null
      }
    }

    lastVersion.value = res.new_version
    saveLastVersion(res.new_version)

    pendingUpserts.value.clear()
    pendingDeletions.value.clear()
    savePendingChanges()

    await new Promise(resolve => setTimeout(resolve, 1200))
    isLoadingFromSync.value = false
    isSyncNeeded.value = false

    window.$notify.success($t('messages.syncComplete'))
  } catch (error) {
    window.$notify.error(`${$t('messages.syncFailed')}: ${error}`)
  } finally {
    isSyncing.value = false
  }
}

// 监听搜索和筛选变化，重置分页
watch([searchQuery, selectedStatusFilter], () => {
  currentPage.value = 1
})

onMounted(async () => {
  lastVersion.value = loadLastVersion()
  loadPendingChanges()
  await getStorageStatus()
  await loadAccounts()
})

onUnmounted(() => {
  if (storageCheckTimer) {
    clearInterval(storageCheckTimer)
    storageCheckTimer = null
  }
})
</script>