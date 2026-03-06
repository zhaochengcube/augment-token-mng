<template>
  <div :class="isPageMode ? 'h-full flex flex-col overflow-hidden' : 'modal-overlay'">
    <div :class="isPageMode ? 'h-full flex flex-col overflow-hidden bg-page' : ['modal-content', 'modal-content-shell']" @click.stop>
      <div :class="isPageMode ? 'px-6 py-5 border-b border-border shrink-0 bg-muted/30' : 'modal-header'">
        <h3 class="m-0 text-2xl font-bold text-text">{{ $t('hmeManager.title') }}</h3>
        <button v-if="!isPageMode" @click="$emit('close')" class="modal-close">&times;</button>
      </div>

      <div :class="isPageMode ? 'flex-1 overflow-y-auto p-6 space-y-6' : ['modal-body', 'modal-body-scroll']">

        <!-- Top Row: Cookie + Generate side by side -->
        <div class="grid grid-cols-2 gap-6">

          <!-- Cookie Section -->
          <section class="rounded-xl border border-border bg-muted/50 p-5 h-full">
            <div class="flex items-center gap-2 mb-3">
              <h4 class="m-0 text-base font-semibold text-text">{{ $t('hmeManager.cookie.title') }}</h4>
              <button @click="showCookieHelp = !showCookieHelp"
                class="inline-flex items-center justify-center w-5 h-5 rounded-full border border-border text-text-muted hover:text-accent hover:border-accent transition"
                :title="$t('hmeManager.cookie.helpTitle')">
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/>
                </svg>
              </button>
            </div>

            <div v-if="showCookieHelp" class="mb-3 rounded-lg border border-border bg-surface p-3 text-xs text-text-secondary leading-relaxed">
              {{ $t('hmeManager.cookie.helpText') }}
            </div>

            <div class="flex items-center gap-2 mb-3 flex-wrap">
              <span :class="[
                'badge badge--sm shrink-0',
                cookieStatus === 'valid' ? 'badge--success-tech' :
                cookieStatus === 'invalid' ? 'badge--danger-tech' :
                cookieStatus === 'loaded' ? 'badge--accent-tech' :
                'badge--warning-tech'
              ]">
                {{ $t(`hmeManager.cookie.status.${cookieStatus}`) }}
              </span>
              <div class="flex gap-2 ml-auto flex-wrap">
                <button @click="saveCookie" :disabled="!cookieInput.trim() || cookieSaving" class="btn btn--primary btn--sm">
                  <span v-if="cookieSaving" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
                  {{ $t('hmeManager.cookie.save') }}
                </button>
                <button @click="clearCookie" :disabled="cookieStatus === 'unset'" class="btn btn-tech-danger btn--sm">
                  {{ $t('hmeManager.cookie.clear') }}
                </button>
              </div>
            </div>

            <input
              type="text"
              v-model="cookieInput"
              :placeholder="$t('hmeManager.cookie.placeholder')"
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-text placeholder-text-muted focus:outline-none focus:border-accent transition"
            />
          </section>

          <!-- Generate Section -->
          <section class="rounded-xl border border-border bg-muted/50 p-5 h-full">
            <h4 class="m-0 mb-4 text-base font-semibold text-text">{{ $t('hmeManager.generate.title') }}</h4>
            <p class="mb-4 text-xs text-text-secondary">{{ $t('hmeManager.generate.limitHint') }}</p>

            <div class="flex items-center gap-4 flex-wrap">
              <div class="flex flex-col gap-1">
                <label class="text-xs font-medium text-text-secondary">{{ $t('hmeManager.generate.count') }}</label>
                <input v-model.number="generateCount" type="number" min="1" max="5"
                  class="w-20 rounded-lg border border-border bg-surface px-3 py-1.5 text-sm text-text focus:outline-none focus:border-accent transition" />
              </div>
              <div class="flex flex-col gap-1 flex-1 min-w-[120px]">
                <label class="text-xs font-medium text-text-secondary">{{ $t('hmeManager.generate.label') }}</label>
                <input v-model="generateLabel" type="text" :placeholder="$t('hmeManager.generate.labelPlaceholder')"
                  class="rounded-lg border border-border bg-surface px-3 py-1.5 text-sm text-text placeholder-text-muted focus:outline-none focus:border-accent transition" />
              </div>
              <button @click="generateEmails" :disabled="generateDisabled" class="btn btn--primary btn--sm">
                <span v-if="generating" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
                {{ generateBtnText }}
              </button>
            </div>
          </section>

        </div>

        <!-- List Section -->
        <section class="rounded-xl border border-border bg-muted/50 p-5">
          <div class="flex items-center gap-3 mb-4">
            <h4 class="m-0 text-base font-semibold text-text">{{ $t('hmeManager.list.title') }}</h4>
            <span class="text-sm text-text-muted">({{ filteredList.length }})</span>
          </div>

          <!-- Tabs -->
          <div class="flex items-center gap-4 mb-4">
            <div class="flex rounded-lg border border-border overflow-hidden">
              <button
                @click="listTab = true; loadList(true)"
                :class="['px-4 py-1.5 text-sm font-medium transition', listTab ? 'bg-accent text-text-inverse' : 'bg-surface text-text-secondary hover:bg-hover']">
                {{ $t('hmeManager.list.active') }}
              </button>
              <button
                @click="listTab = false; loadList(true)"
                :class="['px-4 py-1.5 text-sm font-medium transition', !listTab ? 'bg-accent text-text-inverse' : 'bg-surface text-text-secondary hover:bg-hover']">
                {{ $t('hmeManager.list.inactive') }}
              </button>
            </div>

            <div class="flex-1 min-w-0">
              <input v-model="searchKeyword" :placeholder="$t('hmeManager.list.searchPlaceholder')"
                class="w-full rounded-lg border border-border bg-surface px-3 py-1.5 text-sm text-text placeholder-text-muted focus:outline-none focus:border-accent transition" />
            </div>

            <button @click="loadList(true)" :disabled="listLoading" class="btn btn--secondary btn--sm">
              <span v-if="listLoading" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
              {{ $t('common.refresh') }}
            </button>
          </div>

          <!-- Batch Actions -->
          <div v-if="selectedIds.size > 0" class="flex items-center gap-3 mb-4 flex-wrap">
            <span class="text-sm text-text-secondary">{{ $t('hmeManager.batch.selected', { n: selectedIds.size }) }}</span>
            <button v-if="listTab" @click="batchDeactivate" :disabled="batchLoading" class="btn btn-tech-warning btn--sm">
              {{ $t('hmeManager.batch.deactivate') }}
            </button>
            <button v-if="!listTab" @click="batchDelete" :disabled="batchLoading" class="btn btn-tech-danger btn--sm">
              {{ $t('hmeManager.batch.delete') }}
            </button>
            <button v-if="listTab" @click="batchCleanup" :disabled="batchLoading" class="btn btn-tech-danger btn--sm">
              {{ $t('hmeManager.batch.cleanup') }}
            </button>
          </div>

          <!-- Table -->
          <div v-if="listLoading" class="text-center py-10 text-text-muted text-sm">
            <span class="btn-spinner btn-spinner--lg inline-block mb-3" aria-hidden="true"></span>
            <p>{{ $t('common.loading') }}</p>
          </div>

          <div v-else-if="emailList.length === 0" class="text-center py-10 text-text-muted text-sm">
            {{ $t('hmeManager.list.empty') }}
          </div>

          <div v-else class="overflow-x-auto rounded-lg border border-border">
            <table class="w-full text-sm">
              <thead>
                <tr class="bg-muted/70 text-left">
                  <th class="px-3 py-2 w-10">
                    <input type="checkbox" :checked="allSelected" @change="toggleSelectAll" class="accent-accent" />
                  </th>
                  <th class="px-3 py-2 font-medium text-text-secondary">{{ $t('hmeManager.list.labelCol') }}</th>
                  <th class="px-3 py-2 font-medium text-text-secondary">{{ $t('hmeManager.list.emailCol') }}</th>
                  <th class="px-3 py-2 font-medium text-text-secondary whitespace-nowrap">{{ $t('hmeManager.list.createdAtCol') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in pagedList" :key="item.anonymous_id"
                  class="border-t border-border hover:bg-hover/50 transition">
                  <td class="px-3 py-2">
                    <input type="checkbox" :checked="selectedIds.has(item.anonymous_id)"
                      @change="toggleSelect(item.anonymous_id)" class="accent-accent" />
                  </td>
                  <td class="px-3 py-2 text-text truncate max-w-[180px]">{{ item.label || '-' }}</td>
                  <td class="px-3 py-2 font-mono text-text-secondary truncate max-w-[260px] cursor-pointer hover:text-accent transition"
                    @click="copyText(item.hme)">{{ item.hme }}</td>
                  <td class="px-3 py-2 text-text-muted whitespace-nowrap">{{ formatDate(item.created_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Pagination -->
          <div v-if="totalPages > 1" class="flex items-center justify-center gap-3 mt-4">
            <button
              @click="currentPage--"
              :disabled="currentPage === 1"
              class="btn btn--secondary btn--sm disabled:opacity-40">
              ← {{ $t('pagination.prev') }}
            </button>
            <span class="text-sm text-text-secondary">
              {{ currentPage }} / {{ totalPages }}
            </span>
            <button
              @click="currentPage++"
              :disabled="currentPage === totalPages"
              class="btn btn--secondary btn--sm disabled:opacity-40">
              {{ $t('pagination.next') }} →            </button>
          </div>

        </section>

      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

defineProps({
  isPageMode: { type: Boolean, default: false }
})
defineEmits(['close'])

const { t } = useI18n()

// Cookie
const cookieInput = ref('')
const cookieStatus = ref('unset')
const cookieSaving = ref(false)
const showCookieHelp = ref(false)

// Generate
const generateCount = ref(5)
const generateLabel = ref('ATM HME')
const generating = ref(false)

// Cooldown: iCloud enforces max 5 emails per 30 minutes
const COOLDOWN_MS = 30 * 60 * 1000
const lastGenTimestamp = ref(0)

// List
const PAGE_SIZE = 20
const listTab = ref(true)
const searchKeyword = ref('')
const emailList = ref([])
const listLoading = ref(false)
const selectedIds = ref(new Set())
const batchLoading = ref(false)
const currentPage = ref(1)

const filteredList = computed(() => {
  const kw = searchKeyword.value.trim().toLowerCase()
  if (!kw) return emailList.value
  return emailList.value.filter(i =>
    (i.hme && i.hme.toLowerCase().includes(kw)) ||
    (i.label && i.label.toLowerCase().includes(kw))
  )
})

const totalPages = computed(() => Math.max(1, Math.ceil(filteredList.value.length / PAGE_SIZE)))
const pagedList = computed(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE
  return filteredList.value.slice(start, start + PAGE_SIZE)
})

watch(searchKeyword, () => { currentPage.value = 1 })

const allSelected = computed(() =>
  pagedList.value.length > 0 && pagedList.value.every(i => selectedIds.value.has(i.anonymous_id))
)

const normalizeTimestamp = (timestamp) => {
  if (!Number.isFinite(timestamp)) return 0
  return Math.abs(timestamp) < 1_000_000_000_000 ? timestamp * 1000 : timestamp
}

const updateLastGenTimestamp = (items) => {
  for (const item of items) {
    const ts = Number.isFinite(item?.create_timestamp)
      ? normalizeTimestamp(item.create_timestamp)
      : new Date(item?.created_at).getTime()
    if (ts > lastGenTimestamp.value) lastGenTimestamp.value = ts
  }
}

const getCooldownRemaining = () => {
  if (lastGenTimestamp.value === 0) return 0
  const elapsed = Date.now() - lastGenTimestamp.value
  const remaining = COOLDOWN_MS - elapsed
  return remaining > 0 ? remaining : 0
}

const formatCooldownWait = (ms) => {
  const totalSec = Math.ceil(ms / 1000)
  const min = Math.floor(totalSec / 60)
  const sec = totalSec % 60
  return `${String(min).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
}

const canUseRemoteCookie = computed(() =>
  cookieStatus.value === 'loaded' || cookieStatus.value === 'valid'
)

const generateDisabled = computed(() =>
  generating.value || !canUseRemoteCookie.value
)

const generateBtnText = computed(() => {
  if (generating.value) return t('hmeManager.generate.generating')
  return t('hmeManager.generate.btn')
})

const notify = (msg, type = 'info') => {
  if (window.$notify?.[type]) window.$notify[type](msg)
}

const doConfirm = async (opts) => {
  if (window.$confirm) return window.$confirm(opts)
  return confirm(opts.message || opts.title)
}

const copyText = async (text) => {
  try {
    await navigator.clipboard.writeText(text)
    notify(t('common.copySuccess'), 'success')
  } catch {
    notify(t('common.copyFailed'), 'error')
  }
}

const formatDate = (iso) => {
  if (!iso) return '-'
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}

const getErrorMessage = (error) => {
  if (typeof error === 'string') return error
  if (error?.message) return error.message
  return String(error)
}

const isCookieInvalidError = (error) => {
  const message = getErrorMessage(error).toLowerCase()
  return message.includes('cookie expired or invalid') ||
    message.includes('unexpected html response') ||
    message.includes('cookie is not set')
}

const setEmailList = (items) => {
  emailList.value = Array.isArray(items) ? items : []
  if (listTab.value) updateLastGenTimestamp(emailList.value)
}

// Cookie actions
const checkCookieState = async () => {
  await loadListLocal()

  try {
    const cookie = await invoke('hme_get_cookie')
    if (cookie) {
      cookieInput.value = cookie
      cookieStatus.value = 'loaded'
      await syncFromApi()
    } else {
      cookieStatus.value = 'unset'
    }
  } catch {
    cookieStatus.value = 'unset'
  }
}

const saveCookie = async () => {
  if (!cookieInput.value.trim()) return
  cookieSaving.value = true
  try {
    await invoke('hme_set_cookie', { cookie: cookieInput.value })
    cookieStatus.value = 'loaded'
    notify(t('hmeManager.messages.cookieSaved'), 'success')

    try {
      const valid = await invoke('hme_validate_cookie')
      cookieStatus.value = valid ? 'valid' : 'invalid'
      notify(valid ? t('hmeManager.messages.cookieValid') : t('hmeManager.messages.cookieInvalid'),
        valid ? 'success' : 'warning')
    } catch (e) {
      const errorMessage = getErrorMessage(e)
      if (isCookieInvalidError(errorMessage)) {
        cookieStatus.value = 'invalid'
        notify(t('hmeManager.messages.cookieInvalid'), 'warning')
      } else {
        cookieStatus.value = 'loaded'
        notify(`${t('hmeManager.messages.cookieValidateFailed')}: ${errorMessage}`, 'error')
      }
    }

    if (cookieStatus.value === 'invalid') {
      await loadListLocal()
    } else {
      await syncFromApi()
    }
  } catch (e) {
    notify(`${t('hmeManager.messages.cookieSaveFailed')}: ${e}`, 'error')
  } finally {
    cookieSaving.value = false
  }
}

const clearCookie = async () => {
  try {
    await invoke('hme_clear_cookie')
    cookieInput.value = ''
    cookieStatus.value = 'unset'
    notify(t('hmeManager.messages.cookieCleared'), 'success')
  } catch (e) {
    notify(`${t('hmeManager.messages.cookieClearFailed')}: ${e}`, 'error')
  }
}

// Generate actions
const generateEmails = async () => {
  const cooldownRemaining = getCooldownRemaining()
  if (cooldownRemaining > 0) {
    notify(t('hmeManager.messages.generateRateLimited', {
      time: formatCooldownWait(cooldownRemaining)
    }), 'warning')
    return
  }

  generating.value = true
  try {
    const result = await invoke('hme_generate', {
      count: generateCount.value,
      label: generateLabel.value || null
    })
    if (result.generated.length > 0) {
      lastGenTimestamp.value = Date.now()
      notify(t('hmeManager.messages.generateSuccess', { n: result.generated.length }), 'success')
      await syncFromApi()
    }
    if (result.failed > 0) {
      notify(t('hmeManager.messages.generatePartialFail', { n: result.failed }), 'warning')
    }
  } catch (e) {
    notify(`${t('hmeManager.messages.generateFailed')}: ${e}`, 'error')
  } finally {
    generating.value = false
  }
}

// List actions
const loadListLocal = async (resetPage = false) => {
  if (resetPage) currentPage.value = 1
  listLoading.value = true
  selectedIds.value = new Set()
  try {
    const items = await invoke('hme_list_local', {
      active: listTab.value,
      search: null
    })
    setEmailList(items)
    return items
  } catch (e) {
    emailList.value = []
    return []
  } finally {
    listLoading.value = false
  }
}

const syncFromApi = async (resetPage = false) => {
  if (resetPage) currentPage.value = 1
  listLoading.value = true
  selectedIds.value = new Set()
  try {
    const items = await invoke('hme_sync', {
      active: listTab.value,
      search: null
    })
    setEmailList(items)
    cookieStatus.value = 'valid'
    return items
  } catch (e) {
    const errorMessage = getErrorMessage(e)
    if (isCookieInvalidError(errorMessage)) {
      cookieStatus.value = 'invalid'
    } else if (cookieStatus.value !== 'unset') {
      cookieStatus.value = 'loaded'
    }

    notify(`${t('hmeManager.messages.listFailed')}: ${errorMessage}`, 'error')
    const localItems = await loadListLocal(resetPage)
    if (localItems.length > 0) {
      notify(t('hmeManager.messages.listFallbackLocal'), 'warning')
    }
    return localItems
  } finally {
    listLoading.value = false
  }
}

const loadList = async (resetPage = false) => {
  if (canUseRemoteCookie.value) {
    await syncFromApi(resetPage)
  } else {
    await loadListLocal(resetPage)
  }
}

const toggleSelect = (id) => {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

const toggleSelectAll = () => {
  if (allSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(pagedList.value.map(i => i.anonymous_id))
  }
}

const getSelectedArray = () => [...selectedIds.value]

const batchDeactivate = async () => {
  const ok = await doConfirm({
    title: t('hmeManager.confirm.deactivateTitle'),
    message: t('hmeManager.confirm.deactivateMessage', { n: selectedIds.value.size })
  })
  if (!ok) return
  batchLoading.value = true
  try {
    const res = await invoke('hme_deactivate', { anonymousIds: getSelectedArray() })
    notify(t('hmeManager.messages.deactivateSuccess', { n: res.success }), 'success')
    if (res.failed > 0) notify(t('hmeManager.messages.batchPartialFail', { n: res.failed }), 'warning')
    await loadListLocal()
  } catch (e) {
    notify(`${t('hmeManager.messages.deactivateFailed')}: ${e}`, 'error')
  } finally {
    batchLoading.value = false
  }
}

const batchDelete = async () => {
  const ok = await doConfirm({
    title: t('hmeManager.confirm.deleteTitle'),
    message: t('hmeManager.confirm.deleteMessage', { n: selectedIds.value.size })
  })
  if (!ok) return
  batchLoading.value = true
  try {
    const res = await invoke('hme_delete', { anonymousIds: getSelectedArray() })
    notify(t('hmeManager.messages.deleteSuccess', { n: res.success }), 'success')
    if (res.failed > 0) notify(t('hmeManager.messages.batchPartialFail', { n: res.failed }), 'warning')
    await loadListLocal()
  } catch (e) {
    notify(`${t('hmeManager.messages.deleteFailed')}: ${e}`, 'error')
  } finally {
    batchLoading.value = false
  }
}

const batchCleanup = async () => {
  const ok = await doConfirm({
    title: t('hmeManager.confirm.cleanupTitle'),
    message: t('hmeManager.confirm.cleanupMessage', { n: selectedIds.value.size })
  })
  if (!ok) return
  batchLoading.value = true
  try {
    const res = await invoke('hme_cleanup', { anonymousIds: getSelectedArray() })
    notify(t('hmeManager.messages.cleanupSuccess', { n: res.success }), 'success')
    if (res.failed > 0) notify(t('hmeManager.messages.batchPartialFail', { n: res.failed }), 'warning')
    await loadListLocal()
  } catch (e) {
    notify(`${t('hmeManager.messages.cleanupFailed')}: ${e}`, 'error')
  } finally {
    batchLoading.value = false
  }
}

onMounted(() => {
  checkCookieState()
})
</script>

