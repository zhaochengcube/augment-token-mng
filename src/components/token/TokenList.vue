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
              v-tooltip="isDatabaseAvailable ? $t('tokenList.viewSyncQueueTooltip') : ''"
              @click="isDatabaseAvailable && openSyncQueue()">
              <span :class="['status-dot', storageStatusClass]"></span>
              <span class="text-[11px] font-semibold tracking-[0.3px]">{{ storageStatusText }}</span>
            </div>

            <!-- 功能性操作按钮 -->
            <div class="flex items-center gap-2" @click.stop>
              <button class="btn btn--icon btn--ghost" @click="setToolbarMode('search')" v-tooltip="$t('common.search')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5z" />
                </svg>
              </button>
              <button class="btn btn--icon btn--ghost" @click="setToolbarMode('filter')" v-tooltip="$t('common.filter')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3 4h18l-7 8v6l-4 2v-8L3 4z"/>
                </svg>
              </button>
              <button class="btn btn--icon btn--ghost" @click="setToolbarMode('sort')" v-tooltip="$t('common.sort')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                  stroke-linecap="round">
                  <path d="M7 16V6M4 9l3-3 3 3" />
                  <path d="M17 8v10M14 15l3 3 3-3" />
                </svg>
              </button>
              <button
                class="btn btn--icon btn--ghost"
                @click="toggleViewMode"
                v-tooltip="viewMode === 'card' ? $t('tokenList.switchToTable') : $t('tokenList.switchToCard')"
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
            <button
              v-if="expiringSessionTokens.length > 0"
              class="btn btn--icon btn-tech-warning relative"
              @click="openSessionRefreshModal"
              v-tooltip="$t('tokenList.sessionExpiring', { count: expiringSessionTokens.length })"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
              </svg>
              <span class="absolute -top-1 -right-1 min-w-4 h-4 px-1 text-[10px] font-semibold text-white bg-warning rounded-md flex items-center justify-center shadow-sm pointer-events-none">{{ expiringSessionTokens.length }}</span>
            </button>
            <button @click="handleAddToken" class="btn btn--icon btn--ghost" v-tooltip="$t('tokenList.addToken')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
            </button>
            <button
              v-if="isDatabaseAvailable"
              class="btn btn--icon btn--ghost"
              @click="handleBidirectionalSync"
              :disabled="isSyncing"
              v-tooltip="$t('tokenList.syncTooltip')"
            >
              <svg v-if="!isSyncing" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z" />
              </svg>
              <span v-else class="btn-spinner text-accent" aria-hidden="true"></span>
            </button>
            <button
              class="btn btn--icon btn--ghost"
              @click="handleRefresh"
              :disabled="isRefreshing"
              v-tooltip="$t('tokenList.refresh')"
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
      <!-- Page Body -->
      <!-- Loading State -->
      <div v-if="isLoading" class="text-center py-12 px-6">
        <div class="spinner"></div>
        <p class="text-text-muted m-0">{{ $t('tokenList.loading') }}</p>
      </div>

      <div v-else-if="showEmptyState" class="flex flex-col items-center justify-center py-[60px] px-5 text-text-secondary">
        <div class="text-text-muted mb-[18px] opacity-60">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
          </svg>
        </div>
        <p class="mt-4 text-sm">{{ $t('tokenList.noResults') }}</p>
      </div>

      <template v-else>
        <!-- 卡片布局 -->
        <div v-if="viewMode === 'card'" class="grid grid-cols-[repeat(auto-fill,minmax(330px,1fr))] gap-[18px] p-1.5">
          <TokenCard v-for="token in paginatedTokens" :key="token.id" :ref="el => setTokenCardRef(el, token.id)"
            :token="token" :is-batch-checking="isRefreshing" :is-highlighted="highlightedTokenId === token.id"
            :is-selected="selectedTokenIds.has(token.id)" :selection-mode="isSelectionMode"
            :is-selected-refreshing="isBatchRefreshing"
            :cached-payment-link="paymentLinksCache.get(token.id)"
            :all-tokens="tokens"
            :show-real-email="showRealEmail"
            @delete="deleteToken" @edit="handleEditToken" @token-updated="handleTokenUpdated"
            @select="toggleTokenSelection" @payment-link-fetched="cachePaymentLink" />
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
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                      </svg>
                      <svg v-else-if="isPartialSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M19 13H5v-2h14v2z"/>
                      </svg>
                    </div>
                  </div>
                </th>
                <th class="th w-[85px] max-w-[85px]">{{ $t('tokenList.tableHeaderTag') }}</th>
                <th class="th w-[85px]">{{ $t('tokenList.tableHeaderStatus') }}</th>
                <th class="th min-w-[150px]">{{ $t('tokenList.tableHeaderEmail') }}</th>
                <th class="th w-[85px] text-center">{{ $t('tokenList.tableHeaderBalance') }}</th>
                <th class="th w-[140px] min-w-[140px]">{{ $t('tokenList.tableHeaderDates') }}</th>
                <th class="th w-[230px] text-center">{{ $t('tokenList.tableHeaderActions') }}</th>
              </tr>
            </thead>
            <tbody>
              <TokenTableRow
                v-for="token in paginatedTokens"
                :key="token.id"
                :ref="el => setTokenCardRef(el, token.id)"
                :token="token"
                :is-batch-checking="isRefreshing"
                :is-highlighted="highlightedTokenId === token.id"
                :is-selected="selectedTokenIds.has(token.id)"
                :selection-mode="isSelectionMode"
                :is-selected-refreshing="isBatchRefreshing"
                :cached-payment-link="paymentLinksCache.get(token.id)"
                :all-tokens="tokens"
                :show-real-email="showRealEmail"
                @delete="deleteToken"
                @edit="handleEditToken"
                @token-updated="handleTokenUpdated"
                @select="toggleTokenSelection"
                @payment-link-fetched="cachePaymentLink"
                @open-editor="handleOpenEditor"
                @open-portal="handleOpenPortal"
                @edit-tag="handleEditTag"
              />
            </tbody>
          </table>
        </div>
      </template>
      <template #pagination>
        <Pagination
          :current-page="currentPage"
          :total-pages="totalPages"
          :total-items="filteredTokens.length"
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
          :placeholder="$t('tokenList.searchPlaceholder')"
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
                selectedStatusFilters.size === 0 ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="clearStatusFilter"
            >
              全部
            </button>
            <button
              v-for="option in statusFilterOptions"
              :key="option.key"
              :class="[
                'btn btn--sm',
                selectedStatusFilters.has(option.value) ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectStatusFilter(option.value)"
            >
              <span>{{ option.label }}</span>
              <span class="ml-1 opacity-70">({{ statusStatistics[option.statKey] }})</span>
            </button>
          </div>
        </div>

        <!-- 邮箱后缀筛选 -->
        <div class="flex flex-col gap-2">
          <span class="label">邮箱后缀</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="[
                'btn btn--sm',
                selectedEmailSuffixes.size === 0 ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="clearEmailSuffixFilter"
            >
              全部
            </button>
            <button
              v-for="suffix in emailSuffixes"
              :key="suffix"
              :class="[
                'btn btn--sm',
                selectedEmailSuffixes.has(suffix) ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="selectEmailSuffix(suffix)"
            >
              <span>{{ suffix }}</span>
              <span class="ml-1 opacity-70">({{ emailSuffixStatistics[suffix] }})</span>
            </button>
            <span v-if="emailSuffixes.length === 0" class="text-xs text-text-muted">暂无邮箱后缀</span>
          </div>
        </div>

        <!-- 标签筛选 -->
        <div class="flex flex-col gap-2">
          <span class="label">标签</span>
          <div class="flex flex-wrap gap-2">
            <button
              :class="['btn btn--sm', 'btn--secondary']"
              @click="toggleTagFilterMode"
              v-tooltip="tagFilterMode === 'include' ? '切换到排除模式' : '切换到包含模式'"
            >
              <svg v-if="tagFilterMode === 'include'" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13H5v-2h14v2z"/>
              </svg>
              <span class="ml-1">{{ tagFilterMode === 'include' ? '包含' : '排除' }}</span>
            </button>
            <button
              :class="[
                'btn btn--sm',
                selectedTags.size === 0 ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="clearTagFilter"
            >
              全部
            </button>
            <button
              v-for="tag in allTags"
              :key="tag"
              :class="[
                'btn btn--sm',
                selectedTags.has(tag) ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="toggleTag(tag)"
            >
              {{ tag }}
            </button>
            <span v-if="allTags.length === 0" class="text-xs text-text-muted">暂无标签</span>
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
              @click="setSortOrder('asc')"
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
              @click="setSortOrder('desc')"
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
              @click="setSortField('time')"
            >
              {{ $t('tokenList.sortByTime') }}
            </button>
            <button
              :class="[
                'btn btn--sm',
                sortType === 'balance' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortField('balance')"
            >
              {{ $t('tokenList.sortByBalance') }}
            </button>
            <button
              :class="[
                'btn btn--sm',
                sortType === 'tag' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortField('tag')"
            >
              {{ $t('tokenList.sortByTag') }}
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
          v-tooltip="$t('bookmarkManager.openDataFolder')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z" />
          </svg>
          <span>{{ $t('common.openDataFolder') }}</span>
        </button>

        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="showBatchImportConfirm"
          v-tooltip="$t('tokenList.batchImport')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z" />
          </svg>
          <span>{{ $t('tokenList.batchImport') }}</span>
        </button>

        <button
          class="btn btn--ghost btn--sm inline-flex items-center gap-2"
          @click="handleBatchDelete"
          v-tooltip="$t('tokenList.batchDelete')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
          </svg>
          <span>{{ $t('tokenList.batchDelete') }}</span>
        </button>
      </div>
    </ActionToolbar>

    <!-- 批量操作工具栏 -->
    <BatchToolbar
      :visible="isSelectionMode"
      :selected-count="selectedTokenIds.size"
      @select-all="selectAllOnPage"
      @clear="clearSelection">
      <template #actions>
        <!-- 批量刷新状态 -->
        <button
          @click="batchRefreshSelected"
          class="btn btn--icon btn--ghost"
          :disabled="isBatchRefreshing"
          v-tooltip="$t('tokenList.batchRefreshSelected')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
        </button>

        <!-- 批量刷新 Session -->
        <button
          @click="batchRefreshSessionsSelected"
          class="btn btn--icon btn--ghost"
          :disabled="isBatchRefreshingSessions"
          v-tooltip="$t('tokenList.batchRefreshSessionSelected')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
          </svg>
        </button>

        <!-- 批量导出 -->
        <button
          @click="batchExportSelected"
          class="btn btn--icon btn--ghost"
          v-tooltip="$t('tokenList.batchExportSelected')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>

        <!-- 批量获取绑卡链接 -->
        <button
          @click="batchFetchPaymentLinks"
          class="btn btn--icon btn--ghost"
          :disabled="isBatchFetchingPaymentLinks"
          v-tooltip="$t('tokenList.batchFetchPaymentLinks')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
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
          v-tooltip="$t('tokenList.batchDeleteSelected')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </template>
    </BatchToolbar>

    <!-- Token Form Modal -->
    <TokenForm
      v-if="showTokenFormModal"
      :token="editingToken"
      :all-tokens="tokens"
      @close="closeTokenForm"
      @success="handleTokenFormSuccess"
      @update-token="handleUpdateToken"
      @add-token="handleAddTokenFromForm"
      @auto-import-completed="handleAutoImportCompleted"
      @manual-import-completed="handleManualImportCompleted"
    />

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="tokens.length"
      :title="$t('tokenList.syncQueueTitle')"
      :upserts-title="$t('tokenList.syncQueueUpsertsTitle')"
      :deletions-title="$t('tokenList.syncQueueDeletionsTitle')"
      :empty-text="$t('tokenList.syncQueueEmpty')"
      :full-sync-text="$t('tokenList.fullSync')"
      :sync-text="$t('tokenList.sync')"
      :no-label-text="$t('tokenList.noEmailNote')"
      label-field="email_note"
      fallback-field="id"
      @sync="handleBidirectionalSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />

    <!-- Session Refresh Modal -->
    <SessionRefreshModal
      v-model:visible="showSessionRefreshModal"
      :expiring-tokens="expiringSessionTokens"
      :refreshing="isRefreshingSessions"
      @refresh="handleBatchRefreshSessions"
      @refresh-single="handleSingleRefreshSession"
    />

    <!-- Batch Import Dialog -->
    <BatchImportModal
      v-model:visible="showBatchImportDialog"
      :default-input-count="defaultSessionInputCount"
      @import="handleBatchImportData"
    />

    <!-- Batch Delete Confirmation Dialog -->
    <DeleteConfirmModal
      v-model:visible="showBatchDeleteDialog"
      :title="$t('tokenList.batchDeleteConfirm')"
      :message="$t('tokenList.batchDeleteMessage')"
      @confirm="executeBatchDelete"
    >
      <template #stats>
        <div class="flex items-center justify-between text-sm">
          <span class="text-text-muted">{{ $t('tokenList.bannedCount') }}:</span>
          <span class="font-medium text-text">{{ bannedTokensCount }} {{ $t('tokenList.items') }}</span>
        </div>
        <div class="flex items-center justify-between text-sm">
          <span class="text-text-muted">{{ $t('tokenList.expiredCount') }}:</span>
          <span class="font-medium text-text">{{ expiredTokensCount }} {{ $t('tokenList.items') }}</span>
        </div>
        <div class="flex items-center justify-between text-sm border-t border-border pt-2 mt-1">
          <span class="font-medium text-text">{{ $t('tokenList.totalCount') }}:</span>
          <span class="font-semibold text-danger">{{ deletableTokensCount }} {{ $t('tokenList.items') }}</span>
        </div>
      </template>
    </DeleteConfirmModal>

    <!-- Selected Tokens Delete Confirmation Dialog -->
    <DeleteConfirmModal
      v-model:visible="showSelectedDeleteDialog"
      :title="$t('tokenList.batchDeleteConfirm')"
      :message="$t('tokenList.selectedDeleteMessage')"
      @confirm="executeBatchDeleteSelected"
    >
      <template #stats>
        <div class="flex items-center justify-between text-sm">
          <span class="font-medium text-text">{{ $t('tokenList.selectedCount') }}:</span>
          <span class="font-semibold text-danger">{{ selectedTokenIds.size }} {{ $t('tokenList.items') }}</span>
        </div>
      </template>
    </DeleteConfirmModal>

    <!-- 批量编辑标签模态框 -->
    <TagEditorModal
      v-model:visible="showBatchTagEditor"
      :tokens="selectedTokens"
      :all-tokens="tokens"
      @save="handleBatchTagSave"
      @clear="handleBatchTagClear"
    />
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted, onUnmounted, computed, readonly, watch } from 'vue'
import { watchDebounced } from '@vueuse/core'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import TokenCard from './TokenCard.vue'
import TokenTableRow from './TokenTableRow.vue'
import TokenForm from './TokenForm.vue'
import TagEditorModal from './TagEditorModal.vue'
import BatchImportModal from './BatchImportModal.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import DeleteConfirmModal from '../common/DeleteConfirmModal.vue'
import SessionRefreshModal from './SessionRefreshModal.vue'
import Pagination from '../common/Pagination.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import ActionToolbar from '../common/ActionToolbar.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'
import { useStorageSync } from '@/composables/useStorageSync'

const { t } = useI18n()

// Props
const props = defineProps({
  tokens: {
    type: Array,
    default: null // null表示自主管理，非null表示由父组件传入
  },
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})

// 内部状态管理 - TokenList直接管理tokens和存储状态
// 如果props.tokens不为null，则使用props.tokens，否则自主管理
const internalTokens = ref([])
const tokens = computed(() => props.tokens !== null ? props.tokens : internalTokens.value)
const isLoading = ref(false)

// 初始化就绪标记
const isReady = ref(false)

// Session 刷新相关状态
const showSessionRefreshModal = ref(false)
const isRefreshingSessions = ref(false)

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
  markAllForSync,
  openSyncQueue,
  closeSyncQueue,
  handleSync: composableHandleSync
} = useStorageSync({
  platform: 'augment',
  syncCommand: 'sync_tokens',
  items: internalTokens,
  itemKey: 'token',
  labelField: 'email_note',
  onSyncComplete: async () => {
    // 同步完成后保存到本地文件
    await handleSave()
  }
})

// 事件监听器取消函数
let unlistenTokensUpdated = null

// 排序状态管理
const sortType = ref('time') // 'time' = 按时间排序, 'balance' = 按余额排序, 'tag' = 按标签排序
const sortOrder = ref('desc') // 'desc' = 最新优先/余额从多到少, 'asc' = 最旧优先/余额从少到多
const showSortMenu = ref(false) // 排序下拉菜单显示状态
const showStatusFilterMenu = ref(false) // 状态筛选下拉菜单显示状态

// 视图模式管理
const viewMode = ref('card') // 'card' = 卡片布局, 'table' = 列表布局

// 邮箱显示模式
const showRealEmail = ref(true) // false = 脱敏显示, true = 真实邮箱

// 模态框主体容器引用
const tokenContentScrollRef = ref(null)

// 搜索状态管理
const searchQuery = ref('')
const toolbarMode = ref('hidden')
const toolbarSearchInputRef = ref(null)

// 状态筛选管理 - 多选模式
const selectedStatusFilters = ref(new Set()) // 选中的状态集合，空集合表示"全部"

// 邮箱后缀筛选管理 - 多选模式
const selectedEmailSuffixes = ref(new Set()) // 选中的邮箱后缀集合，空集合表示"全部"
const showEmailSuffixMenu = ref(false) // 邮箱后缀下拉菜单显示状态

// 标签筛选管理
const selectedTags = ref(new Set()) // 选中的标签集合
const tagFilterMode = ref('include') // 'include' = 包含模式, 'exclude' = 排除模式
const showTagFilterMenu = ref(false) // 标签筛选下拉菜单显示状态

// 分页状态管理
const currentPage = ref(1)           // 当前页码
const pageSize = ref(20)             // 每页显示数量(默认 20)
const pageSizeOptions = [10, 20, 50, 100, 200]  // 可选的每页数量

// 高亮状态管理
const highlightedTokenId = ref(null)
let highlightTimer = null

const DEFAULT_TAG_COLOR = '#f97316'

// 批量删除状态
const showBatchDeleteDialog = ref(false)
const isDeleting = ref(false)

// 批量选择状态
const selectedTokenIds = ref(new Set())
const isSelectionMode = computed(() => selectedTokenIds.value.size > 0)
const showSelectedDeleteDialog = ref(false)
const isBatchRefreshing = ref(false)
const isBatchRefreshingSessions = ref(false)
const isBatchFetchingPaymentLinks = ref(false)
const showBatchTagEditor = ref(false)

// 绑卡链接缓存（Session 级别，不持久化）
const paymentLinksCache = ref(new Map())

// 批量导入状态
const showBatchImportDialog = ref(false)
const batchImportTab = ref('session') // 'session' 或 'token'
const importJsonText = ref('')
const isImporting = ref(false)
const importPreview = ref([])
const importErrors = ref([])
const defaultSessionInputCount = 3

// Session 动态输入框状态
const sessionInputs = ref([])
let nextSessionInputId = 1

// localStorage 配置键名
const STORAGE_KEY_DEFAULT_INPUT_COUNT = 'atm-default-session-input-count'

// UI 配置
const defaultInputCount = ref(3)

// 从 localStorage 加载配置
const loadDefaultInputCount = () => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_DEFAULT_INPUT_COUNT)
    if (stored) {
      const count = parseInt(stored, 10)
      if (!isNaN(count) && count >= 1 && count <= 20) {
        return count
      }
    }
  } catch (error) {
    console.warn('Failed to load default input count from localStorage:', error)
  }
  return 3 // 默认值
}

// 保存配置到 localStorage
const saveDefaultInputCount = (count) => {
  try {
    localStorage.setItem(STORAGE_KEY_DEFAULT_INPUT_COUNT, count.toString())
    return true
  } catch (error) {
    console.error('Failed to save default input count to localStorage:', error)
    return false
  }
}

// 智能提取email字段的辅助函数
// 支持多种外部格式的email字段命名
const extractEmail = (item) => {
  // 按优先级顺序查找email字段
  const emailFields = [
    'email_note',    // 优先：当前应用标准格式
    'email',         // 次优：常见外部格式
    'emailNote',     // 驼峰格式
    'Email',         // 首字母大写
    'user_email',    // 带前缀
    'userEmail',     // 驼峰带前缀
    'mail'           // 简写
  ]

  for (const field of emailFields) {
    const value = item[field]
    // 验证字段存在、类型正确且值有效
    if (value && typeof value === 'string' && value.trim()) {
      return value.trim()
    }
  }

  return null
}

// 从邮箱地址中提取后缀的辅助函数
// 返回包含 @ 符号的后缀,如 "@gmail.com"
const extractEmailSuffix = (email) => {
  if (!email || typeof email !== 'string') {
    return null
  }

  const trimmedEmail = email.trim()
  const atIndex = trimmedEmail.indexOf('@')

  // 验证 @ 符号存在且不在末尾
  if (atIndex === -1 || atIndex >= trimmedEmail.length - 1) {
    return null
  }

  // 返回包含 @ 符号的后缀
  return trimmedEmail.substring(atIndex)
}

// 初始化 Session 输入框
const initializeSessionInputs = (count) => {
  const inputs = []
  for (let i = 1; i <= count; i++) {
    inputs.push({ id: i, value: '' })
  }
  sessionInputs.value = inputs
  nextSessionInputId = count + 1
}

// 填充 Session 模板
const fillSessionTemplate = (count = 1) => {
  const sessions = []
  for (let i = 0; i < count; i++) {
    sessions.push(i === 0 ? 'session1' : '')
  }
  importJsonText.value = JSON.stringify(sessions, null, 2)
  validateImportJson()
}

// 填充 Token 模板
const fillTokenTemplate = (count = 1) => {
  const tokens = []
  for (let i = 0; i < count; i++) {
    tokens.push({
      access_token: i === 0 ? 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...' : '',
      tenant_url: i === 0 ? 'https://example.com' : '',
      email_note: i === 0 ? 'user@example.com' : '',
      portal_url: i === 0 ? 'https://portal.example.com' : ''
    })
  }
  importJsonText.value = JSON.stringify(tokens, null, 2)
  validateImportJson()
}

// 计算可删除的 token 数量
const deletableTokensCount = computed(() => {
  return tokens.value.filter(token =>
    token.ban_status === 'SUSPENDED' || token.ban_status === 'EXPIRED'
  ).length
})

// 计算已封禁的 token 数量
const bannedTokensCount = computed(() => {
  return tokens.value.filter(token => token.ban_status === 'SUSPENDED').length
})

// 计算已过期的 token 数量
const expiredTokensCount = computed(() => {
  return tokens.value.filter(token => token.ban_status === 'EXPIRED').length
})

// 只应用搜索的tokens - 用于状态筛选统计
const searchFilteredTokens = computed(() => {
  let result = sortedTokens.value

  // 只应用搜索过滤
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase().trim()
    result = result.filter(token => {
      const matchesOriginalFields = (
        token.access_token?.toLowerCase().includes(query) ||
        token.email_note?.toLowerCase().includes(query) ||
        token.auth_session?.toLowerCase().includes(query) ||
        token.tag_name?.toLowerCase().includes(query)
      )

      const matchesStatus = matchStatusKeyword(token.ban_status, query)

      return matchesOriginalFields || matchesStatus
    })
  }

  return result
})

// 应用状态筛选 - 用于邮箱后缀和标签选项的统计
const statusOnlyFilteredTokens = computed(() => {
  let result = searchFilteredTokens.value

  // 应用状态筛选 - 多选模式
  if (selectedStatusFilters.value.size > 0) {
    result = result.filter(token => {
      const status = token.ban_status
      const portalInfo = token.portal_info
      const creditTotal = portalInfo?.credit_total
      const creditsBalance = portalInfo?.credits_balance

      // 检查token是否匹配任一选中的状态
      return Array.from(selectedStatusFilters.value).some(filterStatus => {
        if (filterStatus === 'ACTIVE') {
          return status === 'ACTIVE' && !(creditTotal && creditsBalance === 0)
        } else if (filterStatus === 'DEPLETE') {
          return status === 'ACTIVE' && creditTotal && creditsBalance === 0
        } else if (filterStatus === 'SUSPENDED') {
          return status === 'SUSPENDED'
        } else if (filterStatus === 'OTHER') {
          return status !== 'ACTIVE' && status !== 'SUSPENDED'
        }
        return false
      })
    })
  }

  return result
})

// 状态统计计算属性 - 基于搜索筛选后的结果
const statusStatistics = computed(() => {
  const stats = {
    ACTIVE: 0,
    DEPLETE: 0,
    SUSPENDED: 0,
    OTHER: 0,
    TOTAL: searchFilteredTokens.value.length
  }

  searchFilteredTokens.value.forEach(token => {
    const status = token.ban_status
    const portalInfo = token.portal_info
    const creditTotal = portalInfo?.credit_total
    const creditsBalance = portalInfo?.credits_balance

    if (status === 'ACTIVE') {
      // 判断是否用尽：credit_total 不为空/不为null 且 credits_balance === 0
      if (creditTotal && creditsBalance === 0) {
        stats.DEPLETE++
      } else {
        stats.ACTIVE++
      }
    } else if (status === 'SUSPENDED') {
      stats.SUSPENDED++
    } else {
      // 所有其他状态(EXPIRED, INVALID_TOKEN, ERROR等)都归入OTHER
      stats.OTHER++
    }
  })

  return stats
})

const statusFilterOptions = computed(() => ([
  {
    key: 'all',
    value: null,
    label: t('tokenList.allStatus'),
    statKey: 'TOTAL',
    chipClass: 'text-[var(--text-muted)] border-[var(--border)] hover:border-[var(--accent)]',
    activeClass: 'bg-[var(--bg-muted)] border-[var(--accent)] text-[var(--text)]'
  },
  {
    key: 'active',
    value: 'ACTIVE',
    label: t('tokenList.activeStatus'),
    statKey: 'ACTIVE',
    chipClass: 'text-emerald-600 border-emerald-200 hover:border-emerald-300',
    activeClass: 'bg-emerald-50 border-emerald-400 text-emerald-700'
  },
  {
    key: 'deplete',
    value: 'DEPLETE',
    label: t('tokenList.depleteStatus'),
    statKey: 'DEPLETE',
    chipClass: 'text-amber-600 border-amber-200 hover:border-amber-300',
    activeClass: 'bg-amber-50 border-amber-400 text-amber-700'
  },
  {
    key: 'suspended',
    value: 'SUSPENDED',
    label: t('tokenList.suspendedStatus'),
    statKey: 'SUSPENDED',
    chipClass: 'text-red-600 border-red-200 hover:border-red-300',
    activeClass: 'bg-red-50 border-red-400 text-red-700'
  },
  {
    key: 'other',
    value: 'OTHER',
    label: t('tokenList.otherStatus'),
    statKey: 'OTHER',
    chipClass: 'text-slate-600 border-slate-200 hover:border-slate-300',
    activeClass: 'bg-slate-100 border-slate-300 text-slate-700'
  }
]))

// 邮箱后缀统计 - 基于状态筛选后的token列表（不包含邮箱后缀筛选）
const emailSuffixStatistics = computed(() => {
  const stats = {}

  statusOnlyFilteredTokens.value.forEach(token => {
    // 使用辅助函数提取邮箱后缀
    const suffix = extractEmailSuffix(token.email_note)
    if (suffix) {
      stats[suffix] = (stats[suffix] || 0) + 1
    }
  })

  return stats
})

// 提取所有唯一的邮箱后缀 - 按数量从大到小排序
const emailSuffixes = computed(() => {
  const stats = emailSuffixStatistics.value

  // 转换为数组并按数量排序(从大到小)
  return Object.keys(stats).sort((a, b) => {
    return stats[b] - stats[a]
  })
})

// 辅助函数：判断 session 是否需要刷新（25-30天）
const shouldRefreshSession = (token) => {
  if (!token.auth_session || !token.created_at) return false

  // 使用 session_updated_at（如果存在）或 created_at 来判断
  const referenceTime = token.session_updated_at || token.created_at
  const referenceDate = new Date(referenceTime)
  const now = new Date()
  const daysSinceReference = Math.floor((now - referenceDate) / (1000 * 60 * 60 * 24))

  return daysSinceReference >= 25 && daysSinceReference < 30
}

// 计算即将过期的 session tokens（25-30天，且状态为正常）
// 按剩余天数从少到多排序（最紧急的排在前面）
const expiringSessionTokens = computed(() => {
  const filtered = tokens.value.filter(token => {
    // 必须有 auth_session 和 created_at
    if (!token.auth_session || !token.created_at) return false

    return shouldRefreshSession(token)
  })

  // 按剩余天数排序（从少到多，最紧急的在前）
  return filtered.sort((a, b) => {
    const daysA = Math.floor((new Date() - new Date(a.created_at)) / (1000 * 60 * 60 * 24))
    const daysB = Math.floor((new Date() - new Date(b.created_at)) / (1000 * 60 * 60 * 24))
    return daysB - daysA // 天数大的在前（剩余天数少的在前）
  })
})

// 提取所有唯一的标签 - 按字母排序（忽略大小写）- 基于状态筛选后的结果（不包含标签筛选）
const allTags = computed(() => {
  const tagMap = new Map() // 使用 Map 来存储小写 -> 原始标签的映射

  statusOnlyFilteredTokens.value.forEach(token => {
    const tagName = token.tag_name?.trim()
    if (tagName) {
      const lowerTag = tagName.toLowerCase()
      // 如果已存在相同的小写标签，保留第一个出现的原始大小写形式
      if (!tagMap.has(lowerTag)) {
        tagMap.set(lowerTag, tagName)
      }
    }
  })

  // 转换为数组并按字母排序（忽略大小写）
  return Array.from(tagMap.values()).sort((a, b) => {
    return a.toLowerCase().localeCompare(b.toLowerCase(), 'zh-CN')
  })
})

// 排序后的tokens计算属性
const sortedTokens = computed(() => {
  if (tokens.value.length === 0) return []

  return [...tokens.value].sort((a, b) => {
    if (sortType.value === 'time') {
      // 按时间排序
      const dateA = new Date(a.created_at)
      const dateB = new Date(b.created_at)

      if (sortOrder.value === 'desc') {
        return dateB - dateA // 最新优先
      } else {
        return dateA - dateB // 最旧优先
      }
    } else if (sortType.value === 'balance') {
      // 按余额排序
      const balanceA = a.portal_info?.credits_balance
      const balanceB = b.portal_info?.credits_balance

      // 处理没有余额信息的情况
      const hasBalanceA = balanceA !== undefined && balanceA !== null
      const hasBalanceB = balanceB !== undefined && balanceB !== null

      // 没有余额信息的排在最后
      if (!hasBalanceA && !hasBalanceB) return 0
      if (!hasBalanceA) return 1
      if (!hasBalanceB) return -1

      // 都有余额信息,按数值排序
      if (sortOrder.value === 'desc') {
        return balanceB - balanceA // 余额从多到少
      } else {
        return balanceA - balanceB // 余额从少到多
      }
    } else if (sortType.value === 'tag') {
      // 按标签排序
      const tagA = a.tag_name || ''
      const tagB = b.tag_name || ''

      // 处理无标签的情况：升序时无标签在前，降序时无标签在后
      if (!tagA && !tagB) return 0
      if (!tagA) return sortOrder.value === 'asc' ? -1 : 1
      if (!tagB) return sortOrder.value === 'asc' ? 1 : -1

      // 都有标签,按字母排序
      const comparison = tagA.localeCompare(tagB, 'zh-CN')
      if (sortOrder.value === 'asc') {
        return comparison // A-Z
      } else {
        return -comparison // Z-A
      }
    }
    return 0
  })
})

// 状态关键词匹配辅助函数 - 支持中英文及常见别名搜索
const matchStatusKeyword = (banStatus, query) => {
  if (!banStatus || !query) return false

  const lowerQuery = query.toLowerCase()

  // 状态关键词映射表（支持中英文及别名）
  const statusKeywords = {
    'ACTIVE': ['active', 'normal', '正常', '激活', '可用'],
    'SUSPENDED': ['suspended', 'banned', 'ban', '封禁', '已封禁', '被封', '禁用'],
    'EXPIRED': ['expired', 'expire', '过期', '已过期', '到期'],
    'INVALID_TOKEN': ['invalid', 'token invalid', '失效', 'token失效', '无效']
  }

  // 获取当前状态的关键词列表
  const keywords = statusKeywords[banStatus] || []

  // 检查是否有任何关键词包含查询词（支持部分匹配）
  return keywords.some(keyword => keyword.includes(lowerQuery))
}

// 最终筛选后的tokens - 应用所有筛选条件
const filteredTokens = computed(() => {
  let result = statusOnlyFilteredTokens.value

  // 1. 应用邮箱后缀筛选 - 多选模式
  if (selectedEmailSuffixes.value.size > 0) {
    result = result.filter(token => {
      const suffix = extractEmailSuffix(token.email_note)
      return suffix && selectedEmailSuffixes.value.has(suffix)
    })
  }

  // 2. 应用标签筛选（忽略大小写）
  if (selectedTags.value.size > 0) {
    const lowerSelectedTags = new Set(
      Array.from(selectedTags.value).map(tag => tag.toLowerCase())
    )

    result = result.filter(token => {
      const tagName = token.tag_name?.trim() || ''
      const lowerTagName = tagName.toLowerCase()

      if (tagFilterMode.value === 'include') {
        return tagName && lowerSelectedTags.has(lowerTagName)
      } else {
        return !tagName || !lowerSelectedTags.has(lowerTagName)
      }
    })
  }

  return result
})

// 总页数
const totalPages = computed(() => {
  return Math.ceil(filteredTokens.value.length / pageSize.value)
})

// 当前页的 tokens
const paginatedTokens = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredTokens.value.slice(start, end)
})

const showEmptyState = computed(() => !isLoading.value && filteredTokens.value.length === 0)
const shouldShowPagination = computed(() => !isLoading.value && tokens.value.length > 0 && filteredTokens.value.length > 0)

// 切换排序方式
const toggleSort = (type = 'time') => {
  // 如果切换到不同的排序类型,重置为降序
  if (sortType.value !== type) {
    sortType.value = type
    sortOrder.value = 'desc'
  } else {
    // 同一类型,切换升序/降序
    sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
  }

  clearSortHighlight()
}

// 切换排序菜单显示
const toggleSortMenu = () => {
  showSortMenu.value = !showSortMenu.value
  // 关闭其他菜单
  if (showSortMenu.value) {
    showStatusFilterMenu.value = false
    showEmailSuffixMenu.value = false
    showTagFilterMenu.value = false
  }
}

// 切换状态筛选菜单显示
const toggleStatusFilterMenu = () => {
  showStatusFilterMenu.value = !showStatusFilterMenu.value
  // 关闭其他菜单
  if (showStatusFilterMenu.value) {
    showSortMenu.value = false
    showEmailSuffixMenu.value = false
    showTagFilterMenu.value = false
  }
}

// 切换视图模式
const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'card' ? 'table' : 'card'
  // 保存到 localStorage
  localStorage.setItem('tokenListViewMode', viewMode.value)
}

// 从 localStorage 加载视图模式
const loadViewMode = () => {
  const saved = localStorage.getItem('tokenListViewMode')
  if (saved === 'card' || saved === 'table') {
    viewMode.value = saved
  }
}

// 设置排序类型和顺序
const setSortType = (type, order) => {
  sortType.value = type
  sortOrder.value = order
  showSortMenu.value = false
  clearSortHighlight()
}

const setSortField = (type) => {
  if (sortType.value === type) return
  sortType.value = type
  clearSortHighlight()
}

const setSortOrder = (order) => {
  if (sortOrder.value === order) return
  sortOrder.value = order
  clearSortHighlight()
}

const clearSortHighlight = () => {
  if (highlightedTokenId.value) {
    highlightedTokenId.value = null
    if (highlightTimer) {
      clearTimeout(highlightTimer)
      highlightTimer = null
    }
  }
}

const closeToolbarMenus = () => {
  showSortMenu.value = false
  showStatusFilterMenu.value = false
  showEmailSuffixMenu.value = false
  showTagFilterMenu.value = false
}

const setToolbarMode = (mode) => {
  toolbarMode.value = toolbarMode.value === mode ? 'hidden' : mode
  closeToolbarMenus()
  if (toolbarMode.value === 'search') {
    nextTick(() => toolbarSearchInputRef.value?.focus())
  }
}

const openToolbarFromHeader = () => {
  if (toolbarMode.value === 'hidden') {
    toolbarMode.value = 'more'
  }
}

// 选择状态筛选 - 多选模式
const selectStatusFilter = (status) => {
  const newSet = new Set(selectedStatusFilters.value)

  if (newSet.has(status)) {
    newSet.delete(status)
  } else {
    newSet.add(status)
  }

  selectedStatusFilters.value = newSet
  // 重置到第一页
  currentPage.value = 1
}

// 清除状态筛选
const clearStatusFilter = () => {
  selectedStatusFilters.value = new Set()
  currentPage.value = 1
}

// 切换邮箱后缀筛选菜单显示
const toggleEmailSuffixMenu = () => {
  showEmailSuffixMenu.value = !showEmailSuffixMenu.value
  // 关闭其他菜单
  if (showEmailSuffixMenu.value) {
    showSortMenu.value = false
    showStatusFilterMenu.value = false
    showTagFilterMenu.value = false
  }
}

// 选择邮箱后缀筛选 - 多选模式
const selectEmailSuffix = (suffix) => {
  const newSet = new Set(selectedEmailSuffixes.value)

  if (newSet.has(suffix)) {
    newSet.delete(suffix)
  } else {
    newSet.add(suffix)
  }

  selectedEmailSuffixes.value = newSet
  // 重置到第一页
  currentPage.value = 1
}

// 清除邮箱后缀筛选
const clearEmailSuffixFilter = () => {
  selectedEmailSuffixes.value = new Set()
  currentPage.value = 1
}

// 提取邮箱后缀到剪贴板
const extractEmailSuffixes = async () => {
  if (emailSuffixes.value.length === 0) {
    window.$notify.warning('暂无邮箱后缀可提取')
    return
  }

  try {
    // 将邮箱后缀以逗号分隔的格式复制到剪贴板
    const suffixText = emailSuffixes.value.join(', ')
    await navigator.clipboard.writeText(suffixText)

    window.$notify.success(`已复制 ${emailSuffixes.value.length} 个邮箱后缀到剪贴板`)

    // 关闭下拉菜单
    showEmailSuffixMenu.value = false
  } catch (error) {
    console.error('Failed to copy email suffixes:', error)
    window.$notify.error('复制失败,请重试')
  }
}

// ========== 标签筛选相关函数 ==========

// 切换标签筛选菜单显示
const toggleTagFilterMenu = () => {
  showTagFilterMenu.value = !showTagFilterMenu.value
  // 关闭其他菜单
  if (showTagFilterMenu.value) {
    showSortMenu.value = false
    showStatusFilterMenu.value = false
    showEmailSuffixMenu.value = false
  }
}

// 切换标签筛选模式
const toggleTagFilterMode = () => {
  tagFilterMode.value = tagFilterMode.value === 'include' ? 'exclude' : 'include'
  // 如果有选中的标签，重置到第一页
  if (selectedTags.value.size > 0) {
    currentPage.value = 1
  }
}

// 选择/取消选择标签
const toggleTag = (tagName) => {
  const newSet = new Set(selectedTags.value)

  if (newSet.has(tagName)) {
    newSet.delete(tagName)
  } else {
    newSet.add(tagName)
  }

  selectedTags.value = newSet
  // 重置到第一页
  currentPage.value = 1
}

// 清除标签筛选
const clearTagFilter = () => {
  selectedTags.value = new Set()
  showTagFilterMenu.value = false
  currentPage.value = 1
}

// 处理页面内容点击 (关闭所有下拉菜单)
const handlePageContentClick = (event) => {
  const target = event.target

  // 关闭排序菜单
  if (showSortMenu.value) {
    const sortDropdown = document.querySelector('.sort-dropdown')
    if (sortDropdown && !sortDropdown.contains(target)) {
      showSortMenu.value = false
    }
  }

  // 关闭状态筛选菜单
  if (showStatusFilterMenu.value) {
    const statusFilterDropdown = document.querySelector('.status-filter-dropdown')
    if (statusFilterDropdown && !statusFilterDropdown.contains(target)) {
      showStatusFilterMenu.value = false
    }
  }

  // 关闭邮箱后缀菜单
  if (showEmailSuffixMenu.value) {
    const emailSuffixDropdown = document.querySelector('.email-suffix-dropdown')
    if (emailSuffixDropdown && !emailSuffixDropdown.contains(target)) {
      showEmailSuffixMenu.value = false
    }
  }

  // 关闭标签筛选菜单
  if (showTagFilterMenu.value) {
    const tagFilterDropdown = document.querySelector('.tag-filter-dropdown')
    if (tagFilterDropdown && !tagFilterDropdown.contains(target)) {
      showTagFilterMenu.value = false
    }
  }
}

// 处理页码变化
const handlePageChange = (page) => {
  currentPage.value = page
  // 滚动到顶部
  nextTick(() => {
    // 直接使用 token-content-scroll 容器
    if (tokenContentScrollRef.value) {
      tokenContentScrollRef.value.scrollTop = 0
    }
  })
}

// 切换每页数量
const changePageSize = (newSize) => {
  pageSize.value = newSize
  // 保存到 localStorage
  localStorage.setItem('tokenListPageSize', newSize.toString())
  // 重新计算当前页(保持当前第一条 token 可见)
  const firstTokenIndex = (currentPage.value - 1) * pageSize.value
  currentPage.value = Math.floor(firstTokenIndex / newSize) + 1
}

// 从 localStorage 加载每页数量
const loadPageSize = () => {
  const saved = localStorage.getItem('tokenListPageSize')
  if (saved) {
    const size = parseInt(saved)
    if (pageSizeOptions.includes(size)) {
      pageSize.value = size
    }
  }
}

// 处理批量删除按钮点击
const handleBatchDelete = () => {
  showBatchDeleteDialog.value = true
}

// ========== 批量选择相关函数 ==========

// 切换单个 token 的选中状态
const toggleTokenSelection = (tokenId) => {
  const newSet = new Set(selectedTokenIds.value)
  if (newSet.has(tokenId)) {
    newSet.delete(tokenId)
  } else {
    newSet.add(tokenId)
  }
  selectedTokenIds.value = newSet
}

// 全选当前页
const selectAllOnPage = () => {
  const newSet = new Set(selectedTokenIds.value)
  paginatedTokens.value.forEach(token => newSet.add(token.id))
  selectedTokenIds.value = newSet
}

// 清除所有选择
const clearSelection = () => {
  selectedTokenIds.value = new Set()
}

// 获取选中的 tokens
const selectedTokens = computed(() => {
  return tokens.value.filter(t => selectedTokenIds.value.has(t.id))
})

// 表格全选相关计算属性
const isAllSelected = computed(() => {
  if (paginatedTokens.value.length === 0) return false
  return paginatedTokens.value.every(token => selectedTokenIds.value.has(token.id))
})

const isPartialSelected = computed(() => {
  if (paginatedTokens.value.length === 0) return false
  const selectedCount = paginatedTokens.value.filter(token => selectedTokenIds.value.has(token.id)).length
  return selectedCount > 0 && selectedCount < paginatedTokens.value.length
})

// 切换全选状态
const toggleSelectAll = () => {
  if (isAllSelected.value) {
    // 取消选择当前页所有项
    const newSet = new Set(selectedTokenIds.value)
    paginatedTokens.value.forEach(token => newSet.delete(token.id))
    selectedTokenIds.value = newSet
  } else {
    // 选择当前页所有项
    selectAllOnPage()
  }
}

// 处理打开编辑器（从 TokenTableRow 触发）
// 通过设置当前 token 并打开编辑表单来实现编辑器选择
const handleOpenEditor = (token) => {
  // 直接使用编辑功能来打开 Token 表单，用户可以在表单中选择编辑器
  editingToken.value = token
  showTokenFormModal.value = true
}

// 处理打开 Portal（从 TokenTableRow 触发）
const handleOpenPortal = (token) => {
  if (token.portal_url) {
    // 直接在浏览器中打开 Portal URL
    window.open(token.portal_url, '_blank')
  }
}

// 处理编辑标签（从 TokenTableRow 触发）
const handleEditTag = (token) => {
  // 打开 Token 编辑表单，用户可以编辑标签
  editingToken.value = token
  showTokenFormModal.value = true
}

// 批量刷新选中的 token 状态
const batchRefreshSelected = async () => {
  if (selectedTokenIds.value.size === 0 || isBatchRefreshing.value) return
  
  isBatchRefreshing.value = true
  const selectedIds = Array.from(selectedTokenIds.value)
  
  try {
    // 获取选中的 tokens（过滤掉标记为跳过检测的）
    const tokensToCheck = tokens.value
      .filter(t => selectedIds.includes(t.id) && !t.skip_check)
    
    if (tokensToCheck.length === 0) {
      window.$notify.warning(t('messages.noTokensToCheck'))
      return
    }
    
    window.$notify.info(t('tokenList.batchRefreshing', { count: tokensToCheck.length }))
    
    // 使用公共方法批量检测
    const result = await batchCheckTokensStatus(tokensToCheck)
    
    if (result.hasChanges) {
      await handleSave()
      if (isDatabaseAvailable.value) {
        isSyncNeeded.value = true
      }
    }
    
    window.$notify.success(t('tokenList.batchRefreshComplete', { count: tokensToCheck.length }))
  } catch (error) {
    console.error('Batch refresh error:', error)
    window.$notify.error(`${t('messages.refreshFailed')}: ${error.message || error}`)
  } finally {
    isBatchRefreshing.value = false
    // 操作完成后清除选择
    clearSelection()
  }
}

// 批量刷新选中的 Session
const batchRefreshSessionsSelected = async () => {
  if (selectedTokenIds.value.size === 0 || isBatchRefreshingSessions.value) return

  // 构建请求列表：{ id, session }
  const requests = selectedTokens.value
    .filter(token => token.auth_session)
    .map(token => ({
      id: token.id,
      session: token.auth_session
    }))

  if (requests.length === 0) {
    window.$notify.warning(t('tokenList.noTokensWithSession'))
    return
  }

  isBatchRefreshingSessions.value = true

  try {
    window.$notify.info(t('tokenList.batchRefreshingSessions', { count: requests.length }))

    // 调用后端刷新接口
    const results = await invoke('batch_refresh_sessions', { requests })

    let successCount = 0
    let failCount = 0

    // 更新 tokens（使用 token_id 匹配）
    const now = new Date().toISOString()
    results.forEach(result => {
      if (result.success && result.new_session) {
        successCount++
        // 在 tokens 数组中查找对应的 token
        const token = tokens.value.find(t => t.id === result.token_id)
        if (token) {
          token.auth_session = result.new_session
          token.session_updated_at = now  // 设置 session 更新时间
          token.updated_at = now

          // 添加到待同步队列
          markItemUpsert(token)
        }
      } else {
        failCount++
        console.error(`Failed to refresh session for token ${result.token_id}:`, result.error)
      }
    })

    // 保存更新后的 tokens（由前端统一处理双向存储）
    await saveTokens()

    // 显示结果
    if (successCount > 0) {
      window.$notify.success(t('messages.sessionRefreshSuccess', { count: successCount }))
    }
    if (failCount > 0) {
      window.$notify.warning(t('messages.sessionRefreshPartialFail', { success: successCount, fail: failCount }))
    }
  } catch (error) {
    console.error('Failed to refresh sessions:', error)
    window.$notify.error(t('messages.sessionRefreshFailed'))
  } finally {
    isBatchRefreshingSessions.value = false
    // 操作完成后清除选择
    clearSelection()
  }
}

// 批量导出选中的 token
const batchExportSelected = async () => {
  if (selectedTokenIds.value.size === 0) return
  
  const exportData = selectedTokens.value.map(token => ({
    access_token: token.access_token,
    tenant_url: token.tenant_url,
    portal_url: token.portal_url || undefined,
    email_note: token.email_note || undefined,
    auth_session: token.auth_session || undefined,
    tag_name: token.tag_name || undefined,
    tag_color: token.tag_color || undefined
  }))
  
  const jsonStr = JSON.stringify(exportData, null, 2)
  
  try {
    await navigator.clipboard.writeText(jsonStr)
    window.$notify.success(t('tokenList.batchExportSuccess', { count: exportData.length }))
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
    window.$notify.error(t('tokenList.batchExportFailed'))
  }
  
  // 操作完成后清除选择
  clearSelection()
}

// 批量获取绑卡链接
const batchFetchPaymentLinks = async () => {
  if (selectedTokenIds.value.size === 0 || isBatchFetchingPaymentLinks.value) return
  
  isBatchFetchingPaymentLinks.value = true
  const selectedIds = Array.from(selectedTokenIds.value)
  
  try {
    // 获取选中的 tokens（需要有 auth_session）
    const tokensToFetch = tokens.value.filter(t => 
      selectedIds.includes(t.id) && t.auth_session
    )
    
    if (tokensToFetch.length === 0) {
      window.$notify.warning(t('tokenList.noTokensWithSession'))
      return
    }
    
    window.$notify.info(t('tokenList.batchFetchingPaymentLinks', { count: tokensToFetch.length }))
    
    let successCount = 0
    let failedCount = 0
    const links = []
    
    // 并行获取所有绑卡链接
    const fetchPromises = tokensToFetch.map(async (token) => {
      try {
        const result = await invoke('fetch_payment_method_link_command', {
          authSession: token.auth_session
        })
        if (result.payment_method_link) {
          // 缓存绑卡链接
          paymentLinksCache.value.set(token.id, result.payment_method_link)
          links.push(result.payment_method_link)
          successCount++
        } else {
          failedCount++
        }
      } catch (error) {
        console.error(`Failed to fetch payment link for token ${token.id}:`, error)
        failedCount++
      }
    })
    
    await Promise.all(fetchPromises)
    
    // 复制所有链接到剪贴板
    if (links.length > 0) {
      const linksText = links.join('\n')
      await invoke('copy_to_clipboard', { text: linksText })
      window.$notify.success(t('tokenList.batchPaymentLinksCopied', { 
        success: successCount, 
        failed: failedCount 
      }))
    } else {
      window.$notify.error(t('tokenList.batchPaymentLinksFailed'))
    }
  } catch (error) {
    console.error('Batch fetch payment links error:', error)
    window.$notify.error(`${t('tokenList.batchPaymentLinksFailed')}: ${error}`)
  } finally {
    isBatchFetchingPaymentLinks.value = false
    // 操作完成后清除选择
    clearSelection()
  }
}

// 缓存单个绑卡链接（供 TokenCard 调用）
const cachePaymentLink = (tokenId, link) => {
  paymentLinksCache.value.set(tokenId, link)
}

// 获取缓存的绑卡链接
const getCachedPaymentLink = (tokenId) => {
  return paymentLinksCache.value.get(tokenId) || null
}

// 显示批量删除选中项对话框
const showBatchDeleteSelectedConfirm = () => {
  if (selectedTokenIds.value.size === 0) return
  showSelectedDeleteDialog.value = true
}

// 执行批量删除选中项
const executeBatchDeleteSelected = async () => {
  if (selectedTokenIds.value.size === 0) return

  const selectedIds = Array.from(selectedTokenIds.value)
  let deletedCount = 0

  for (const tokenId of selectedIds) {
    const index = internalTokens.value.findIndex(t => t.id === tokenId)
    if (index !== -1) {
      const tokenToDelete = internalTokens.value[index]
      internalTokens.value.splice(index, 1)
      deletedCount++

      // 标记删除
      markItemDeletion(tokenToDelete)
    }
  }

  // 关闭对话框
  showSelectedDeleteDialog.value = false

  // 清除选择
  clearSelection()

  // 保存更改
  await saveTokens()

  window.$notify.success(t('tokenList.batchDeleteSelectedSuccess', { count: deletedCount }))
}

// 批量编辑标签 - 保存
const handleBatchTagSave = async ({ tagName, tagColor }) => {
  if (selectedTokenIds.value.size === 0) return

  const selectedIds = Array.from(selectedTokenIds.value)
  let updatedCount = 0

  for (const tokenId of selectedIds) {
    const token = internalTokens.value.find(t => t.id === tokenId)
    if (token) {
      token.tag_name = tagName
      token.tag_color = tagColor
      token.updated_at = new Date().toISOString()
      updatedCount++

      // 添加到待更新队列
      markItemUpsert(token)
    }
  }

  // 保存更改
  await saveTokens()

  // 清除选择
  clearSelection()

  window.$notify.success(t('tokenList.batchTagUpdated', { count: updatedCount }))
}

// 批量编辑标签 - 清除
const handleBatchTagClear = async () => {
  if (selectedTokenIds.value.size === 0) return

  const selectedIds = Array.from(selectedTokenIds.value)
  let clearedCount = 0

  for (const tokenId of selectedIds) {
    const token = tokens.value.find(t => t.id === tokenId)
    if (token) {
      token.tag_name = ''
      token.tag_color = ''
      token.updated_at = new Date().toISOString()
      clearedCount++

      // 添加到待更新队列
      markItemUpsert(token)
    }
  }

  // 保存更改
  await saveTokens()

  // 清除选择
  clearSelection()

  window.$notify.success(t('tokenList.batchTagCleared', { count: clearedCount }))
}

// ========== 批量选择相关函数结束 ==========

// 显示批量删除确认对话框
const showBatchDeleteConfirm = () => {
  if (deletableTokensCount.value > 0) {
    showBatchDeleteDialog.value = true
  }
}

// 显示批量导入对话框
const showBatchImportConfirm = () => {
  // 使用配置的默认数量重置 Session 输入框
  initializeSessionInputs(defaultInputCount.value)

  // 重置 Token JSON 输入
  importJsonText.value = '[\n  \n]'
  importPreview.value = []
  importErrors.value = []

  // 默认显示 Session Tab
  batchImportTab.value = 'session'

  showBatchImportDialog.value = true
}

// Session 动态输入框方法
const addSessionInput = () => {
  sessionInputs.value.push({
    id: nextSessionInputId++,
    value: ''
  })
}

const removeSessionInput = (id) => {
  if (sessionInputs.value.length <= 1) {
    window.$notify.warning(t('tokenList.atLeastOneInput'))
    return
  }
  sessionInputs.value = sessionInputs.value.filter(input => input.id !== id)
}

// 获取有效的 Session 输入数量
const validSessionCount = computed(() => {
  return sessionInputs.value.filter(input => input.value.trim()).length
})

// 从 Session 动态输入框执行批量导入
const executeBatchImportFromSessionInputs = async () => {
  // 获取所有非空的 session
  const validSessions = sessionInputs.value
    .map(input => input.value.trim())
    .filter(value => value.length > 0)

  if (validSessions.length === 0) {
    window.$notify.warning(t('tokenList.noValidSessions'))
    return
  }

  isImporting.value = true

  try {
    let successCount = 0
    let skippedCount = 0
    let sessionExtractionErrors = []
    let duplicateIds = []

    // 遍历所有 session,调用后端提取
    for (let i = 0; i < validSessions.length; i++) {
      const session = validSessions[i]

      try {
        // 调用后端 API 从 session 提取 token
        const result = await invoke('add_token_from_session', {
          session: session
        })

        // 提取成功,添加 token
        const tokenData = {
          tenantUrl: result.tenant_url,
          accessToken: result.access_token,
          portalUrl: null,  // Session 导入不再获取 portal_url
          emailNote: result.email || null,  // 从 get-models API 获取的邮箱
          authSession: session,
          suspensions: null,  // Session 导入不再获取 suspensions
          creditsBalance: null,  // Session 导入不再获取余额
          expiryDate: null,  // Session 导入不再获取过期时间
          banStatus: 'ACTIVE'  // Session 导入默认设置为 ACTIVE 状态
        }

        const addResult = addToken(tokenData)
        if (addResult.success) {
          successCount++
        } else if (addResult.duplicateId) {
          duplicateIds.push(addResult.duplicateId)
          skippedCount++
        } else {
          skippedCount++
        }
      } catch (error) {
        console.error('Failed to extract token from session:', error)
        sessionExtractionErrors.push({
          index: i + 1,
          error: error.toString()
        })
        skippedCount++
      }
    }

    // 关闭对话框
    showBatchImportDialog.value = false

    // 显示结果
    if (sessionExtractionErrors.length > 0) {
      const errorDetails = sessionExtractionErrors
        .map(e => `[${e.index}] ${e.error}`)
        .join('\n')

      window.$notify.warning(
        t('messages.batchImportSuccessWithSkipped', {
          success: successCount,
          skipped: skippedCount
        }) + `\n\n${t('tokenList.sessionExtractionFailed')}:\n${errorDetails}`
      )
    } else if (duplicateIds.length > 0) {
      if (duplicateIds.length === 1 && successCount === 0) {
        highlightAndScrollTo(duplicateIds[0])
      } else {
        window.$notify.success(
          t('messages.batchImportSuccessWithSkipped', {
            success: successCount,
            skipped: skippedCount
          })
        )
      }
    } else if (skippedCount > 0) {
      window.$notify.success(
        t('messages.batchImportSuccessWithSkipped', {
          success: successCount,
          skipped: skippedCount
        })
      )
    } else {
      window.$notify.success(t('messages.batchImportSuccess', { count: successCount }))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.batchImportFailed')}: ${error}`)
  } finally {
    isImporting.value = false
  }
}

// 验证并解析导入的 JSON
const validateImportJson = () => {
  importErrors.value = []
  importPreview.value = []

  if (!importJsonText.value.trim()) {
    importErrors.value.push(t('tokenList.importJsonEmpty'))
    return false
  }

  try {
    const parsed = JSON.parse(importJsonText.value)

    if (!Array.isArray(parsed)) {
      importErrors.value.push(t('tokenList.importJsonNotArray'))
      return false
    }

    if (parsed.length === 0) {
      importErrors.value.push(t('tokenList.importJsonEmptyArray'))
      return false
    }

    // 检测数组类型: 字符串数组(Session) 或 对象数组(Token)
    const firstItem = parsed[0]
    const isSessionArray = typeof firstItem === 'string'
    const isTokenArray = typeof firstItem === 'object' && firstItem !== null

    if (!isSessionArray && !isTokenArray) {
      importErrors.value.push('数组元素必须是字符串(Session)或对象(Token)')
      return false
    }

    // 验证 Session 数组
    if (isSessionArray) {
      const validSessions = []
      parsed.forEach((item, index) => {
        if (typeof item !== 'string') {
          importErrors.value.push(`[${index + 1}] 必须是字符串`)
        } else if (!item.trim()) {
          importErrors.value.push(`[${index + 1}] Session 不能为空`)
        } else if (item.length < 10) {
          importErrors.value.push(`[${index + 1}] ${t('tokenList.invalidSession')}`)
        } else {
          validSessions.push({ auth_session: item })
        }
      })
      importPreview.value = validSessions
      return validSessions.length > 0
    }

    // 验证 Token 对象数组
    if (isTokenArray) {
      const validTokens = []
      parsed.forEach((item, index) => {
        const errors = []

        if (typeof item !== 'object' || item === null) {
          importErrors.value.push(`[${index + 1}] 必须是对象`)
          return
        }

        // 验证必需字段
        if (!item.access_token || typeof item.access_token !== 'string' || !item.access_token.trim()) {
          errors.push(`[${index + 1}] ${t('tokenList.missingAccessToken')}`)
        }

        if (!item.tenant_url || typeof item.tenant_url !== 'string' || !item.tenant_url.trim()) {
          errors.push(`[${index + 1}] ${t('tokenList.missingTenantUrl')}`)
        }

        // 验证 URL 格式
        if (item.tenant_url) {
          try {
            new URL(item.tenant_url)
          } catch {
            errors.push(`[${index + 1}] ${t('tokenList.invalidTenantUrl')}`)
          }
        }

        if (item.portal_url) {
          try {
            new URL(item.portal_url)
          } catch {
            errors.push(`[${index + 1}] ${t('tokenList.invalidPortalUrl')}`)
          }
        }

        if (errors.length > 0) {
          importErrors.value.push(...errors)
        } else {
          validTokens.push(item)
        }
      })
      importPreview.value = validTokens
      return validTokens.length > 0
    }

    return false
  } catch (error) {
    importErrors.value.push(`${t('tokenList.importJsonParseError')}: ${error.message}`)
    return false
  }
}

// 处理从 BatchImportModal 组件传来的导入数据
const handleBatchImportData = async ({ type, data }) => {
  if (type === 'session') {
    // 将 sessions 数组转换为 sessionInputs 格式并调用原有逻辑
    sessionInputs.value = data.map((value, index) => ({ id: index + 1, value }))
    await executeBatchImportFromSessionInputs()
  } else {
    // Token 导入
    importPreview.value = data
    await executeBatchImportFromTokens()
  }
}

// 从 Token JSON 执行批量导入
const executeBatchImportFromTokens = async () => {
  isImporting.value = true

  try {
    let successCount = 0
    let skippedCount = 0
    let duplicateIds = []

    // Token 模式: 直接添加所有 token
    importPreview.value.forEach(item => {
      const tokenData = {
        tenantUrl: item.tenant_url,
        accessToken: item.access_token,
        portalUrl: item.portal_url || null,
        emailNote: extractEmail(item),
        tagName: item.tag_name || null,
        tagColor: item.tag_color || null,
        authSession: null,
        suspensions: item.suspensions || null
      }

      const result = addToken(tokenData)
      if (result.success) {
        successCount++
      } else if (result.duplicateId) {
        duplicateIds.push(result.duplicateId)
        skippedCount++
      } else {
        skippedCount++
      }
    })

    // 关闭对话框
    showBatchImportDialog.value = false

    // 显示结果
    if (duplicateIds.length > 0) {
      if (duplicateIds.length === 1 && successCount === 0) {
        highlightAndScrollTo(duplicateIds[0])
      } else {
        window.$notify.success(
          t('messages.batchImportSuccessWithSkipped', {
            success: successCount,
            skipped: skippedCount
          })
        )
      }
    } else if (skippedCount > 0) {
      window.$notify.success(
        t('messages.batchImportSuccessWithSkipped', {
          success: successCount,
          skipped: skippedCount
        })
      )
    } else {
      window.$notify.success(t('messages.batchImportSuccess', { count: successCount }))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.batchImportFailed')}: ${error}`)
  } finally {
    isImporting.value = false
  }
}

// 执行批量导入
const executeBatchImport = async () => {
  // 如果是 Session Tab,从动态输入框导入
  if (batchImportTab.value === 'session') {
    await executeBatchImportFromSessionInputs()
    return
  }

  // Token Tab: 使用原有的 JSON 导入逻辑
  if (!validateImportJson()) {
    return
  }

  isImporting.value = true

  try {
    let successCount = 0
    let skippedCount = 0
    let sessionExtractionErrors = []
    let duplicateIds = []  // 收集重复的 token ID

    // 检测导入模式: Session 数组 或 Token 对象数组
    const firstItem = importPreview.value[0]
    const isSessionMode = firstItem.auth_session && !firstItem.access_token

    if (isSessionMode) {
      // Session 模式: 遍历所有 session,调用后端提取
      for (let i = 0; i < importPreview.value.length; i++) {
        const item = importPreview.value[i]

        try {
          // 更新进度提示
          const progressMsg = t('tokenList.extractingFromSession', {
            current: i + 1,
            total: importPreview.value.length
          })
          console.log(progressMsg)

          // 调用后端 API 从 session 提取 token
          const result = await invoke('add_token_from_session', {
            session: item.auth_session
          })

          // 提取成功,添加 token
          const tokenData = {
            tenantUrl: result.tenant_url,
            accessToken: result.access_token,
            portalUrl: null,  // Session 导入不再获取 portal_url
            emailNote: result.email || null,  // 从 get-models API 获取的邮箱
            authSession: item.auth_session,
            suspensions: null,  // Session 导入不再获取 suspensions
            creditsBalance: null,  // Session 导入不再获取余额
            expiryDate: null  // Session 导入不再获取过期时间
          }

          const addResult = addToken(tokenData)
          if (addResult.success) {
            successCount++
          } else if (addResult.duplicateId) {
            // 记录重复的 token ID
            duplicateIds.push(addResult.duplicateId)
            skippedCount++
          } else {
            skippedCount++
          }
        } catch (error) {
          console.error('Failed to extract token from session:', error)
          sessionExtractionErrors.push({
            index: i + 1,
            error: error.toString()
          })
          skippedCount++
        }
      }
    } else {
      // Token 模式: 直接添加所有 token
      importPreview.value.forEach(item => {
        const tokenData = {
          tenantUrl: item.tenant_url,
          accessToken: item.access_token,
          portalUrl: item.portal_url || null,
          emailNote: extractEmail(item),  // 智能提取email字段
          tagName: item.tag_name || null,
          tagColor: item.tag_color || null,
          authSession: null,
          suspensions: item.suspensions || null
        }

        const result = addToken(tokenData)
        if (result.success) {
          successCount++
        } else if (result.duplicateId) {
          // 记录重复的 token ID
          duplicateIds.push(result.duplicateId)
          skippedCount++
        } else {
          skippedCount++
        }
      })
    }

    // 关闭对话框
    showBatchImportDialog.value = false

    // 显示结果
    if (sessionExtractionErrors.length > 0) {
      // 有 session 提取失败的情况
      const errorDetails = sessionExtractionErrors
        .map(e => `[${e.index}] ${e.error}`)
        .join('\n')

      window.$notify.warning(
        t('messages.batchImportSuccessWithSkipped', {
          success: successCount,
          skipped: skippedCount
        }) + `\n\n${t('tokenList.sessionExtractionFailed')}:\n${errorDetails}`
      )
    } else if (duplicateIds.length > 0) {
      // 有重复的 token
      if (duplicateIds.length === 1 && successCount === 0) {
        // 只有一个重复且没有成功导入的,高亮并滚动到重复的 token
        highlightAndScrollTo(duplicateIds[0])
      } else {
        // 有多个重复或有部分成功导入的,显示提示
        window.$notify.success(
          t('messages.batchImportSuccessWithSkipped', {
            success: successCount,
            skipped: skippedCount
          })
        )
      }
    } else if (skippedCount > 0) {
      window.$notify.success(
        t('messages.batchImportSuccessWithSkipped', {
          success: successCount,
          skipped: skippedCount
        })
      )
    } else {
      window.$notify.success(t('messages.batchImportSuccess', { count: successCount }))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.batchImportFailed')}: ${error}`)
  } finally {
    isImporting.value = false
  }
}

// 执行批量删除（已封禁/过期的 tokens）
const executeBatchDelete = async () => {
  isDeleting.value = true

  try {
    // 获取要删除的 tokens
    const tokensToDelete = internalTokens.value.filter(token =>
      token.ban_status === 'SUSPENDED' || token.ban_status === 'EXPIRED'
    )

    let deletedCount = 0

    for (const token of tokensToDelete) {
      // 从本地列表移除
      const index = internalTokens.value.findIndex(t => t.id === token.id)
      if (index !== -1) {
        internalTokens.value.splice(index, 1)
        deletedCount++
      }

      // 标记删除
      markItemDeletion(token)
    }

    // 关闭对话框
    showBatchDeleteDialog.value = false

    // 保存更改到本地文件
    await saveTokens()

    // 显示结果消息
    if (deletedCount > 0) {
      window.$notify.success(t('messages.batchDeleteSuccess', { count: deletedCount }))
    }
  } catch (error) {
    console.error('Batch delete failed:', error)
    window.$notify.error(`${t('messages.batchDeleteFailed')}: ${error}`)
  } finally {
    isDeleting.value = false
  }
}


// Additional state for new components
const isSaving = ref(false)
const isRefreshing = ref(false)

// TokenForm state management
const showTokenFormModal = ref(false)
const editingToken = ref(null)

// Token card refs for accessing child methods
const tokenCardRefs = ref({})

// 打开 Session 刷新 Modal
const openSessionRefreshModal = () => {
  showSessionRefreshModal.value = true
}

// 单个刷新 session
const handleSingleRefreshSession = async ({ tokenId, newSession, updatedAt }) => {
  try {
    // 在 tokens 数组中查找对应的 token
    const token = tokens.value.find(t => t.id === tokenId)
    if (token) {
      token.auth_session = newSession
      token.session_updated_at = updatedAt
      token.updated_at = updatedAt

      // 添加到待同步队列
      markItemUpsert(token)

      // 保存更新后的 tokens
      await saveTokens()
    }
  } catch (error) {
    console.error('Failed to save token after session refresh:', error)
    window.$notify?.error(t('messages.saveTokenFailed'))
  }
}

// 批量刷新 sessions
const handleBatchRefreshSessions = async () => {
  if (expiringSessionTokens.value.length === 0) {
    window.$notify.warning(t('messages.noExpiringSession'))
    return
  }

  // 构建请求列表：{ id, session }
  const requests = expiringSessionTokens.value
    .filter(token => token.auth_session)
    .map(token => ({
      id: token.id,
      session: token.auth_session
    }))

  if (requests.length === 0) {
    window.$notify.warning(t('messages.noExpiringSession'))
    return
  }

  isRefreshingSessions.value = true

  try {

    // 调用后端刷新接口
    const results = await invoke('batch_refresh_sessions', { requests })

    let successCount = 0
    let failCount = 0

    // 更新 tokens（使用 token_id 匹配）
    const now = new Date().toISOString()
    results.forEach(result => {
      if (result.success && result.new_session) {
        successCount++
        // 在 tokens 数组中查找对应的 token
        const token = tokens.value.find(t => t.id === result.token_id)
        if (token) {
          token.auth_session = result.new_session
          token.session_updated_at = now  // 设置 session 更新时间
          token.updated_at = now

          // 添加到待同步队列
          markItemUpsert(token)
        }
      } else {
        failCount++
        console.error(`Failed to refresh session for token ${result.token_id}:`, result.error)
      }
    })

    // 保存更新后的 tokens（由前端统一处理双向存储）
    await saveTokens()

    // 显示结果
    if (successCount > 0) {
      window.$notify.success(t('messages.sessionRefreshSuccess', { count: successCount }))
    }
    if (failCount > 0) {
      window.$notify.warning(t('messages.sessionRefreshPartialFail', { success: successCount, fail: failCount }))
    }

    // 关闭 Modal
    showSessionRefreshModal.value = false
  } catch (error) {
    console.error('Failed to refresh sessions:', error)
    window.$notify.error(t('messages.sessionRefreshFailed'))
  } finally {
    isRefreshingSessions.value = false
  }
}

// 标记所有 token 为待同步（使用 composable 的 markAllForSync）
const handleMarkAllForSync = () => {
  if (tokens.value.length === 0) {
    window.$notify.warning(t('messages.noTokensToSync'))
    return
  }

  if (markAllForSync()) {
    window.$notify.success(t('messages.allTokensMarkedForSync', { count: tokens.value.length }))
  }
}

// 初始化就绪等待方法
const waitUntilReady = async () => {
  if (isReady.value && !isLoading.value) return
  await new Promise((resolve) => {
    const stop = watch([isReady, isLoading], ([ready, loading]) => {
      if (ready && !loading) {
        stop()
        resolve()
      }
    })
  })
}

// 设置ref的函数
const setTokenCardRef = (el, tokenId) => {
  if (el) {
    tokenCardRefs.value[tokenId] = el
  } else {
    // 当组件被移除时，清理引用
    delete tokenCardRefs.value[tokenId]
  }
}

// 处理 Token 更新事件
const handleTokenUpdated = async (updatedToken) => {
  // 如果有 updatedToken 参数，记录到待更新集合
  if (updatedToken && updatedToken.id) {
    markItemUpsert(updatedToken)

    // 保存到本地文件
    await saveTokens()
  }
}

// 深度比对两个对象是否相等
const isEqual = (obj1, obj2) => {
  if (obj1 === obj2) return true
  if (obj1 == null || obj2 == null) return false
  if (typeof obj1 !== 'object' || typeof obj2 !== 'object') return obj1 === obj2

  const keys1 = Object.keys(obj1)
  const keys2 = Object.keys(obj2)

  if (keys1.length !== keys2.length) return false

  for (const key of keys1) {
    if (!keys2.includes(key)) return false
    if (!isEqual(obj1[key], obj2[key])) return false
  }

  return true
}

// 检查所有Token的账号状态
const checkAllAccountStatus = async () => {
  if (tokens.value.length === 0) {
    return { success: true, hasChanges: false, message: t('messages.noTokensToCheck') }
  }

  try {
    // 准备批量检测的数据，过滤掉标记为跳过检测的账号
    const tokensToCheck = tokens.value.filter(token => !token.skip_check)

    const tokenInfos = tokensToCheck.map(token => ({
      id: token.id,
      access_token: token.access_token,
      tenant_url: token.tenant_url,
      portal_url: token.portal_url || null,
      auth_session: token.auth_session || null,
      email_note: token.email_note || null,
      should_refresh_session: shouldRefreshSession(token)  // 前端判断是否需要刷新 session
    }))

    // 单次批量API调用
    const results = await invoke('batch_check_tokens_status', {
      tokens: tokenInfos
    })


    // 批量更新tokens状态，返回是否有变化
    const hasChanges = updateTokensFromResults(results)

    return { success: true, hasChanges }

  } catch (error) {
    console.error('Batch check error:', error)
    return {
      success: false,
      hasChanges: false,
      message: `${t('messages.accountStatusCheckError')}: ${error}`
    }
  }
}

// 根据批量检测结果更新tokens状态
const updateTokensFromResults = (results) => {
  let anyChanges = false

  results.forEach(result => {
    const token = tokens.value.find(t => t.id === result.token_id)
    if (token) {
      const statusResult = result.status_result
      let hasChanges = false

      // 比对并更新 access_token
      if (token.access_token !== result.access_token) {
        token.access_token = result.access_token
        hasChanges = true
      }

      // 比对并更新 tenant_url
      if (token.tenant_url !== result.tenant_url) {
        token.tenant_url = result.tenant_url
        hasChanges = true
      }

      // 比对并更新 ban_status
      if (token.ban_status !== statusResult.status) {
        token.ban_status = statusResult.status
        hasChanges = true
      }

      // 自动禁用封禁或过期的账号检测
      if ((statusResult.status === 'SUSPENDED' || statusResult.status === 'EXPIRED') && !token.skip_check) {
        token.skip_check = true
        hasChanges = true
        // 显示通知
        const autoDisableMsg = statusResult.status === 'SUSPENDED'
          ? t('messages.autoDisabledBanned')
          : t('messages.autoDisabledExpired')
        window.$notify.info(autoDisableMsg)
      }

      // 比对并更新 suspensions 信息（如果有）
      if (result.suspensions) {
        if (!isEqual(token.suspensions, result.suspensions)) {
          token.suspensions = result.suspensions
          hasChanges = true
          console.log(`Updated suspensions for token ${token.id}:`, result.suspensions)
        }
      }

      // 比对并更新 Portal 信息（如果有）
      if (result.portal_info) {
        const newPortalInfo = {
          credits_balance: result.portal_info.credits_balance,
          credit_total: result.portal_info.credit_total,
          expiry_date: result.portal_info.expiry_date
        }

        if (!isEqual(token.portal_info, newPortalInfo)) {
          // 使用 Object.assign 确保触发响应式更新
          if (!token.portal_info) {
            token.portal_info = {}
          }
          Object.assign(token.portal_info, newPortalInfo)
          hasChanges = true
        }
      } else if (result.portal_error) {
        console.warn(`Failed to get portal info for token ${token.id}:`, result.portal_error)
      }

      // 比对并更新 email_note（如果有）
      if (result.email_note && token.email_note !== result.email_note) {
        token.email_note = result.email_note
        hasChanges = true
      }

      // 只有在有实际变化时才更新时间戳
      if (hasChanges) {
        token.updated_at = new Date().toISOString()
        anyChanges = true
      }
    }
  })

  return anyChanges
}

const loadTokens = async (showSuccessMessage = false) => {
  // 如果tokens由父组件管理，则不加载
  if (props.tokens !== null) {
    return
  }

  isLoading.value = true
  try {
    const jsonString = await invoke('load_tokens_json')
    const parsedTokens = JSON.parse(jsonString)

    // 确保是数组
    if (Array.isArray(parsedTokens)) {
      // 使用展开运算符创建新数组，确保触发响应式更新
      internalTokens.value = [...parsedTokens]
    } else {
      internalTokens.value = []
    }

    if (showSuccessMessage) {
      window.$notify.success(t('messages.tokenLoadSuccess'))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.tokenLoadFailed')}: ${error}`)
    internalTokens.value = []
  } finally {
    isLoading.value = false
  }
}

const saveTokens = async (showSuccessMessage = false) => {
  try {
    const jsonString = JSON.stringify(tokens.value, null, 2)
    await invoke('save_tokens_json', { jsonString })
    if (showSuccessMessage) {
      window.$notify.success(t('messages.tokenSaved'))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.tokenSaveFailed')}: ${error}`)
    throw error
  }
}

// 删除token - 使用新的增量同步协议，不再直接调用后端删除
const deleteToken = async (tokenId) => {
  const tokenIndex = internalTokens.value.findIndex(token => token.id === tokenId)
  if (tokenIndex === -1) {
    window.$notify.error(t('messages.tokenNotFound'))
    return
  }

  // 获取要删除的 token 信息
  const tokenToDelete = internalTokens.value[tokenIndex]

  // 从内存中删除
  internalTokens.value = internalTokens.value.filter(token => token.id !== tokenId)
  window.$notify.success(t('messages.tokenDeleted'))

  // 标记删除
  markItemDeletion(tokenToDelete)

  // 保存到本地文件
  await saveTokens()
}

// TokenForm event handlers
const handleAddToken = () => {
  editingToken.value = null
  showTokenFormModal.value = true
}

const handleEditToken = (token) => {
  editingToken.value = token
  showTokenFormModal.value = true
}

const closeTokenForm = () => {
  showTokenFormModal.value = false
  editingToken.value = null
}

// 用于标记最后一次添加是否成功
const lastAddTokenSuccess = ref(true)

const handleTokenFormSuccess = () => {
  // 只有在添加成功时才关闭对话框和显示提示
  if (editingToken.value) {
    // 编辑模式总是关闭
    closeTokenForm()
    window.$notify.success(t('messages.tokenUpdated'))
  } else {
    // 添加模式：只有成功时才关闭和提示
    if (lastAddTokenSuccess.value) {
      closeTokenForm()
      window.$notify.success(t('messages.tokenSaved'))
    }
    // 如果失败（重复），不关闭对话框，已经显示了警告并高亮了重复的 token
  }
}

const handleUpdateToken = async (updatedTokenData) => {
  const index = internalTokens.value.findIndex(token => token.id === updatedTokenData.id)
  if (index !== -1) {
    const tagName = updatedTokenData.tagName ? updatedTokenData.tagName.trim() : ''
    const tagColor = updatedTokenData.tagColor || DEFAULT_TAG_COLOR

    // Update the token in the list
    const updatedToken = {
      ...internalTokens.value[index],
      tenant_url: updatedTokenData.tenantUrl,
      access_token: updatedTokenData.accessToken,
      portal_url: updatedTokenData.portalUrl || null,
      email_note: updatedTokenData.emailNote || null,
      tag_name: tagName || null,
      tag_color: tagName ? tagColor : null,
      updated_at: new Date().toISOString()  // 更新 updated_at 时间戳
    }
    internalTokens.value[index] = updatedToken

    // 记录到待更新集合
    markItemUpsert(updatedToken)

    // 保存到本地文件
    await saveTokens()
  }
}

const handleAddTokenFromForm = async (tokenData) => {
  const result = await addToken(tokenData)
  lastAddTokenSuccess.value = result.success

  // 如果是重复邮箱，高亮并滚动到重复的 token
  if (!result.success && result.duplicateId) {
    highlightAndScrollTo(result.duplicateId)
  }
}

// 处理自动导入完成事件
const handleAutoImportCompleted = () => {
  if (lastAddTokenSuccess.value) {
    // 添加成功,显示成功提示
    window.$notify.success(t('messages.sessionAutoImported'))
  }
  // 无论成功失败都关闭对话框
  closeTokenForm()
}

// 处理手动导入完成事件
const handleManualImportCompleted = () => {
  if (lastAddTokenSuccess.value) {
    // 添加成功,显示成功提示
    window.$notify.success(t('messages.sessionImportSuccess'))
  }
  // 无论成功失败都关闭对话框
  closeTokenForm()
}


// 添加token
const addToken = async (tokenData) => {
  // 如果有邮箱，检查是否重复
  if (tokenData.emailNote && tokenData.emailNote.trim()) {
    const emailToCheck = tokenData.emailNote.trim().toLowerCase()
    const duplicate = tokens.value.find(token =>
      token.email_note &&
      token.email_note.trim().toLowerCase() === emailToCheck
    )

    if (duplicate) {
      window.$notify.warning(
        t('messages.duplicateEmailSkipped', { email: tokenData.emailNote })
      )
      return { success: false, duplicateId: duplicate.id }  // 返回重复的 token ID
    }
  }

  const now = new Date().toISOString()
  const tagName = tokenData.tagName ? tokenData.tagName.trim() : ''
  const tagColor = tokenData.tagColor || DEFAULT_TAG_COLOR

  // 构建 portal_info (如果有 creditsBalance 或 expiryDate)
  let portalInfo = null
  if (tokenData.creditsBalance !== undefined && tokenData.creditsBalance !== null) {
    portalInfo = {
      credits_balance: tokenData.creditsBalance,
      expiry_date: tokenData.expiryDate || null
    }
  }

  const newToken = {
    id: crypto.randomUUID(),
    tenant_url: tokenData.tenantUrl,
    access_token: tokenData.accessToken,
    created_at: now,
    updated_at: now,  // 添加 updated_at 字段
    portal_url: tokenData.portalUrl || null,
    ban_status: tokenData.banStatus || null,  // 使用传入的 banStatus，Session 导入时为 'ACTIVE'
    portal_info: portalInfo,  // 使用构建的 portal_info
    email_note: tokenData.emailNote || null,
    tag_name: tagName || null,
    tag_color: tagName ? tagColor : null,
    auth_session: tokenData.authSession || null,  // 添加 auth_session 字段
    suspensions: tokenData.suspensions || null,  // 添加 suspensions 字段
    skip_check: false,  // 默认不跳过检测
    balance_color_mode: null,  // 默认为 null，将使用绿色
    session_updated_at: tokenData.authSession ? now : null,  // 如果有 session，设置初始更新时间
    version: 0  // 本地创建时版本号为0，由数据库分配
  }

  internalTokens.value.push(newToken)

  // 记录到待更新集合
  markItemUpsert(newToken)

  // 保存到本地文件
  await saveTokens()

  return { success: true, token: newToken }
}

// 高亮并滚动到指定 token
const highlightAndScrollTo = (tokenId) => {
  // 清除之前的定时器
  if (highlightTimer) {
    clearTimeout(highlightTimer)
    highlightTimer = null
  }

  // 先清空高亮状态，确保即使是同一个 token 也能重新触发动画
  highlightedTokenId.value = null

  // 查找 token 在 filteredTokens 中的索引
  const tokenIndex = filteredTokens.value.findIndex(token => token.id === tokenId)

  if (tokenIndex === -1) {
    console.warn('Token not found in filtered list:', tokenId)
    return
  }

  // 计算 token 所在的页码
  const targetPage = Math.floor(tokenIndex / pageSize.value) + 1

  // 如果不在当前页,先跳转到目标页
  if (currentPage.value !== targetPage) {
    currentPage.value = targetPage
  }

  // 使用 requestAnimationFrame 确保 DOM 完全更新
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      // 重新设置高亮
      highlightedTokenId.value = tokenId

      nextTick(() => {
        const element = tokenCardRefs.value[tokenId]

        if (element) {
          // 尝试多种方式获取 DOM 元素
          let domElement = null

          // 如果 $el 是文本节点，尝试获取下一个元素节点
          if (element.$el) {
            if (element.$el.nodeType === Node.ELEMENT_NODE) {
              domElement = element.$el
            } else if (element.$el.nextElementSibling) {
              domElement = element.$el.nextElementSibling
            } else if (element.$el.parentElement) {
              // 如果是文本节点，尝试在父元素中查找 .token-card
              domElement = element.$el.parentElement.querySelector('.token-card')
            }
          } else if (element instanceof HTMLElement) {
            domElement = element
          } else if (element.value) {
            domElement = element.value
          }

          if (domElement && typeof domElement.scrollIntoView === 'function') {
            domElement.scrollIntoView({ behavior: 'smooth', block: 'center' })
          }
        }

        // 3秒后取消高亮
        highlightTimer = setTimeout(() => {
          highlightedTokenId.value = null
          highlightTimer = null
        }, 3000)
      })
    })
  })
}

// 打开数据文件夹
const openDataFolder = async () => {
  try {
    await invoke('open_data_folder')
    // 静默执行，不显示状态提示
  } catch (error) {
    window.$notify.error(`${t('bookmarkManager.messages.openFolderFailed')}: ${error}`)
  }
}

// ========== 公共批量检测方法 ==========
// 批量检测指定 tokens 的状态（公共方法）
const batchCheckTokensStatus = async (tokensToCheck) => {
  if (tokensToCheck.length === 0) {
    return { hasChanges: false }
  }

  try {
    // 准备批量检测的数据
    const tokenInfos = tokensToCheck.map(token => ({
      id: token.id,
      access_token: token.access_token,
      tenant_url: token.tenant_url,
      portal_url: token.portal_url || null,
      auth_session: token.auth_session || null,
      email_note: token.email_note || null,
      should_refresh_session: shouldRefreshSession(token)  // 前端判断是否需要刷新 session
    }))

    // 单次批量API调用检测所有tokens
    const results = await invoke('batch_check_tokens_status', {
      tokens: tokenInfos
    })

    // 批量更新tokens状态
    let hasChanges = false

    results.forEach(result => {
      const token = tokens.value.find(t => t.id === result.token_id)
      if (token) {
        const statusResult = result.status_result
        let tokenHasChanges = false

        // 比对并更新 access_token
        if (token.access_token !== result.access_token) {
          token.access_token = result.access_token
          tokenHasChanges = true
        }

        // 比对并更新 tenant_url
        if (token.tenant_url !== result.tenant_url) {
          token.tenant_url = result.tenant_url
          tokenHasChanges = true
        }

        // 比对并更新 ban_status
        if (token.ban_status !== statusResult.status) {
          token.ban_status = statusResult.status
          tokenHasChanges = true
        }

        // 自动禁用封禁或过期的账号检测
        if ((statusResult.status === 'SUSPENDED' || statusResult.status === 'EXPIRED') && !token.skip_check) {
          token.skip_check = true
          tokenHasChanges = true
          // 显示通知
          const autoDisableMsg = statusResult.status === 'SUSPENDED'
            ? t('messages.autoDisabledBanned')
            : t('messages.autoDisabledExpired')
          window.$notify.info(autoDisableMsg)
        }

        // 比对并更新 suspensions 信息（如果有）
        if (result.suspensions) {
          if (!isEqual(token.suspensions, result.suspensions)) {
            token.suspensions = result.suspensions
            tokenHasChanges = true
            console.log(`Updated suspensions for token ${token.id}:`, result.suspensions)
          }
        }

        // 比对并更新 Portal 信息（如果有）
        if (result.portal_info) {
          const newPortalInfo = {
            credits_balance: result.portal_info.credits_balance,
            credit_total: result.portal_info.credit_total,
            expiry_date: result.portal_info.expiry_date
          }

          if (!isEqual(token.portal_info, newPortalInfo)) {
            // 使用 Object.assign 确保触发响应式更新
            if (!token.portal_info) {
              token.portal_info = {}
            }
            Object.assign(token.portal_info, newPortalInfo)
            tokenHasChanges = true
            console.log(`Updated token ${token.id} portal info:`, token.portal_info)
          }
        } else if (result.portal_error) {
          // 如果获取Portal信息失败，记录错误但不影响状态更新
          console.warn(`Failed to fetch portal info for token ${token.id}:`, result.portal_error)
        }

        // 比对并更新 email_note（如果有）
        if (result.email_note && token.email_note !== result.email_note) {
          token.email_note = result.email_note
          tokenHasChanges = true
        }

        // 比对并更新 auth_session（如果后端刷新了 session）
        if (result.auth_session && token.auth_session !== result.auth_session) {
          token.auth_session = result.auth_session
          token.session_updated_at = new Date().toISOString()  // 设置 session 更新时间
          tokenHasChanges = true
        }

        // 只有在有实际变化时才更新时间戳
        if (tokenHasChanges) {
          token.updated_at = new Date().toISOString()
          hasChanges = true
        }
      }
    })

    // 强制触发响应式更新,确保UI刷新
    if (hasChanges) {
      await nextTick()
    }

    return { hasChanges }
  } catch (error) {
    console.error('Batch check tokens error:', error)
    throw error
  }
}

// 检查当前页账号状态
const checkPageAccountStatus = async () => {
  // 获取当前页需要检测的tokens(过滤掉标记为跳过检测的)
  const tokensToCheck = paginatedTokens.value.filter(token => !token.skip_check)
  return await batchCheckTokensStatus(tokensToCheck)
}

// 处理刷新事件 - 只刷新当前页
const handleRefresh = async () => {
  if (isRefreshing.value) return
  isRefreshing.value = true

  try {
    window.$notify.info(t('messages.refreshingTokenStatus'))

    // 加载最新的 tokens
    await loadTokens(false)
    await nextTick()

    // 只检查当前页的账号状态
    if (paginatedTokens.value.length > 0) {
      const result = await checkPageAccountStatus()

      // 只有在有实际变化时才保存和标记未同步
      if (result.hasChanges) {
        // 刷新完成后手动保存更新的状态
        await handleSave()

        // 如果是双向存储模式，标记需要同步
        if (isDatabaseAvailable.value) {
          isSyncNeeded.value = true
        }
      }

      window.$notify.success(t('messages.refreshComplete'))
    } else {
      window.$notify.warning(t('messages.noTokensToCheck'))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.refreshFailed')}: ${error.message || error}`)
  } finally {
    // 延迟重置 isRefreshing，确保 watchDebounced 的 debounce timer 已经被清除
    // watchDebounced 的 debounce 时间是 2000ms，这里等待 2100ms 确保安全
    await new Promise(resolve => setTimeout(resolve, 2100))
    isRefreshing.value = false
  }
}



// 自动保存方法（静默保存，不显示提示）
// 只做本地保存，不触发同步
const handleSave = async () => {
  if (isSaving.value) return

  isSaving.value = true
  try {
    await saveTokens(false)
  } catch (error) {
    // 保存失败时抛出错误，由调用方处理
    throw error
  } finally {
    isSaving.value = false
  }
}

// 使用 composable 提供的 handleSync
const handleSync = composableHandleSync

// 保留旧的双向同步方法作为兼容，内部调用新的 handleSync
const handleBidirectionalSync = handleSync

// 组件挂载时自动加载tokens和存储状态
onMounted(async () => {
  // 加载分页配置
  loadPageSize()

  // 加载视图模式配置
  loadViewMode()

  // 加载默认输入框数量配置
  defaultInputCount.value = loadDefaultInputCount()

  // 初始化输入框
  initializeSessionInputs(defaultInputCount.value)

  // 初始化同步状态（从 composable）
  await initSync()

  // 使用 isLoadingFromSync 标记初始加载，避免触发自动保存
  isLoadingFromSync.value = true
  await loadTokens(false) // 显示成功消息

  // 延迟重置标记，确保 watchDebounced 的 debounce timer 已经被清除
  await new Promise(resolve => setTimeout(resolve, 2100))
  isLoadingFromSync.value = false

  // 初始化时，如果是双向存储模式，默认不标记需要同步
  // 只有在用户修改后才标记
  isSyncNeeded.value = false

  isReady.value = true

  // 监听后端发送的 tokens-updated 事件
  unlistenTokensUpdated = await listen('tokens-updated', async () => {
    console.log('📡 Received tokens-updated event from backend, reloading tokens...')
    await loadTokens(false)
  })
})

// 组件卸载时清理定时器和事件监听器
onUnmounted(() => {

  // 取消事件监听
  if (unlistenTokensUpdated) {
    unlistenTokensUpdated()
    unlistenTokensUpdated = null
  }
})

// 搜索时重置到第一页
watch(searchQuery, () => {
  currentPage.value = 1
})

// 防抖自动保存 - 监听 tokens 变化
watchDebounced(
  tokens,
  async (newTokens, oldTokens) => {
    // 只有在组件就绪后才自动保存
    if (!isReady.value) return

    // 如果正在保存,跳过
    if (isSaving.value) return

    // 如果正在同步导致的加载,跳过（避免循环触发）
    if (isLoadingFromSync.value) return

    // 如果正在批量刷新,跳过（刷新完成后会手动保存）
    if (isRefreshing.value) return

    // 如果tokens为空且之前也为空,跳过
    if (newTokens.length === 0 && (!oldTokens || oldTokens.length === 0)) return

    try {
      await handleSave()
      window.$notify.success(t('messages.autoSaveSuccess'))

      // 如果是双向存储模式，标记需要同步
      if (isDatabaseAvailable.value) {
        isSyncNeeded.value = true
      }
    } catch (error) {
      window.$notify.error(t('messages.autoSaveFailed') + ': ' + error)
    }
  },
  {
    debounce: 2000,  // 2秒防抖
    deep: true       // 深度监听
  }
)

// 暴露方法给父组件
defineExpose({
  addToken,    // 允许App.vue添加token
  deleteToken, // 允许App.vue删除token
  tokens: readonly(tokens), // 只读访问tokens
  saveTokens,   // 允许App.vue保存tokens
  waitUntilReady, // 暴露就绪等待方法
  highlightAndScrollTo // 暴露高亮和滚动方法
})
</script>
