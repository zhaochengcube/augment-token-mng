<template>
  <tr class="border-b border-border hover:bg-hover transition-colors">
    <!-- 网站名称 -->
    <td class="td">
      <span class="font-medium text-text">{{ subscription.website }}</span>
    </td>
    
    <!-- 网站地址 -->
    <td class="td">
      <a
        v-if="subscription.website_url"
        :href="normalizedUrl"
        class="text-accent no-underline hover:underline"
        @click.stop.prevent="openExternalUrl"
      >{{ displayUrl }}</a>
      <span v-else class="text-text-muted">-</span>
    </td>

    <!-- 过期时间 -->
    <td class="td">
      <span :class="expiryStatusClass">{{ formattedExpiryDate }}</span>
    </td>
    
    <!-- 费用 -->
    <td class="td">
      <span class="text-text">{{ formattedCost || '-' }}</span>
    </td>
    
    <!-- 备注 -->
    <td class="td">
      <span
        v-if="subscription.notes"
        class="text-text-secondary truncate block max-w-[200px] cursor-pointer hover:text-text"
        :title="subscription.notes"
        v-tooltip="$t('common.copy')"
        @click.stop="copyNotes"
      >
        {{ subscription.notes }}
      </span>
      <span v-else class="text-text-muted">-</span>
    </td>

    <!-- 标签 -->
    <td class="td">
      <span
        v-if="subscription.tag"
        class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
        :style="{ backgroundColor: subscription.tag_color || '#e5e5e5', color: getContrastColor(subscription.tag_color) }"
      >
        {{ subscription.tag }}
      </span>
      <span v-else class="text-text-muted">-</span>
    </td>
    
    <!-- 操作 -->
    <td class="td text-center">
      <div class="flex items-center justify-center gap-1">
        <button class="btn btn--icon-sm btn--ghost" @click="$emit('edit', subscription)" v-tooltip="$t('common.edit')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>
        <button class="btn btn--icon-sm btn--ghost text-danger" @click="$emit('delete', subscription)" v-tooltip="$t('common.delete')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </div>
    </td>
  </tr>
</template>

<script setup>
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t: $t, locale } = useI18n()

const props = defineProps({
  subscription: {
    type: Object,
    required: true
  }
})

defineEmits(['edit', 'delete'])

const copyNotes = async () => {
  if (!props.subscription.notes) return
  try {
    await navigator.clipboard.writeText(props.subscription.notes)
    window.$notify?.success($t('common.copySuccess'))
  } catch (error) {
    window.$notify?.error($t('common.copyFailed'))
  }
}

// 日期格式化 locale 映射
const dateLocale = computed(() => locale.value === 'zh-CN' ? 'zh-CN' : 'en-US')

// 打开外部链接
const openExternalUrl = async () => {
  const url = normalizedUrl.value
  if (!url) return
  try {
    await invoke('open_url', { url })
  } catch (error) {
    console.error('Failed to open URL:', error)
  }
}

// 标准化 URL（确保有协议前缀）
const normalizedUrl = computed(() => {
  const url = props.subscription.website_url
  if (!url) return ''
  if (/^https?:\/\//i.test(url)) return url
  return `https://${url}`
})

// 简化显示的 URL（移除 https:// 等前缀）
const displayUrl = computed(() => {
  if (!props.subscription.website_url) return ''
  return props.subscription.website_url.replace(/^https?:\/\//i, '').replace(/\/$/, '')
})

// 格式化过期日期
const formattedExpiryDate = computed(() => {
  if (!props.subscription.expiry_date) return $t('subscriptions.noExpiry')
  const date = new Date(props.subscription.expiry_date)
  return date.toLocaleDateString(dateLocale.value, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
})

const totalDurationDays = computed(() => {
  if (!props.subscription.start_date || !props.subscription.expiry_date) return null
  const start = new Date(props.subscription.start_date)
  const expiry = new Date(props.subscription.expiry_date)
  const duration = Math.ceil((expiry - start) / (1000 * 60 * 60 * 24))
  return duration > 0 ? duration : null
})

const daysLeft = computed(() => {
  if (!props.subscription.expiry_date) return null
  const now = new Date()
  const expiry = new Date(props.subscription.expiry_date)
  return Math.ceil((expiry - now) / (1000 * 60 * 60 * 24))
})

const remainingRatio = computed(() => {
  if (daysLeft.value === null || totalDurationDays.value === null) return null
  return daysLeft.value / totalDurationDays.value
})

// 过期状态样式
const expiryStatusClass = computed(() => {
  if (!props.subscription.expiry_date) return 'text-text-muted'
  if (daysLeft.value === null) return 'text-text-muted'
  if (daysLeft.value < 0) return 'text-danger'
  if (remainingRatio.value !== null) {
    if (remainingRatio.value <= 0.2) return 'text-warning'
    if (remainingRatio.value <= 0.5) return 'text-text-secondary'
    return 'text-success'
  }
  if (daysLeft.value <= 7) return 'text-warning'
  if (daysLeft.value <= 30) return 'text-text-secondary'
  return 'text-success'
})

// 格式化费用
const formattedCost = computed(() => {
  const cost = props.subscription.cost
  if (!cost) return ''
  return typeof cost === 'number' ? `¥${cost.toFixed(2)}` : cost
})

// 获取对比色
const getContrastColor = (bgColor) => {
  if (!bgColor) return '#333'
  const hex = bgColor.replace('#', '')
  const r = parseInt(hex.substr(0, 2), 16)
  const g = parseInt(hex.substr(2, 2), 16)
  const b = parseInt(hex.substr(4, 2), 16)
  const brightness = (r * 299 + g * 587 + b * 114) / 1000
  return brightness > 128 ? '#333' : '#fff'
}
</script>
