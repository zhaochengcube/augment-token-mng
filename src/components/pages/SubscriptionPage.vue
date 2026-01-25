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
        <!-- Page Header -->
        <div class="flex items-center justify-between gap-4 px-5 py-4 border-b border-border bg-surface shrink-0">
          <!-- 左侧：存储状态 + 功能性操作 -->
          <div class="flex items-center gap-3 shrink-0">
            <!-- 存储状态徽章 -->
            <div
              :class="['badge', storageStatusClass, { clickable: isDatabaseAvailable }]"
              v-tooltip="isDatabaseAvailable ? $t('subscriptions.viewSyncQueueTooltip') : ''"
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
            <button @click="showAddDialog = true" class="btn btn--icon btn--ghost" v-tooltip="$t('subscriptions.add')">
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
          <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z"/>
          </svg>
        </div>
        <p class="mt-4 text-sm">{{ searchQuery ? $t('common.noSearchResults') : $t('subscriptions.emptyState') }}</p>
        <button v-if="!searchQuery" class="btn btn--primary mt-4" @click="showAddDialog = true">
          {{ $t('subscriptions.addFirst') }}
        </button>
      </div>

      <!-- Subscription List -->
      <template v-else>
        <!-- 卡片布局 -->
        <div v-if="viewMode === 'card'" class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3 p-1">
          <SubscriptionCard
            v-for="sub in paginatedSubscriptions"
            :key="sub.id"
            :subscription="sub"
            @edit="handleEdit"
            @delete="handleDelete"
          />
        </div>

        <!-- 列表布局 -->
        <div v-else class="table-container">
          <table class="table table-fixed">
            <thead>
              <tr>
                <th class="th w-[120px]">{{ $t('subscriptions.fields.website') }}</th>
                <th class="th">{{ $t('subscriptions.fields.websiteUrl') }}</th>
                <th class="th w-[100px]">{{ $t('subscriptions.fields.expiryDate') }}</th>
                <th class="th w-[80px]">{{ $t('subscriptions.fields.cost') }}</th>
                <th class="th">{{ $t('subscriptions.fields.notes') }}</th>
                <th class="th w-[60px]">{{ $t('subscriptions.fields.tag') }}</th>
                <th class="th w-[80px] text-center">{{ $t('common.actions') }}</th>
              </tr>
            </thead>
            <tbody>
              <SubscriptionTableRow
                v-for="sub in paginatedSubscriptions"
                :key="sub.id"
                :subscription="sub"
                @edit="handleEdit"
                @delete="handleDelete"
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
          :total-items="filteredSubscriptions.length"
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
          :placeholder="$t('subscriptions.searchPlaceholder')"
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
    <SubscriptionDialog
      v-if="showAddDialog || editingSubscription"
      :subscription="editingSubscription"
      :all-subscriptions="subscriptions"
      @save="handleSave"
      @close="closeDialog"
    />

    <!-- Sync Queue Modal -->
    <SyncQueueModal
      v-model:visible="showSyncQueueModal"
      :pending-upserts="pendingUpsertsList"
      :pending-deletions="pendingDeletionsList"
      :syncing="isSyncing"
      :total-count="subscriptions.length"
      :title="$t('subscriptions.syncQueueTitle')"
      :upserts-title="$t('subscriptions.syncQueueUpsertsTitle')"
      :deletions-title="$t('subscriptions.syncQueueDeletionsTitle')"
      :empty-text="$t('subscriptions.syncQueueEmpty')"
      :full-sync-text="$t('subscriptions.fullSync')"
      :sync-text="$t('subscriptions.sync')"
      label-field="website"
      fallback-field="id"
      @sync="handleSync"
      @mark-all-for-sync="handleMarkAllForSync"
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
import Pagination from '../common/Pagination.vue'
import SyncQueueModal from '../common/SyncQueueModal.vue'
import SubscriptionCard from '../subscription/SubscriptionCard.vue'
import SubscriptionTableRow from '../subscription/SubscriptionTableRow.vue'
import SubscriptionDialog from '../subscription/SubscriptionDialog.vue'

const { t: $t } = useI18n()

const props = defineProps({
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})

const { isSidebarCollapsed } = toRefs(props)

// 本地存储键（用于视图模式）
const VIEW_MODE_KEY = 'atm-subscriptions-view-mode'

// 状态
const subscriptions = ref([])
const isLoading = ref(false)
const isRefreshing = ref(false)
const showAddDialog = ref(false)
const editingSubscription = ref(null)
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
  platform: 'subscription',
  syncCommand: 'subscription_sync_accounts',
  items: subscriptions,
  itemKey: 'account',
  labelField: 'website',
  onSyncComplete: async () => {
    // 同步完成后重新加载数据以确保本地文件更新
    try {
      const response = await invoke('subscription_list')
      subscriptions.value = response?.subscriptions || []
    } catch (error) {
      console.error('Failed to reload subscriptions after sync:', error)
    }
  }
})

// 标记全部同步
const handleMarkAllForSync = () => {
  markAllForSync()
}

// 辅助函数：计算订阅的剩余天数
const getDaysLeft = (sub) => {
  if (!sub.expiry_date) return null
  const expiryTime = new Date(sub.expiry_date).getTime()
  return Math.ceil((expiryTime - Date.now()) / (1000 * 60 * 60 * 24))
}

// 辅助函数：应用到期时间过滤
const applyExpiryFilter = (subs, filter) => {
  if (filter === 'all') return subs
  return subs.filter((sub) => {
    const daysLeft = getDaysLeft(sub)
    if (daysLeft === null) return false
    if (filter === 'expired') return daysLeft < 0
    if (filter === '7d') return daysLeft >= 0 && daysLeft <= 7
    if (filter === '30d') return daysLeft >= 0 && daysLeft <= 30
    if (filter === 'gt30') return daysLeft > 30
    return true
  })
}

// 只应用搜索的订阅列表
const searchFilteredSubscriptions = computed(() => {
  let result = subscriptions.value
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(sub =>
      sub.website?.toLowerCase().includes(query) ||
      sub.website_url?.toLowerCase().includes(query) ||
      sub.tag?.toLowerCase().includes(query)
    )
  }
  return result
})

// 应用搜索 + 标签筛选（用于计算到期时间统计）
const tagFilteredSubscriptions = computed(() => {
  let result = searchFilteredSubscriptions.value
  if (selectedTagFilter.value) {
    result = result.filter(sub => sub.tag === selectedTagFilter.value)
  }
  return result
})

// 应用搜索 + 到期时间筛选（用于计算标签统计）
const expiryFilteredSubscriptions = computed(() => {
  return applyExpiryFilter(searchFilteredSubscriptions.value, selectedExpiryFilter.value)
})

// 到期时间统计 - 基于搜索+标签筛选
const expiryStatistics = computed(() => {
  const subs = tagFilteredSubscriptions.value
  const stats = {
    all: subs.length,
    expired: 0,
    '7d': 0,
    '30d': 0,
    gt30: 0
  }
  subs.forEach(sub => {
    const daysLeft = getDaysLeft(sub)
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
  const subs = expiryFilteredSubscriptions.value
  let taggedCount = 0
  const stats = {}
  subs.forEach(sub => {
    if (sub.tag) {
      stats[sub.tag] = (stats[sub.tag] || 0) + 1
      taggedCount++
    }
  })
  stats.all = taggedCount
  return stats
})

// 计算属性：过滤并排序后的订阅列表
const filteredSubscriptions = computed(() => {
  // 应用到期时间筛选
  let result = applyExpiryFilter(searchFilteredSubscriptions.value, selectedExpiryFilter.value)

  // 过滤：标签
  if (selectedTagFilter.value) {
    result = result.filter(sub => sub.tag === selectedTagFilter.value)
  }

  // 排序
  result = [...result].sort((a, b) => {
    if (sortType.value === 'created') {
      const timeA = a.created_at || 0
      const timeB = b.created_at || 0
      return sortOrder.value === 'desc' ? timeB - timeA : timeA - timeB
    } else if (sortType.value === 'expiry') {
      // 将日期字符串转为时间戳进行比较，无日期的放到最后
      const getExpiryTime = (sub) => {
        if (!sub.expiry_date) return sortOrder.value === 'desc' ? -Infinity : Infinity
        return new Date(sub.expiry_date).getTime()
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
  subscriptions.value.forEach((sub) => {
    if (sub.tag && !tagMap.has(sub.tag)) {
      tagMap.set(sub.tag, { name: sub.tag })
    }
  })
  return Array.from(tagMap.values()).sort((a, b) => a.name.localeCompare(b.name))
})

// 分页后的订阅列表
const paginatedSubscriptions = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredSubscriptions.value.slice(start, end)
})

// 是否显示分页
const totalPages = computed(() => Math.ceil(filteredSubscriptions.value.length / pageSize.value))

const shouldShowPagination = computed(() => !isLoading.value && filteredSubscriptions.value.length > 0)

// 空状态判断
const showEmptyState = computed(() => filteredSubscriptions.value.length === 0)

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

// 加载订阅数据
const loadSubscriptions = async () => {
  isLoading.value = true
  try {
    await initSync()
    // 从后端加载订阅列表
    const response = await invoke('subscription_list')
    subscriptions.value = response?.subscriptions || []
  } catch (error) {
    console.error('Failed to load subscriptions:', error)
    subscriptions.value = []
  } finally {
    isLoading.value = false
  }
}

// 刷新数据
const handleRefresh = async () => {
  isRefreshing.value = true
  try {
    await initSync()
    const response = await invoke('subscription_list')
    subscriptions.value = response?.subscriptions || []
    window.$notify?.success($t('common.refreshSuccess'))
  } catch (error) {
    console.error('Failed to refresh subscriptions:', error)
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

// 生成唯一ID
const generateId = () => `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

// 保存订阅（新增或编辑）
const handleSave = async (formData) => {
  const now = Math.floor(Date.now() / 1000)

  try {
    if (editingSubscription.value) {
      // 编辑模式
      const updated = {
        ...editingSubscription.value,
        ...formData,
        updated_at: now
      }

      // 调用后端更新
      await invoke('subscription_update', { subscription: updated })

      // 更新前端状态
      const index = subscriptions.value.findIndex(s => s.id === editingSubscription.value.id)
      if (index !== -1) {
        subscriptions.value[index] = updated
      }
      markItemUpsert(updated)
      window.$notify?.success($t('subscriptions.messages.updateSuccess'))
    } else {
      // 新增模式
      const newSub = {
        id: generateId(),
        ...formData,
        created_at: now,
        updated_at: now,
        version: 0,
        deleted: false
      }

      // 调用后端添加
      await invoke('subscription_add', { subscription: newSub })

      // 更新前端状态
      subscriptions.value.unshift(newSub)
      markItemUpsert(newSub)
      window.$notify?.success($t('subscriptions.messages.addSuccess'))
    }
    closeDialog()
  } catch (error) {
    console.error('Failed to save subscription:', error)
    window.$notify?.error($t('subscriptions.messages.saveFailed'))
  }
}

// 编辑订阅
const handleEdit = (subscription) => {
  editingSubscription.value = subscription
}

// 删除订阅
const handleDelete = async (subscription) => {
  const confirmed = await window.$confirm?.({
    title: $t('subscriptions.deleteConfirm.title'),
    message: $t('subscriptions.deleteConfirm.message', { name: subscription.website }),
    confirmText: $t('common.delete'),
    cancelText: $t('common.cancel'),
    type: 'danger'
  })
  if (confirmed) {
    try {
      // 调用后端删除
      await invoke('subscription_delete', { id: subscription.id })

      // 更新前端状态
      subscriptions.value = subscriptions.value.filter(s => s.id !== subscription.id)
      markItemDeletion(subscription)
      window.$notify?.success($t('subscriptions.messages.deleteSuccess'))
    } catch (error) {
      console.error('Failed to delete subscription:', error)
      window.$notify?.error($t('subscriptions.messages.deleteFailed'))
    }
  }
}

// 关闭弹窗
const closeDialog = () => {
  showAddDialog.value = false
  editingSubscription.value = null
}

// 初始化
onMounted(async () => {
  loadViewMode()
  await loadSubscriptions()
})
</script>
