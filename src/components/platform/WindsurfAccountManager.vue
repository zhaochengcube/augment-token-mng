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
              v-tooltip="isDatabaseAvailable ? $t('platform.windsurf.viewSyncQueueTooltip') : ''"
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
            <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('platform.windsurf.addAccount')">
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
              v-tooltip="$t('platform.windsurf.refreshQuota')"
            >
              <svg v-if="!isRefreshing" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
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
          <img src="/icons/windsurf.svg" alt="Windsurf" width="64" height="64" />
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
            :all-accounts="accounts"
            @switch="handleSwitch"
            @refresh="handleRefreshQuota"
            @delete="handleDelete"
            @select="toggleAccountSelection"
            @account-updated="handleAccountUpdated"
          />
        </div>

        <!-- 列表布局 -->
        <div v-else class="table-container">
          <table class="table">
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
                <th class="th w-[100px] text-center">{{ $t('platform.windsurf.table.status') }}</th>
                <th class="th w-24">{{ $t('platform.windsurf.table.tag') }}</th>
                <th class="th">{{ $t('platform.windsurf.table.info') }}</th>
                <th class="th w-[85px] text-center">{{ $t('platform.windsurf.table.quota') }}</th>
                <th class="th w-[100px]">{{ $t('platform.windsurf.table.expires') }}</th>
                <th class="th w-[120px] text-center">{{ $t('platform.windsurf.table.actions') }}</th>
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
          :placeholder="$t('platform.windsurf.searchPlaceholder')"
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
              :class="['btn btn--sm', selectedStatusFilter === null ? 'btn--primary' : 'btn--secondary']"
              @click="selectStatusFilter(null)"
            >
              <span>{{ $t('platform.windsurf.filter.all') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.total }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedStatusFilter === 'available' ? 'btn--primary' : 'btn--secondary']"
              @click="selectStatusFilter('available')"
            >
              <span>{{ $t('platform.windsurf.filter.available') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.available }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedStatusFilter === 'low' ? 'btn--primary' : 'btn--secondary']"
              @click="selectStatusFilter('low')"
            >
              <span>{{ $t('platform.windsurf.filter.low') }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics.low }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedStatusFilter === 'expired' ? 'btn--primary' : 'btn--secondary']"
              @click="selectStatusFilter('expired')"
            >
              <span>{{ $t('platform.windsurf.filter.expired') }}</span>
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
                sortType === 'quota' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('quota', sortOrder)"
            >
              {{ $t('platform.windsurf.sortByQuota') }}
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

        <!-- 自定义 Windsurf 路径按钮 -->
        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="showCustomPathDialog = true"
          v-tooltip="$t('customPath.windsurfDescription')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z"/>
          </svg>
          <span>{{ $t('customPath.customPathButton') }}</span>
          <span v-if="customWindsurfPath" class="text-xs text-accent">•</span>
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
          v-tooltip="$t('platform.windsurf.batchRefresh')"
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
          @click="batchDeleteSelected"
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

    <!-- 自定义路径对话框 -->
    <CustomPathDialog
      :visible="showCustomPathDialog"
      :title="$t('customPath.windsurfTitle')"
      :description="$t('customPath.windsurfDescription')"
      :current-path="customWindsurfPath"
      :default-path="defaultWindsurfPath"
      select-command="windsurf_select_executable_path"
      validate-command="windsurf_validate_path"
      save-command="windsurf_set_custom_path"
      @close="showCustomPathDialog = false"
      @saved="handleCustomPathSaved"
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
      :title="$t('platform.windsurf.syncQueueTitle')"
      :upserts-title="$t('platform.windsurf.syncQueueUpsertsTitle')"
      :deletions-title="$t('platform.windsurf.syncQueueDeletionsTitle')"
      :empty-text="$t('platform.windsurf.syncQueueEmpty')"
      :full-sync-text="$t('platform.windsurf.fullSync')"
      :sync-text="$t('platform.windsurf.sync')"
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
import { useI18n } from 'vue-i18n'
import AccountCard from '../windsurf/AccountCard.vue'
import AccountTableRow from '../windsurf/AccountTableRow.vue'
import AddAccountDialog from '../windsurf/AddAccountDialog.vue'
import Pagination from '../common/Pagination.vue'
import ActionToolbar from '../common/ActionToolbar.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'
import CustomPathDialog from '../common/CustomPathDialog.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
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
const isRefreshing = ref(false)
const switchingAccountId = ref(null)
const refreshingIds = ref(new Set())

// 使用存储同步 composable
const {
  isDatabaseAvailable,
  isStorageInitializing,
  isSyncing,
  isSyncNeeded,
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
  platform: 'windsurf',
  syncCommand: 'windsurf_sync_accounts',
  items: accounts,
  currentItemId: currentAccountId,
  itemKey: 'account',
  labelField: 'email',
  onSyncComplete: async () => {
    // 同步完成后重新从本地加载，确保数据一致性
    await loadAccounts()
  }
})

// 自定义路径相关状态
const showCustomPathDialog = ref(false)
const customWindsurfPath = ref(null)
const defaultWindsurfPath = ref('')

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
const showRealEmail = ref(true)

// 分页
const currentPage = ref(1)
const pageSize = ref(20)
const pageSizeOptions = [10, 20, 50, 100, 200]

// 批量操作
const selectedAccountIds = ref(new Set())
const isBatchRefreshing = ref(false)
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

const statusStatistics = computed(() => {
  const stats = { total: accounts.value.length, available: 0, low: 0, expired: 0 }

  accounts.value.forEach(account => {
    if (!account.quota) {
      stats.available++
    } else {
      const remaining = 100 - (account.quota.usage_percentage || 0)
      if (remaining >= 20) stats.available++
      else if (remaining > 0) stats.low++
      else stats.expired++
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
      const remaining = account.quota ? 100 - (account.quota.usage_percentage || 0) : 100

      if (selectedStatusFilter.value === 'available') return remaining >= 20
      if (selectedStatusFilter.value === 'low') return remaining > 0 && remaining < 20
      if (selectedStatusFilter.value === 'expired') return remaining <= 0
      return true
    })
  }

  // 排序
  result = [...result].sort((a, b) => {
    const currentId = currentAccountId.value
    if (currentId) {
      if (a.id === currentId) return -1
      if (b.id === currentId) return 1
    }

    if (sortType.value === 'time') {
      const timeA = a.last_used || a.created_at
      const timeB = b.last_used || b.created_at
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'quota') {
      const quotaA = a.quota ? 100 - (a.quota.usage_percentage || 0) : 100
      const quotaB = b.quota ? 100 - (b.quota.usage_percentage || 0) : 100
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
    const data = await invoke('windsurf_list_accounts')
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

const handleSwitch = async (accountId) => {
  switchingAccountId.value = accountId
  try {
    await invoke('windsurf_switch_account', { accountId })
    await loadAccounts()
    markItemUpsertById(accountId)
    window.$notify?.success($t('platform.windsurf.messages.switchSuccess'))
  } catch (error) {
    console.error('Failed to switch account:', error)
    window.$notify?.error(error?.message || error)
  } finally {
    switchingAccountId.value = null
  }
}

const handleRefreshQuota = async (accountId) => {
  refreshingIds.value.add(accountId)
  try {
    const updatedAccount = await invoke('windsurf_fetch_quota', { accountId })

    // 只更新该账户的数据，保持滚动位置
    const index = accounts.value.findIndex(a => a.id === accountId)
    if (index !== -1 && updatedAccount) {
      accounts.value[index] = updatedAccount
    } else {
      await loadAccounts()
    }

    markItemUpsertById(accountId)
    window.$notify?.success($t('platform.windsurf.messages.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh quota:', error)
    window.$notify?.error(error?.message || error)
  } finally {
    refreshingIds.value.delete(accountId)
  }
}

const handleRefresh = async () => {
  isRefreshing.value = true
  try {
    await invoke('windsurf_fetch_all_quotas')
    await loadAccounts()
    // 标记所有账户待同步
    for (const account of accounts.value) {
      markItemUpsert(account)
    }
    window.$notify?.success($t('platform.windsurf.messages.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh quotas:', error)
    window.$notify?.error(error?.message || error)
  } finally {
    isRefreshing.value = false
  }
}

const handleAccountAdded = async (account) => {
  showAddDialog.value = false
  await loadAccounts()
  if (account?.id) {
    markItemUpsertById(account.id)
  }
  window.$notify?.success($t('platform.windsurf.messages.addSuccess'))
}

const handleDelete = async (accountId) => {
  try {
    const account = accounts.value.find(a => a.id === accountId)
    await invoke('windsurf_delete_account', { accountId })
    if (account) {
      markItemDeletion(account)
    }
    await loadAccounts()
    window.$notify?.success($t('platform.windsurf.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to delete account:', error)
    window.$notify?.error(error?.message || error)
  }
}

const handleMarkAllForSync = () => {
  if (markAllForSync()) {
    window.$notify?.success($t('platform.windsurf.allAccountsMarkedForSync', { count: accounts.value.length }))
  } else {
    window.$notify?.warning($t('platform.windsurf.messages.noSelection'))
  }
}

// 账户更新处理（标签等属性）
const handleAccountUpdated = async (updatedAccount) => {
  if (!updatedAccount?.id) return
  try {
    // 更新本地状态
    const index = accounts.value.findIndex(a => a.id === updatedAccount.id)
    if (index !== -1) {
      accounts.value[index] = { ...accounts.value[index], ...updatedAccount }
    }
    // 调用后端保存到本地存储
    await invoke('windsurf_update_account', { account: updatedAccount })
    // 标记待同步到远程
    markItemUpsert(updatedAccount)
  } catch (error) {
    console.error('Failed to update account:', error)
    window.$notify?.error($t('messages.updateFailed'))
  }
}

// 工具栏模式切换
const setToolbarMode = (mode) => {
  toolbarMode.value = toolbarMode.value === mode ? 'hidden' : mode
  if (toolbarMode.value === 'search') {
    nextTick(() => toolbarSearchInputRef.value?.focus())
  }
}

const openDataFolder = async () => {
  try {
    await invoke('open_data_folder')
  } catch (error) {
    console.error('Failed to open data folder:', error)
  }
}

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

const handlePageContentClick = () => {}

const handlePageChange = (page) => {
  currentPage.value = page
}

const changePageSize = (size) => {
  pageSize.value = size
  currentPage.value = 1
}

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

// 批量刷新选中账户
const batchRefreshSelected = async () => {
  if (selectedAccountIds.value.size === 0 || isBatchRefreshing.value) return

  isBatchRefreshing.value = true
  try {
    for (const accountId of selectedAccountIds.value) {
      await handleRefreshQuota(accountId)
    }
  } finally {
    isBatchRefreshing.value = false
  }
}

// 批量删除选中账户
const batchDeleteSelected = async () => {
  if (selectedAccountIds.value.size === 0) return

  try {
    for (const accountId of selectedAccountIds.value) {
      const account = accounts.value.find(a => a.id === accountId)
      await invoke('windsurf_delete_account', { accountId })
      if (account) {
        markItemDeletion(account)
      }
    }
    selectedAccountIds.value = new Set()
    await loadAccounts()
    window.$notify?.success($t('platform.windsurf.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to batch delete accounts:', error)
    window.$notify?.error(error?.message || error)
  }
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
    await invoke('windsurf_save_accounts', { accounts: accounts.value })
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
    await invoke('windsurf_save_accounts', { accounts: accounts.value })
  } catch (error) {
    console.error('Failed to save accounts:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagCleared', { count: clearedCount }))
}

watch([searchQuery, selectedStatusFilter], () => {
  currentPage.value = 1
})

// 自定义 Windsurf 路径相关函数
const loadCustomPath = async () => {
  try {
    customWindsurfPath.value = await invoke('windsurf_get_custom_path')
    try {
      defaultWindsurfPath.value = await invoke('windsurf_get_default_path')
    } catch (e) {
      defaultWindsurfPath.value = ''
    }
  } catch (error) {
    console.error('Failed to load custom path:', error)
  }
}

const handleCustomPathSaved = (newPath) => {
  customWindsurfPath.value = newPath
}

onMounted(async () => {
  await initSync()
  await loadAccounts()
  await loadCustomPath()
})
</script>
