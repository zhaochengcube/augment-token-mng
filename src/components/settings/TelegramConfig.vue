<template>
  <BaseModal
    :visible="true"
    :title="$t('telegram.title')"
    :body-scroll="false"
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-5">
      <!-- 启用 Telegram 通知开关 -->
      <div class="flex flex-col gap-2">
        <label class="flex items-center cursor-pointer font-medium text-text">
          <input
            type="checkbox"
            v-model="config.enabled"
            :disabled="isLoading"
            class="mr-2.5 w-[18px] h-[18px] cursor-pointer accent-accent disabled:cursor-not-allowed"
          >
          <span>{{ $t('telegram.enableNotification') }}</span>
        </label>
        <p class="text-xs text-text-muted">{{ $t('telegram.description') }}</p>
      </div>

      <!-- Bot Token -->
      <div class="form-group !mb-0">
        <label for="botToken" class="label">
          {{ $t('telegram.botToken') }}:
        </label>
        <input
          id="botToken"
          v-model="config.bot_token"
          type="password"
          class="input font-mono"
          :placeholder="$t('telegram.placeholders.botToken')"
          :disabled="!config.enabled || isLoading"
        >
        <small class="text-[12px] text-text-muted mt-1 block">{{ $t('telegram.botTokenHelp') }}</small>
      </div>

      <!-- Chat ID -->
      <div class="form-group !mb-0">
        <label for="chatId" class="label">
          {{ $t('telegram.chatId') }}:
        </label>
        <input
          id="chatId"
          v-model="config.chat_id"
          type="text"
          class="input font-mono"
          :placeholder="$t('telegram.placeholders.chatId')"
          :disabled="!config.enabled || isLoading"
        >
        <small class="text-[12px] text-text-muted mt-1 block">{{ $t('telegram.chatIdHelp1') }}</small>
        <small class="text-[12px] text-text-muted mt-0.5 block">{{ $t('telegram.chatIdHelp2') }}</small>
      </div>

      <!-- 提醒天数配置 -->
      <div class="form-group !mb-0">
        <label class="label">{{ $t('telegram.notifyDays') }}:</label>
        <div class="flex flex-wrap gap-3 mt-2">
          <label
            v-for="day in availableDays"
            :key="day"
            class="flex items-center cursor-pointer"
          >
            <input
              type="checkbox"
              :checked="config.notify_days.includes(day)"
              @change="toggleNotifyDay(day)"
              :disabled="!config.enabled || isLoading"
              class="mr-2 w-4 h-4 cursor-pointer accent-accent disabled:cursor-not-allowed"
            >
            <span class="text-sm text-text">{{ $t('telegram.daysBefore', { days: day }) }}</span>
          </label>
        </div>
        <small class="text-[12px] text-text-muted mt-2 block">{{ $t('telegram.notifyDaysHelp') }}</small>
      </div>

      <!-- 检查间隔配置 -->
      <div class="form-group !mb-0">
        <label for="checkInterval" class="label">
          {{ $t('telegram.checkInterval') }}:
        </label>
        <div class="flex items-center gap-2">
          <input
            id="checkInterval"
            v-model.number="config.check_interval_hours"
            type="number"
            min="1"
            max="24"
            class="input w-24 font-mono"
            :disabled="!config.enabled || isLoading"
          >
          <span class="text-sm text-text">{{ $t('telegram.hours') }}</span>
        </div>
        <small class="text-[12px] text-text-muted mt-1 block">{{ $t('telegram.checkIntervalHelp') }}</small>
      </div>
    </div>

    <template #footer>
      <div class="flex gap-2.5 items-center mr-auto">
        <button
          @click="testConnection"
          class="btn btn--secondary"
          :disabled="!canTest || isTesting || isLoading"
        >
          <span v-if="isTesting" class="btn-spinner" aria-hidden="true"></span>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
          </svg>
          {{ $t('telegram.testConnection') }}
        </button>
        <button
          @click="manualCheck"
          class="btn btn--secondary"
          :disabled="!canTest || isChecking || isLoading"
        >
          <span v-if="isChecking" class="btn-spinner" aria-hidden="true"></span>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3h-4.18C14.4 1.84 13.3 1 12 1c-1.3 0-2.4.84-2.82 2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-7 0c.55 0 1 .45 1 1s-.45 1-1 1-1-.45-1-1 .45-1 1-1zm2 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z"/>
          </svg>
          {{ $t('telegram.manualCheck') }}
        </button>
      </div>
      <div class="flex gap-2.5">
        <button @click="$emit('close')" class="btn btn--secondary">
          {{ $t('common.cancel') }}
        </button>
        <button
          @click="saveConfig"
          class="btn btn--primary"
          :disabled="isSaving || isLoading"
        >
          <span v-if="isSaving" class="btn-spinner" aria-hidden="true"></span>
          {{ $t('common.save') }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

const emit = defineEmits(['close', 'saved'])
const { t } = useI18n()

// 配置状态
const config = ref({
  bot_token: '',
  chat_id: '',
  enabled: false,
  notify_days: [15, 7, 3],
  check_interval_hours: 6
})

// 可选的提醒天数
const availableDays = [30, 15, 7, 3, 1]

// 加载状态
const isLoading = ref(true)
const isSaving = ref(false)
const isTesting = ref(false)
const isChecking = ref(false)

// 计算属性
const canTest = computed(() => {
  return config.value.bot_token.trim() && config.value.chat_id.trim()
})

// 切换提醒天数
const toggleNotifyDay = (day) => {
  const index = config.value.notify_days.indexOf(day)
  if (index > -1) {
    config.value.notify_days.splice(index, 1)
  } else {
    config.value.notify_days.push(day)
    config.value.notify_days.sort((a, b) => b - a)
  }
}

// 加载配置
const loadConfig = async () => {
  isLoading.value = true
  try {
    const savedConfig = await invoke('load_telegram_config')
    if (savedConfig) {
      config.value = {
        ...config.value,
        ...savedConfig
      }
    }
  } catch (error) {
    console.error('Failed to load telegram config:', error)
  } finally {
    isLoading.value = false
  }
}

// 保存配置
const saveConfig = async () => {
  isSaving.value = true
  try {
    await invoke('save_telegram_config', { config: config.value })
    window.$notify?.success(t('telegram.saveSuccess'))
    emit('saved')
    emit('close')
  } catch (error) {
    console.error('Failed to save telegram config:', error)
    window.$notify?.error(t('telegram.saveFailed'))
  } finally {
    isSaving.value = false
  }
}

// 测试连接
const testConnection = async () => {
  isTesting.value = true
  try {
    await invoke('test_telegram_connection_cmd', {
      botToken: config.value.bot_token,
      chatId: config.value.chat_id
    })
    window.$notify?.success(t('telegram.testSuccess'))
  } catch (error) {
    console.error('Failed to test telegram connection:', error)
    window.$notify?.error(`${t('telegram.testFailed')}: ${error}`)
  } finally {
    isTesting.value = false
  }
}

// 手动触发检查
const manualCheck = async () => {
  isChecking.value = true
  try {
    // 先保存当前配置
    await invoke('save_telegram_config', { config: config.value })
    // 然后触发检查
    await invoke('check_subscriptions_expiry')
    window.$notify?.success(t('telegram.checkComplete'))
  } catch (error) {
    console.error('Failed to check subscriptions:', error)
    window.$notify?.error(`${t('telegram.checkFailed')}: ${error}`)
  } finally {
    isChecking.value = false
  }
}

onMounted(() => {
  loadConfig()
})
</script>
