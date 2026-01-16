<template>
  <BaseModal
    :visible="visible"
    :show-close="true"
    :close-on-overlay="true"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="close"
  >
    <template #header>
      <div class="flex items-center gap-3 text-text-strong">
        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
        </svg>
        <h3 class="text-base font-semibold m-0">{{ title }}</h3>
      </div>
    </template>

    <!-- Tabs -->
    <div class="flex gap-2 pb-3 mb-3 border-b border-border">
      <button 
        :class="[
          'flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-semibold transition-all duration-200 border',
          activeTab === 'upserts' 
            ? 'bg-accent/10 border-accent/40 text-accent' 
            : 'bg-transparent border-transparent text-text-muted hover:text-text hover:bg-muted'
        ]"
        @click="activeTab = 'upserts'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
        <span>{{ upsertsTitle }}</span>
        <span class="px-1.5 py-0.5 rounded-full text-[10px] font-bold bg-success/15 text-success">
          {{ pendingUpserts.length }}
        </span>
      </button>
      <button 
        :class="[
          'flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-semibold transition-all duration-200 border',
          activeTab === 'deletions' 
            ? 'bg-accent/10 border-accent/40 text-accent' 
            : 'bg-transparent border-transparent text-text-muted hover:text-text hover:bg-muted'
        ]"
        @click="activeTab = 'deletions'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
        </svg>
        <span>{{ deletionsTitle }}</span>
        <span class="px-1.5 py-0.5 rounded-full text-[10px] font-bold bg-danger/15 text-danger">
          {{ pendingDeletions.length }}
        </span>
      </button>
    </div>

    <!-- Tab Content -->
    <div class="min-h-[200px] max-h-[350px] overflow-y-auto">
      <!-- Upserts Tab -->
      <div v-show="activeTab === 'upserts'">
        <div v-if="pendingUpserts.length === 0" class="flex flex-col items-center justify-center gap-2.5 py-8 text-text-muted text-sm">
          <svg class="w-10 h-10 opacity-20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
          <span>{{ emptyText }}</span>
        </div>
        <div v-else class="flex flex-col gap-2.5">
          <div v-for="item in pendingUpserts" :key="item.id" 
               class="flex items-center gap-2.5 p-2.5 rounded-lg bg-muted/30 border border-border hover:border-accent/50 hover:bg-accent/5 transition-all duration-200">
            <div class="w-6 h-6 rounded-md flex items-center justify-center bg-success/10 text-success shrink-0">
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <span class="text-xs font-semibold text-text truncate block">{{ getItemLabel(item) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Deletions Tab -->
      <div v-show="activeTab === 'deletions'">
        <div v-if="pendingDeletions.length === 0" class="flex flex-col items-center justify-center gap-2.5 py-8 text-text-muted text-sm">
          <svg class="w-10 h-10 opacity-20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
          <span>{{ emptyText }}</span>
        </div>
        <div v-else class="flex flex-col gap-2.5">
          <div v-for="item in pendingDeletions" :key="item.id" 
               class="flex items-center gap-2.5 p-2.5 rounded-lg bg-muted/30 border border-border opacity-85 hover:border-danger/50 transition-all duration-200">
            <div class="w-6 h-6 rounded-md flex items-center justify-center bg-danger/10 text-danger shrink-0">
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13H5v-2h14v2z"/>
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <span class="text-xs font-semibold text-text truncate block">{{ getItemLabel(item) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full gap-3 flex-wrap">
        <button @click="handleMarkAllForSync" 
                class="btn btn--primary btn--sm"
                :disabled="syncing || totalCount === 0">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
          </svg>
          {{ fullSyncText }}
        </button>
        <div class="flex items-center gap-2">
          <button @click="close" class="btn btn--secondary btn--sm">
            {{ $t('common.close') }}
          </button>
          <button v-if="pendingUpserts.length > 0 || pendingDeletions.length > 0" 
                  @click="handleSync" 
                  class="btn btn--primary btn--sm"
                  :disabled="syncing">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6 0 1.01-.25 1.97-.7 2.8l1.46 1.46C19.54 15.03 20 13.57 20 12c0-4.42-3.58-8-8-8zm0 14c-3.31 0-6-2.69-6-6 0-1.01.25-1.97.7-2.8L5.24 7.74C4.46 8.97 4 10.43 4 12c0 4.42 3.58 8 8 8v3l4-4-4-4v3z"/>
            </svg>
            {{ syncText }}
          </button>
        </div>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref } from 'vue'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  visible: { type: Boolean, default: false },
  pendingUpserts: { type: Array, default: () => [] },
  pendingDeletions: { type: Array, default: () => [] },
  syncing: { type: Boolean, default: false },
  totalCount: { type: Number, default: 0 },
  // i18n text props
  title: { type: String, required: true },
  upsertsTitle: { type: String, required: true },
  deletionsTitle: { type: String, required: true },
  emptyText: { type: String, required: true },
  fullSyncText: { type: String, required: true },
  syncText: { type: String, required: true },
  noLabelText: { type: String, default: '' },
  // Label field configuration
  labelField: { type: String, default: 'email' },
  fallbackField: { type: String, default: 'id' }
})

const emit = defineEmits(['update:visible', 'sync', 'mark-all-for-sync'])
const activeTab = ref('upserts')

const getItemLabel = (item) => {
  return item[props.labelField] || item[props.fallbackField] || props.noLabelText || item.id
}

const close = () => emit('update:visible', false)
const handleSync = () => { close(); emit('sync') }
const handleMarkAllForSync = () => emit('mark-all-for-sync')
</script>

