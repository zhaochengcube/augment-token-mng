<template>
  <div class="flex items-center justify-between gap-4 px-5 py-4 border-b border-border bg-surface shrink-0">
    <!-- 左侧：存储状态 + 功能性操作 -->
    <div class="flex items-center gap-3 shrink-0">
      <!-- 存储状态徽章 -->
      <div
        :class="['badge', storageStatusClass, { clickable: isDatabaseAvailable }]"
        v-tooltip="isDatabaseAvailable ? syncQueueTooltip : ''"
        @click="isDatabaseAvailable && $emit('open-sync-queue')"
      >
        <span :class="['status-dot', storageStatusClass]"></span>
        <span class="text-[11px] font-semibold tracking-[0.3px]">{{ storageStatusText }}</span>
      </div>

      <!-- 功能性操作按钮 -->
      <div class="flex items-center gap-2" @click.stop>
        <button class="btn btn--icon btn--ghost relative" @click="$emit('search')" v-tooltip="$t('common.search')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5z" />
          </svg>
          <span v-if="searchActive" class="indicator-dot"></span>
        </button>
        <button class="btn btn--icon btn--ghost relative" @click="$emit('filter')" v-tooltip="$t('common.filter')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 4h18l-7 8v6l-4 2v-8L3 4z"/>
          </svg>
          <span v-if="filterActive" class="indicator-dot"></span>
        </button>
        <button class="btn btn--icon btn--ghost relative" @click="$emit('sort')" v-tooltip="$t('common.sort')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M7 16V6M4 9l3-3 3 3" />
            <path d="M17 8v10M14 15l3 3 3-3" />
          </svg>
          <span v-if="sortActive" class="indicator-dot"></span>
        </button>
        <button
          class="btn btn--icon btn--ghost"
          @click="$emit('toggle-view')"
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
        <!-- 一键清除 -->
        <button
          v-if="hasActive"
          class="btn btn--icon btn--ghost text-accent"
          @click="$emit('clear-all')"
          v-tooltip="$t('common.clearFilters')"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M3 4h18l-7 8v6l-4 2v-8L3 4z"/>
            <line x1="19" y1="19" x2="5" y2="5" stroke="currentColor" stroke-width="2.5"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 右侧：各平台自定义 -->
    <div class="flex items-center gap-2 shrink-0" @click.stop>
      <slot />
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

const props = defineProps({
  storageStatusText: { type: String, default: '' },
  storageStatusClass: { type: String, default: '' },
  isDatabaseAvailable: { type: Boolean, default: false },
  syncQueueTooltip: { type: String, default: '' },
  searchActive: { type: Boolean, default: false },
  filterActive: { type: Boolean, default: false },
  sortActive: { type: Boolean, default: false },
  viewMode: { type: String, default: 'card' }
})

defineEmits([
  'open-sync-queue',
  'search', 'filter', 'sort',
  'toggle-view', 'clear-all'
])

const hasActive = computed(() => props.searchActive || props.filterActive || props.sortActive)
</script>

<style scoped>
.indicator-dot {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background-color: var(--color-accent);
  pointer-events: none;
}
</style>
