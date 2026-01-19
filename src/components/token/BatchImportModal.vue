<template>
  <BaseModal
    :visible="visible"
    :title="$t('tokenList.batchImportTitle')"
    :show-close="true"
    :close-on-overlay="!isImporting"
    :close-on-esc="!isImporting"
    :body-scroll="true"
    modal-class="max-w-[560px]"
    @close="handleClose"
  >
    <template #header>
      <h3 class="modal-title">{{ $t('tokenList.batchImportTitle') }}</h3>
    </template>

    <!-- Tab Navigation -->
    <div class="mb-4 flex gap-1 rounded-lg border border-border bg-muted p-1">
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-all',
          activeTab === 'session'
            ? 'bg-surface text-text shadow-sm'
            : 'text-text-muted hover:text-text'
        ]"
        @click="activeTab = 'session'"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm0 4c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm6 12H6v-1.4c0-2 4-3.1 6-3.1s6 1.1 6 3.1V19z" />
        </svg>
        {{ $t('tokenList.sessionImportTab') }}
      </button>
      <button
        :class="[
          'flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-all',
          activeTab === 'token'
            ? 'bg-surface text-text shadow-sm'
            : 'text-text-muted hover:text-text'
        ]"
        @click="activeTab = 'token'"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
        </svg>
        {{ $t('tokenList.tokenImportTab') }}
      </button>
    </div>

    <!-- Session Tab Content -->
    <div v-if="activeTab === 'session'">
      <p class="mb-4 text-sm text-text-secondary">{{ $t('tokenList.sessionImportMessage') }}</p>

      <!-- Session Input List -->
      <div class="flex max-h-[300px] flex-col gap-2 overflow-y-auto">
        <div
          v-for="(input, index) in sessionInputs"
          :key="input.id"
          class="flex items-center gap-2"
        >
          <span class="w-6 text-right text-sm text-text-muted">{{ index + 1 }}.</span>
          <input
            v-model="input.value"
            type="text"
            :placeholder="$t('tokenList.sessionInputPlaceholder')"
            class="input flex-1"
          />
          <button
            @click="removeSessionInput(input.id)"
            :disabled="sessionInputs.length <= 1"
            class="btn btn--ghost btn--icon-sm text-text-muted hover:text-danger disabled:opacity-30"
            v-tooltip="$t('tokenList.deleteInput')"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Add More Button -->
      <button
        @click="addSessionInput"
        @contextmenu="handleContextMenu($event, 'session')"
        class="mt-3 flex w-full items-center justify-center gap-2 rounded-lg border border-dashed border-border py-2 text-sm text-text-muted transition-colors hover:border-accent hover:text-accent"
      >
        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
        </svg>
        {{ $t('tokenList.addMore') }}
      </button>
    </div>

    <!-- Token Tab Content -->
    <div v-else-if="activeTab === 'token'">
      <p class="mb-4 text-sm text-text-secondary">{{ $t('tokenList.tokenImportMessage') }}</p>

      <!-- Format Info -->
      <div class="mb-4 rounded-lg border border-border bg-muted/50 p-3">
        <div class="mb-1 text-sm font-medium text-text">{{ $t('tokenList.tokenFormatTitle') }}</div>
        <p class="mb-2 text-xs text-text-muted">{{ $t('tokenList.tokenFormatDesc') }}</p>
        <button
          @click="fillTokenTemplate()"
          @contextmenu="handleContextMenu($event, 'token')"
          class="text-xs font-medium text-accent hover:underline"
        >
          {{ $t('tokenList.fillTemplate') }}
        </button>
      </div>

      <!-- JSON Input -->
      <textarea
        v-model="importJsonText"
        rows="8"
        class="input w-full resize-none font-mono text-xs"
        @input="validateImportJson"
      ></textarea>

      <!-- Errors -->
      <div v-if="importErrors.length > 0" class="mt-3 rounded-lg border border-danger/30 bg-danger/10 p-3">
        <div class="mb-2 flex items-center gap-2 text-sm font-medium text-danger">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
          </svg>
          {{ $t('tokenList.importErrorsFound', { count: importErrors.length }) }}
        </div>
        <ul class="max-h-24 list-inside list-disc overflow-y-auto text-xs text-danger/80">
          <li v-for="(error, index) in importErrors" :key="index">{{ error }}</li>
        </ul>
      </div>

      <!-- Preview -->
      <div v-if="importPreview.length > 0" class="mt-3 rounded-lg border border-success/30 bg-success/10 p-3">
        <div class="flex items-center gap-2 text-sm font-medium text-success">
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
          </svg>
          {{ $t('tokenList.importPreviewReady', { count: importPreview.length }) }}
        </div>
      </div>
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary" :disabled="isImporting">
        {{ $t('common.cancel') }}
      </button>
      <button
        @click="handleImport"
        class="btn btn--primary"
        :disabled="isImporting || !canImport"
      >
        <span v-if="isImporting" class="btn-spinner btn-spinner--sm" aria-hidden="true"></span>
        {{ importButtonText }}
      </button>
    </template>
  </BaseModal>

  <!-- Context Menu for Session Default Count -->
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="showContextMenu" class="fixed inset-0 z-[9998]" @click="closeContextMenu">
        <div
          class="dropdown-menu fixed z-[9999] w-[220px]"
          :style="contextMenuStyle"
          @click.stop
        >
          <div class="px-3 py-2 text-xs font-medium text-text-muted border-b border-border mb-1">
            {{ $t('tokenList.selectFillCount') }}
          </div>
          <div class="flex items-center gap-2 px-2 py-1.5">
            <input
              v-model.number="customFillCount"
              type="number"
              min="1"
              max="20"
              :placeholder="$t('tokenList.customCount')"
              class="input w-16 text-center text-sm py-1"
              @click.stop
            />
            <button @click="setDefaultCountFromInput" class="btn btn--primary btn--sm flex-1">
              {{ $t('tokenList.setAsDefault') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'

const { t } = useI18n()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  defaultInputCount: {
    type: Number,
    default: 3
  }
})

const emit = defineEmits(['update:visible', 'import'])

// Tab state
const activeTab = ref('session')

// Session inputs
const sessionInputs = ref([])
let nextSessionInputId = 1

// Token JSON import
const importJsonText = ref('')
const importErrors = ref([])
const importPreview = ref([])

// Loading state
const isImporting = ref(false)

// Context Menu state
const showContextMenu = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const customFillCount = ref(3)

// Context Menu style - 固定宽度，防止超出屏幕
const MENU_WIDTH = 220
const contextMenuStyle = computed(() => {
  const x = Math.min(contextMenuPosition.value.x, window.innerWidth - MENU_WIDTH - 10)
  const y = contextMenuPosition.value.y
  return {
    left: x + 'px',
    top: y + 'px'
  }
})

// localStorage 配置键名
const STORAGE_KEY_DEFAULT_INPUT_COUNT = 'atm-default-session-input-count'

// Initialize session inputs
const initializeSessionInputs = (count) => {
  const inputs = []
  for (let i = 1; i <= count; i++) {
    inputs.push({ id: i, value: '' })
  }
  sessionInputs.value = inputs
  nextSessionInputId = count + 1
}

// Watch visibility to reset state
watch(() => props.visible, (isVisible) => {
  if (isVisible) {
    activeTab.value = 'session'
    initializeSessionInputs(props.defaultInputCount)
    importJsonText.value = '[\n  \n]'
    importErrors.value = []
    importPreview.value = []
  }
})

// Session input methods
const addSessionInput = () => {
  sessionInputs.value.push({ id: nextSessionInputId++, value: '' })
}

const removeSessionInput = (id) => {
  if (sessionInputs.value.length <= 1) return
  sessionInputs.value = sessionInputs.value.filter(input => input.id !== id)
}

// Valid session count
const validSessionCount = computed(() => {
  return sessionInputs.value.filter(input => input.value.trim()).length
})

// Can import check
const canImport = computed(() => {
  if (activeTab.value === 'session') {
    return validSessionCount.value > 0
  }
  return importPreview.value.length > 0
})

// Import button text
const importButtonText = computed(() => {
  if (isImporting.value) {
    return t('tokenList.importing')
  }
  if (activeTab.value === 'session') {
    return t('tokenList.batchAdd', { count: validSessionCount.value })
  }
  return t('tokenList.confirmImport')
})

// Fill token template
const fillTokenTemplate = (count = 1) => {
  const tokens = []
  for (let i = 0; i < count; i++) {
    tokens.push({
      access_token: i === 0 ? 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...' : '',
      tenant_url: i === 0 ? 'https://example.com' : '',
      email_note: ''
    })
  }
  importJsonText.value = JSON.stringify(tokens, null, 2)
  validateImportJson()
}

// Validate import JSON
const validateImportJson = () => {
  importErrors.value = []
  importPreview.value = []

  const text = importJsonText.value.trim()
  if (!text) return

  try {
    const data = JSON.parse(text)
    if (!Array.isArray(data)) {
      importErrors.value.push(t('tokenList.importErrorNotArray'))
      return
    }

    const validTokens = []
    data.forEach((item, index) => {
      if (!item.access_token || typeof item.access_token !== 'string') {
        importErrors.value.push(t('tokenList.importErrorMissingToken', { index: index + 1 }))
      } else {
        validTokens.push(item)
      }
    })

    importPreview.value = validTokens
  } catch (e) {
    importErrors.value.push(t('tokenList.importErrorInvalidJson'))
  }
}

// Handle import
const handleImport = async () => {
  isImporting.value = true
  try {
    if (activeTab.value === 'session') {
      const sessions = sessionInputs.value
        .filter(input => input.value.trim())
        .map(input => input.value.trim())
      await emit('import', { type: 'session', data: sessions })
    } else {
      await emit('import', { type: 'token', data: importPreview.value })
    }
  } finally {
    isImporting.value = false
    emit('update:visible', false)
  }
}

// Handle close
const handleClose = () => {
  if (isImporting.value) return
  emit('update:visible', false)
}

// Context Menu handlers
const handleContextMenu = (event) => {
  event.preventDefault()
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  showContextMenu.value = true
}

const closeContextMenu = () => {
  showContextMenu.value = false
}

const setDefaultCountFromInput = () => {
  const count = parseInt(customFillCount.value)
  if (count >= 1 && count <= 20) {
    try {
      localStorage.setItem(STORAGE_KEY_DEFAULT_INPUT_COUNT, count.toString())
      // 重新初始化 session inputs
      initializeSessionInputs(count)
      window.$notify?.success(t('tokenList.defaultCountSaved', { count }))
    } catch (e) {
      window.$notify?.error(t('tokenList.saveDefaultFailed'))
    }
  }
  closeContextMenu()
}
</script>
