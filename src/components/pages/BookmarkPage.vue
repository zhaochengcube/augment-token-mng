<template>
  <div class="flex flex-col h-full w-full min-h-0">
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
          :sync-queue-tooltip="$t('bookmarks.viewSyncQueueTooltip')"
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
          <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('bookmarks.add')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
            </svg>
          </button>
          <button @click="showImportDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('bookmarks.import.tooltip')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/>
            </svg>
          </button>
          <button @click="showRaindropDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('raindrop.title')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2c-5.33 4.55-8 8.48-8 11.8 0 4.98 3.8 8.2 8 8.2s8-3.22 8-8.2C20 10.48 17.33 6.55 12 2z"/>
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
          <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z"/>
          </svg>
        </div>
        <p class="mt-4 text-sm">{{ searchQuery ? $t('common.noSearchResults') : $t('bookmarks.emptyState') }}</p>
        <button v-if="!searchQuery" class="btn btn--primary mt-4" @click="showAddDialog = true">
          {{ $t('bookmarks.addFirst') }}
        </button>
      </div>

      <!-- Bookmark List -->
      <template v-else>
        <!-- 卡片布局 -->
        <div v-if="viewMode === 'card'" class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3 p-1">
          <BookmarkCard
            v-for="item in paginatedBookmarks"
            :key="item.id"
            :bookmark="item"
            :is-selected="selectedBookmarkIds.has(item.id)"
            :selection-mode="isSelectionMode"
            :all-bookmarks="bookmarks"
            @edit="handleEdit"
            @delete="handleDelete"
            @select="toggleBookmarkSelection"
            @bookmark-updated="handleBookmarkUpdated"
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
                <th class="th w-[120px]">{{ $t('bookmarks.fields.name') }}</th>
                <th class="th">{{ $t('bookmarks.fields.url') }}</th>
                <th class="th">{{ $t('bookmarks.fields.description') }}</th>
                <th class="th w-[60px]">{{ $t('bookmarks.fields.tag') }}</th>
                <th class="th w-[80px] text-center">{{ $t('common.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <BookmarkTableRow
                v-for="item in paginatedBookmarks"
                :key="item.id"
                :bookmark="item"
                :is-selected="selectedBookmarkIds.has(item.id)"
                :selection-mode="isSelectionMode"
                :all-bookmarks="bookmarks"
                @edit="handleEdit"
                @delete="handleDelete"
                @select="toggleBookmarkSelection"
                @bookmark-updated="handleBookmarkUpdated"
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
          :total-items="filteredBookmarks.length"
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
          :placeholder="$t('bookmarks.searchPlaceholder')"
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
        <!-- Tag Filter -->
        <div class="flex flex-col gap-2">
          <span class="label">{{ $t('bookmarks.filters.tag') }}</span>
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
              <span>{{ $t('bookmarks.filters.all') }}</span>
              <span class="ml-1 opacity-70">({{ tagStatistics.all }})</span>
            </button>
            <button
              v-for="tag in tagOptions"
              :key="tag.name"
              :class="['btn btn--sm', selectedTags.has(tag.name) ? 'btn--primary' : 'btn--secondary']"
              @click="toggleTag(tag.name)"
            >
              <span>{{ tag.name }}</span>
              <span class="ml-1 opacity-70">({{ tagStatistics[tag.name] || 0 }})</span>
            </button>
            <button
              :class="['btn btn--sm', selectedTags.has('__no_tag__') ? 'btn--primary' : 'btn--secondary']"
              @click="toggleTag('__no_tag__')"
            >
              <span>无标签</span>
              <span class="ml-1 opacity-70">({{ tagStatistics.__no_tag__ || 0 }})</span>
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
              {{ $t('bookmarks.sortByCreated') }}
            </button>
            <button
              :class="[
                'btn btn--sm',
                sortType === 'name' ? 'btn--primary' : 'btn--secondary'
              ]"
              @click="setSortType('name', sortOrder)"
            >
              {{ $t('bookmarks.sortByName') }}
            </button>
          </div>
        </div>
      </div>
    </ActionToolbar>

    <!-- Import Bookmarks Dialog -->
    <ImportBookmarksDialog
      v-if="showImportDialog"
      @close="showImportDialog = false"
      @imported="handleBookmarksImported"
    />

    <!-- Raindrop.io Sync Dialog -->
    <RaindropSyncDialog
      v-if="showRaindropDialog"
      @close="showRaindropDialog = false"
      @synced="handleRaindropSynced"
    />

    <!-- Add/Edit Dialog -->
    <BookmarkDialog
      v-if="showAddDialog || editingBookmark"
      :bookmark="editingBookmark"
      :all-bookmarks="bookmarks"
      @save="handleSave"
      @close="closeDialog"
    />

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="bookmarks.length"
      :title="$t('bookmarks.syncQueueTitle')"
      :upserts-title="$t('bookmarks.syncQueueUpsertsTitle')"
      :deletions-title="$t('bookmarks.syncQueueDeletionsTitle')"
      :empty-text="$t('bookmarks.syncQueueEmpty')"
      :full-sync-text="$t('bookmarks.fullSync')"
      :sync-text="$t('bookmarks.sync')"
      label-field="name"
      fallback-field="id"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
    />

    <!-- 批量操作工具栏 -->
    <BatchToolbar
      :visible="isSelectionMode"
      :selected-count="selectedBookmarkIds.size"
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
          @click="handleBatchDeleteSelected"
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
      :tokens="selectedBookmarksForTag"
      :all-tokens="allBookmarksAsTokens"
      @save="handleBatchTagSave"
      @clear="handleBatchTagClear"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, nextTick, toRefs } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useStorageSync } from '@/composables/useStorageSync'
import ActionToolbar from '../common/ActionToolbar.vue'
import FixedPaginationLayout from '../common/FixedPaginationLayout.vue'
import AccountManagerHeader from '../common/AccountManagerHeader.vue'
import Pagination from '../common/Pagination.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import BatchToolbar from '../common/BatchToolbar.vue'
import TagEditorModal from '../token/TagEditorModal.vue'
import BookmarkCard from '../bookmark/BookmarkCard.vue'
import BookmarkTableRow from '../bookmark/BookmarkTableRow.vue'
import BookmarkDialog from '../bookmark/BookmarkDialog.vue'
import ImportBookmarksDialog from '../bookmark/ImportBookmarksDialog.vue'
import RaindropSyncDialog from '../bookmark/RaindropSyncDialog.vue'

const { t: $t } = useI18n()

const props = defineProps({
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})

const { isSidebarCollapsed } = toRefs(props)

// 本地存储键（用于视图模式）
const VIEW_MODE_KEY = 'atm-bookmarks-view-mode'

// 状态
const bookmarks = ref([])
const isLoading = ref(false)
const isRefreshing = ref(false)
const showImportDialog = ref(false)
const showRaindropDialog = ref(false)
const showAddDialog = ref(false)
const editingBookmark = ref(null)
const viewMode = ref('card')
const searchQuery = ref('')
const toolbarMode = ref('hidden')
const searchInputRef = ref(null)
const selectedTags = ref(new Set())
const tagFilterMode = ref('include')

// 排序状态
const sortType = ref('created')
const sortOrder = ref('desc')

// 分页状态
const currentPage = ref(1)
const pageSize = ref(20)

// 批量操作状态
const selectedBookmarkIds = ref(new Set())
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
  markItemUpsertById,
  markItemDeletion,
  markAllForSync,
  openSyncQueue,
  handleSync
} = useStorageSync({
  platform: 'bookmark',
  syncCommand: 'bookmark_sync_accounts',
  items: bookmarks,
  itemKey: 'account',
  labelField: 'name',
  onSyncComplete: async () => {
    try {
      const response = await invoke('bookmark_list')
      bookmarks.value = response?.bookmarks || []
    } catch (error) {
      console.error('Failed to reload bookmarks after sync:', error)
    }
  }
})

// 标记全部同步
const handleMarkAllForSync = () => {
  markAllForSync()
}

// 只应用搜索的书签列表
const searchFilteredBookmarks = computed(() => {
  let result = bookmarks.value
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(b =>
      b.name?.toLowerCase().includes(query) ||
      b.url?.toLowerCase().includes(query) ||
      b.tag?.toLowerCase().includes(query)
    )
  }
  return result
})

// 标签统计
const tagStatistics = computed(() => {
  const items = bookmarks.value
  const stats = {}
  let taggedCount = 0
  items.forEach(b => {
    if (b.tag) {
      stats[b.tag] = (stats[b.tag] || 0) + 1
      taggedCount++
    }
  })
  stats.all = items.length
  stats.__no_tag__ = items.length - taggedCount
  return stats
})

// 计算属性：过滤并排序后的书签列表
const filteredBookmarks = computed(() => {
  let result = searchFilteredBookmarks.value

  // 过滤：标签
  if (selectedTags.value.size > 0) {
    const hasNoTagFilter = selectedTags.value.has('__no_tag__')
    const selectedTagNames = new Set(
      Array.from(selectedTags.value).filter(t => t !== '__no_tag__')
    )
    result = result.filter(b => {
      const tag = b.tag || ''
      const isNoTag = !tag
      let matches = false
      if (isNoTag) {
        matches = hasNoTagFilter
      } else {
        matches = selectedTagNames.has(tag)
      }
      return tagFilterMode.value === 'include' ? matches : !matches
    })
  }

  // 排序
  result = [...result].sort((a, b) => {
    if (sortType.value === 'created') {
      const timeA = a.created_at || 0
      const timeB = b.created_at || 0
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'name') {
      const nameA = (a.name || '').toLowerCase()
      const nameB = (b.name || '').toLowerCase()
      const cmp = nameA.localeCompare(nameB)
      return sortOrder.value === 'desc' ? -cmp : cmp
    }
    return 0
  })

  return result
})

const tagOptions = computed(() => {
  const tagMap = new Map()
  bookmarks.value.forEach((b) => {
    if (b.tag && !tagMap.has(b.tag)) {
      tagMap.set(b.tag, { name: b.tag })
    }
  })
  return Array.from(tagMap.values()).sort((a, b) => a.name.localeCompare(b.name))
})

// 分页后的书签列表
const paginatedBookmarks = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredBookmarks.value.slice(start, end)
})

// 是否显示分页
const totalPages = computed(() => Math.ceil(filteredBookmarks.value.length / pageSize.value))

const shouldShowPagination = computed(() => !isLoading.value && filteredBookmarks.value.length > 0)

// 搜索/筛选/排序活跃状态
const isSearchActive = computed(() => searchQuery.value.trim() !== '')
const isFilterActive = computed(() => selectedTags.value.size > 0)
const isSortNonDefault = computed(() => sortType.value !== 'created' || sortOrder.value !== 'desc')

const clearAllFilters = () => {
  searchQuery.value = ''
  selectedTags.value = new Set()
  tagFilterMode.value = 'include'
  sortType.value = 'created'
  sortOrder.value = 'desc'
}

// 空状态判断
const showEmptyState = computed(() => filteredBookmarks.value.length === 0)

// 批量操作计算属性
const isSelectionMode = computed(() => selectedBookmarkIds.value.size > 0)

const selectedBookmarksForTag = computed(() => {
  return bookmarks.value
    .filter(b => selectedBookmarkIds.value.has(b.id))
    .map(b => ({
      id: b.id,
      tag_name: b.tag || '',
      tag_color: b.tag_color || ''
    }))
})

const allBookmarksAsTokens = computed(() =>
  bookmarks.value.map(b => ({
    tag_name: b.tag || '',
    tag_color: b.tag_color || ''
  }))
)

const isAllSelected = computed(() => {
  return paginatedBookmarks.value.length > 0 && paginatedBookmarks.value.every(b => selectedBookmarkIds.value.has(b.id))
})

const isPartialSelected = computed(() => {
  return selectedBookmarkIds.value.size > 0 && !isAllSelected.value
})

// 工具栏模式切换
const setToolbarMode = (mode) => {
  toolbarMode.value = toolbarMode.value === mode ? 'hidden' : mode
  if (toolbarMode.value === 'search') {
    nextTick(() => searchInputRef.value?.focus())
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

// 加载书签数据
const loadBookmarks = async () => {
  isLoading.value = true
  try {
    const response = await invoke('bookmark_load_local')
    bookmarks.value = response?.bookmarks || []
  } catch (error) {
    console.error('Failed to load bookmarks:', error)
    bookmarks.value = []
  } finally {
    isLoading.value = false
  }
}

// 刷新数据
const handleRefresh = async () => {
  isRefreshing.value = true
  try {
    await loadBookmarks()
    window.$notify?.success($t('common.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh bookmarks:', error)
    window.$notify?.error($t('common.refreshFailed'))
  } finally {
    isRefreshing.value = false
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

// 导入书签成功处理
const handleBookmarksImported = async (result) => {
  showImportDialog.value = false
  await loadBookmarks()
  // 标记导入的书签到同步队列
  if (result.imported_ids?.length) {
    result.imported_ids.forEach(id => markItemUpsertById(id))
  }
  let msg = $t('bookmarks.import.importSuccess', { success: result.success_count })
  if (result.skipped_count > 0) msg += $t('bookmarks.import.importSkipped', { skipped: result.skipped_count })
  if (result.failed_count > 0) msg += $t('bookmarks.import.importFailedCount', { failed: result.failed_count })
  window.$notify?.success(msg)
}

// Raindrop 同步完成处理
const handleRaindropSynced = async (result) => {
  await loadBookmarks()
  // 标记新增/更新的书签到同步队列
  if (result?.affected_ids?.length) {
    result.affected_ids.forEach(id => markItemUpsertById(id))
  }
}

// 生成唯一ID
const generateId = () => `bm_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

// 保存书签（新增或编辑）
const handleSave = async (formData) => {
  const now = Math.floor(Date.now() / 1000)

  try {
    if (editingBookmark.value) {
      // 编辑模式
      const updated = {
        ...editingBookmark.value,
        ...formData,
        updated_at: now
      }

      await invoke('bookmark_update', { bookmark: updated })

      const index = bookmarks.value.findIndex(b => b.id === editingBookmark.value.id)
      if (index !== -1) {
        bookmarks.value[index] = updated
      }
      markItemUpsert(updated)
      window.$notify?.success($t('bookmarks.messages.updateSuccess'))
    } else {
      // 新增模式
      const newBookmark = {
        id: generateId(),
        ...formData,
        created_at: now,
        updated_at: now,
        version: 0,
        deleted: false
      }

      await invoke('bookmark_add', { bookmark: newBookmark })

      bookmarks.value.unshift(newBookmark)
      markItemUpsert(newBookmark)
      window.$notify?.success($t('bookmarks.messages.addSuccess'))
    }
    closeDialog()
  } catch (error) {
    console.error('Failed to save bookmark:', error)
    window.$notify?.error($t('bookmarks.messages.saveFailed'))
  }
}

// 编辑书签
const handleEdit = (bookmark) => {
  editingBookmark.value = bookmark
}

// 删除书签
const handleDelete = async (bookmark) => {
  const confirmed = await window.$confirm?.({
    title: $t('bookmarks.deleteConfirm.title'),
    message: $t('bookmarks.deleteConfirm.message', { name: bookmark.name }),
    confirmText: $t('common.delete'),
    cancelText: $t('common.cancel'),
    type: 'danger'
  })
  if (confirmed) {
    try {
      await invoke('bookmark_delete', { id: bookmark.id })

      bookmarks.value = bookmarks.value.filter(b => b.id !== bookmark.id)
      markItemDeletion(bookmark)
      window.$notify?.success($t('bookmarks.messages.deleteSuccess'))
    } catch (error) {
      console.error('Failed to delete bookmark:', error)
      window.$notify?.error($t('bookmarks.messages.deleteFailed'))
    }
  }
}

// 关闭弹窗
const closeDialog = () => {
  showAddDialog.value = false
  editingBookmark.value = null
}

// 批量操作方法
const toggleBookmarkSelection = (bookmarkId) => {
  const newSet = new Set(selectedBookmarkIds.value)
  if (newSet.has(bookmarkId)) {
    newSet.delete(bookmarkId)
  } else {
    newSet.add(bookmarkId)
  }
  selectedBookmarkIds.value = newSet
}

const toggleSelectAll = () => {
  const currentIds = paginatedBookmarks.value.map(b => b.id)
  const allSelected = currentIds.every(id => selectedBookmarkIds.value.has(id))

  const newSet = new Set(selectedBookmarkIds.value)
  if (allSelected) {
    currentIds.forEach(id => newSet.delete(id))
  } else {
    currentIds.forEach(id => newSet.add(id))
  }
  selectedBookmarkIds.value = newSet
}

const selectAllOnPage = () => {
  const newSet = new Set(selectedBookmarkIds.value)
  paginatedBookmarks.value.forEach(b => newSet.add(b.id))
  selectedBookmarkIds.value = newSet
}

const clearSelection = () => {
  selectedBookmarkIds.value = new Set()
}

// 批量删除
const handleBatchDeleteSelected = async () => {
  const confirmed = await window.$confirm?.({
    title: $t('bookmarks.deleteConfirm.title'),
    message: $t('bookmarks.deleteConfirm.message', { name: `${selectedBookmarkIds.value.size} items` }),
    confirmText: $t('common.delete'),
    cancelText: $t('common.cancel'),
    type: 'danger'
  })
  if (confirmed) {
    try {
      for (const bookmarkId of selectedBookmarkIds.value) {
        const bookmark = bookmarks.value.find(b => b.id === bookmarkId)
        await invoke('bookmark_delete', { id: bookmarkId })
        if (bookmark) {
          markItemDeletion(bookmark)
        }
      }
      selectedBookmarkIds.value = new Set()
      await loadBookmarks()
      window.$notify?.success($t('bookmarks.messages.deleteSuccess'))
    } catch (error) {
      console.error('Failed to batch delete bookmarks:', error)
      window.$notify?.error($t('bookmarks.messages.deleteFailed'))
    }
  }
}

// 批量编辑标签 - 保存
const handleBatchTagSave = async ({ tagName, tagColor }) => {
  if (selectedBookmarkIds.value.size === 0) return

  const selectedIds = Array.from(selectedBookmarkIds.value)
  let updatedCount = 0

  for (const bookmarkId of selectedIds) {
    const bookmark = bookmarks.value.find(b => b.id === bookmarkId)
    if (bookmark) {
      bookmark.tag = tagName
      bookmark.tag_color = tagColor
      bookmark.updated_at = Math.floor(Date.now() / 1000)
      updatedCount++
      markItemUpsert(bookmark)
    }
  }

  try {
    for (const bookmarkId of selectedIds) {
      const bookmark = bookmarks.value.find(b => b.id === bookmarkId)
      if (bookmark) {
        await invoke('bookmark_update', { bookmark })
      }
    }
  } catch (error) {
    console.error('Failed to save bookmarks:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagUpdated', { count: updatedCount }))
}

// 批量编辑标签 - 清除
const handleBatchTagClear = async () => {
  if (selectedBookmarkIds.value.size === 0) return

  const selectedIds = Array.from(selectedBookmarkIds.value)
  let clearedCount = 0

  for (const bookmarkId of selectedIds) {
    const bookmark = bookmarks.value.find(b => b.id === bookmarkId)
    if (bookmark) {
      bookmark.tag = ''
      bookmark.tag_color = ''
      bookmark.updated_at = Math.floor(Date.now() / 1000)
      clearedCount++
      markItemUpsert(bookmark)
    }
  }

  try {
    for (const bookmarkId of selectedIds) {
      const bookmark = bookmarks.value.find(b => b.id === bookmarkId)
      if (bookmark) {
        await invoke('bookmark_update', { bookmark })
      }
    }
  } catch (error) {
    console.error('Failed to save bookmarks:', error)
  }

  clearSelection()
  window.$notify?.success($t('tokenList.batchTagCleared', { count: clearedCount }))
}

// 书签更新处理（标签等属性）
const handleBookmarkUpdated = async (updatedBookmark) => {
  if (!updatedBookmark?.id) return
  try {
    const index = bookmarks.value.findIndex(b => b.id === updatedBookmark.id)
    if (index !== -1) {
      bookmarks.value[index] = { ...bookmarks.value[index], ...updatedBookmark }
    }
    await invoke('bookmark_update', { bookmark: updatedBookmark })
    markItemUpsert(updatedBookmark)
  } catch (error) {
    console.error('Failed to update bookmark:', error)
    window.$notify?.error($t('messages.updateFailed'))
  }
}

// 初始化
onMounted(() => {
  loadViewMode()
  loadBookmarks()
  initSync()
})
</script>
