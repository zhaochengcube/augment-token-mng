<template>
  <div class="antigravity-manager">
    <div class="page-container">
      <div class="page-content antigravity-manager-content" @click.stop="handlePageContentClick">
        <!-- Header -->
        <div class="page-header">
          <!-- 左侧：状态徽章 -->
          <div
            :class="['status-badge', storageStatusClass, { clickable: isDatabaseAvailable }]"
            v-tooltip="isDatabaseAvailable ? $t('platform.antigravity.viewSyncQueueTooltip') : ''"
            @click="isDatabaseAvailable && openSyncQueue()"
          >
            <span class="status-dot" :class="storageStatusClass"></span>
            <span class="status-text">{{ storageStatusText }}</span>
          </div>

          <!-- 中间：搜索框 -->
          <div class="header-search-box">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="search-icon">
              <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
            </svg>
            <input type="text" v-model="searchQuery" :placeholder="$t('platform.antigravity.searchPlaceholder')" class="search-input" />
            <button v-if="searchQuery.trim()" @click="searchQuery = ''" class="modal-close clear-search-btn" v-tooltip="$t('common.clear')">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
          </div>

          <!-- 右侧：操作按钮 -->
          <div class="header-actions">
            <button v-if="isDatabaseAvailable" @click="handleSync" class="btn primary small" :disabled="isSyncing" v-tooltip="$t('tokenList.syncTooltip')">
              <svg v-if="!isSyncing" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.41-.5 2.69-1.32 3.7l1.46 1.46C19.29 15.77 20 13.96 20 12c0-4.42-3.58-8-8-8zm-6.68 2.3L3.86 7.76C2.71 9.23 2 11.04 2 13c0 4.42 3.58 8 8 8v3l4-4-4-4v3c-3.31 0-6-2.69-6-6 0-1.41.5-2.69 1.32-3.7z" />
              </svg>
              {{ isSyncing ? $t('tokenList.syncing') : $t('tokenList.sync') }}
            </button>
            <button @click="showAddDialog = true" class="btn primary small">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
              {{ $t('platform.antigravity.addAccount') }}
            </button>
          </div>
        </div>

        <div class="page-body">
          <!-- Loading State -->
          <div v-if="isLoading" class="loading-state">
            <div class="spinner"></div>
            <p>{{ $t('common.loading') }}</p>
          </div>

          <!-- Empty State -->
          <div v-else-if="accounts.length === 0" class="empty-state">
            <div class="empty-icon">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z" />
              </svg>
            </div>
            <h3>{{ $t('platform.antigravity.noAccounts') }}</h3>
            <p>{{ $t('platform.antigravity.noAccountsHint') }}</p>
          </div>

          <!-- Account List -->
          <div v-else class="account-list">
            <!-- 工具栏：筛选 + 操作按钮 -->
            <div class="list-toolbar">
              <!-- 左侧：筛选和视图控制按钮 -->
              <div class="toolbar-left">
                <!-- 状态筛选下拉菜单 -->
                <div class="status-filter-dropdown">
                  <button class="filter-btn status-filter-btn" @click.stop="toggleStatusFilterMenu" :class="{ 'active': selectedStatusFilter !== null }">
                    <svg v-if="selectedStatusFilter === 'available'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon active">
                      <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                    </svg>
                    <svg v-else-if="selectedStatusFilter === 'low'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon suspended">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 11c-.55 0-1-.45-1-1V8c0-.55.45-1 1-1s1 .45 1 1v4c0 .55-.45 1-1 1zm1 4h-2v-2h2v2z" />
                    </svg>
                    <svg v-else-if="selectedStatusFilter === 'forbidden'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon other">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z" />
                    </svg>
                    <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon all">
                      <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
                    </svg>
                    <span class="filter-label">{{ selectedStatusFilter ? $t(`platform.antigravity.filter.${selectedStatusFilter}`) : $t('platform.antigravity.filter.all') }}</span>
                    <span class="filter-count">{{ selectedStatusFilter ? statusStatistics[selectedStatusFilter] : statusStatistics.total }}</span>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="dropdown-arrow">
                      <path d="M7 10l5 5 5-5z" />
                    </svg>
                  </button>

                  <!-- 下拉菜单 -->
                  <Transition name="dropdown">
                    <div v-if="showStatusFilterMenu" class="status-filter-menu" @click.stop>
                      <button :class="['status-option', { active: selectedStatusFilter === null }]" @click="selectStatusFilter(null)">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon all">
                          <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
                        </svg>
                        <span>{{ $t('platform.antigravity.filter.all') }}</span>
                        <span class="status-count">{{ statusStatistics.total }}</span>
                        <svg v-if="selectedStatusFilter === null" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'active-status', { active: selectedStatusFilter === 'available' }]" @click="selectStatusFilter('available')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon active">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                        <span>{{ $t('platform.antigravity.filter.available') }}</span>
                        <span class="status-count">{{ statusStatistics.available }}</span>
                        <svg v-if="selectedStatusFilter === 'available'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'suspended-status', { active: selectedStatusFilter === 'low' }]" @click="selectStatusFilter('low')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon suspended">
                          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 11c-.55 0-1-.45-1-1V8c0-.55.45-1 1-1s1 .45 1 1v4c0 .55-.45 1-1 1zm1 4h-2v-2h2v2z" />
                        </svg>
                        <span>{{ $t('platform.antigravity.filter.low') }}</span>
                        <span class="status-count">{{ statusStatistics.low }}</span>
                        <svg v-if="selectedStatusFilter === 'low'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'other-status', { active: selectedStatusFilter === 'forbidden' }]" @click="selectStatusFilter('forbidden')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon other">
                          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31C15.55 19.37 13.85 20 12 20zm6.31-3.1L7.1 5.69C8.45 4.63 10.15 4 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9z" />
                        </svg>
                        <span>{{ $t('platform.antigravity.filter.forbidden') }}</span>
                        <span class="status-count">{{ statusStatistics.forbidden }}</span>
                        <svg v-if="selectedStatusFilter === 'forbidden'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>
                    </div>
                  </Transition>
                </div>

                <!-- 排序下拉菜单 -->
                <div class="sort-dropdown">
                  <button class="btn-icon" @click.stop="toggleSortMenu" v-tooltip="$t('common.sort')">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                      <path d="M7 16V6M4 9l3-3 3 3" />
                      <path d="M17 8v10M14 15l3 3 3-3" />
                    </svg>
                  </button>

                  <Transition name="dropdown">
                    <div v-if="showSortMenu" class="sort-menu" @click.stop>
                      <button :class="['sort-option', { active: sortType === 'time' && sortOrder === 'desc' }]" @click="setSortType('time', 'desc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM7 10h5v5H7z" />
                        </svg>
                        <span>{{ $t('common.sortByTime') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-down">
                          <path d="M16 10l-4 4-4-4h8z" />
                        </svg>
                        <svg v-if="sortType === 'time' && sortOrder === 'desc'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['sort-option', { active: sortType === 'time' && sortOrder === 'asc' }]" @click="setSortType('time', 'asc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM7 10h5v5H7z" />
                        </svg>
                        <span>{{ $t('common.sortByTime') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-up">
                          <path d="M8 14l4-4 4 4H8z" />
                        </svg>
                        <svg v-if="sortType === 'time' && sortOrder === 'asc'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <div class="sort-divider"></div>

                      <button :class="['sort-option', { active: sortType === 'quota' && sortOrder === 'desc' }]" @click="setSortType('quota', 'desc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z" />
                        </svg>
                        <span>{{ $t('platform.antigravity.sortByQuota') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-down">
                          <path d="M16 10l-4 4-4-4h8z" />
                        </svg>
                        <svg v-if="sortType === 'quota' && sortOrder === 'desc'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['sort-option', { active: sortType === 'quota' && sortOrder === 'asc' }]" @click="setSortType('quota', 'asc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z" />
                        </svg>
                        <span>{{ $t('platform.antigravity.sortByQuota') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-up">
                          <path d="M8 14l4-4 4 4H8z" />
                        </svg>
                        <svg v-if="sortType === 'quota' && sortOrder === 'asc'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>
                    </div>
                  </Transition>
                </div>

                <!-- 布局切换按钮 -->
                <button class="btn-icon view-toggle-btn" @click="toggleViewMode" v-tooltip="viewMode === 'card' ? $t('common.switchToTable') : $t('common.switchToCard')" :class="{ 'active': viewMode === 'table' }">
                  <svg v-if="viewMode === 'table'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M4 11h5V5H4v6zm0 7h5v-6H4v6zm6 0h5v-6h-5v6zm6 0h5v-6h-5v6zm-6-7h5V5h-5v6zm6-6v6h5V5h-5z" />
                  </svg>
                  <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M3 14h4v-4H3v4zm0 5h4v-4H3v4zM3 9h4V5H3v4zm5 5h13v-4H8v4zm0 5h13v-4H8v4zM8 5v4h13V5H8z" />
                  </svg>
                </button>
              </div>

              <!-- 右侧：其他操作按钮 -->
              <div class="toolbar-right">
                <!-- 邮箱显示切换按钮 -->
                <button
                  class="btn-icon email-visibility-btn"
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
                </button>

                <!-- 刷新按钮 -->
                <button class="btn-icon refresh-btn" @click="handleRefresh" :disabled="isRefreshing" v-tooltip="$t('platform.antigravity.refreshQuota')">
                  <svg v-if="!isRefreshing" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                  </svg>
                  <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="spinning">
                    <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                  </svg>
                </button>

                <!-- 打开文件夹按钮 -->
                <button class="btn-icon open-folder-btn" @click="openDataFolder" v-tooltip="$t('common.openDataFolder')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z" />
                  </svg>
                </button>

                <!-- 批量删除按钮 -->
                <button class="btn-icon batch-delete-btn" @click="handleBatchDelete" v-tooltip="$t('common.batchDelete')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
                  </svg>
                </button>
              </div>
            </div>

            <!-- 无搜索结果提示 -->
            <div v-if="searchQuery.trim() && filteredAccounts.length === 0" class="no-search-results">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
              </svg>
              <p>{{ $t('common.noSearchResults') }}</p>
            </div>

            <template v-else>
              <!-- 可滚动内容区域 -->
              <div class="account-content-scroll">
                <!-- 卡片布局 -->
                <div v-if="viewMode === 'card'" class="account-grid">
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
                <div v-else class="account-table-wrapper">
                  <table class="account-table">
                    <thead>
                      <tr>
                        <th class="th-checkbox">
                          <div class="header-checkbox" @click="toggleSelectAll">
                            <div class="checkbox-inner" :class="{ 'checked': isAllSelected, 'indeterminate': isPartialSelected }">
                              <svg v-if="isAllSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                              </svg>
                              <svg v-else-if="isPartialSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M19 13H5v-2h14v2z" />
                              </svg>
                            </div>
                          </div>
                        </th>
                        <th class="th-info">{{ $t('platform.antigravity.table.info') }}</th>
                        <th class="th-quota">{{ $t('platform.antigravity.table.quota') }}</th>
                        <th class="th-actions">{{ $t('platform.antigravity.table.actions') }}</th>
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
              </div>

              <!-- 分页组件 -->
              <Pagination
                v-if="filteredAccounts.length > 0"
                :current-page="currentPage"
                :total-pages="totalPages"
                :total-items="filteredAccounts.length"
                :page-size="pageSize"
                :page-size-options="pageSizeOptions"
                @update:current-page="handlePageChange"
                @update:page-size="changePageSize"
              />
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 批量操作工具栏 -->
    <Teleport to="body">
      <Transition name="slide-up">
        <div v-if="isSelectionMode" class="batch-toolbar">
          <div class="batch-toolbar-content">
            <div class="batch-info">
              <span class="selected-count">{{ $t('common.selected', { count: selectedAccountIds.size }) }}</span>
              <button @click="selectAllOnPage" class="btn-text">{{ $t('common.selectAllPage') }}</button>
            </div>

            <div class="batch-actions">
              <button @click="batchRefreshSelected" class="btn-icon" :disabled="isBatchRefreshing" v-tooltip="$t('platform.antigravity.batchRefresh')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                </svg>
              </button>

              <button @click="showBatchDeleteSelectedConfirm" class="btn-icon danger" v-tooltip="$t('common.batchDelete')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
                </svg>
              </button>

              <button @click="clearSelection" class="btn-icon close" v-tooltip="$t('common.cancelSelection')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

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
      :total-accounts-count="accounts.length"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import AccountCard from '../antigravity/AccountCard.vue'
import AccountTableRow from '../antigravity/AccountTableRow.vue'
import AddAccountDialog from '../antigravity/AddAccountDialog.vue'
import ModelsModal from '../antigravity/ModelsModal.vue'
import SyncQueueModal from '../antigravity/SyncQueueModal.vue'
import Pagination from '../common/Pagination.vue'

const { t: $t } = useI18n()

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
const showStatusFilterMenu = ref(false)

// 排序
const sortType = ref('time')
const sortOrder = ref('desc')
const showSortMenu = ref(false)

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
    return 'initializing'
  }

  if (isDatabaseAvailable.value && hasPendingChanges.value) {
    return 'unsaved'
  }

  return 'saved'
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

// 筛选和排序
const toggleStatusFilterMenu = () => {
  showStatusFilterMenu.value = !showStatusFilterMenu.value
  if (showStatusFilterMenu.value) {
    showSortMenu.value = false
  }
}

const selectStatusFilter = (filter) => {
  selectedStatusFilter.value = filter
  showStatusFilterMenu.value = false
  currentPage.value = 1
}

const toggleSortMenu = () => {
  showSortMenu.value = !showSortMenu.value
  if (showSortMenu.value) {
    showStatusFilterMenu.value = false
  }
}

const setSortType = (type, order) => {
  sortType.value = type
  sortOrder.value = order
  showSortMenu.value = false
}

const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'card' ? 'table' : 'card'
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

// 点击外部关闭菜单
const handleClickOutside = (event) => {
  if (!event.target.closest('.status-filter-dropdown')) {
    showStatusFilterMenu.value = false
  }
  if (!event.target.closest('.sort-dropdown')) {
    showSortMenu.value = false
  }
}

onMounted(async () => {
  lastVersion.value = loadLastVersion()
  loadPendingChanges()
  await getStorageStatus()
  await loadAccounts()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  if (storageCheckTimer) {
    clearInterval(storageCheckTimer)
    storageCheckTimer = null
  }
})
</script>

<style scoped>
/* 主容器 */
.antigravity-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.antigravity-manager .page-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.antigravity-manager-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
  max-width: 100%;
  max-height: 100%;
  border-radius: 0;
  background: transparent;
}

/* Header 样式 - 科技风 */
.antigravity-manager-content .page-header {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface);
  flex-shrink: 0;
}

.antigravity-manager-content .page-header > .status-badge {
  flex-shrink: 0;
}

.antigravity-manager-content .page-header > .header-actions {
  flex-shrink: 0;
  align-self: center;
}

.antigravity-manager-content .page-header .btn {
  margin: 0;
}

/* Header 搜索框 - 科技风 */
.header-search-box {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 0 16px;
  height: 36px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  transition: all 0.25s ease;
}

.header-search-box:focus-within {
  border-color: color-mix(in srgb, var(--accent) 60%, transparent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 15%, transparent),
              0 0 20px var(--tech-glow-primary);
}

.header-search-box .search-icon {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: all 0.2s ease;
}

.header-search-box:focus-within .search-icon {
  color: var(--accent);
}

.header-search-box .search-input {
  flex: 1;
  border: none;
  outline: none !important;
  background: transparent;
  color: var(--text);
  font-size: 14px;
  box-shadow: none !important;
  padding: 0;
  margin: 0;
  height: 100%;
  line-height: normal;
}

.header-search-box .search-input:focus {
  outline: none !important;
  box-shadow: none !important;
}

.header-search-box .search-input::placeholder {
  color: var(--text-muted);
  opacity: 0.6;
}

.header-search-box .clear-search-btn {
  min-width: 22px;
  min-height: 22px;
  padding: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  height: 32px;
}

.status-text {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.3px;
}

/* 按钮样式 - 科技风 */


/* Body 样式 */
.antigravity-manager-content .page-body {
  flex: 1;
  overflow: hidden;
  padding: 4px 12px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

/* Loading 和 Empty 状态 - 科技风 */
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 20px;
  color: var(--text-muted);
}

.spinner {
  width: 48px;
  height: 48px;
  border: 3px solid color-mix(in srgb, var(--accent) 20%, transparent);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.empty-icon {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.empty-icon svg {
  opacity: 0.4;
  color: var(--accent);
}

.empty-state h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text);
}

.empty-state p {
  margin: 0;
  font-size: 14px;
  color: var(--text-muted);
  max-width: 400px;
  text-align: center;
  line-height: 1.6;
}

/* 工具栏 - 科技风 */
.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 14px 18px;
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  box-shadow: var(--tech-border-glow);
  position: relative;
  z-index: 100;
  margin-bottom: 16px;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

/* 操作按钮 - 科技风 */
.btn-icon {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  padding: 0;
}

/* 筛选按钮样式 */
.status-filter-dropdown,
.sort-dropdown {
  position: relative;
}

/* 通用筛选按钮样式 - 科技风 */
.filter-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 36px;
  padding: 0 14px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  font-weight: 500;
}

.filter-btn:hover {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.filter-btn.active {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  color: var(--accent);
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.filter-btn .dropdown-arrow {
  margin-left: 2px;
  transition: transform 0.2s;
  opacity: 0.7;
}

/* 排序按钮样式 */
.sort-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 10px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--text);
  font-weight: 500;
}

.sort-btn:hover {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

.filter-label {
  font-weight: 500;
}

.filter-count {
  padding: 2px 6px;
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
}

.dropdown-arrow {
  color: var(--text-muted);
  transition: transform 0.2s;
}

/* 状态筛选下拉菜单 */
.status-filter-dropdown {
  position: relative;
}

.status-filter-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 280px;
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--tech-card-bg));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  z-index: 1100;
  overflow: hidden;
  padding: 6px;
}

/* 排序下拉菜单 */
.sort-dropdown {
  position: relative;
}

.sort-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 220px;
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--tech-card-bg));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  padding: 6px;
  z-index: 1100;
}

.status-option,
.sort-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: transparent;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  text-align: left;
  color: var(--text);
  font-size: 13px;
}

.status-option:hover,
.sort-option:hover {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent);
}

.status-option.active,
.sort-option.active {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
  font-weight: 500;
}

.status-icon {
  flex-shrink: 0;
}

.status-icon.active {
  color: var(--state-success);
}

.status-icon.suspended {
  color: var(--state-warning);
}

.status-icon.other {
  color: var(--state-danger);
}

.status-option span:not(.status-count):not(.check-icon) {
  flex: 1;
}

.status-option .status-count {
  margin-left: auto;
  padding: 2px 8px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.status-option.active .status-count {
  background: color-mix(in srgb, var(--accent) 25%, transparent);
  color: var(--accent);
}

.status-option .check-icon {
  flex-shrink: 0;
  margin-left: 8px;
  color: var(--accent);
}


.check-icon {
  margin-left: auto;
  color: var(--accent);
}

.sort-divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}

.arrow-down,
.arrow-up {
  margin-left: auto;
  color: var(--text-secondary);
}

/* 无搜索结果 */
.no-search-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  gap: 12px;
  color: var(--text-secondary);
}

.no-search-results p {
  margin: 0;
  font-size: 14px;
}

/* 账号网格 */
.account-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
  padding: 4px;
}

/* 账号表格 */
.account-table-wrapper {
  overflow-x: auto;
  border-radius: 8px;
  border: 1px solid var(--border);
}

.account-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--bg-muted);
}

.account-table thead {
  background: var(--bg-hover);
  border-bottom: 2px solid var(--border);
}

.account-table th {
  padding: 12px 16px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.th-checkbox {
  width: 40px;
}

.th-info {
  width: 220px;
}

.th-quota {
  width: auto;
}

.th-actions {
  width: 88px;
}

.header-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.checkbox-inner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.checkbox-inner.checked,
.checkbox-inner.indeterminate {
  background: var(--accent);
  border-color: var(--accent);
}

.checkbox-inner svg {
  color: white;
}

/* ========== 批量操作工具栏样式 ========== */
.batch-toolbar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;
  background: var(--bg-surface, #ffffff);
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 12px;
  box-shadow: var(--shadow-elevated);
  padding: 12px 24px;
  min-width: 520px;
}

.batch-toolbar-content {
  display: flex;
  align-items: center;
  gap: 24px;
}

.batch-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selected-count {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent, #3b82f6);
}

.btn-text {
  background: transparent;
  border: none;
  color: var(--accent, #3b82f6);
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s;
  font-weight: 500;
}

.btn-text:hover {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.batch-actions {
  display: flex;
  gap: 8px;
}

/* 动画 */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.spinning {
  animation: spin 1s linear infinite;
}

/* Dropdown 过渡 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* Slide up 过渡 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px);
}

/* Account list wrapper with fixed pagination */
.antigravity-manager .page-body > .account-list {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* 可滚动内容区域 */
.antigravity-manager .account-content-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  min-height: 0;
  padding-bottom: 8px;
}

/* 确保表格包装器在滚动容器内正常工作 */
.antigravity-manager .account-content-scroll > .account-table-wrapper {
  overflow-x: auto;
  max-height: none;
}

/* 分页组件固定样式 */
.antigravity-manager .account-list :deep(.pagination-container) {
  flex-shrink: 0;
}
</style>
