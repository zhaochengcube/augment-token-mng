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
        <AccountManagerHeader
          :storage-status-text="storageStatusText"
          :storage-status-class="storageStatusClass"
          :is-database-available="isDatabaseAvailable"
          :sync-queue-tooltip="$t('platform.cursor.viewSyncQueueTooltip')"
          :search-active="isSearchActive"
          :filter-active="isFilterActive"
          :sort-active="isSortNonDefault"
          :view-mode="viewMode"
          @open-sync-queue="openSyncQueue"
          @search="setToolbarMode('search')"
          @filter="setToolbarMode('filter')"
          @sort="setToolbarMode('sort')"
          @toggle-view="toggleViewMode"
          @clear-all="clearAllFilters"
        >
          <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('platform.cursor.addAccount')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
            </svg>
          </button>
          <button @click="showImportDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('platform.cursor.importAccounts')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/>
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
            @click="handleBatchRefreshQuota"
            :disabled="isRefreshing"
            v-tooltip="$t('platform.cursor.batchRefreshQuota')"
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
        </AccountManagerHeader>
      </template>

      <!-- Loading State -->
      <div v-if="isLoading" class="flex flex-col items-center justify-center h-full gap-5 text-text-muted py-12 px-6">
        <div class="spinner"></div>
        <p class="m-0">{{ $t('common.loading') }}</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="showEmptyState" class="flex flex-col items-center justify-center py-[60px] px-5 text-text-secondary">
        <div class="text-text-muted mb-[18px] opacity-60">
          <img src="/icons/cursor.svg" alt="Cursor" width="64" height="64" />
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
            :is-refreshing="refreshingAccountId === account.id"
            :is-selected="selectedAccountIds.has(account.id)"
            :selection-mode="isSelectionMode"
            :show-real-email="showRealEmail"
            :all-accounts="accounts"
            @switch="handleSwitch"
            @delete="handleDelete"
            @select="toggleAccountSelection"
            @account-updated="handleAccountUpdated"
            @account-synced="handleAccountSynced"
            @machine-id-generated="handleMachineIdGenerated"
            @refresh-quota="handleRefreshQuota"
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
                <th class="th w-[60px]">{{ $t('platform.cursor.table.tag') }}</th>
                <th class="th">{{ $t('platform.cursor.table.email') }}</th>
                <th class="th w-[130px]">{{ $t('platform.cursor.table.usage') }}</th>
                <th class="th w-[105px]">{{ $t('platform.cursor.table.expiry') }}</th>
                <th class="th w-[100px]">{{ $t('platform.cursor.table.quota') }}</th>
                <th class="th w-[80px] text-center">{{ $t('platform.cursor.table.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <AccountTableRow
                v-for="account in paginatedAccounts"
                :key="account.id"
                :account="account"
                :is-current="account.id === currentAccountId"
                :is-switching="switchingAccountId === account.id"
                :is-selected="selectedAccountIds.has(account.id)"
                :selection-mode="isSelectionMode"
                :show-real-email="showRealEmail"
                :all-accounts="accounts"
                @switch="handleSwitch"
                @delete="handleDelete"
                @select="toggleAccountSelection"
                @account-updated="handleAccountUpdated"
                @account-synced="handleAccountSynced"
                @machine-id-generated="handleMachineIdGenerated"
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
          :placeholder="$t('platform.cursor.searchPlaceholder')"
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
        <div class="flex flex-col gap-2">
          <span class="label">{{ $t('tokenList.filterByStatus') }}</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="['btn btn--sm', selectedStatusFilter === null ? 'btn--primary' : 'btn--secondary']"
              @click="selectedStatusFilter = null"
            >
              <span>{{ $t('platform.cursor.filter.all') }}</span>
            </button>
            <button
              :class="['btn btn--sm', selectedStatusFilter === 'available' ? 'btn--primary' : 'btn--secondary']"
              @click="selectedStatusFilter = 'available'"
            >
              <span>{{ $t('platform.cursor.filter.available') }}</span>
            </button>
            <button
              :class="['btn btn--sm', selectedStatusFilter === 'disabled' ? 'btn--primary' : 'btn--secondary']"
              @click="selectedStatusFilter = 'disabled'"
            >
              <span>{{ $t('platform.cursor.filter.forbidden') }}</span>
            </button>
          </div>
        </div>
        <!-- 会员类型筛选 -->
        <div v-if="Object.keys(membershipStatistics).length > 2" class="flex flex-col gap-2">
          <span class="label">会员类型</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="['btn btn--sm', selectedMembershipFilter === null ? 'btn--primary' : 'btn--secondary']"
              @click="selectedMembershipFilter = null"
            >
              <span>{{ $t('platform.cursor.filter.all') }}</span>
              <span class="ml-1 opacity-70">({{ membershipStatistics.total }})</span>
            </button>
            <button
              v-for="(count, type) in membershipStatistics"
              :key="type"
              v-show="type !== 'total'"
              :class="['btn btn--sm', selectedMembershipFilter === type ? 'btn--primary' : 'btn--secondary']"
              @click="selectedMembershipFilter = type"
            >
              <span>{{ type }}</span>
              <span class="ml-1 opacity-70">({{ count }})</span>
            </button>
          </div>
        </div>

        <div v-if="allTags.length > 0" class="flex flex-col gap-2">
          <span class="label">{{ $t('platform.cursor.table.tag') }}</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="['btn btn--sm', 'btn--secondary']"
              @click="toggleTagFilterMode"
              v-tooltip="tagFilterMode === 'include' ? '切换到排除模式' : '切换到包含模式'"
              data-toolbar-keep-open
            >
              <svg v-if="tagFilterMode === 'include'" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
              <span class="ml-1">{{ tagFilterMode === 'include' ? '包含' : '排除' }}</span>
            </button>
            <button
              :class="['btn btn--sm', selectedTags.size === 0 ? 'btn--primary' : 'btn--secondary']"
              @click="clearTagFilter"
            >
              {{ $t('platform.cursor.filter.all') }}
              <span class="ml-1 opacity-70">({{ accounts.length }})</span>
            </button>
            <button
              v-for="tag in allTags"
              :key="tag"
              :class="['btn btn--sm', selectedTags.has(tag) ? 'btn--primary' : 'btn--secondary']"
              @click="toggleTag(tag)"
            >
              {{ tag }}
              <span class="ml-1 opacity-70">({{ tagCounts[tag] || 0 }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedTags.has('__no_tag__') ? 'btn--primary' : 'btn--secondary']"
              @click="toggleTag('__no_tag__')"
            >
              无标签
              <span class="ml-1 opacity-70">({{ noTagCount }})</span>
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
        <div class="flex flex-col gap-2">
          <span class="label">排序方向</span>
          <div class="flex items-center gap-2 flex-wrap">
            <button :class="['btn btn--sm', sortOrder === 'asc' ? 'btn--primary' : 'btn--secondary']" @click="sortOrder = 'asc'">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M8 14l4-4 4 4H8z" /></svg>
              <span class="ml-1">升序</span>
            </button>
            <button :class="['btn btn--sm', sortOrder === 'desc' ? 'btn--primary' : 'btn--secondary']" @click="sortOrder = 'desc'">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M16 10l-4 4-4-4h8z" /></svg>
              <span class="ml-1">降序</span>
            </button>
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <span class="label">排序字段</span>
          <div class="flex items-center gap-2 flex-wrap">
            <button :class="['btn btn--sm', sortType === 'time' ? 'btn--primary' : 'btn--secondary']" @click="sortType = 'time'">
              {{ $t('common.sortByTime') }}
            </button>
            <button :class="['btn btn--sm', sortType === 'apiUsage' ? 'btn--primary' : 'btn--secondary']" @click="sortType = 'apiUsage'">
              按额度
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
        <!-- 禁用/启用自动更新按钮 -->
        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          :class="{ 'active': isAutoUpdateDisabled }"
          :disabled="isTogglingAutoUpdate"
          @click="toggleAutoUpdate"
          v-tooltip="isAutoUpdateDisabled ? '点击启用自动更新' : '点击禁用自动更新'"
        >
          <span v-if="isTogglingAutoUpdate" class="btn-spinner btn-spinner--sm text-accent"></span>
          <svg v-else-if="isAutoUpdateDisabled" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M21 10.12h-6.78l2.74-2.82c-2.73-2.7-7.15-2.8-9.88-.1-2.73 2.71-2.73 7.08 0 9.79s7.15 2.71 9.88 0C18.32 15.65 19 14.08 19 12.1h2c0 1.98-.88 4.55-2.64 6.29-3.51 3.48-9.21 3.48-12.72 0-3.5-3.47-3.5-9.11 0-12.58 3.51-3.47 9.14-3.49 12.65 0L21 3v7.12z"/>
          </svg>
          <span>{{ isAutoUpdateDisabled ? '自动更新已禁用' : '禁用自动更新' }}</span>
        </button>
        <!-- 自定义 Cursor 路径按钮 -->
        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="showCustomPathDialog = true"
          v-tooltip="$t('customPath.cursorDescription')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z"/>
          </svg>
          <span>{{ $t('customPath.customPathButton') }}</span>
          <span v-if="customCursorPath" class="text-xs text-accent">•</span>
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

        <!-- 批量导出 -->
        <button
          @click="handleBatchExport"
          class="btn btn--icon btn--ghost"
          v-tooltip="$t('accountCard.batchExport')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
          </svg>
        </button>

        <!-- 批量删除 -->
        <button
          @click="handleBatchDelete"
          class="btn btn--icon btn--ghost text-danger"
          v-tooltip="$t('common.delete')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
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
    <AddAccountDialog
      v-if="showAddDialog"
      :existing-accounts="accounts"
      @close="showAddDialog = false"
      @added="handleAccountAdded"
      @updated="handleAccountAdded"
    />

    <!-- Import Accounts Dialog -->
    <ImportAccountsDialog v-if="showImportDialog" @close="showImportDialog = false" @imported="handleAccountsImported" />

    <!-- Machine ID Option Dialog -->
    <MachineIdOptionDialog
      v-if="showMachineIdOptionDialog"
      :account="pendingSwitchAccount"
      @close="handleMachineIdOptionClose"
      @confirm="handleMachineIdOptionConfirm"
    />

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="accounts.length"
      :title="$t('platform.cursor.syncQueueTitle')"
      :upserts-title="$t('platform.cursor.syncQueueUpsertsTitle')"
      :deletions-title="$t('platform.cursor.syncQueueDeletionsTitle')"
      :empty-text="$t('platform.cursor.syncQueueEmpty')"
      :full-sync-text="$t('platform.cursor.fullSync')"
      :sync-text="$t('platform.cursor.sync')"
      label-field="email"
      fallback-field="id"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />

    <!-- 自定义路径对话框 -->
    <CustomPathDialog
      :visible="showCustomPathDialog"
      :title="$t('customPath.cursorTitle')"
      :description="$t('customPath.cursorDescription')"
      :current-path="customCursorPath"
      :default-path="defaultCursorPath"
      select-command="cursor_select_executable_path"
      validate-command="cursor_validate_path"
      save-command="cursor_set_custom_path"
      @close="showCustomPathDialog = false"
      @saved="handleCustomPathSaved"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import AccountCard from '../cursor/AccountCard.vue'
import AccountTableRow from '../cursor/AccountTableRow.vue'
import AddAccountDialog from '../cursor/AddAccountDialog.vue'
import ImportAccountsDialog from '../cursor/ImportAccountsDialog.vue'
import MachineIdOptionDialog from '../cursor/MachineIdOptionDialog.vue'
import Pagination from '../common/Pagination.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'
import AccountManagerHeader from '../common/AccountManagerHeader.vue'
import ActionToolbar from '../common/ActionToolbar.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import CustomPathDialog from '../common/CustomPathDialog.vue'
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
const showImportDialog = ref(false)
const showMachineIdOptionDialog = ref(false)
const pendingSwitchAccount = ref(null) // 待切换的账号（用于机器码选项对话框）
const isLoading = ref(false)
const isRefreshing = ref(false)
const switchingAccountId = ref(null)
const refreshingAccountId = ref(null)

// 使用存储同步 composable
const {
  isDatabaseAvailable,
  isSyncing,
  storageStatusText,
  storageStatusClass,
  showSyncQueueModal,
  pendingUpsertsList,
  pendingDeletionsList,
  initSync,
  markItemUpsertById,
  markItemDeletion,
  markAllForSync,
  openSyncQueue,
  handleSync
} = useStorageSync({
  platform: 'cursor',
  syncCommand: 'cursor_sync_accounts',
  items: accounts,
  currentItemId: currentAccountId,
  itemKey: 'account',
  labelField: 'email',
  onSyncComplete: async () => {
    await loadAccounts()
  }
})

// 自定义路径相关状态
const showCustomPathDialog = ref(false)
const customCursorPath = ref(null)
const defaultCursorPath = ref('')

// 自动更新状态
const isAutoUpdateDisabled = ref(false)
const isTogglingAutoUpdate = ref(false)

// 搜索和筛选
const searchQuery = ref('')
const selectedStatusFilter = ref(null)
const selectedTags = ref(new Set())
const tagFilterMode = ref('include')
const toolbarMode = ref('hidden')
const toolbarSearchInputRef = ref(null)

// 会员类型筛选
const selectedMembershipFilter = ref(null)

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
const showBatchTagEditor = ref(false)

// 计算属性
const isSelectionMode = computed(() => selectedAccountIds.value.size > 0)

// 选中的账号列表（转换为TagEditorModal需要的格式）
const selectedAccounts = computed(() =>
  accounts.value
    .filter(a => selectedAccountIds.value.has(a.id))
    .map(acc => ({
      tag_name: acc.tag || '',
      tag_color: acc.tag_color || '',
      _account: acc // 保存原始账号引用
    }))
)

// 所有账号转换为标签格式
const allAccountsAsTokens = computed(() =>
  accounts.value.map(acc => ({
    tag_name: acc.tag || '',
    tag_color: acc.tag_color || ''
  }))
)

// 提取所有唯一标签
const allTags = computed(() => {
  const tags = new Set()
  accounts.value.forEach(a => {
    if (a.tag?.trim()) {
      tags.add(a.tag.trim())
    }
  })
  return Array.from(tags).sort((a, b) => a.localeCompare(b, 'zh-CN'))
})

const tagCounts = computed(() => {
  const counts = {}
  accounts.value.forEach(a => {
    const tag = a.tag?.trim()
    if (tag) {
      counts[tag] = (counts[tag] || 0) + 1
    }
  })
  return counts
})

const noTagCount = computed(() => {
  return accounts.value.filter(a => !a.tag?.trim()).length
})

const membershipStatistics = computed(() => {
  const stats = { total: accounts.value.length }
  const counts = {}
  accounts.value.forEach(a => {
    const type = (a.membership_type || 'free').toLowerCase()
    counts[type] = (counts[type] || 0) + 1
  })
  return { ...stats, ...counts }
})

const filteredAccounts = computed(() => {
  let result = accounts.value

  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(a => a.email.toLowerCase().includes(query))
  }

  if (selectedStatusFilter.value) {
    result = result.filter(account => {
      if (selectedStatusFilter.value === 'disabled') {
        return account.disabled
      }
      if (selectedStatusFilter.value === 'available') {
        return !account.disabled
      }
      return true
    })
  }

  // 会员类型筛选
  if (selectedMembershipFilter.value) {
    result = result.filter(account => {
      const type = (account.membership_type || 'free').toLowerCase()
      return type === selectedMembershipFilter.value
    })
  }

  // 标签筛选
  if (selectedTags.value.size > 0) {
    const hasNoTagFilter = selectedTags.value.has('__no_tag__')
    const lowerSelectedTags = new Set(
      Array.from(selectedTags.value)
        .filter(t => t !== '__no_tag__')
        .map(tag => tag.toLowerCase())
    )
    result = result.filter(account => {
      const tagName = account.tag?.trim() || ''
      const isNoTag = !tagName
      let matches = false
      if (isNoTag) {
        matches = hasNoTagFilter
      } else {
        matches = lowerSelectedTags.has(tagName.toLowerCase())
      }
      return tagFilterMode.value === 'include' ? matches : !matches
    })
  }

  result = [...result].sort((a, b) => {
    const currentId = currentAccountId.value
    if (currentId) {
      if (a.id === currentId) return -1
      if (b.id === currentId) return 1
    }

    if (sortType.value === 'time') {
      const timeA = a.last_used || a.created_at || 0
      const timeB = b.last_used || b.created_at || 0
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'apiUsage') {
      const usageA = a.individual_usage?.plan?.apiPercentUsed
      const usageB = b.individual_usage?.plan?.apiPercentUsed
      const valA = usageA != null ? usageA : (sortOrder.value === 'desc' ? -1 : Infinity)
      const valB = usageB != null ? usageB : (sortOrder.value === 'desc' ? -1 : Infinity)
      return sortOrder.value === 'desc' ? valB - valA : valA - valB
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

// 搜索/筛选/排序活跃状态
const isSearchActive = computed(() => searchQuery.value.trim() !== '')
const isFilterActive = computed(() => selectedStatusFilter.value !== null || selectedMembershipFilter.value !== null || selectedTags.value.size > 0)
const isSortNonDefault = computed(() => sortType.value !== 'time' || sortOrder.value !== 'desc')

const clearAllFilters = () => {
  searchQuery.value = ''
  selectedStatusFilter.value = null
  selectedMembershipFilter.value = null
  selectedTags.value = new Set()
  tagFilterMode.value = 'include'
  sortType.value = 'time'
  sortOrder.value = 'desc'
}

// 方法
const loadAccounts = async () => {
  isLoading.value = true
  try {
    // 使用后端提供的 cursor_list_accounts 命令
    const data = await invoke('cursor_list_accounts')
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

// 检查账号是否有有效的机器码信息
const hasMachineInfo = (account) => {
  if (!account?.machine_info) return false
  const info = account.machine_info
  return info['telemetry.machineId'] ||
    info['telemetry.macMachineId'] ||
    info['telemetry.devDeviceId'] ||
    info['telemetry.sqmId'] ||
    info['system.machineGuid'] ||
    info['storage.serviceMachineId']
}

const handleSwitch = async (accountId) => {
  const account = accounts.value.find(a => a.id === accountId)
  if (!account) return

  // 如果账号有绑定的机器码，弹出选项对话框
  if (hasMachineInfo(account)) {
    pendingSwitchAccount.value = account
    showMachineIdOptionDialog.value = true
    return
  }

  // 没有机器码，直接使用随机生成
  await performSwitch(accountId, false)
}

// 执行切换操作
const performSwitch = async (accountId, useBoundMachineId) => {
  switchingAccountId.value = accountId
  try {
    await invoke('cursor_switch_account', { accountId, useBoundMachineId })
    await loadAccounts()
    markItemUpsertById(accountId)
  } catch (error) {
    console.error('Failed to switch account:', error)
    window.$notify?.error(error?.message || error)
  } finally {
    switchingAccountId.value = null
  }
}

// 机器码选项确认
const handleMachineIdOptionConfirm = async (useBoundMachineId) => {
  showMachineIdOptionDialog.value = false
  if (pendingSwitchAccount.value) {
    await performSwitch(pendingSwitchAccount.value.id, useBoundMachineId)
    pendingSwitchAccount.value = null
  }
}

// 机器码选项取消
const handleMachineIdOptionClose = () => {
  showMachineIdOptionDialog.value = false
  pendingSwitchAccount.value = null
}

const handleBatchRefreshQuota = async () => {
  const pageAccounts = paginatedAccounts.value.filter(a => a.token?.workos_cursor_session_token)
  if (pageAccounts.length === 0) {
    window.$notify?.error($t('platform.cursor.noQuotaData'))
    return
  }
  isRefreshing.value = true
  let success = 0
  let fail = 0
  for (const account of pageAccounts) {
    try {
      const summary = await invoke('cursor_get_usage_summary', { sessionToken: account.token.workos_cursor_session_token })
      account.membership_type = summary.membershipType || null
      const usage = summary.individualUsage || {}
      if (summary.billingCycleStart) usage.billingCycleStart = summary.billingCycleStart
      if (summary.billingCycleEnd) usage.billingCycleEnd = summary.billingCycleEnd
      account.individual_usage = Object.keys(usage).length > 0 ? usage : null
      account.updated_at = Math.floor(Date.now() / 1000)
      await invoke('cursor_update_account', { account })
      markItemUpsertById(account.id)
      success++
    } catch (e) {
      console.error(`Failed to refresh quota for ${account.email}:`, e)
      fail++
    }
  }
  isRefreshing.value = false
  if (fail === 0) {
    window.$notify?.success($t('platform.cursor.batchRefreshQuota') + ` (${success}/${pageAccounts.length})`)
  } else {
    window.$notify?.warning(`${success} ${$t('common.success')}, ${fail} ${$t('common.failed')}`)
  }
}

const handleAccountAdded = async (account) => {
  showAddDialog.value = false
  await loadAccounts()
  if (account?.id) {
    markItemUpsertById(account.id)
    handleRefreshQuota(account.id)
  }
  window.$notify?.success($t('platform.cursor.messages.addSuccess'))
}

// 导入账号成功处理
const handleAccountsImported = async (result) => {
  showImportDialog.value = false
  await loadAccounts()
  // 标记成功导入的账号
  if (result?.results) {
    result.results.forEach(r => {
      if (r.success && r.account?.id) {
        markItemUpsertById(r.account.id)
      }
    })
  }
  window.$notify?.success($t('platform.cursor.messages.importSuccess', { count: result.success_count }))
}

const handleDelete = async (accountId) => {
  try {
    const account = accounts.value.find(a => a.id === accountId)
    await invoke('cursor_delete_account', { accountId })
    if (account) {
      markItemDeletion(account)
    }
    await loadAccounts()
    window.$notify?.success($t('platform.cursor.messages.deleteSuccess'))
  } catch (error) {
    console.error('Failed to delete account:', error)
    window.$notify?.error($t('platform.cursor.messages.deleteFailed', { error: error?.message || error }))
  }
}

// 工具栏模式切换
const setToolbarMode = (mode) => {
  toolbarMode.value = toolbarMode.value === mode ? 'hidden' : mode
  if (toolbarMode.value === 'search') {
    nextTick(() => toolbarSearchInputRef.value?.focus())
  }
}

// 标签筛选方法
const toggleTagFilterMode = () => {
  tagFilterMode.value = tagFilterMode.value === 'include' ? 'exclude' : 'include'
}

const toggleTag = (tagName) => {
  const newSet = new Set(selectedTags.value)
  if (newSet.has(tagName)) {
    newSet.delete(tagName)
  } else {
    newSet.add(tagName)
  }
  selectedTags.value = newSet
}

const clearTagFilter = () => {
  selectedTags.value = new Set()
}

// 处理账号更新（标签等）
const handleAccountUpdated = async (updatedAccount) => {
  try {
    // 更新本地状态
    const index = accounts.value.findIndex(a => a.id === updatedAccount.id)
    if (index !== -1) {
      accounts.value[index] = { ...accounts.value[index], ...updatedAccount }
    }
    // 调用后端保存
    await invoke('cursor_update_account', { account: updatedAccount })
    // 标记同步
    markItemUpsertById(updatedAccount.id)
  } catch (error) {
    console.error('Failed to update account:', error)
    window.$notify?.error($t('messages.updateFailed'))
  }
}

// 处理模态框内已保存的账号同步标记（不再重复保存）
const handleAccountSynced = (accountId) => {
  markItemUpsertById(accountId)
}

const handleMachineIdGenerated = async (accountId) => {
  // 重新加载账号列表以获取更新后的机器码信息
  await loadAccounts()
  // 触发同步队列更新（标记该账号待同步，不自动执行同步）
  if (accountId && isDatabaseAvailable.value) {
    markItemUpsertById(accountId)
  }
}

// 刷新单个账号配额
const handleRefreshQuota = async (accountId) => {
  const account = accounts.value.find(a => a.id === accountId)
  if (!account) return
  const sessionToken = account.token?.workos_cursor_session_token
  if (!sessionToken) {
    window.$notify?.error($t('platform.cursor.noQuotaData'))
    return
  }
  refreshingAccountId.value = accountId
  try {
    const summary = await invoke('cursor_get_usage_summary', { sessionToken })
    account.membership_type = summary.membershipType || null
    const usage = summary.individualUsage || {}
    if (summary.billingCycleStart) usage.billingCycleStart = summary.billingCycleStart
    if (summary.billingCycleEnd) usage.billingCycleEnd = summary.billingCycleEnd
    account.individual_usage = Object.keys(usage).length > 0 ? usage : null
    account.updated_at = Math.floor(Date.now() / 1000)
    await invoke('cursor_update_account', { account })
    markItemUpsertById(accountId)
  } catch (e) {
    console.error('Failed to refresh quota:', e)
    window.$notify?.error(e?.message || e)
  } finally {
    refreshingAccountId.value = null
  }
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

// 批量工具栏方法
const selectAllOnPage = () => {
  const newSet = new Set(selectedAccountIds.value)
  paginatedAccounts.value.forEach(a => newSet.add(a.id))
  selectedAccountIds.value = newSet
}

const clearSelection = () => {
  selectedAccountIds.value = new Set()
}

const handleBatchTagSave = async ({ tagName, tagColor }) => {
  const ids = Array.from(selectedAccountIds.value)
  for (const id of ids) {
    const account = accounts.value.find(a => a.id === id)
    if (account) {
      account.tag = tagName
      account.tag_color = tagColor
      account.updated_at = Math.floor(Date.now() / 1000)
      try {
        await invoke('cursor_update_account', { account })
        markItemUpsertById(id)
      } catch (error) {
        console.error('Failed to update account tag:', error)
      }
    }
  }
  window.$notify?.success($t('tokenList.batchTagUpdated', { count: ids.length }))
}

const handleBatchTagClear = async () => {
  const ids = Array.from(selectedAccountIds.value)
  for (const id of ids) {
    const account = accounts.value.find(a => a.id === id)
    if (account) {
      account.tag = ''
      account.tag_color = ''
      account.updated_at = Math.floor(Date.now() / 1000)
      try {
        await invoke('cursor_update_account', { account })
        markItemUpsertById(id)
      } catch (error) {
        console.error('Failed to clear account tag:', error)
      }
    }
  }
  window.$notify?.success($t('messages.tagCleared'))
}

const handleBatchDelete = async () => {
  if (selectedAccountIds.value.size === 0) return
  const ids = Array.from(selectedAccountIds.value)
  for (const id of ids) {
    try {
      const account = accounts.value.find(a => a.id === id)
      await invoke('cursor_delete_account', { accountId: id })
      if (account) {
        markItemDeletion(account)
      }
    } catch (error) {
      console.error('Failed to delete account:', error)
    }
  }
  await loadAccounts()
  clearSelection()
  window.$notify?.success($t('platform.cursor.batchDeleteSuccess', { count: ids.length }))
}

const handleBatchExport = async () => {
  if (selectedAccountIds.value.size === 0) return
  const ids = Array.from(selectedAccountIds.value)

  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const { writeTextFile } = await import('@tauri-apps/plugin-fs')

    // 获取导出数据
    const jsonData = await invoke('cursor_export_accounts', {
      accountIds: ids
    })

    // 生成默认文件名
    const defaultFileName = `cursor_accounts_${ids.length}.json`

    // 让用户选择保存位置
    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [
        {
          name: 'JSON',
          extensions: ['json']
        }
      ]
    })

    if (!filePath) {
      return // 用户取消
    }

    // 写入文件
    await writeTextFile(filePath, jsonData)

    window.$notify?.success($t('platform.cursor.messages.exportSuccess'))
  } catch (err) {
    console.error('Batch export error:', err)
    if (err?.message?.includes('Cancelled') || err?.code === 'Cancelled') {
      return // 用户取消，不显示错误
    }
    window.$notify?.error(err?.message || err || $t('platform.cursor.messages.exportFailed'))
  }
}

watch([searchQuery, selectedStatusFilter, selectedMembershipFilter, selectedTags, tagFilterMode], () => {
  currentPage.value = 1
})

const openDataFolder = async () => {
  try {
    await invoke('open_data_folder')
  } catch (error) {
    console.error('Failed to open data folder:', error)
  }
}

const handleMarkAllForSync = () => {
  markAllForSync()
  window.$notify?.success($t('platform.cursor.allAccountsMarkedForSync', { count: accounts.value.length }))
}

// 自定义 Cursor 路径相关函数
const loadCustomPath = async () => {
  try {
    customCursorPath.value = await invoke('cursor_get_custom_path')
    try {
      defaultCursorPath.value = await invoke('cursor_get_default_path')
    } catch (e) {
      defaultCursorPath.value = ''
    }
  } catch (error) {
    console.error('Failed to load custom path:', error)
  }
}

// 自动更新相关函数
const loadAutoUpdateStatus = async () => {
  try {
    isAutoUpdateDisabled.value = await invoke('cursor_check_auto_update_disabled')
  } catch (error) {
    console.error('Failed to check auto-update status:', error)
    isAutoUpdateDisabled.value = false
  }
}

const toggleAutoUpdate = async () => {
  if (isTogglingAutoUpdate.value) return
  isTogglingAutoUpdate.value = true
  try {
    if (isAutoUpdateDisabled.value) {
      await invoke('cursor_enable_auto_update')
      isAutoUpdateDisabled.value = false
      window.$notify?.success('自动更新已启用')
    } else {
      await invoke('cursor_disable_auto_update')
      isAutoUpdateDisabled.value = true
      window.$notify?.success('自动更新已禁用')
    }
  } catch (error) {
    console.error('Failed to toggle auto-update:', error)
    window.$notify?.error(error?.message || error)
  } finally {
    isTogglingAutoUpdate.value = false
  }
}

const handleCustomPathSaved = (newPath) => {
  customCursorPath.value = newPath
}

onMounted(async () => {
  await initSync()
  await loadAccounts()
  await loadCustomPath()
  await loadAutoUpdateStatus()
})
</script>
