<template>
  <div class="token-list-modal">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <div class="header-title">
            <h2>{{ $t('tokenList.title') }}</h2>
            <div :class="['status-badge', storageStatusClass]">
              <span :class="['status-dot', storageStatusClass]"></span>
              <span class="status-text">{{ storageStatusText }}</span>
            </div>
          </div>
          <div class="header-actions">
            <!-- 数据库配置按钮 -->
            <button @click="showDatabaseConfig = true" class="btn info small">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 3C7.58 3 4 4.79 4 7s3.58 4 8 4 8-1.79 8-4-3.58-4-8-4zM4 9v3c0 2.21 3.58 4 8 4s8-1.79 8-4V9c0 2.21-3.58 4-8 4s-8-1.79-8-4zM4 16v3c0 2.21 3.58 4 8 4s8-1.79 8-4v-3c0 2.21-3.58 4-8 4s-8-1.79-8-4z"/>
              </svg>
              {{ $t('tokenList.databaseConfig') }}
            </button>
            <!-- 同步按钮 - 仅双向存储模式显示 -->
            <button
              v-if="isDatabaseAvailable"
              @click="handleBidirectionalSync"
              class="btn info small"
              :disabled="isSyncing"
              :title="$t('tokenList.syncTooltip')"
            >
              <svg v-if="!isSyncing" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
              </svg>
              {{ isSyncing ? $t('tokenList.syncing') : $t('tokenList.sync') }}
            </button>
            <button @click="handleAddToken" class="btn primary small">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
              {{ $t('tokenList.addToken') }}
            </button>
            <button @click="handleRefresh" class="btn secondary small" :disabled="isRefreshing">
              <svg v-if="!isRefreshing" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              {{ isRefreshing ? $t('loading.refreshing') : $t('tokenList.refresh') }}
            </button>
            <button class="close-btn" @click="handleClose">×</button>
          </div>
        </div>
        
        <div class="modal-body">
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
            <button class="batch-import-btn-empty" @click="showBatchImportConfirm" :title="$t('tokenList.batchImport')">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z" />
              </svg>
              {{ $t('tokenList.batchImport') }}
            </button>
          </div>

          <!-- Token List -->
          <div v-else class="token-list">
            <div class="list-header">
              <div class="list-title-section">
                <h3>{{
                  searchQuery.trim()
                    ? $t('tokenList.searchResults', { count: filteredTokens.length })
                    : $t('tokenList.listTitle', { count: tokens.length })
                }}</h3>
                <button :class="['sort-btn', { active: sortType === 'time' }]" @click="toggleSort('time')"
                  :title="$t('tokenList.sortByTime')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="sort-icon">
                    <path
                      d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM7 10h5v5H7z" />
                  </svg>
                  <svg v-if="sortType === 'time'" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"
                    :class="['arrow-icon', sortOrder === 'desc' ? 'arrow-down' : 'arrow-up']">
                    <path d="M7 14l5-5 5 5z" />
                  </svg>
                </button>
                <button :class="['sort-btn', { active: sortType === 'balance' }]" @click="toggleSort('balance')"
                  :title="$t('tokenList.sortByBalance')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="sort-icon">
                    <path
                      d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z" />
                  </svg>
                  <svg v-if="sortType === 'balance'" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"
                    :class="['arrow-icon', sortOrder === 'desc' ? 'arrow-down' : 'arrow-up']">
                    <path d="M7 14l5-5 5 5z" />
                  </svg>
                </button>
                <button class="open-folder-btn" @click="openDataFolder" :title="$t('bookmarkManager.openDataFolder')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path
                      d="M10 4H4c-1.11 0-2 .89-2 2v12c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V8c0-1.11-.89-2-2-2h-8l-2-2z" />
                  </svg>
                </button>
                <button class="batch-delete-btn" @click="showBatchDeleteConfirm" :disabled="deletableTokensCount === 0"
                  :title="deletableTokensCount === 0 ? $t('tokenList.noDeletableTokens') : $t('tokenList.batchDelete')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
                  </svg>
                </button>
                <button class="batch-import-btn" @click="showBatchImportConfirm" :title="$t('tokenList.batchImport')">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path
                      d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2z" />
                  </svg>
                </button>
                <div class="search-box">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="search-icon">
                    <path
                      d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
                  </svg>
                  <input type="text" v-model="searchQuery" :placeholder="$t('tokenList.searchPlaceholder')"
                    class="search-input" />
                  <button v-if="searchQuery.trim()" @click="searchQuery = ''" class="clear-search-btn" title="清空搜索">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                      <path
                        d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                    </svg>
                  </button>
                </div>
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

            <div v-else class="token-grid">
              <TokenCard v-for="token in filteredTokens" :key="token.id" :ref="el => setTokenCardRef(el, token.id)"
                :token="token" :is-batch-checking="isRefreshing" :is-highlighted="highlightedTokenId === token.id"
                @delete="deleteToken" @edit="handleEditToken" @token-updated="handleTokenUpdated" />
            </div>


          </div>
        </div>
      </div>
    </div>

    <!-- Database Config Modal -->
    <DatabaseConfig v-if="showDatabaseConfig" @close="showDatabaseConfig = false"
      @config-saved="handleDatabaseConfigSaved" @config-deleted="handleDatabaseConfigDeleted" />

    <!-- Token Form Modal -->
    <TokenForm v-if="showTokenFormModal" :token="editingToken" @close="closeTokenForm" @success="handleTokenFormSuccess"
      @update-token="handleUpdateToken" @add-token="handleAddTokenFromForm"
      @auto-import-completed="handleAutoImportCompleted" @manual-import-completed="handleManualImportCompleted" />

    <!-- Batch Import Dialog -->
    <Teleport to="body">
      <Transition name="modal" appear>
        <div v-if="showBatchImportDialog" class="batch-import-overlay" @click="showBatchImportDialog = false">
          <div class="batch-import-dialog" @click.stop>
            <div class="dialog-header">
              <h3>{{ $t('tokenList.batchImportTitle') }}</h3>
              <button @click="showBatchImportDialog = false" class="dialog-close">
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

            <div class="dialog-body">
              <!-- Session Tab Content -->
              <div v-if="batchImportTab === 'session'" class="tab-content">
                <p class="dialog-message">{{ $t('tokenList.sessionImportMessage') }}</p>

                <!-- Session 动态输入框列表 -->
                <div class="session-inputs-container">
                  <div v-for="(input, index) in sessionInputs" :key="input.id" class="session-input-item">
                    <span class="session-input-number">{{ index + 1 }}.</span>
                    <input v-model="input.value" type="text" :placeholder="$t('tokenList.sessionInputPlaceholder')"
                      class="session-input-field" />
                    <button @click="removeSessionInput(input.id)" class="session-input-delete"
                      :title="$t('tokenList.deleteInput')" :disabled="sessionInputs.length <= 1">
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
                <p class="dialog-message">{{ $t('tokenList.tokenImportMessage') }}</p>

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

            <div class="dialog-footer">
              <button @click="showBatchImportDialog = false" class="btn-cancel">
                {{ $t('tokenList.cancel') }}
              </button>
              <button @click="executeBatchImport" class="btn-confirm"
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
        <div v-if="showBatchDeleteDialog" class="batch-delete-overlay" @click="showBatchDeleteDialog = false">
          <div class="batch-delete-dialog" @click.stop>
            <div class="dialog-header">
              <h3>{{ $t('tokenList.batchDeleteConfirm') }}</h3>
              <button @click="showBatchDeleteDialog = false" class="dialog-close">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path
                    d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
                </svg>
              </button>
            </div>
            <div class="dialog-body">
              <p class="dialog-message">{{ $t('tokenList.batchDeleteMessage') }}</p>
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
              <p class="dialog-warning">{{ $t('tokenList.cannotUndo') }}</p>
            </div>
            <div class="dialog-footer">
              <button @click="executeBatchDelete" class="btn btn-danger" :disabled="isDeleting">
                {{ isDeleting ? $t('tokenList.deleting') : $t('tokenList.confirmDelete') }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted, onUnmounted, computed, readonly, watch } from 'vue'
import { watchDebounced } from '@vueuse/core'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import TokenCard from './TokenCard.vue'
import DatabaseConfig from './DatabaseConfig.vue'
import TokenForm from './TokenForm.vue'

const { t } = useI18n()

// Props - 移除存储状态相关的props，TokenList自主管理
const props = defineProps({
  // 如果将来需要其他props可以在这里添加
})

// 内部状态管理 - TokenList直接管理tokens和存储状态
const tokens = ref([])
const isLoading = ref(false)
const isDatabaseAvailable = ref(false)
const isStorageInitializing = ref(false)

// 初始化就绪标记
const isReady = ref(false)

// 同步状态标记 - 用于防止同步时触发自动保存
const isSyncing = ref(false)
const isLoadingFromSync = ref(false)

// 同步需求标记 - 标识本地有未同步到数据库的更改
const isSyncNeeded = ref(false)

// 存储状态检查定时器
let storageCheckTimer = null

// 排序状态管理
const sortType = ref('time') // 'time' = 按时间排序, 'balance' = 按余额排序
const sortOrder = ref('desc') // 'desc' = 最新优先/余额从多到少, 'asc' = 最旧优先/余额从少到多

// 搜索状态管理
const searchQuery = ref('')

// 高亮状态管理
const highlightedTokenId = ref(null)
let highlightTimer = null

const DEFAULT_TAG_COLOR = '#f97316'

// 批量删除状态
const showBatchDeleteDialog = ref(false)
const isDeleting = ref(false)

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
    }
    return 0
  })
})

// 过滤后的tokens计算属性（搜索 + 排序）
const filteredTokens = computed(() => {
  if (!searchQuery.value.trim()) {
    return sortedTokens.value
  }

  const query = searchQuery.value.toLowerCase().trim()
  return sortedTokens.value.filter(token => {
    return (
      token.access_token?.toLowerCase().includes(query) ||
      token.email_note?.toLowerCase().includes(query) ||
      token.auth_session?.toLowerCase().includes(query)
    )
  })
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
          portalUrl: result.user_info?.portal_url || null,
          emailNote: result.user_info ? extractEmail(result.user_info) : null,  // 智能提取email字段
          authSession: session,
          suspensions: result.user_info?.suspensions || null
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
            portalUrl: result.user_info?.portal_url || null,
            emailNote: result.user_info ? extractEmail(result.user_info) : null,  // 智能提取email字段
            authSession: item.auth_session,
            suspensions: result.user_info?.suspensions || null
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

// 执行批量删除
const executeBatchDelete = async () => {
  isDeleting.value = true

  try {
    // 获取要删除的 tokens
    const tokensToDelete = tokens.value.filter(token =>
      token.ban_status === 'SUSPENDED' || token.ban_status === 'EXPIRED'
    )

    // 并行删除所有 tokens
    const deletePromises = tokensToDelete.map(token =>
      invoke('delete_token', { tokenId: token.id })
        .then(() => {
          // 删除成功,从本地列表移除
          const index = tokens.value.findIndex(t => t.id === token.id)
          if (index !== -1) {
            tokens.value.splice(index, 1)
          }
          return { success: true, id: token.id }
        })
        .catch(error => {
          console.error(`Failed to delete token ${token.id}:`, error)
          return { success: false, id: token.id, error }
        })
    )

    // 等待所有删除操作完成
    const results = await Promise.allSettled(deletePromises)

    // 统计成功和失败的数量
    const successCount = results.filter(r =>
      r.status === 'fulfilled' && r.value.success
    ).length
    const failedCount = tokensToDelete.length - successCount

    // 关闭对话框
    showBatchDeleteDialog.value = false

    // 显示结果消息
    if (failedCount === 0) {
      console.log(`Successfully deleted ${successCount} tokens`)
    } else {
      console.warn(`Deleted ${successCount} tokens, ${failedCount} failed`)
    }
  } catch (error) {
    console.error('Batch delete failed:', error)
  } finally {
    isDeleting.value = false
  }
}

const emit = defineEmits(['close'])

// Additional state for new components
const showDatabaseConfig = ref(false)
const isSaving = ref(false)
const isRefreshing = ref(false)

// TokenForm state management
const showTokenFormModal = ref(false)
const editingToken = ref(null)

// Token card refs for accessing child methods
const tokenCardRefs = ref({})

// Computed properties for storage status display
const storageStatusText = computed(() => {
  if (isStorageInitializing.value) {
    return t('storage.initializing')
  }
  if (isDatabaseAvailable.value) {
    return isSyncNeeded.value
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
  // 如果是双向存储且未同步，显示警告样式
  if (isDatabaseAvailable.value && isSyncNeeded.value) {
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
const handleTokenUpdated = () => {
  // Token 更新时不再设置未保存状态，关闭时会自动保存
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
      auth_session: token.auth_session || null
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
          expiry_date: result.portal_info.expiry_date,
          can_still_use: result.portal_info.can_still_use
        }

        if (!isEqual(token.portal_info, newPortalInfo)) {
          token.portal_info = newPortalInfo
          hasChanges = true
          console.log(`Updated token ${token.id} portal info:`, token.portal_info)
        }
      } else if (result.portal_error) {
        console.warn(`Failed to get portal info for token ${token.id}:`, result.portal_error)
      }

      // 只有在有实际变化时才更新时间戳
      if (hasChanges) {
        token.updated_at = new Date().toISOString()
        console.log(`Updated token ${token.id} status to: ${statusResult.status}`)
        anyChanges = true
      } else {
        console.log(`No changes for token ${token.id}, skipping update`)
      }
    }
  })

  return anyChanges
}

const loadTokens = async (showSuccessMessage = false) => {
  isLoading.value = true
  try {
    const jsonString = await invoke('load_tokens_json')
    const parsedTokens = JSON.parse(jsonString)

    // 确保是数组
    if (Array.isArray(parsedTokens)) {
      // 使用展开运算符创建新数组，确保触发响应式更新
      tokens.value = [...parsedTokens]
    } else {
      tokens.value = []
    }

    if (showSuccessMessage) {
      window.$notify.success(t('messages.tokenLoadSuccess'))
    }
  } catch (error) {
    window.$notify.error(`${t('messages.tokenLoadFailed')}: ${error}`)
    tokens.value = []
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

// 删除token
const deleteToken = (tokenId) => {
  const tokenIndex = tokens.value.findIndex(token => token.id === tokenId)
  if (tokenIndex === -1) {
    window.$notify.error(t('messages.tokenNotFound'))
    return
  }

  // 从内存中删除
  tokens.value = tokens.value.filter(token => token.id !== tokenId)
  window.$notify.success(t('messages.tokenDeleted'))

  // 异步删除后端数据（不阻塞UI）
  invoke('delete_token', { tokenId }).catch(error => {
    console.log('Backend delete failed:', error)
  })
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

const handleUpdateToken = (updatedTokenData) => {
  const index = tokens.value.findIndex(token => token.id === updatedTokenData.id)
  if (index !== -1) {
    const tagName = updatedTokenData.tagName ? updatedTokenData.tagName.trim() : ''
    const tagColor = updatedTokenData.tagColor || DEFAULT_TAG_COLOR

    // Update the token in the list
    tokens.value[index] = {
      ...tokens.value[index],
      tenant_url: updatedTokenData.tenantUrl,
      access_token: updatedTokenData.accessToken,
      portal_url: updatedTokenData.portalUrl || null,
      email_note: updatedTokenData.emailNote || null,
      tag_name: tagName || null,
      tag_color: tagName ? tagColor : null,
      updated_at: new Date().toISOString()  // 更新 updated_at 时间戳
    }
  }
}

const handleAddTokenFromForm = (tokenData) => {
  const result = addToken(tokenData)
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
const addToken = (tokenData) => {
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

  const newToken = {
    id: crypto.randomUUID(),
    tenant_url: tokenData.tenantUrl,
    access_token: tokenData.accessToken,
    created_at: now,
    updated_at: now,  // 添加 updated_at 字段
    portal_url: tokenData.portalUrl || null,
    ban_status: null,
    portal_info: null,
    email_note: tokenData.emailNote || null,
    tag_name: tagName || null,
    tag_color: tagName ? tagColor : null,
    auth_session: tokenData.authSession || null,  // 添加 auth_session 字段
    suspensions: tokenData.suspensions || null,  // 添加 suspensions 字段
    skip_check: false,  // 默认不跳过检测
    balance_color_mode: null  // 默认为 null，将使用绿色
  }

  tokens.value.push(newToken)
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

// 处理关闭事件
const handleClose = () => {
  // 防抖自动保存会处理保存,直接关闭即可
  emit('close')
}

// 处理刷新事件 - 刷新账号状态
const handleRefresh = async () => {
  if (isRefreshing.value) return
  isRefreshing.value = true

  try {
    window.$notify.info(t('messages.refreshingTokenStatus'))

    // 加载最新的 tokens
    await loadTokens(false)
    await nextTick()

    // 检查账号状态
    if (tokens.value.length > 0) {
      const result = await checkAllAccountStatus()

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



const handleDatabaseConfigSaved = async () => {
  window.$notify.success(t('messages.databaseConfigSaved'))
  // 重新获取存储状态
  await getStorageStatus()
  // 自动执行刷新操作
  await handleRefresh()
}

const handleDatabaseConfigDeleted = async () => {
  window.$notify.info(t('messages.databaseConfigDeleted'))
  // 重新获取存储状态
  await getStorageStatus()
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

// 双向同步方法（手动触发）
const handleBidirectionalSync = async () => {
  if (isSyncing.value) return
  if (!isDatabaseAvailable.value) {
    window.$notify.warning(t('messages.databaseNotAvailable'))
    return
  }

  isSyncing.value = true
  try {
    window.$notify.info(t('messages.syncingData'))

    // 执行双向同步
    const tokensJson = JSON.stringify(tokens.value)
    await invoke('bidirectional_sync_tokens_with_data', { tokensJson })

    // 同步后需要刷新，但要避免触发 watch
    isLoadingFromSync.value = true
    await loadTokens(false)

    // 延迟重置 isLoadingFromSync，确保 watchDebounced 的 debounce timer 已经被清除
    // watchDebounced 的 debounce 时间是 2000ms，这里等待 2100ms 确保安全
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

// 组件挂载时自动加载tokens和存储状态
onMounted(async () => {
  // 加载默认输入框数量配置
  defaultInputCount.value = loadDefaultInputCount()

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
})

// 组件卸载时清理定时器
onUnmounted(() => {
  if (storageCheckTimer) {
    clearInterval(storageCheckTimer)
    storageCheckTimer = null
  }
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
.token-list-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2000;
}

.modal-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-content {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  width: 95vw;
  /* 使用视口宽度的 95%,自适应屏幕大小 */
  max-width: none;
  /* 移除固定最大宽度限制 */
  height: 90vh;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
}

/* 移除旧的 modal-header 样式，使用新的样式 */

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.close-btn:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

.modal-body {
  padding: 24px;
  flex: 1;
  overflow-y: auto;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
}

.empty-icon {
  color: var(--color-border-strong, #d1d5db);
  margin-bottom: 16px;
}

.empty-state h3 {
  color: var(--color-text-primary, #374151);
  margin: 0 0 8px 0;
  font-size: 1.25rem;
}

.empty-state p {
  color: var(--color-text-muted, #6b7280);
  margin: 0 0 24px 0;
}

/* 空状态下的批量导入按钮 */
.batch-import-btn-empty {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: var(--color-primary, #2563eb);
  color: #ffffff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 2px 4px rgba(37, 99, 235, 0.2);
}

.batch-import-btn-empty:hover {
  background: var(--color-primary-hover, #1d4ed8);
  box-shadow: 0 4px 8px rgba(37, 99, 235, 0.3);
  transform: translateY(-1px);
}

.batch-import-btn-empty:active {
  transform: translateY(0);
  box-shadow: 0 2px 4px rgba(37, 99, 235, 0.2);
}

.batch-import-btn-empty svg {
  flex-shrink: 0;
}

.loading-state {
  text-align: center;
  padding: 40px 20px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border, #e5e7eb);
  border-top: 3px solid var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
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
  color: var(--color-text-muted, #6b7280);
  margin: 0;
}

.token-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(330px, 1fr));
  gap: 16px;
  padding: 4px;
}

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
  .modal-content {
    margin: 10px;
    width: calc(100vw - 20px);
  }

  .modal-header {
    padding: 16px;
  }

  .modal-body {
    padding: 16px;
  }

  .header-actions {
    gap: 6px;
  }

  .token-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .list-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .list-title-section {
    gap: 8px;
  }

  .list-header h3 {
    font-size: 1rem;
  }

  .sort-btn {
    padding: 4px 6px;
  }

  .sort-icon {
    width: 14px;
    height: 14px;
  }

  .arrow-icon {
    width: 10px;
    height: 10px;
  }
}

/* 打开文件夹按钮 - 与排序按钮样式一致 */
.open-folder-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 6px;
  color: var(--color-text-secondary, #6b7280);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0;
}

.open-folder-btn:hover {
  background: var(--color-surface-hover, #e9ecef);
  color: var(--color-primary, #667eea);
  border-color: var(--color-primary, #667eea);
}

.open-folder-btn svg {
  flex-shrink: 0;
}

/* 批量删除按钮 - 与排序按钮样式一致 */
.batch-delete-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-secondary, #6b7280);
  border: 1px solid var(--color-border, #d1d5db);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0;
}

.batch-delete-btn:hover:not(:disabled) {
  background: var(--color-surface-hover, #e9ecef);
  color: var(--color-danger, #dc2626);
  border-color: var(--color-danger, #dc2626);
}

.batch-delete-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.batch-delete-btn svg {
  flex-shrink: 0;
}

/* 批量导入按钮 */
.batch-import-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-secondary, #6b7280);
  border: 1px solid var(--color-border, #d1d5db);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0;
}

.batch-import-btn:hover {
  background: var(--color-surface-hover, #e9ecef);
  color: var(--color-primary, #2563eb);
  border-color: var(--color-primary, #2563eb);
}

.batch-import-btn svg {
  flex-shrink: 0;
}

/* 批量导入对话框 */
.batch-import-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
}

.batch-import-dialog {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  max-width: 896px;
  /* 2xl: 56rem = 896px */
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.batch-import-dialog .dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.batch-import-dialog .dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.batch-import-dialog .dialog-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  border-radius: 4px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.batch-import-dialog .dialog-close:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

/* Tab Navigation */
.batch-import-tabs {
  display: flex;
  gap: 0;
  padding: 0 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
  background: var(--color-surface-alt, #f9fafb);
}

.batch-import-tab {
  padding: 12px 20px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary, #6b7280);
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
  color: var(--color-text-primary, #374151);
  background: var(--color-surface-hover, #f3f4f6);
}

.batch-import-tab.active {
  color: var(--color-primary, #2563eb);
  border-bottom-color: var(--color-primary, #2563eb);
}

.batch-import-tab svg {
  flex-shrink: 0;
}

.batch-import-dialog .dialog-body {
  padding: 24px;
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

.batch-import-dialog .dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--color-divider, #e1e5e9);
  background: var(--color-surface, #ffffff);
}

.batch-import-dialog .btn-cancel {
  padding: 8px 16px;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 6px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.batch-import-dialog .btn-cancel:hover {
  background: var(--color-surface-hover, #f3f4f6);
  border-color: var(--color-border-hover, #9ca3af);
}

.batch-import-dialog .btn-confirm {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  background: var(--color-primary, #2563eb);
  color: #ffffff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.batch-import-dialog .btn-confirm:hover:not(:disabled) {
  background: var(--color-primary-hover, #1d4ed8);
}

.batch-import-dialog .btn-confirm:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.import-textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
  transition: border-color 0.2s ease;
}

.import-textarea:focus {
  outline: none;
  border-color: var(--color-primary, #2563eb);
}

.import-textarea::placeholder {
  color: var(--color-text-muted, #9ca3af);
}

.format-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-bottom: 20px;
}

.format-option-single {
  padding: 16px;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  background: var(--color-surface-secondary, #f9fafb);
  margin-bottom: 16px;
}

.format-option {
  padding: 16px;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  background: var(--color-surface-alt, #f9fafb);
}

.format-header {
  margin-bottom: 8px;
}

.format-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.format-desc {
  font-size: 13px;
  color: var(--color-text-secondary, #6b7280);
  margin: 0 0 12px 0;
  line-height: 1.5;
}

.btn-fill-template {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-primary, #2563eb);
  border-radius: 6px;
  background: var(--color-surface, #ffffff);
  color: var(--color-primary, #2563eb);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-fill-template:hover {
  background: var(--color-primary, #2563eb);
  color: #ffffff;
}

.import-input-section {
  margin: 16px 0;
}

.import-errors {
  margin-top: 16px;
  padding: 12px;
  background: var(--color-danger-light, #fee2e2);
  border: 1px solid var(--color-danger, #dc2626);
  border-radius: 8px;
}

.import-errors .error-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-danger, #dc2626);
  font-weight: 600;
  margin-bottom: 8px;
}

.import-errors .error-list {
  margin: 0;
  padding-left: 24px;
  color: var(--color-danger, #dc2626);
  font-size: 13px;
}

.import-errors .error-list li {
  margin: 4px 0;
}

.import-preview {
  margin-top: 16px;
  padding: 12px;
  background: var(--color-success-light, #d1fae5);
  border: 1px solid var(--color-success, #10b981);
  border-radius: 8px;
}

.import-preview .preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-success, #10b981);
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
  color: var(--color-text-secondary, #6b7280);
  text-align: right;
}

.session-input-field {
  flex: 1;
  height: 40px;
  padding: 0 12px;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 6px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #374151);
  transition: all 0.2s;
}

.session-input-field:focus {
  outline: none;
  border-color: var(--color-primary, #2563eb);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.session-input-field::placeholder {
  color: var(--color-text-muted, #9ca3af);
}

.session-input-delete {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  padding: 0;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 6px;
  background: var(--color-surface, #ffffff);
  color: var(--color-text-muted, #6b7280);
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.session-input-delete:hover:not(:disabled) {
  background: var(--color-danger-light, #fee2e2);
  border-color: var(--color-danger, #dc2626);
  color: var(--color-danger, #dc2626);
}

.session-input-delete:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.add-more-btn {
  width: 100%;
  padding: 10px 16px;
  border: 2px dashed var(--color-divider, #e1e5e9);
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary, #6b7280);
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
  border-color: var(--color-primary, #2563eb);
  color: var(--color-primary, #2563eb);
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
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e1e5e9);
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
  color: var(--color-text-muted, #6b7280);
  background: var(--color-surface-hover, #f3f4f6);
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.context-menu-item {
  padding: 8px 16px;
  cursor: pointer;
  color: var(--color-text-primary, #374151);
  transition: background 0.2s ease;
  font-size: 14px;
}

.context-menu-item:hover {
  background: var(--color-primary-light, #e0f2fe);
  color: var(--color-primary, #0ea5e9);
}

/* 右键菜单操作项样式 */
.context-menu-action {
  display: flex;
  align-items: center;
  font-weight: 500;
  color: var(--color-primary, #0ea5e9);
}

.context-menu-action:hover {
  background: var(--color-primary-light, #e0f2fe);
  color: var(--color-primary-dark, #0284c7);
}

.context-menu-action svg {
  flex-shrink: 0;
}

.context-menu-divider {
  height: 1px;
  background: var(--color-divider, #e1e5e9);
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
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 4px;
  font-size: 14px;
  color: var(--color-text-primary, #374151);
  background: var(--color-surface, #ffffff);
  outline: none;
  transition: border-color 0.2s ease;
}

.custom-count-input:focus {
  border-color: var(--color-primary, #0ea5e9);
}

.custom-count-input::placeholder {
  color: var(--color-text-muted, #9ca3af);
}

.btn-custom-fill {
  padding: 6px 12px;
  background: var(--color-primary, #0ea5e9);
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
  background: var(--color-primary-dark, #0284c7);
}

/* 批量删除对话框 */
.batch-delete-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  padding: 20px;
}

.batch-delete-dialog {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 500px;
  width: 100%;
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.batch-delete-dialog .dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.batch-delete-dialog .dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.batch-delete-dialog .dialog-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted, #6b7280);
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.batch-delete-dialog .dialog-close:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

.batch-delete-dialog .dialog-body {
  padding: 24px;
}

.dialog-message {
  margin: 0 0 16px 0;
  color: var(--color-text-secondary, #6b7280);
  font-size: 14px;
  white-space: pre-line;
  line-height: 1.6;
}

.delete-stats {
  background: var(--color-surface-secondary, #f9fafb);
  border: 1px solid var(--color-divider, #e1e5e9);
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
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.stat-item.total {
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.stat-label {
  color: var(--color-text-secondary, #6b7280);
  font-size: 14px;
}

.stat-value {
  color: var(--color-text-primary, #374151);
  font-size: 14px;
  font-weight: 500;
}

.dialog-warning {
  margin: 0;
  color: var(--color-warning-text, #92400e);
  background: var(--color-warning-surface, #fef3c7);
  border: 1px solid var(--color-warning-border, #fde68a);
  border-radius: 6px;
  padding: 12px;
  font-size: 13px;
}

.batch-delete-dialog .dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--color-divider, #e1e5e9);
  background: var(--color-surface, #ffffff);
}

.btn-danger {
  background: var(--color-danger, #dc2626);
  color: white;
  border: 1px solid var(--color-danger, #dc2626);
}

.btn-danger:hover:not(:disabled) {
  background: var(--color-danger-hover, #b91c1c);
  border-color: var(--color-danger-hover, #b91c1c);
}

.btn-danger:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 黑暗模式 */
[data-theme='dark'] .batch-import-dialog {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .batch-import-dialog .dialog-footer {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .batch-delete-dialog {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .delete-stats {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .dialog-warning {
  background: rgba(245, 158, 11, 0.2);
  border-color: rgba(245, 158, 11, 0.4);
  color: #fbbf24;
}

[data-theme='dark'] .format-option,
[data-theme='dark'] .format-option-single {
  background: rgba(55, 65, 81, 0.3);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .btn-fill-template {
  background: rgba(37, 99, 235, 0.1);
  border-color: var(--color-primary, #3b82f6);
  color: var(--color-primary, #3b82f6);
}

[data-theme='dark'] .btn-fill-template:hover {
  background: var(--color-primary, #3b82f6);
  color: #ffffff;
}

@media (max-width: 480px) {
  .modal-overlay {
    padding: 10px;
  }

  .modal-content {
    max-height: 95vh;
  }

  .modal-header h2 {
    font-size: 1.25rem;
  }

  .empty-state {
    padding: 20px 10px;
  }

  .empty-state h3 {
    font-size: 1.125rem;
  }

  .btn.small {
    padding: 4px 8px;
    font-size: 11px;
  }

  .sync-actions {
    flex-direction: column;
  }

  .btn.sync-btn {
    min-width: auto;
  }
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
}

.list-title-section {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.list-header h3 {
  margin: 0;
  color: var(--color-text-primary, #374151);
  font-size: 1.125rem;
  font-weight: 600;
}

/* 搜索框样式 */
.search-box {
  position: relative;
  display: flex;
  align-items: center;
  margin-left: auto;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--color-text-secondary, #6b7280);
  pointer-events: none;
}

.search-input {
  width: 300px;
  padding: 6px 32px 6px 32px;
  border: 1px solid var(--color-divider, #e1e5e9);
  border-radius: 6px;
  font-size: 13px;
  color: var(--color-text-primary, #374151);
  background: var(--color-surface, #ffffff);
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.search-input::placeholder {
  color: var(--color-text-secondary, #9ca3af);
}

.clear-search-btn {
  position: absolute;
  right: 6px;
  padding: 4px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-secondary, #6b7280);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.clear-search-btn:hover {
  background: var(--color-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

/* 无搜索结果样式 */
.no-search-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--color-text-secondary, #6b7280);
}

.no-search-results p {
  margin-top: 16px;
  font-size: 14px;
}

.sort-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  background: var(--color-surface-muted, #f8f9fa);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 6px;
  color: var(--color-text-secondary, #6b7280);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0;
}

.sort-btn:hover {
  background: var(--color-surface-hover, #e9ecef);
  color: var(--color-text-primary, #374151);
  border-color: var(--color-border-hover, #d1d5db);
}

.sort-btn.active {
  background: var(--color-primary-surface, #dbeafe);
  color: var(--color-primary, #2563eb);
  border-color: var(--color-primary-border, #93c5fd);
}

.sort-btn.active:hover {
  background: var(--color-primary-surface-hover, #bfdbfe);
  border-color: var(--color-primary-border-hover, #60a5fa);
}

.sort-icon {
  flex-shrink: 0;
}

.arrow-icon {
  flex-shrink: 0;
  transition: transform 0.2s ease;
}

.arrow-icon.arrow-down {
  transform: rotate(180deg);
}

.arrow-icon.arrow-up {
  transform: rotate(0deg);
}

.btn {
  padding: 8px 16px;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 36px;
  box-sizing: border-box;
}

.btn.secondary {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
  border: 1px solid var(--color-border-strong, #d1d5db);
}

.btn.secondary:hover {
  background: var(--color-border, #e5e7eb);
  border-color: var(--color-border-hover, #9ca3af);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(55, 65, 81, 0.2);
}

.btn.success {
  background: var(--color-success-surface, #d1fae5);
  color: var(--color-success-text, #065f46);
  border: 1px solid var(--color-success-border, #a7f3d0);
}

.btn.success:hover:not(:disabled) {
  background: var(--color-success-surface, #a7f3d0);
  border-color: var(--color-success-border, #6ee7b7);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(6, 95, 70, 0.3);
}

.btn.success:disabled {
  background: var(--color-border-strong, #d1d5db);
  color: var(--color-text-soft, #9ca3af);
  border-color: var(--color-border-strong, #d1d5db);
  cursor: not-allowed;
}

.btn.info {
  background: var(--color-info-surface, #dbeafe);
  color: var(--color-info-text, #1e40af);
  border: 1px solid var(--color-info-border, #93c5fd);
}

.btn.info:hover:not(:disabled) {
  background: var(--color-info-surface, #bfdbfe);
  border-color: var(--color-info-border, #60a5fa);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(30, 64, 175, 0.3);
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
  height: 32px;
}

/* Header layout */
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-surface-alt, #f9fafb);
  min-height: 60px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.header-title h2 {
  margin: 0;
  color: var(--color-text-strong, #111827);
  font-size: 1.25rem;
  font-weight: 600;
  line-height: 1.2;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  flex-wrap: wrap;
}



/* Status badge styles */
.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  height: fit-content;
}

.status-badge.saved {
  background-color: var(--color-success-surface, #d1fae5);
  color: var(--color-success-text, #065f46);
}

.status-badge.unsaved {
  background-color: var(--color-warning-surface, #fef3c7);
  color: var(--color-warning-text, #92400e);
}

.status-badge.initializing {
  background-color: var(--color-info-surface, #dbeafe);
  color: var(--color-info-text, #1e40af);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}

.status-dot.saved {
  background-color: var(--color-success-bg, #10b981);
}

.status-dot.unsaved {
  background-color: var(--color-warning-bg, #f59e0b);
}

.status-dot.initializing {
  background-color: var(--color-info-bg, #3b82f6);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.5;
  }
}

.status-text {
  font-size: 11px;
  font-weight: 500;
}



@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.btn.loading {
  opacity: 0.7;
  pointer-events: none;
}

/* 浅色主题按钮样式统一 */
.btn.primary {
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border: 1px solid var(--color-blue-soft-border, #90caf9);
}

.btn.primary:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(25, 118, 210, 0.3);
}

/* 黑暗主题下的按钮样式 */
[data-theme='dark'] .btn.secondary {
  background: rgba(148, 163, 184, 0.2);
  color: #cbd5e1;
  border: 1px solid rgba(148, 163, 184, 0.4);
}

[data-theme='dark'] .btn.secondary:hover {
  background: rgba(148, 163, 184, 0.3);
  border-color: rgba(148, 163, 184, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(148, 163, 184, 0.4);
}

[data-theme='dark'] .btn.success {
  background: rgba(34, 197, 94, 0.2);
  color: #86efac;
  border: 1px solid rgba(134, 239, 172, 0.4);
}

[data-theme='dark'] .btn.success:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.3);
  border-color: rgba(110, 231, 183, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(34, 197, 94, 0.4);
}

[data-theme='dark'] .btn.success:disabled {
  background: rgba(100, 116, 139, 0.2);
  color: rgba(148, 163, 184, 0.6);
  border-color: rgba(100, 116, 139, 0.4);
  cursor: not-allowed;
}

[data-theme='dark'] .btn.info {
  background: rgba(14, 165, 233, 0.2);
  color: #7dd3fc;
  border: 1px solid rgba(125, 211, 252, 0.4);
}

[data-theme='dark'] .btn.info:hover:not(:disabled) {
  background: rgba(14, 165, 233, 0.3);
  border-color: rgba(56, 189, 248, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(14, 165, 233, 0.4);
}

[data-theme='dark'] .btn.primary {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
  border: 1px solid rgba(147, 197, 253, 0.4);
}

[data-theme='dark'] .btn.primary:hover {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(96, 165, 250, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(59, 130, 246, 0.4);
}

[data-theme='dark'] .search-input {
  background: var(--color-surface, #1f2937);
  border-color: var(--color-divider, #374151);
  color: var(--color-text-primary, #f3f4f6);
}

[data-theme='dark'] .search-input:focus {
  border-color: var(--color-primary, #3b82f6);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

[data-theme='dark'] .clear-search-btn:hover {
  background: var(--color-hover, #374151);
}
</style>
