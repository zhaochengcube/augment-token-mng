<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.cursor.addAccountDialog.title')"
    :show-close="true"
    :close-on-overlay="!isLoading && !showConfirmDialog"
    :close-on-esc="!isLoading && !showConfirmDialog"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="handleClose"
  >
    <!-- Session Token 方式 -->
    <div class="animate-fade-in">
      <!-- Info -->
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-accent/20 bg-accent/10 p-4">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <p class="text-[13px] leading-relaxed text-text-secondary">{{ $t('platform.cursor.addAccountDialog.sessionInfo') }}</p>
      </div>

      <div class="form-group mb-0">
        <label class="label">{{ $t('platform.cursor.addAccountDialog.sessionToken') }}</label>
        <textarea
          v-model="sessionToken"
          :placeholder="$t('platform.cursor.addAccountDialog.sessionTokenPlaceholder')"
          class="input resize-none"
          rows="4"
          :disabled="isLoading"
        ></textarea>
        <p class="mt-1.5 text-xs text-text-muted">
          {{ $t('platform.cursor.addAccountDialog.sessionTokenHint') }}
        </p>
      </div>
    </div>

    <!-- Error Message -->
    <div v-if="error" class="mt-4 flex items-center gap-2 rounded-lg border border-danger/30 bg-danger/10 p-3 text-[13px] text-danger">
      <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      {{ error }}
    </div>

    <!-- 覆盖确认对话框 -->
    <div v-if="showConfirmDialog" class="mt-4 rounded-lg border border-warning/30 bg-warning/10 p-4">
      <div class="flex items-start gap-3">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-warning" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        <div class="flex-1">
          <p class="text-sm font-medium text-text">{{ $t('platform.cursor.addAccountDialog.accountExists') }}</p>
          <p class="mt-1 text-xs text-text-secondary">
            {{ $t('platform.cursor.addAccountDialog.accountExistsHint', { email: userInfo?.email }) }}
          </p>
        </div>
      </div>
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary" :disabled="isLoading">
        {{ showConfirmDialog ? $t('common.cancel') : $t('common.cancel') }}
      </button>
      <button
        v-if="!showConfirmDialog"
        @click="handleAdd"
        class="btn btn--primary"
        :disabled="!canSubmit || isLoading"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">
            {{ $t('platform.cursor.addAccountDialog.add') }}
          </span>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
      </button>
      <button
        v-else
        @click="handleConfirmAdd"
        class="btn btn--primary"
        :disabled="isLoading"
      >
        <span class="relative inline-flex items-center justify-center">
          <span :style="{ visibility: isLoading ? 'hidden' : 'visible' }">
            {{ $t('platform.cursor.addAccountDialog.overwrite') }}
          </span>
          <span v-if="isLoading" class="btn-spinner absolute inset-0 m-auto" aria-hidden="true"></span>
        </span>
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()

const props = defineProps({
  // 现有账号列表，用于检查邮箱重复
  existingAccounts: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['close', 'added'])

const handleClose = () => {
  if (isLoading.value) return
  emit('close')
}

const sessionToken = ref('')
const isLoading = ref(false)
const error = ref('')
const showConfirmDialog = ref(false)
const userInfo = ref(null)
const existingAccountId = ref(null)

const canSubmit = computed(() => {
  return sessionToken.value.trim().length > 0
})

// 检查邮箱是否已存在
const checkEmailExists = (email) => {
  if (!props.existingAccounts || props.existingAccounts.length === 0) {
    return null
  }
  const normalizedEmail = email.toLowerCase().trim()
  return props.existingAccounts.find(acc => acc.email.toLowerCase().trim() === normalizedEmail)
}

const handleAdd = async () => {
  if (!canSubmit.value) return

  error.value = ''
  isLoading.value = true

  try {
    // 1. 先获取用户信息
    const info = await invoke('cursor_get_user_info_from_session', {
      sessionToken: sessionToken.value.trim()
    })
    userInfo.value = info

    // 2. 检查邮箱是否已存在
    const existing = checkEmailExists(info.email)
    if (existing) {
      existingAccountId.value = existing.id
      showConfirmDialog.value = true
      isLoading.value = false
      return
    }

    // 3. 邮箱不存在，直接添加
    await confirmAddAccount()
  } catch (err) {
    console.error('Get user info error:', err)
    error.value = err?.message || err || $t('platform.cursor.addAccountDialog.addFailed')
  } finally {
    if (!showConfirmDialog.value) {
      isLoading.value = false
    }
  }
}

const handleConfirmAdd = async () => {
  error.value = ''
  isLoading.value = true

  try {
    await confirmAddAccount()
  } catch (err) {
    console.error('Confirm add error:', err)
    error.value = err?.message || err || $t('platform.cursor.addAccountDialog.addFailed')
    isLoading.value = false
  }
}

const confirmAddAccount = async () => {
  if (existingAccountId.value) {
    // 覆盖已有账号的 token
    const account = await invoke('cursor_refresh_account_tokens', {
      accountId: existingAccountId.value,
      sessionToken: sessionToken.value.trim()
    })
    emit('added', account)
  } else {
    // 添加新账号
    const account = await invoke('cursor_add_account_with_session', {
      sessionToken: sessionToken.value.trim()
    })
    emit('added', account)
  }
}
</script>

<style scoped>
.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
