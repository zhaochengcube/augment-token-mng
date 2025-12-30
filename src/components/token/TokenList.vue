<template>
  <div class="token-list-page">
    <div class="page-container">
      <div class="page-content" @click.stop="handlePageContentClick">
        <div class="page-header">
          <!-- 左侧：存储状态 -->
          <div
            :class="['status-badge', storageStatusClass, { clickable: isDatabaseAvailable }]"
            v-tooltip="isDatabaseAvailable ? $t('tokenList.viewSyncQueueTooltip') : ''"
            @click="isDatabaseAvailable && openSyncQueue()"
          >
            <span :class="['status-dot', storageStatusClass]"></span>
            <span class="status-text">{{ storageStatusText }}</span>
          </div>

          <!-- 中间：搜索框 -->
          <div class="header-search-box">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="search-icon">
              <path
                d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
            </svg>
            <input type="text" v-model="searchQuery" :placeholder="$t('tokenList.searchPlaceholder')"
              class="search-input" />
            <button v-if="searchQuery.trim()" @click="searchQuery = ''" class="modal-close clear-search-btn" v-tooltip="'清空搜索'">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
              </svg>
            </button>
          </div>

          <!-- 右侧：操作按钮 -->
          <div class="header-actions">
            <!-- 同步按钮 - 仅双向存储模式显示 -->
            <button v-if="isDatabaseAvailable" @click="handleBidirectionalSync" class="btn primary small"
              :disabled="isSyncing" v-tooltip="$t('tokenList.syncTooltip')">
              <svg v-if="!isSyncing" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z" />
              </svg>
              {{ isSyncing ? $t('tokenList.syncing') : $t('tokenList.sync') }}
            </button>
            <button @click="handleAddToken" class="btn primary small">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
              {{ $t('tokenList.addToken') }}
            </button>
          </div>
        </div>

        <div ref="pageBodyRef" class="page-body">
          <!-- Loading State -->
          <div v-if="isLoading" class="loading-state">
            <div class="spinner"></div>
            <p>{{ $t('tokenList.loading') }}</p>
          </div>

          <!-- Empty State -->
          <div v-else-if="tokens.length === 0" class="empty-state">
            <div class="empty-icon">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
              </svg>
            </div>
            <h3>{{ $t('tokenList.empty') }}</h3>
            <button class="batch-import-btn-empty" @click="showBatchImportConfirm" v-tooltip="$t('tokenList.batchImport')">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z" />
              </svg>
              {{ $t('tokenList.batchImport') }}
            </button>
          </div>

          <!-- Token List -->
          <div v-else class="token-list">
            <!-- 工具栏：筛选 + 操作按钮 -->
            <div class="list-toolbar">
              <!-- 左侧：筛选和视图控制按钮 -->
              <div class="toolbar-left">
                <!-- 状态筛选下拉菜单 -->
                <div class="status-filter-dropdown">
                  <button class="filter-btn status-filter-btn" @click.stop="toggleStatusFilterMenu"
                    :class="{ 'active': selectedStatusFilter !== null }">
                    <!-- 状态图标 -->
                    <svg v-if="selectedStatusFilter === 'ACTIVE'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon active">
                      <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                    </svg>
                    <svg v-else-if="selectedStatusFilter === 'DEPLETE'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon deplete">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11h-4v4h-2v-4H7v-2h4V7h2v4h4v2z" />
                    </svg>
                    <svg v-else-if="selectedStatusFilter === 'SUSPENDED'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon suspended">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 11c-.55 0-1-.45-1-1V8c0-.55.45-1 1-1s1 .45 1 1v4c0 .55-.45 1-1 1zm1 4h-2v-2h2v2z" />
                    </svg>
                    <svg v-else-if="selectedStatusFilter === 'OTHER'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon other">
                      <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" />
                    </svg>
                    <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon all">
                      <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
                    </svg>
                    <span class="filter-label">{{ selectedStatusFilter ? $t(`tokenList.${selectedStatusFilter.toLowerCase()}Status`) : $t('tokenList.allStatus') }}</span>
                    <span class="filter-count">{{ selectedStatusFilter ? statusStatistics[selectedStatusFilter] : statusStatistics.TOTAL }}</span>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="dropdown-arrow">
                      <path d="M7 10l5 5 5-5z" />
                    </svg>
                  </button>

                  <!-- 下拉菜单 -->
                  <Transition name="dropdown">
                    <div v-if="showStatusFilterMenu" class="status-filter-menu" @click.stop>
                      <button :class="['status-option', { active: selectedStatusFilter === null }]"
                        @click="selectStatusFilter(null)">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon all">
                          <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
                        </svg>
                        <span>{{ $t('tokenList.allStatus') }}</span>
                        <span class="status-count">{{ statusStatistics.TOTAL }}</span>
                        <svg v-if="selectedStatusFilter === null" width="16" height="16" viewBox="0 0 24 24"
                          fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'active-status', { active: selectedStatusFilter === 'ACTIVE' }]"
                        @click="selectStatusFilter('ACTIVE')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon active">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                        <span>{{ $t('tokenList.activeStatus') }}</span>
                        <span class="status-count">{{ statusStatistics.ACTIVE }}</span>
                        <svg v-if="selectedStatusFilter === 'ACTIVE'" width="16" height="16" viewBox="0 0 24 24"
                          fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'deplete-status', { active: selectedStatusFilter === 'DEPLETE' }]"
                        @click="selectStatusFilter('DEPLETE')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon deplete">
                          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm5 11h-4v4h-2v-4H7v-2h4V7h2v4h4v2z" />
                        </svg>
                        <span>{{ $t('tokenList.depleteStatus') }}</span>
                        <span class="status-count">{{ statusStatistics.DEPLETE }}</span>
                        <svg v-if="selectedStatusFilter === 'DEPLETE'" width="16" height="16" viewBox="0 0 24 24"
                          fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'suspended-status', { active: selectedStatusFilter === 'SUSPENDED' }]"
                        @click="selectStatusFilter('SUSPENDED')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon suspended">
                          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 11c-.55 0-1-.45-1-1V8c0-.55.45-1 1-1s1 .45 1 1v4c0 .55-.45 1-1 1zm1 4h-2v-2h2v2z" />
                        </svg>
                        <span>{{ $t('tokenList.suspendedStatus') }}</span>
                        <span class="status-count">{{ statusStatistics.SUSPENDED }}</span>
                        <svg v-if="selectedStatusFilter === 'SUSPENDED'" width="16" height="16" viewBox="0 0 24 24"
                          fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['status-option', 'other-status', { active: selectedStatusFilter === 'OTHER' }]"
                        @click="selectStatusFilter('OTHER')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="status-icon other">
                          <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" />
                        </svg>
                        <span>{{ $t('tokenList.otherStatus') }}</span>
                        <span class="status-count">{{ statusStatistics.OTHER }}</span>
                        <svg v-if="selectedStatusFilter === 'OTHER'" width="16" height="16" viewBox="0 0 24 24"
                          fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>
                    </div>
                  </Transition>
                </div>

                <!-- 标签筛选下拉菜单 -->
                <div class="tag-filter-dropdown">
                  <button class="btn-icon tag-filter-btn" @click.stop="toggleTagFilterMenu"
                    v-tooltip="selectedTags.size > 0 ? `${tagFilterMode === 'include' ? '包含' : '排除'}: ${Array.from(selectedTags).join(', ')}` : '按标签筛选'"
                    :class="{ 'active': selectedTags.size > 0 }">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58.55 0 1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41 0-.55-.23-1.06-.59-1.42zM5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7z"/>
                    </svg>
                    <span v-if="selectedTags.size > 0" class="tag-count-badge">{{ selectedTags.size }}</span>
                  </button>

                  <!-- 下拉菜单 -->
                  <Transition name="dropdown">
                    <div v-if="showTagFilterMenu" class="tag-filter-menu" @click.stop>
                      <!-- 顶部操作栏 -->
                      <div class="tag-menu-header">
                        <!-- 全部选项 -->
                        <button
                          :class="['tag-option', 'tag-option-all', { active: selectedTags.size === 0 }]"
                          @click="clearTagFilter">
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path
                              d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
                          </svg>
                          <span>全部</span>
                          <svg v-if="selectedTags.size === 0" width="16" height="16" viewBox="0 0 24 24"
                            fill="currentColor" class="check-icon">
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                          </svg>
                        </button>

                        <!-- 筛选模式切换按钮 -->
                        <button class="tag-mode-toggle-btn" @click="toggleTagFilterMode"
                          v-tooltip="tagFilterMode === 'include' ? '切换到排除模式' : '切换到包含模式'">
                          <svg v-if="tagFilterMode === 'include'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                          </svg>
                          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M19 13H5v-2h14v2z"/>
                          </svg>
                          <span>{{ tagFilterMode === 'include' ? '包含' : '排除' }}</span>
                        </button>
                      </div>

                      <div v-if="allTags.length > 0" class="sort-divider"></div>

                      <!-- 标签选项 -->
                      <div v-if="allTags.length > 0" class="tag-list">
                        <button v-for="tag in allTags" :key="tag"
                          :class="['tag-option', { active: selectedTags.has(tag) }]"
                          @click="toggleTag(tag)">
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path
                              d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58.55 0 1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41 0-.55-.23-1.06-.59-1.42zM5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7z" />
                          </svg>
                          <span class="tag-text">{{ tag }}</span>
                          <svg v-if="selectedTags.has(tag)" width="16" height="16" viewBox="0 0 24 24"
                            fill="currentColor" class="check-icon">
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                          </svg>
                        </button>
                      </div>

                      <!-- 无标签提示 -->
                      <div v-else class="no-tag-hint">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
                          <path
                            d="M11 15h2v2h-2zm0-8h2v6h-2zm.99-5C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z" />
                        </svg>
                        <p>暂无标签数据</p>
                      </div>
                    </div>
                  </Transition>
                </div>

                <!-- 邮箱后缀筛选下拉菜单 -->
                <div class="email-suffix-dropdown">
                  <button class="btn-icon" @click.stop="toggleEmailSuffixMenu"
                    v-tooltip="selectedEmailSuffix ? `筛选: ${selectedEmailSuffix}` : '按邮箱后缀筛选'"
                    :class="{ 'active': selectedEmailSuffix }">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path
                        d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
                    </svg>
                  </button>

                  <!-- 下拉菜单 -->
                  <Transition name="dropdown">
                    <div v-if="showEmailSuffixMenu" class="email-suffix-menu" @click.stop>
                      <!-- 顶部操作栏 -->
                      <div class="suffix-menu-header">
                        <!-- 全部选项 -->
                        <button
                          :class="['suffix-option', 'suffix-option-all', { active: selectedEmailSuffix === null }]"
                          @click="clearEmailSuffixFilter">
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path
                              d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z" />
                          </svg>
                          <span>全部</span>
                          <span class="suffix-count">{{ statusFilteredTokens.length }}</span>
                          <svg v-if="selectedEmailSuffix === null" width="16" height="16" viewBox="0 0 24 24"
                            fill="currentColor" class="check-icon">
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                          </svg>
                        </button>

                        <!-- 提取按钮 -->
                        <button class="suffix-extract-btn" @click="extractEmailSuffixes"
                          :disabled="emailSuffixes.length === 0"
                          v-tooltip="emailSuffixes.length === 0 ? '暂无邮箱后缀可提取' : '复制所有邮箱后缀到剪贴板'">
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path
                              d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z" />
                          </svg>
                          <span>提取</span>
                        </button>
                      </div>

                      <div v-if="emailSuffixes.length > 0" class="sort-divider"></div>

                      <!-- 邮箱后缀选项 -->
                      <div v-if="emailSuffixes.length > 0" class="suffix-list">
                        <button v-for="suffix in emailSuffixes" :key="suffix"
                          :class="['suffix-option', { active: selectedEmailSuffix === suffix }]"
                          @click="selectEmailSuffix(suffix)">
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path
                              d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
                          </svg>
                          <span class="suffix-text">{{ suffix }}</span>
                          <span class="suffix-count">{{ emailSuffixStatistics[suffix] || 0 }}</span>
                          <svg v-if="selectedEmailSuffix === suffix" width="16" height="16" viewBox="0 0 24 24"
                            fill="currentColor" class="check-icon">
                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                          </svg>
                        </button>
                      </div>

                      <!-- 无邮箱后缀提示 -->
                      <div v-else class="no-suffix-hint">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
                          <path
                            d="M11 15h2v2h-2zm0-8h2v6h-2zm.99-5C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z" />
                        </svg>
                        <p>暂无邮箱后缀数据</p>
                      </div>
                    </div>
                  </Transition>
                </div>

                <!-- 排序下拉菜单 -->
                <div class="sort-dropdown">
                <button class="btn-icon" @click.stop="toggleSortMenu" v-tooltip="$t('tokenList.sort')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                    stroke-linecap="round">
                    <!-- 左边向上箭头 -->
                    <path d="M7 16V6M4 9l3-3 3 3" />
                    <!-- 右边向下箭头 -->
                    <path d="M17 8v10M14 15l3 3 3-3" />
                  </svg>
                </button>

                  <!-- 下拉菜单 -->
                  <Transition name="dropdown">
                    <div v-if="showSortMenu" class="sort-menu" @click.stop>
                      <button :class="['sort-option', { active: sortType === 'time' && sortOrder === 'desc' }]"
                        @click="setSortType('time', 'desc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path
                            d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM7 10h5v5H7z" />
                        </svg>
                        <span>{{ $t('tokenList.sortByTime') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-down">
                          <path d="M16 10l-4 4-4-4h8z" />
                        </svg>
                        <svg v-if="sortType === 'time' && sortOrder === 'desc'" width="16" height="16"
                          viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['sort-option', { active: sortType === 'time' && sortOrder === 'asc' }]"
                        @click="setSortType('time', 'asc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path
                            d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM7 10h5v5H7z" />
                        </svg>
                        <span>{{ $t('tokenList.sortByTime') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-up">
                          <path d="M8 14l4-4 4 4H8z" />
                        </svg>
                        <svg v-if="sortType === 'time' && sortOrder === 'asc'" width="16" height="16"
                          viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <div class="sort-divider"></div>

                      <button :class="['sort-option', { active: sortType === 'balance' && sortOrder === 'desc' }]"
                        @click="setSortType('balance', 'desc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path
                            d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z" />
                        </svg>
                        <span>{{ $t('tokenList.sortByBalance') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-down">
                          <path d="M16 10l-4 4-4-4h8z" />
                        </svg>
                        <svg v-if="sortType === 'balance' && sortOrder === 'desc'" width="16" height="16"
                          viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['sort-option', { active: sortType === 'balance' && sortOrder === 'asc' }]"
                        @click="setSortType('balance', 'asc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path
                            d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z" />
                        </svg>
                        <span>{{ $t('tokenList.sortByBalance') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-up">
                          <path d="M8 14l4-4 4 4H8z" />
                        </svg>
                        <svg v-if="sortType === 'balance' && sortOrder === 'asc'" width="16" height="16"
                          viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <div class="sort-divider"></div>

                      <button :class="['sort-option', { active: sortType === 'tag' && sortOrder === 'asc' }]"
                        @click="setSortType('tag', 'asc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58.55 0 1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41 0-.55-.23-1.06-.59-1.42zM5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7z"/>
                        </svg>
                        <span>{{ $t('tokenList.sortByTag') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-up">
                          <path d="M8 14l4-4 4 4H8z" />
                        </svg>
                        <svg v-if="sortType === 'tag' && sortOrder === 'asc'" width="16" height="16"
                          viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>

                      <button :class="['sort-option', { active: sortType === 'tag' && sortOrder === 'desc' }]"
                        @click="setSortType('tag', 'desc')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58.55 0 1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41 0-.55-.23-1.06-.59-1.42zM5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7z"/>
                        </svg>
                        <span>{{ $t('tokenList.sortByTag') }}</span>
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" class="arrow-down">
                          <path d="M16 10l-4 4-4-4h8z" />
                        </svg>
                        <svg v-if="sortType === 'tag' && sortOrder === 'desc'" width="16" height="16"
                          viewBox="0 0 24 24" fill="currentColor" class="check-icon">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                        </svg>
                      </button>
                    </div>
                  </Transition>
                </div>

                <!-- 布局切换按钮 -->
                <button
                  class="btn-icon"
                  @click="toggleViewMode"
                  v-tooltip="viewMode === 'card' ? $t('tokenList.switchToTable') : $t('tokenList.switchToCard')"
                  :class="{ 'active': viewMode === 'table' }"
                >
                  <!-- 卡片图标 -->
                  <svg v-if="viewMode === 'table'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M4 11h5V5H4v6zm0 7h5v-6H4v6zm6 0h5v-6h-5v6zm6 0h5v-6h-5v6zm-6-7h5V5h-5v6zm6-6v6h5V5h-5z"/>
                  </svg>
                  <!-- 列表图标 -->
                  <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M3 14h4v-4H3v4zm0 5h4v-4H3v4zM3 9h4V5H3v4zm5 5h13v-4H8v4zm0 5h13v-4H8v4zM8 5v4h13V5H8z"/>
                  </svg>
                </button>
              </div>

              <!-- 右侧：其他操作按钮 -->
              <div class="toolbar-right">
                <!-- Session 即将过期警告按钮 -->
                <button
                  v-if="expiringSessionTokens.length > 0"
                  class="btn-icon warning"
                  @click="openSessionRefreshModal"
                  v-tooltip="$t('tokenList.sessionExpiring', { count: expiringSessionTokens.length })"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
                  </svg>
                  <span class="warning-badge">{{ expiringSessionTokens.length }}</span>
                </button>

                <!-- 邮箱显示切换按钮 -->
              <button
                  class="btn-icon"
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
              <button
                class="btn-icon"
                @click="handleRefresh"
                :disabled="isRefreshing"
                v-tooltip="$t('tokenList.refresh')"
              >
                <svg v-if="!isRefreshing" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="spinning">
                  <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
                </svg>
              </button>

              <!-- 打开文件夹按钮 -->
              <button
                class="btn-icon"
                @click="openDataFolder"
                v-tooltip="$t('bookmarkManager.openDataFolder')"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z" />
                </svg>
              </button>

              <!-- 批量导入按钮 -->
              <button
                class="btn-icon"
                @click="showBatchImportConfirm"
                v-tooltip="$t('tokenList.batchImport')"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z" />
                </svg>
              </button>

              <!-- 批量删除按钮 -->
              <button
                class="btn-icon"
                @click="handleBatchDelete"
                v-tooltip="$t('tokenList.batchDelete')"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
                </svg>
              </button>
              </div>
            </div>

            <!-- 无搜索结果提示 -->
            <div v-if="searchQuery.trim() && filteredTokens.length === 0" class="no-search-results">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
                <path
                  d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
              </svg>
              <p>{{ $t('tokenList.noSearchResults') }}</p>
            </div>

            <!-- 无筛选结果提示 -->
            <div v-else-if="filteredTokens.length === 0" class="no-search-results">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
              <p>{{ $t('tokenList.noFilterResults') }}</p>
            </div>

            <template v-else>
              <!-- 卡片布局 -->
              <div v-if="viewMode === 'card'" class="token-grid">
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
              <div v-else class="token-table-wrapper">
                <table class="token-table">
                  <thead>
                    <tr>
                      <th class="th-checkbox">
                        <div class="header-checkbox" @click="toggleSelectAll">
                          <div
                            class="checkbox-inner"
                            :class="{
                              'checked': isAllSelected,
                              'indeterminate': isPartialSelected
                            }"
                          >
                            <svg v-if="isAllSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                            </svg>
                            <svg v-else-if="isPartialSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                              <path d="M19 13H5v-2h14v2z"/>
                            </svg>
                          </div>
                        </div>
                      </th>
                      <th class="th-tag">{{ $t('tokenList.tableHeaderTag') }}</th>
                      <th class="th-status">{{ $t('tokenList.tableHeaderStatus') }}</th>
                      <th class="th-email">{{ $t('tokenList.tableHeaderEmail') }}</th>
                      <th class="th-balance">{{ $t('tokenList.tableHeaderBalance') }}</th>
                      <th class="th-dates">{{ $t('tokenList.tableHeaderDates') }}</th>
                      <th class="th-actions">{{ $t('tokenList.tableHeaderActions') }}</th>
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

              <!-- 分页组件 -->
              <Pagination
                v-if="filteredTokens.length > 0"
                :current-page="currentPage"
                :total-pages="totalPages"
                :total-items="filteredTokens.length"
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

    <!-- 批量操作工具栏 - 使用 Teleport 确保正确定位 -->
    <Teleport to="body">
      <Transition name="slide-up">
        <div v-if="isSelectionMode" class="batch-toolbar">
            <div class="batch-toolbar-content">
              <!-- 左侧：选中数量 -->
              <div class="batch-info">
                <span class="selected-count">
                  {{ $t('tokenList.selected', { count: selectedTokenIds.size }) }}
                </span>
                <button @click="selectAllOnPage" class="btn-text">
                  {{ $t('tokenList.selectAllPage') }}
                </button>
              </div>

              <!-- 右侧：操作按钮 -->
              <div class="batch-actions">
                <!-- 批量刷新状态 -->
                <button @click="batchRefreshSelected" class="btn-icon"
                        :disabled="isBatchRefreshing"
                        v-tooltip="$t('tokenList.batchRefreshSelected')">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
                  </svg>
                </button>

                <!-- 批量导出 -->
                <button @click="batchExportSelected" class="btn-icon"
                        v-tooltip="$t('tokenList.batchExportSelected')">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
                  </svg>
                </button>

                <!-- 批量获取绑卡链接 -->
                <button @click="batchFetchPaymentLinks" class="btn-icon"
                        :disabled="isBatchFetchingPaymentLinks"
                        v-tooltip="$t('tokenList.batchFetchPaymentLinks')">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
                  </svg>
                </button>

                <!-- 批量编辑标签 -->
                <button @click="showBatchTagEditor = true" class="btn-icon"
                        v-tooltip="$t('tokenList.batchEditTag')">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M21.41 11.58l-9-9C12.05 2.22 11.55 2 11 2H4c-1.1 0-2 .9-2 2v7c0 .55.22 1.05.59 1.42l9 9c.36.36.86.58 1.41.58.55 0 1.05-.22 1.41-.59l7-7c.37-.36.59-.86.59-1.41 0-.55-.23-1.06-.59-1.42zM5.5 7C4.67 7 4 6.33 4 5.5S4.67 4 5.5 4 7 4.67 7 5.5 6.33 7 5.5 7z"/>
                  </svg>
                </button>

                <!-- 批量删除 -->
                <button @click="showBatchDeleteSelectedConfirm" class="btn-icon danger"
                        v-tooltip="$t('tokenList.batchDeleteSelected')">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                  </svg>
                </button>

                <!-- 取消选择 -->
                <button @click="clearSelection" class="btn-icon close"
                        v-tooltip="$t('tokenList.cancelSelection')">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </Transition>
      </Teleport>

    <!-- Token Form Modal -->
    <Teleport to="body">
      <TokenForm v-if="showTokenFormModal" :token="editingToken" :all-tokens="tokens" @close="closeTokenForm" @success="handleTokenFormSuccess"
        @update-token="handleUpdateToken" @add-token="handleAddTokenFromForm"
        @auto-import-completed="handleAutoImportCompleted" @manual-import-completed="handleManualImportCompleted" />
    </Teleport>

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-tokens-count="tokens.length"
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
    <Teleport to="body">
      <Transition name="modal" appear>
        <div v-if="showBatchImportDialog" class="modal-overlay batch-import-overlay" @click="showBatchImportDialog = false">
          <div class="modal-content batch-import-modal" @click.stop>
            <div class="modal-header">
              <h3>{{ $t('tokenList.batchImportTitle') }}</h3>
              <button @click="showBatchImportDialog = false" class="modal-close">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                </svg>
              </button>
            </div>

            <!-- Tab Navigation -->
            <div class="batch-import-tabs">
              <button :class="['batch-import-tab', { active: batchImportTab === 'session' }]"
                @click="batchImportTab = 'session'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm0 4c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm6 12H6v-1.4c0-2 4-3.1 6-3.1s6 1.1 6 3.1V19z" />
                </svg>
                {{ $t('tokenList.sessionImportTab') }}
              </button>
              <button :class="['batch-import-tab', { active: batchImportTab === 'token' }]"
                @click="batchImportTab = 'token'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
                </svg>
                {{ $t('tokenList.tokenImportTab') }}
              </button>
            </div>

            <div class="modal-body">
              <!-- Session Tab Content -->
              <div v-if="batchImportTab === 'session'" class="tab-content">
                <p class="modal-message">{{ $t('tokenList.sessionImportMessage') }}</p>

                <!-- Session 动态输入框列表 -->
                <div class="session-inputs-container">
                  <div v-for="(input, index) in sessionInputs" :key="input.id" class="session-input-item">
                    <span class="session-input-number">{{ index + 1 }}.</span>
                    <input v-model="input.value" type="text" :placeholder="$t('tokenList.sessionInputPlaceholder')"
                      class="session-input-field" />
                    <button @click="removeSessionInput(input.id)" class="session-input-delete"
                      v-tooltip="$t('tokenList.deleteInput')" :disabled="sessionInputs.length <= 1">
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
                      </svg>
                    </button>
                  </div>
                </div>

                <!-- 添加更多按钮 -->
                <button @click="addSessionInput" @contextmenu="handleContextMenu($event, 'session')"
                  class="add-more-btn">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
                  </svg>
                  {{ $t('tokenList.addMore') }}
                </button>
              </div>

              <!-- Token Tab Content -->
              <div v-else-if="batchImportTab === 'token'" class="tab-content">
                <p class="modal-message">{{ $t('tokenList.tokenImportMessage') }}</p>

                <!-- 格式说明和填充按钮 -->
                <div class="format-option-single">
                  <div class="format-header">
                    <span class="format-title">{{ $t('tokenList.tokenFormatTitle') }}</span>
                  </div>
                  <p class="format-desc">{{ $t('tokenList.tokenFormatDesc') }}</p>
                  <button @click="fillTokenTemplate()" @contextmenu="handleContextMenu($event, 'token')"
                    class="btn-fill-template">
                    {{ $t('tokenList.fillTemplate') }}
                  </button>
                </div>

                <div class="import-input-section">
                  <textarea v-model="importJsonText" rows="10" class="import-textarea"
                    @input="validateImportJson"></textarea>
                </div>

                <!-- 错误信息 -->
                <div v-if="importErrors.length > 0" class="import-errors">
                  <div class="error-header">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path
                        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
                    </svg>
                    <span>{{ $t('tokenList.importErrorsFound', { count: importErrors.length }) }}</span>
                  </div>
                  <ul class="error-list">
                    <li v-for="(error, index) in importErrors" :key="index">{{ error }}</li>
                  </ul>
                </div>

                <!-- 预览信息 -->
                <div v-if="importPreview.length > 0" class="import-preview">
                  <div class="preview-header">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                    </svg>
                    <span>{{ $t('tokenList.importPreviewReady', { count: importPreview.length }) }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div class="modal-footer">
              <button @click="showBatchImportDialog = false" class="btn secondary">
                {{ $t('tokenList.cancel') }}
              </button>
              <button @click="executeBatchImport" class="btn primary"
                :disabled="isImporting || (batchImportTab === 'session' ? validSessionCount === 0 : importPreview.length === 0)">
                <template v-if="isImporting">
                  {{ $t('tokenList.importing') }}
                </template>
                <template v-else>
                  {{ batchImportTab === 'session'
                    ? $t('tokenList.batchAdd', { count: validSessionCount })
                    : $t('tokenList.confirmImport')
                  }}
                </template>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Context Menu for Fill Template -->
    <Teleport to="body">
      <div v-if="showContextMenu" class="context-menu-overlay" @click="closeContextMenu">
        <div class="context-menu" :style="{ left: contextMenuPosition.x + 'px', top: contextMenuPosition.y + 'px' }"
          @click.stop>
          <div class="context-menu-header">{{ $t('tokenList.selectFillCount') }}</div>

          <!-- Session Tab: 简化菜单 -->
          <template v-if="contextMenuType === 'session'">
            <div class="context-menu-custom">
              <input v-model.number="customFillCount" type="number" min="1" max="20"
                :placeholder="$t('tokenList.customCount')" class="custom-count-input" @click.stop />
              <button @click="setDefaultCountFromInput" class="btn-custom-fill">
                {{ $t('tokenList.setAsDefault') }}
              </button>
            </div>
          </template>

          <!-- Token Tab: 完整菜单 -->
          <template v-else>
            <div class="context-menu-custom">
              <input v-model.number="customFillCount" type="number" min="1" max="100"
                :placeholder="$t('tokenList.customCount')" class="custom-count-input" @click.stop />
              <button @click="fillWithCustomCount" class="btn-custom-fill">
                {{ $t('common.confirm') }}
              </button>
            </div>
            <div class="context-menu-divider"></div>
            <div class="context-menu-item" @click="selectFillCount(1)">1</div>
            <div class="context-menu-item" @click="selectFillCount(3)">3</div>
            <div class="context-menu-item" @click="selectFillCount(5)">5</div>
            <div class="context-menu-item" @click="selectFillCount(10)">10</div>
            <div class="context-menu-item" @click="selectFillCount(20)">20</div>
          </template>
        </div>
      </div>
    </Teleport>

    <!-- Batch Delete Confirmation Dialog -->
    <Teleport to="body">
      <Transition name="modal" appear>
        <div v-if="showBatchDeleteDialog" class="modal-overlay batch-delete-overlay" @click="showBatchDeleteDialog = false">
          <div class="modal-content batch-delete-modal" @click.stop>
            <div class="modal-header">
              <h3>{{ $t('tokenList.batchDeleteConfirm') }}</h3>
              <button @click="showBatchDeleteDialog = false" class="modal-close">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                </svg>
              </button>
            </div>
            <div class="modal-body">
              <p class="modal-message">{{ $t('tokenList.batchDeleteMessage') }}</p>
              <div class="delete-stats">
                <div class="stat-item">
                  <span class="stat-label">{{ $t('tokenList.bannedCount') }}:</span>
                  <span class="stat-value">{{ bannedTokensCount }} {{ $t('tokenList.items') }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">{{ $t('tokenList.expiredCount') }}:</span>
                  <span class="stat-value">{{ expiredTokensCount }} {{ $t('tokenList.items') }}</span>
                </div>
                <div class="stat-item total">
                  <span class="stat-label">{{ $t('tokenList.totalCount') }}:</span>
                  <span class="stat-value">{{ deletableTokensCount }} {{ $t('tokenList.items') }}</span>
                </div>
              </div>
              <p class="modal-warning">{{ $t('tokenList.cannotUndo') }}</p>
            </div>
            <div class="modal-footer">
              <button @click="executeBatchDelete" class="btn danger" :disabled="isDeleting">
                {{ isDeleting ? $t('tokenList.deleting') : $t('tokenList.confirmDelete') }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Selected Tokens Delete Confirmation Dialog -->
    <Teleport to="body">
      <Transition name="modal" appear>
        <div v-if="showSelectedDeleteDialog" class="modal-overlay batch-delete-overlay" @click="showSelectedDeleteDialog = false">
          <div class="modal-content batch-delete-modal" @click.stop>
            <div class="modal-header">
              <h3>{{ $t('tokenList.batchDeleteConfirm') }}</h3>
              <button @click="showSelectedDeleteDialog = false" class="modal-close">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                </svg>
              </button>
            </div>
            <div class="modal-body">
              <p class="modal-message">{{ $t('tokenList.selectedDeleteMessage') }}</p>
              <div class="delete-stats">
                <div class="stat-item total">
                  <span class="stat-label">{{ $t('tokenList.selectedCount') }}:</span>
                  <span class="stat-value">{{ selectedTokenIds.size }} {{ $t('tokenList.items') }}</span>
                </div>
              </div>
              <p class="modal-warning">{{ $t('tokenList.cannotUndo') }}</p>
            </div>
            <div class="modal-footer">
              <button @click="executeBatchDeleteSelected" class="btn danger">
                {{ $t('tokenList.confirmDelete') }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

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
import SyncQueueModal from './SyncQueueModal.vue'
import SessionRefreshModal from './SessionRefreshModal.vue'
import Pagination from '../common/Pagination.vue'

const { t } = useI18n()

// Props
const props = defineProps({
  tokens: {
    type: Array,
    default: null // null表示自主管理，非null表示由父组件传入
  }
})

// 内部状态管理 - TokenList直接管理tokens和存储状态
// 如果props.tokens不为null，则使用props.tokens，否则自主管理
const internalTokens = ref([])
const tokens = computed(() => props.tokens !== null ? props.tokens : internalTokens.value)
const isLoading = ref(false)
const isDatabaseAvailable = ref(false)
const isStorageInitializing = ref(false)

// 初始化就绪标记
const isReady = ref(false)

// 同步状态标记 - 用于防止同步时触发自动保存
const isSyncing = ref(false)
const isLoadingFromSync = ref(false)

// Session 刷新相关状态
const showSessionRefreshModal = ref(false)
const isRefreshingSessions = ref(false)

// 同步需求标记 - 标识本地有未同步到数据库的更改
const isSyncNeeded = ref(false)

// === 新增量同步协议状态 ===
// localStorage 中的存储键
const STORAGE_KEY_LAST_VERSION = 'atm-sync-last-version'
const STORAGE_KEY_PENDING_UPSERTS = 'atm-sync-pending-upserts'
const STORAGE_KEY_PENDING_DELETIONS = 'atm-sync-pending-deletions'

// 上次同步的版本号（从 localStorage 读取）
const lastVersion = ref(0)

// 待同步的本地变更
const pendingUpserts = ref(new Map())   // id -> token
const pendingDeletions = ref(new Map()) // id -> { id, email_note }

// 加载上次同步版本号
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
    console.warn('Failed to load lastVersion from localStorage:', error)
  }
  return 0
}

// 保存版本号到 localStorage
const saveLastVersion = (version) => {
  try {
    localStorage.setItem(STORAGE_KEY_LAST_VERSION, version.toString())
  } catch (error) {
    console.error('Failed to save lastVersion to localStorage:', error)
  }
}

// 保存待同步变更到 localStorage
const savePendingChanges = () => {
  try {
    const upsertsArr = Array.from(pendingUpserts.value.entries()).map(([id, token]) => ({ id, token }))
    const deletionsArr = Array.from(pendingDeletions.value.values())

    localStorage.setItem(STORAGE_KEY_PENDING_UPSERTS, JSON.stringify(upsertsArr))
    localStorage.setItem(STORAGE_KEY_PENDING_DELETIONS, JSON.stringify(deletionsArr))
  } catch (error) {
    console.error('Failed to save pending changes to localStorage:', error)
  }
}

// 从 localStorage 加载待同步变更
const loadPendingChanges = () => {
  try {
    const upsertsJson = localStorage.getItem(STORAGE_KEY_PENDING_UPSERTS)
    if (upsertsJson) {
      const arr = JSON.parse(upsertsJson)
      if (Array.isArray(arr)) {
        pendingUpserts.value = new Map(
          arr
            .filter(item => item && item.id && item.token)
            .map(item => [item.id, item.token])
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
            .map(item => [item.id, { id: item.id, email_note: item.email_note || null }])
        )
      }
    }

    // 如果存在未同步的本地变更，标记需要同步
    if (pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0) {
      isSyncNeeded.value = true
    }
  } catch (error) {
    console.warn('Failed to load pending changes from localStorage:', error)
  }
}

// 存储状态检查定时器
let storageCheckTimer = null

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
const pageBodyRef = ref(null)

// 搜索状态管理
const searchQuery = ref('')

// 状态筛选管理 - 改为单选模式
const selectedStatusFilter = ref(null) // null表示"全部", 其他值为具体状态

// 邮箱后缀筛选管理
const selectedEmailSuffix = ref(null) // null表示"全部", 其他值为具体邮箱后缀
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

// Session 动态输入框状态
const sessionInputs = ref([])
let nextSessionInputId = 1

// 右键菜单状态
const showContextMenu = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const contextMenuType = ref('') // 'session' 或 'token'
const customFillCount = ref(1)

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

// 基础筛选后的tokens - 包含除状态筛选外的所有筛选（搜索、标签、邮箱后缀）
// 用于状态统计，确保统计数字反映其他筛选条件
const baseFilteredTokens = computed(() => {
  let result = sortedTokens.value

  // 1. 应用邮箱后缀筛选
  if (selectedEmailSuffix.value !== null) {
    result = result.filter(token => {
      const suffix = extractEmailSuffix(token.email_note)
      return suffix === selectedEmailSuffix.value
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

  // 3. 应用搜索过滤
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

// 状态统计计算属性 - 基于其他筛选条件后的结果
const statusStatistics = computed(() => {
  const stats = {
    ACTIVE: 0,
    DEPLETE: 0,
    SUSPENDED: 0,
    OTHER: 0,
    TOTAL: baseFilteredTokens.value.length
  }

  baseFilteredTokens.value.forEach(token => {
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

// 状态筛选后的tokens - 用于邮箱后缀统计
const statusFilteredTokens = computed(() => {
  let result = baseFilteredTokens.value

  // 应用状态筛选
  if (selectedStatusFilter.value !== null) {
    result = result.filter(token => {
      const status = token.ban_status
      const portalInfo = token.portal_info
      const creditTotal = portalInfo?.credit_total
      const creditsBalance = portalInfo?.credits_balance

      if (selectedStatusFilter.value === 'ACTIVE') {
        // 可用状态：ban_status === 'ACTIVE' 且余额大于0（排除用尽状态）
        return status === 'ACTIVE' && !(creditTotal && creditsBalance === 0)
      } else if (selectedStatusFilter.value === 'DEPLETE') {
        // 用尽状态：ban_status === 'ACTIVE' 且 credit_total 不为空/不为null 且 credits_balance === 0
        return status === 'ACTIVE' && creditTotal && creditsBalance === 0
      } else if (selectedStatusFilter.value === 'SUSPENDED') {
        return status === 'SUSPENDED'
      } else if (selectedStatusFilter.value === 'OTHER') {
        // OTHER包含除ACTIVE和SUSPENDED以外的所有状态
        return status !== 'ACTIVE' && status !== 'SUSPENDED'
      }

      return true
    })
  }

  return result
})

// 邮箱后缀统计 - 基于状态筛选后的token列表
const emailSuffixStatistics = computed(() => {
  const stats = {}

  statusFilteredTokens.value.forEach(token => {
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

// 提取所有唯一的标签 - 按字母排序（忽略大小写）
const allTags = computed(() => {
  const tagMap = new Map() // 使用 Map 来存储小写 -> 原始标签的映射

  statusFilteredTokens.value.forEach(token => {
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

// 过滤后的tokens计算属性 - 直接使用 statusFilteredTokens（已包含所有筛选）
const filteredTokens = computed(() => {
  return statusFilteredTokens.value
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

  // 清空高亮状态，避免排序时重新触发动画
  if (highlightedTokenId.value) {
    highlightedTokenId.value = null
    if (highlightTimer) {
      clearTimeout(highlightTimer)
      highlightTimer = null
    }
  }
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

  // 清空高亮状态
  if (highlightedTokenId.value) {
    highlightedTokenId.value = null
    if (highlightTimer) {
      clearTimeout(highlightTimer)
      highlightTimer = null
    }
  }
}

// 选择状态筛选 - 单选模式
const selectStatusFilter = (status) => {
  // 如果点击当前已选中的状态,则取消选择(回到"全部")
  if (selectedStatusFilter.value === status) {
    selectedStatusFilter.value = null
  } else {
    selectedStatusFilter.value = status
  }
  // 重置到第一页
  currentPage.value = 1
  // 关闭菜单
  showStatusFilterMenu.value = false
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

// 选择邮箱后缀筛选
const selectEmailSuffix = (suffix) => {
  // 如果点击当前已选中的后缀,则取消选择(回到"全部")
  if (selectedEmailSuffix.value === suffix) {
    selectedEmailSuffix.value = null
  } else {
    selectedEmailSuffix.value = suffix
  }
  showEmailSuffixMenu.value = false
  // 重置到第一页
  currentPage.value = 1
}

// 清除邮箱后缀筛选
const clearEmailSuffixFilter = () => {
  selectedEmailSuffix.value = null
  showEmailSuffixMenu.value = false
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
    // 查找真正的滚动容器（向上遍历 DOM 树）
    let element = pageBodyRef.value
    while (element && element !== document.body) {
      const style = window.getComputedStyle(element)
      const hasScroll = element.scrollHeight > element.clientHeight
      const canScroll = style.overflowY === 'auto' || style.overflowY === 'scroll'

      if (hasScroll && canScroll) {
        element.scrollTop = 0
        break
      }

      element = element.parentElement
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

// 处理右键菜单
const handleContextMenu = (event, type) => {
  event.preventDefault()
  contextMenuType.value = type
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  showContextMenu.value = true
}

// 关闭右键菜单
const closeContextMenu = () => {
  showContextMenu.value = false
}

// 选择填充数量
const selectFillCount = (count) => {
  if (contextMenuType.value === 'session') {
    fillSessionTemplate(count)
  } else if (contextMenuType.value === 'token') {
    fillTokenTemplate(count)
  }
  closeContextMenu()
}

// 从输入框设置默认数量
const setDefaultCountFromInput = () => {
  const count = parseInt(customFillCount.value)

  // 验证范围
  if (isNaN(count) || count < 1 || count > 20) {
    window.$notify.warning(t('tokenList.invalidDefaultCount'))
    return
  }

  // 保存到 localStorage
  if (saveDefaultInputCount(count)) {
    defaultInputCount.value = count
    // 立即重新初始化输入框
    initializeSessionInputs(count)
    window.$notify.success(t('tokenList.defaultCountSaved', { count: count }))
  } else {
    window.$notify.error(t('tokenList.saveDefaultFailed'))
  }

  closeContextMenu()
}

// 使用自定义数量填充（Token Tab）
const fillWithCustomCount = () => {
  const count = parseInt(customFillCount.value)
  if (isNaN(count) || count < 1) {
    window.$notify.warning(t('tokenList.invalidFillCount'))
    return
  }
  if (count > 100) {
    window.$notify.warning(t('tokenList.fillCountTooLarge'))
    return
  }
  selectFillCount(count)
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
      const emailNote = tokenToDelete?.email_note || null

      // 检查是否是仅在本地新增但从未同步过的 token
      const wasOnlyLocal = pendingUpserts.value.has(tokenId)

      internalTokens.value.splice(index, 1)
      deletedCount++

      // 从待更新队列中移除
      pendingUpserts.value.delete(tokenId)

      // 只有已同步到服务器的 token 才需要记录到待删除队列
      if (!wasOnlyLocal) {
        pendingDeletions.value.set(tokenId, { id: tokenId, email_note: emailNote })
      }
    }
  }
  
  // 标记需要同步（如果还有待同步内容）
  isSyncNeeded.value = pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0
  
  // 持久化待同步队列
  savePendingChanges()
  
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
      pendingUpserts.value.set(tokenId, token)
      pendingDeletions.value.delete(tokenId)
    }
  }

  // 标记需要同步
  isSyncNeeded.value = true
  
  // 持久化待同步队列
  savePendingChanges()

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
      pendingUpserts.value.set(tokenId, token)
      pendingDeletions.value.delete(tokenId)
    }
  }

  // 标记需要同步
  isSyncNeeded.value = true
  
  // 持久化待同步队列
  savePendingChanges()

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
      const tokenId = token.id
      const emailNote = token.email_note || null

      // 检查是否是仅在本地新增但从未同步过的 token
      const wasOnlyLocal = pendingUpserts.value.has(tokenId)

      // 从本地列表移除
      const index = internalTokens.value.findIndex(t => t.id === tokenId)
      if (index !== -1) {
        internalTokens.value.splice(index, 1)
        deletedCount++
      }

      // 从待更新队列中移除
      pendingUpserts.value.delete(tokenId)

      // 只有已同步到服务器的 token 才需要记录到待删除队列
      if (!wasOnlyLocal) {
        pendingDeletions.value.set(tokenId, { id: tokenId, email_note: emailNote })
      }
    }

    // 标记需要同步（如果还有待同步内容）
    isSyncNeeded.value = pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0

    // 持久化待同步队列
    savePendingChanges()

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

// 存储状态展示与同步队列查看
const showSyncQueueModal = ref(false)

const pendingUpsertsList = computed(() => Array.from(pendingUpserts.value.values()))
const pendingDeletionsList = computed(() => Array.from(pendingDeletions.value.values()))

const openSyncQueue = () => {
  if (!isDatabaseAvailable.value) return
  showSyncQueueModal.value = true
}

const closeSyncQueue = () => {
  showSyncQueueModal.value = false
}

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

  isRefreshingSessions.value = true

  try {
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

// 标记所有 token 为待同步
const handleMarkAllForSync = () => {
  if (tokens.value.length === 0) {
    window.$notify.warning(t('messages.noTokensToSync'))
    return
  }
  
  for (const token of tokens.value) {
    pendingUpserts.value.set(token.id, token)
  }
  
  // 持久化队列
  savePendingChanges()
  
  // 标记需要同步
  isSyncNeeded.value = true
  
  window.$notify.success(t('messages.allTokensMarkedForSync', { count: tokens.value.length }))
}

// 是否有待同步的变更（基于队列是否为空）
const hasPendingChanges = computed(() => {
  return pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0
})

// Computed properties for storage status display
const storageStatusText = computed(() => {
  if (isStorageInitializing.value) {
    return t('storage.initializing')
  }
  if (isDatabaseAvailable.value) {
    return hasPendingChanges.value
      ? `${t('storage.dualStorage')}-${t('storage.notSynced')}`
      : t('storage.dualStorage')
  }
  return t('storage.localStorage')
})

const storageStatusClass = computed(() => {
  // 如果正在初始化，显示加载样式
  if (isStorageInitializing.value) {
    return 'initializing'
  }
  // 如果是双向存储且有未同步变更，显示警告样式
  if (isDatabaseAvailable.value && hasPendingChanges.value) {
    return 'unsaved'
  }
  return 'saved'
})



// 存储状态管理方法
const getStorageStatus = async () => {
  try {
    const status = await invoke('get_storage_status')

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
    console.error('Failed to get storage status:', error)
    isDatabaseAvailable.value = false
    isStorageInitializing.value = false

    // 停止定时器
    if (storageCheckTimer) {
      clearInterval(storageCheckTimer)
      storageCheckTimer = null
    }
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
    pendingUpserts.value.set(updatedToken.id, updatedToken)
    pendingDeletions.value.delete(updatedToken.id)

    // 标记需要同步
    isSyncNeeded.value = true

    // 持久化待同步队列
    savePendingChanges()
    
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

  // 获取要删除的 token 信息（用于记录邮箱）
  const tokenToDelete = internalTokens.value[tokenIndex]
  const emailNote = tokenToDelete?.email_note || null

  // 检查是否是仅在本地新增但从未同步过的 token
  const wasOnlyLocal = pendingUpserts.value.has(tokenId)

  // 从内存中删除
  internalTokens.value = internalTokens.value.filter(token => token.id !== tokenId)
  window.$notify.success(t('messages.tokenDeleted'))

  // 从待更新队列中移除
  pendingUpserts.value.delete(tokenId)

  // 只有已同步到服务器的 token 才需要记录到待删除队列
  // 如果 token 仅在本地存在（从未同步），则无需通知服务器删除
  if (!wasOnlyLocal) {
    pendingDeletions.value.set(tokenId, { id: tokenId, email_note: emailNote })
  }

  // 标记需要同步（如果还有待同步内容）
  isSyncNeeded.value = pendingUpserts.value.size > 0 || pendingDeletions.value.size > 0

  // 持久化待同步队列
  savePendingChanges()

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
    pendingUpserts.value.set(updatedToken.id, updatedToken)
    pendingDeletions.value.delete(updatedToken.id)

    // 标记需要同步
    isSyncNeeded.value = true

    // 持久化待同步队列
    savePendingChanges()

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
  pendingUpserts.value.set(newToken.id, newToken)
  pendingDeletions.value.delete(newToken.id)
  isSyncNeeded.value = true

  // 持久化待同步队列
  savePendingChanges()

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

// 新的基于 version + tombstone 的增量同步方法
const handleSync = async () => {
  if (isSyncing.value) return
  if (!isDatabaseAvailable.value) {
    window.$notify.warning(t('messages.databaseNotAvailable'))
    return
  }

  isSyncing.value = true
  try {
    window.$notify.info(t('messages.syncingData'))

    // 构建同步请求
    const req = {
      last_version: lastVersion.value,
      upserts: Array.from(pendingUpserts.value.values()).map(token => ({ token })),
      deletions: Array.from(pendingDeletions.value.values()).map(item => ({ id: item.id })),
    }

    // 调用新的增量同步接口
    const res = await invoke('sync_tokens', { reqJson: JSON.stringify(req) })

    // 标记正在从同步加载，避免触发自动保存
    isLoadingFromSync.value = true

    // 应用服务器的 upserts
    for (const serverToken of res.upserts) {
      const idx = internalTokens.value.findIndex(t => t.id === serverToken.id)
      if (idx !== -1) {
        internalTokens.value[idx] = serverToken
      } else {
        internalTokens.value.push(serverToken)
      }
    }

    // 应用服务器 deletions
    for (const id of res.deletions) {
      const idx = internalTokens.value.findIndex(t => t.id === id)
      if (idx !== -1) {
        internalTokens.value.splice(idx, 1)
      }
    }

    // 更新 lastVersion
    lastVersion.value = res.new_version
    saveLastVersion(res.new_version)

    // 清空本地 pending 变更
    pendingUpserts.value.clear()
    pendingDeletions.value.clear()

    // 持久化空队列
    savePendingChanges()

    // 把最终 tokens 全量写入本地文件
    await handleSave()

    // 延迟重置标记
    await new Promise(resolve => setTimeout(resolve, 2100))
    isLoadingFromSync.value = false

    // 同步完成，清除同步需求标记
    isSyncNeeded.value = false

    window.$notify.success(t('messages.syncComplete'))
  } catch (error) {
    window.$notify.error(`${t('messages.syncFailed')}: ${error}`)
  } finally {
    isSyncing.value = false
  }
}

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

  // 加载上次同步版本号
  lastVersion.value = loadLastVersion()

  // 加载未同步的本地变更
  loadPendingChanges()

  // 初始化输入框
  initializeSessionInputs(defaultInputCount.value)

  // 首先获取存储状态
  await getStorageStatus()

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
  if (storageCheckTimer) {
    clearInterval(storageCheckTimer)
    storageCheckTimer = null
  }

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

<style scoped>
/* ============================================
   TokenList - Modern Tech Style
   ============================================ */

/* 空状态 - 科技风 */
.empty-state {
  text-align: center;
  padding: 50px 24px;
}

.empty-icon {
  color: var(--text-muted);
  margin-bottom: 18px;
  opacity: 0.6;
}

.empty-state h3 {
  color: var(--text-strong);
  margin: 0 0 10px 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.empty-state p {
  color: var(--text-muted);
  margin: 0 0 28px 0;
}

/* 空状态下的批量导入按钮 - 科技风 */
.batch-import-btn-empty {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 14px 28px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  font-weight: 600;
  box-shadow: 0 4px 15px var(--tech-glow-primary);
}

.batch-import-btn-empty:hover {
  filter: brightness(1.1);
  box-shadow: 0 6px 20px var(--tech-glow-primary);
  transform: translateY(-2px);
}

.batch-import-btn-empty:active {
  transform: translateY(0);
}

.batch-import-btn-empty svg {
  flex-shrink: 0;
}

/* 加载状态 - 科技风 */
.loading-state {
  text-align: center;
  padding: 50px 24px;
}

.spinner {
  width: 36px;
  height: 36px;
  border: 3px solid var(--tech-glass-border);
  border-top: 3px solid var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 18px;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.loading-state p {
  color: var(--text-muted);
  margin: 0;
}

/* ============================================
   Token Grid & Table - Modern Tech Style
   ============================================ */

.token-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(330px, 1fr));
  gap: 18px;
  padding: 6px;
}

/* 表格容器 - 磨砂玻璃 */
.token-table-wrapper {
  overflow-x: auto;
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  box-shadow: var(--tech-border-glow);
}

.token-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.token-table thead {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  position: sticky;
  top: 0;
  z-index: 10;
}

.token-table th {
  padding: 14px 10px;
  text-align: left;
  font-weight: 600;
  color: var(--text-muted);
  border-bottom: 1px solid var(--tech-glass-border);
  white-space: nowrap;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

.token-table th:first-child {
  padding-left: 16px;
}

.token-table .th-checkbox {
  width: 44px;
  text-align: center;
}

.token-table .th-tag {
  width: 85px;
  max-width: 85px;
}

.token-table .th-status {
  width: 85px;
}

.token-table .th-email {
  min-width: 150px;
}

.token-table .th-balance {
  width: 85px;
  text-align: center;
}

.token-table .th-dates {
  width: 140px;
  min-width: 140px;
}

.token-table .th-actions {
  width: 230px;
  text-align: center;
}

/* 表头多选框 */
.header-checkbox {
  display: inline-flex;
  cursor: pointer;
}

.header-checkbox .checkbox-inner {
  width: 18px;
  height: 18px;
  border-radius: 5px;
  border: 1.5px solid var(--border);
  background: var(--bg-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.header-checkbox .checkbox-inner:hover {
  border-color: var(--accent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
}

.header-checkbox .checkbox-inner.checked {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

.header-checkbox .checkbox-inner.indeterminate {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  box-shadow: 0 0 10px var(--tech-glow-primary);
}

/* 暗黑模式下样式已通过 CSS 变量自动适配 */

/* 响应式处理 */

/* 超大屏幕优化 */
@media (min-width: 1920px) {
  .token-grid {
    /* 超大屏幕: 每列最小 320px,允许更多列 */
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 20px;
  }
}

/* 中等屏幕 */
@media (max-width: 768px) {
  .token-list-page .page-header {
    padding: 16px;
  }

  .token-list-page .page-body {
    padding: 16px;
  }

  .header-actions {
    gap: 6px;
  }

  .token-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .list-toolbar {
    flex-wrap: wrap;
    gap: 8px;
  }

  .search-box {
    min-width: 150px;
  }
}

/* 批量导入对话框 */
.batch-import-overlay {
  padding: 20px;
  z-index: 3000;
}

.batch-import-modal {
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  max-width: 896px;
  /* 2xl: 56rem = 896px */
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}


/* Tab Navigation */
.batch-import-tabs {
  display: flex;
  gap: 0;
  padding: 0 24px;
  border-bottom: 1px solid var(--divider, #e1e5e9);
  background: var(--bg-surface-alt, #f9fafb);
}

.batch-import-tab {
  padding: 12px 20px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6b7280);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border-bottom: 2px solid transparent;
  position: relative;
}

.batch-import-tab:hover {
  color: var(--text, #374151);
  background: var(--bg-hover, #f3f4f6);
}

.batch-import-tab.active {
  color: var(--accent, #2563eb);
  border-bottom-color: var(--accent, #2563eb);
}

.batch-import-tab svg {
  flex-shrink: 0;
}

.batch-import-modal .modal-body {
  overflow-y: auto;
  flex: 1;
}

.tab-content {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.batch-import-modal .modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.batch-import-modal .btn {
  margin: 0;
}

.import-textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 8px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  background: var(--bg-surface, #ffffff);
  color: var(--text, #374151);
  transition: border-color 0.2s ease;
}

.import-textarea:focus {
  outline: none;
  border-color: var(--accent, #2563eb);
}

.import-textarea::placeholder {
  color: var(--text-muted, #9ca3af);
}

.format-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 20px;
}

.format-option-single {
  padding: 16px;
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 8px;
  background: var(--bg-surface-secondary, #f9fafb);
  margin-bottom: 16px;
}

.format-option {
  padding: 16px;
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 8px;
  background: var(--bg-surface-alt, #f9fafb);
}

.format-header {
  margin-bottom: 8px;
}

.format-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text, #374151);
}

.format-desc {
  font-size: 13px;
  color: var(--text-secondary, #6b7280);
  margin: 0 0 12px 0;
  line-height: 1.5;
}

.btn-fill-template {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--accent, #2563eb);
  border-radius: 6px;
  background: var(--accent-soft, #e8efff);
  color: var(--accent, #2563eb);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-fill-template:hover {
  background: var(--accent, #2563eb);
  color: var(--text-inverse, #ffffff);
}

.import-input-section {
  margin: 16px 0;
}

.import-errors {
  margin-top: 16px;
  padding: 12px;
  background: var(--state-danger-soft, #fee2e2);
  border: 1px solid var(--state-danger, #dc2626);
  border-radius: 8px;
}

.import-errors .error-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--state-danger, #dc2626);
  font-weight: 600;
  margin-bottom: 8px;
}

.import-errors .error-list {
  margin: 0;
  padding-left: 24px;
  color: var(--state-danger, #dc2626);
  font-size: 13px;
}

.import-errors .error-list li {
  margin: 4px 0;
}

.import-preview {
  margin-top: 16px;
  padding: 12px;
  background: var(--state-success-soft, #d1fae5);
  border: 1px solid var(--state-success, #10b981);
  border-radius: 8px;
}

.import-preview .preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--state-success, #10b981);
  font-weight: 600;
}

/* Session 动态输入框样式 */
.session-inputs-container {
  margin-bottom: 16px;
}

.session-input-item {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.session-input-number {
  flex-shrink: 0;
  width: 24px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary, #6b7280);
  text-align: right;
}

.session-input-field {
  flex: 1;
  height: 40px;
  padding: 0 12px;
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 6px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  background: var(--bg-surface, #ffffff);
  color: var(--text, #374151);
  transition: all 0.2s;
}

.session-input-field:focus {
  outline: none;
  border-color: var(--accent, #2563eb);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.session-input-field::placeholder {
  color: var(--text-muted, #9ca3af);
}

.session-input-delete {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  padding: 0;
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 6px;
  background: var(--bg-surface, #ffffff);
  color: var(--text-muted, #6b7280);
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.session-input-delete:hover:not(:disabled) {
  background: var(--state-danger-soft, #fee2e2);
  border-color: var(--state-danger, #dc2626);
  color: var(--state-danger, #dc2626);
}

.session-input-delete:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.add-more-btn {
  width: 100%;
  padding: 10px 16px;
  border: 2px dashed var(--divider, #e1e5e9);
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary, #6b7280);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-more-btn:hover {
  border-color: var(--accent, #2563eb);
  color: var(--accent, #2563eb);
  background: rgba(37, 99, 235, 0.05);
}

.add-more-btn svg {
  flex-shrink: 0;
}

/* 右键菜单 */
.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 20000;
}

.context-menu {
  position: fixed;
  background: var(--bg-surface, #ffffff);
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 20001;
  min-width: 180px;
  overflow: hidden;
}

.context-menu-header {
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted, #6b7280);
  background: var(--bg-hover, #f3f4f6);
  border-bottom: 1px solid var(--divider, #e1e5e9);
}

.context-menu-item {
  padding: 8px 16px;
  cursor: pointer;
  color: var(--text, #374151);
  transition: background 0.2s ease;
  font-size: 14px;
}

.context-menu-item:hover {
  background: var(--accent-light, #e0f2fe);
  color: var(--accent, #0ea5e9);
}

/* 右键菜单操作项样式 */
.context-menu-action {
  display: flex;
  align-items: center;
  font-weight: 500;
  color: var(--accent, #0ea5e9);
}

.context-menu-action:hover {
  background: var(--accent-light, #e0f2fe);
  color: var(--accent-dark, #0284c7);
}

.context-menu-action svg {
  flex-shrink: 0;
}

.context-menu-divider {
  height: 1px;
  background: var(--divider, #e1e5e9);
  margin: 4px 0;
}

.context-menu-custom {
  padding: 8px 12px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.custom-count-input {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 4px;
  font-size: 14px;
  color: var(--text, #374151);
  background: var(--bg-surface, #ffffff);
  outline: none;
  transition: border-color 0.2s ease;
}

.custom-count-input:focus {
  border-color: var(--accent, #0ea5e9);
}

.custom-count-input::placeholder {
  color: var(--text-muted, #9ca3af);
}

.btn-custom-fill {
  padding: 6px 12px;
  background: var(--accent, #0ea5e9);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
  white-space: nowrap;
}

.btn-custom-fill:hover {
  background: var(--accent-dark, #0284c7);
}

/* 批量删除对话框 */
.batch-delete-overlay {
  padding: 20px;
  z-index: 3000;
}

.batch-delete-modal {
  max-width: 500px;
  width: 100%;
  overflow: hidden;
}


.batch-delete-modal .modal-body {
  padding: 24px;
}


.delete-stats {
  background: var(--bg-surface-secondary, #f9fafb);
  border: 1px solid var(--divider, #e1e5e9);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.stat-item:not(:last-child) {
  border-bottom: 1px solid var(--divider, #e1e5e9);
}

.stat-item.total {
  font-weight: 600;
  color: var(--text, #374151);
}

.stat-label {
  color: var(--text-secondary, #6b7280);
  font-size: 14px;
}

.stat-value {
  color: var(--text, #374151);
  font-size: 14px;
  font-weight: 500;
}


.batch-delete-modal .modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
.batch-delete-modal .btn {
  margin: 0;
}

@media (max-width: 480px) {
  .batch-import-overlay {
    padding: 10px;
  }

  .batch-delete-overlay {
    padding: 10px;
  }

  .empty-state {
    padding: 20px 10px;
  }

  .empty-state h3 {
    font-size: 1.125rem;
  }

  .sync-actions {
    flex-direction: column;
  }

}

/* 状态统计栏样式 */
.status-stats-bar {
  margin-bottom: 16px;
  padding: 0 16px;
}

.status-stats-container {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}

.status-stat-card {
  flex: 1;
  min-width: 120px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-page-soft, #f9fafb);
  border: 2px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  user-select: none;
}

.status-stat-card:hover {
  background: var(--bg-hover, #f3f4f6);
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.status-stat-card.selected {
  border-color: var(--accent, #3b82f6);
  background: var(--accent-soft, #eff6ff);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.2);
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  flex-shrink: 0;
}

.stat-icon.all {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
  color: white;
}

.stat-icon.active {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
}

.stat-icon.suspended {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
}

.stat-icon.expired {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  color: white;
}

.stat-icon.invalid {
  background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
  color: white;
}

.stat-icon.error {
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
  color: white;
}

.stat-icon.other {
  background: linear-gradient(135deg, #6b7280 0%, #4b5563 100%);
  color: white;
}

.stat-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.stat-label {
  font-size: 12px;
  color: var(--text-muted, #6b7280);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text, #1f2937);
  line-height: 1;
}

.filter-hint {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--accent-soft, #eff6ff);
  border: 1px solid var(--accent, #3b82f6);
  border-radius: 6px;
  font-size: 13px;
  color: var(--accent, #3b82f6);
  font-weight: 500;
}

.clear-filter-btn {
  padding: 4px 12px;
  background: var(--accent, #3b82f6);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.clear-filter-btn:hover {
  background: var(--accent-hover, #2563eb);
  transform: scale(1.05);
}

.list-header {
  margin-bottom: 16px;
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

/* 选择模式提示 */
.selection-info {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  background: var(--accent-soft, rgba(37, 99, 235, 0.1));
  border: 1px solid var(--accent, #2563eb);
  border-radius: 6px;
  color: var(--accent, #2563eb);
  font-size: 14px;
  font-weight: 500;
  margin-left: auto;
}



/* 无搜索结果样式 */
.no-search-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary, #6b7280);
}

.no-search-results p {
  margin-top: 16px;
  font-size: 14px;
}

/* 排序下拉菜单 - 科技风 */
.sort-dropdown {
  position: relative;
}

/* 下拉菜单容器 - 科技风 */
.sort-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 200px;
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

/* 下拉菜单选项 - 科技风 */
.sort-option {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  font-family: inherit;
  border-radius: 8px;
}

.sort-option:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.sort-option.active {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  color: var(--accent);
}

.sort-option span {
  flex: 1;
  font-size: 14px;
}

.sort-option .arrow-down,
.sort-option .arrow-up {
  opacity: 0.7;
  transition: opacity 0.2s;
}

.sort-option:hover .arrow-down,
.sort-option:hover .arrow-up {
  opacity: 1;
}

.sort-option .check-icon {
  color: var(--accent);
}

/* 分隔线 - 科技风 */
.sort-divider {
  height: 1px;
  background: var(--tech-glass-border);
  margin: 4px 6px;
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

.status-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  text-align: left;
  font-size: 14px;
  font-family: inherit;
  border-radius: 8px;
}

.status-option:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.status-option.active {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  color: var(--accent);
}

.status-option .status-icon {
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  opacity: 0.8;
}

.status-option:hover .status-icon {
  opacity: 1;
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

/* 邮箱后缀筛选下拉菜单 - 科技风 */
.email-suffix-dropdown {
  position: relative;
}

/* 邮箱后缀下拉菜单容器 - 科技风 */
.email-suffix-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 220px;
  max-width: 300px;
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--tech-card-bg));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  z-index: 1100;
  overflow: hidden;
}

/* 邮箱后缀菜单顶部操作栏 - 科技风 */
.suffix-menu-header {
  display: flex;
  gap: 4px;
  padding: 6px;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-bottom: 1px solid var(--tech-glass-border);
}

.suffix-option-all {
  flex: 1;
  min-width: 0;
}

/* 提取按钮样式 - 科技风 */
.suffix-extract-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--accent);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.suffix-extract-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
}

.suffix-extract-btn:active:not(:disabled) {
  transform: scale(0.98);
}

.suffix-extract-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  color: var(--text-muted);
}

/* 邮箱后缀列表容器 - 科技风 */
.suffix-list {
  max-height: 300px;
  overflow-y: auto;
  padding: 6px;
}

/* 邮箱后缀选项 - 科技风 */
.suffix-option {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  font-family: inherit;
  border-radius: 8px;
}

.suffix-option:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.suffix-option.active {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  color: var(--accent);
}

.suffix-option .suffix-text {
  flex: 1;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.suffix-option .suffix-count {
  font-size: 12px;
  color: var(--text-muted);
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 600;
  min-width: 24px;
  text-align: center;
}

.suffix-option.active .suffix-count {
  background: color-mix(in srgb, var(--accent) 25%, transparent);
  color: var(--accent);
}

.suffix-option .check-icon {
  color: var(--accent);
}

/* 无邮箱后缀提示 - 科技风 */
.no-suffix-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  color: var(--text-muted);
}

.no-suffix-hint p {
  margin-top: 12px;
  font-size: 14px;
  color: var(--text-muted);
}

/* 标签筛选下拉菜单 - 科技风 */
.tag-filter-dropdown {
  position: relative;
}

.tag-filter-btn {
  position: relative;
}

.tag-count-badge {
  position: absolute;
  top: -6px;
  right: -6px;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  background: var(--accent);
  color: white;
  font-size: 11px;
  font-weight: 600;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px var(--tech-glow-primary);
}

/* 标签筛选下拉菜单容器 - 科技风 */
.tag-filter-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 220px;
  max-width: 300px;
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--tech-card-bg));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  z-index: 1100;
  overflow: hidden;
}

/* 标签菜单顶部操作栏 - 科技风 */
.tag-menu-header {
  display: flex;
  gap: 4px;
  padding: 6px;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
  border-bottom: 1px solid var(--tech-glass-border);
}

.tag-option-all {
  flex: 1;
  min-width: 0;
}

/* 筛选模式切换按钮 - 科技风 */
.tag-mode-toggle-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--text);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.tag-mode-toggle-btn:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}

/* 提取按钮样式 - 科技风 */
.tag-extract-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--accent);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.tag-extract-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  box-shadow: 0 0 8px var(--tech-glow-primary);
}

.tag-extract-btn:active:not(:disabled) {
  transform: scale(0.98);
}

.tag-extract-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  color: var(--text-muted);
}

/* 标签列表容器 - 科技风 */
.tag-list {
  max-height: 300px;
  overflow-y: auto;
  padding: 6px;
}

/* 标签选项 - 科技风 */
.tag-option {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  font-family: inherit;
  border-radius: 8px;
}

.tag-option:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.tag-option.active {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  color: var(--accent);
}

.tag-option .tag-text {
  flex: 1;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-option .check-icon {
  color: var(--accent);
}

/* 无标签提示 - 科技风 */
.no-tag-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  color: var(--text-muted);
}

.no-tag-hint p {
  margin-top: 12px;
  font-size: 14px;
  color: var(--text-muted);
}

/* 下拉菜单动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* 分页控制 */
.pagination-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-page-soft, #f9fafb);
  border-radius: 8px;
  margin-bottom: 16px;
}

.pagination-info {
  font-size: 14px;
  color: var(--text, #374151);
}

.pagination-size {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pagination-size label {
  font-size: 14px;
  color: var(--text, #374151);
}

.pagination-size select {
  padding: 4px 8px;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 4px;
  background: var(--bg-page, #ffffff);
  color: var(--text, #374151);
  cursor: pointer;
}

/* 分页导航 */
.pagination-nav {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  padding: 16px;
  margin-top: 16px;
}

.pagination-btn {
  padding: 8px 16px;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 4px;
  background: var(--bg-page, #ffffff);
  color: var(--text, #374151);
  cursor: pointer;
  transition: all 0.2s;
}

.pagination-btn:hover:not(:disabled) {
  background: var(--bg-page-soft, #f9fafb);
  border-color: var(--accent, #2563eb);
}

.pagination-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination-pages {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text, #374151);
}

.page-current {
  font-weight: 600;
  color: var(--accent, #2563eb);
}

/* 工具栏内联分页信息和选择器 */
.pagination-combined {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 36px;
  padding: 0 12px;
  background: var(--bg-page, #ffffff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 6px;
}

.pagination-info-text {
  font-size: 13px;
  color: var(--text-secondary, #6b7280);
  white-space: nowrap;
}

.pagination-size-select {
  height: 28px;
  font-size: 13px;
  background: var(--bg-page, #ffffff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 4px;
  color: var(--text, #374151);
}

.pagination-size-select:hover {
  border-color: var(--accent, #3b82f6);
}

.pagination-size-select:focus {
  outline: none;
  border-color: var(--accent, #3b82f6);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.2);
}

/* Header layout */
.token-list-page .page-header {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border, #e5e7eb);
  background: var(--bg-surface-alt, #f9fafb);
}

.token-list-page .page-header > .status-badge {
  flex-shrink: 0;
}

.token-list-page .page-header > .header-actions {
  flex-shrink: 0;
  align-self: center;
}

.token-list-page .page-header .btn {
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

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.header-title h2 {
  margin: 0;
  color: var(--text-strong, #111827);
  font-size: 1.25rem;
  font-weight: 600;
  line-height: 1.2;
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



@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
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
  background: none;
  border: none;
  color: var(--text-secondary, #6b7280);
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.btn-text:hover {
  background: var(--bg-surface-soft, #f3f4f6);
  color: var(--accent, #3b82f6);
}

.batch-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 工具栏滑入动画 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}

/* Page Mode Styles */
.token-list-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.page-container {
  position: static;
  background: transparent;
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.token-list-page .page-content {
  position: static;
  transform: none;
  max-width: none;
  max-height: none;
  width: 100%;
  height: 100%;
  margin: 0;
  border-radius: 0;
  box-shadow: none;
  background: transparent;
  border: none;
}

.token-list-page .page-container {
  position: static;
  background: transparent;
}

.token-list-page .page-header {
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  padding: 16px 20px;
  border-radius: 0;
}

.token-list-page .page-body {
  background: transparent;
  padding: 16px 24px;
}
</style>
