<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm" @mousedown.self="$emit('close')">
    <div class="card w-full max-w-[480px] rounded-xl p-6 shadow-2xl hover:translate-y-0">
      <!-- Header -->
      <div class="mb-5 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" class="text-accent">
            <path d="M17 3H7c-1.1 0-1.99.9-1.99 2L5 21l7-3 7 3V5c0-1.1-.9-2-2-2z"/>
          </svg>
          <h3 class="text-base font-semibold text-text">{{ $t('raindrop.title') }}</h3>
        </div>
        <button @click="$emit('close')" class="btn btn--icon-sm btn--ghost">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <!-- Description -->
      <p class="mb-4 text-xs text-text-muted">{{ $t('raindrop.description') }}</p>

      <!-- Token Input -->
      <div class="mb-4 flex items-center gap-2">
        <input
          v-model="tokenInput"
          type="password"
          :placeholder="config ? config.token_preview : $t('raindrop.tokenPlaceholder')"
          class="flex-1 rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text outline-none transition-colors placeholder:text-text-muted focus:border-accent"
        />
        <button
          @click="handleSaveToken"
          :disabled="!tokenInput.trim() || isSaving"
          class="btn btn--sm btn--primary"
        >
          <span v-if="isSaving" class="btn-spinner" aria-hidden="true"></span>
          {{ $t('raindrop.saveToken') }}
        </button>
        <button
          v-if="config"
          @click="handleDeleteConfig"
          class="btn btn--sm btn--ghost text-red-500 hover:text-red-600"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </div>

      <!-- Sync Section (only when configured) -->
      <template v-if="config">
        <div class="mb-4 border-t border-border pt-4">
          <!-- Last Sync Info -->
          <div class="mb-3 text-xs text-text-muted">
            {{ config.last_sync_at ? $t('raindrop.lastSync') + ': ' + formatTime(config.last_sync_at) : $t('raindrop.neverSynced') }}
          </div>

          <!-- Sync Buttons -->
          <div class="flex items-center gap-2">
            <button
              @click="handleSync(false)"
              :disabled="isSyncing"
              class="btn btn--sm btn--primary"
            >
              <span v-if="isSyncing" class="btn-spinner" aria-hidden="true"></span>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              {{ $t('raindrop.sync') }}
            </button>
            <button
              @click="handleSync(true)"
              :disabled="isSyncing"
              class="btn btn--sm btn--secondary"
            >
              {{ $t('raindrop.fullSync') }}
            </button>
          </div>
        </div>

        <!-- Sync Result -->
        <div v-if="syncResult" class="rounded-md border border-border bg-muted/50 px-3 py-2 text-xs text-text-muted">
          {{ $t('raindrop.syncResult', {
            total: syncResult.total_fetched,
            created: syncResult.created,
            updated: syncResult.updated,
            skipped: syncResult.skipped
          }) }}
        </div>
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()
const emit = defineEmits(['close', 'synced'])

const tokenInput = ref('')
const config = ref(null)
const isSaving = ref(false)
const isSyncing = ref(false)
const syncResult = ref(null)

const loadConfig = async () => {
  try {
    config.value = await invoke('raindrop_load_config')
  } catch {
    config.value = null
  }
}

const handleSaveToken = async () => {
  const token = tokenInput.value.trim()
  if (!token) return
  isSaving.value = true
  try {
    await invoke('raindrop_save_config', { token })
    tokenInput.value = ''
    await loadConfig()
    window.$notify?.success($t('raindrop.tokenSaved'))
  } catch (error) {
    window.$notify?.error($t('raindrop.tokenFailed') + ': ' + error)
  } finally {
    isSaving.value = false
  }
}

const handleDeleteConfig = async () => {
  try {
    await invoke('raindrop_delete_config')
    config.value = null
    syncResult.value = null
    window.$notify?.success($t('raindrop.configDeleted'))
  } catch (error) {
    console.error('Failed to delete raindrop config:', error)
  }
}

const handleSync = async (full = false) => {
  isSyncing.value = true
  syncResult.value = null
  try {
    const result = await invoke(full ? 'raindrop_full_sync' : 'raindrop_sync')
    syncResult.value = result
    await loadConfig()
    window.$notify?.success($t('raindrop.syncSuccess'))
    emit('synced', result)
  } catch (error) {
    window.$notify?.error($t('raindrop.syncFailed') + ': ' + error)
  } finally {
    isSyncing.value = false
  }
}

const formatTime = (isoStr) => {
  try {
    return new Date(isoStr).toLocaleString()
  } catch {
    return isoStr
  }
}

onMounted(loadConfig)
</script>
