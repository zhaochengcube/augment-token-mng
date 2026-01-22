<template>
  <div
    class="bg-surface border border-border rounded-lg p-3 cursor-pointer relative group transition-all duration-150 hover:bg-hover hover:border-border-strong"
    @click="$emit('edit', subscription)"
  >
    <!-- 头部：只有标题 -->
    <div class="flex items-center gap-2 mb-3 pr-8">
      <h3 class="text-[15px] font-semibold text-text m-0 flex-1 truncate">{{ subscription.website }}</h3>
    </div>

    <!-- 删除按钮（悬停显示） -->
    <button
      class="absolute top-2 right-2 w-7 h-7 border-none rounded bg-surface text-text-secondary cursor-pointer flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity shadow-sm hover:bg-danger hover:text-white"
      @click.stop="$emit('delete', subscription)"
      :title="$t('common.delete')"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
      </svg>
    </button>

    <!-- 属性列表 -->
    <div class="flex flex-col gap-1.5">
      <!-- 网站地址 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.websiteUrl') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text truncate">
          <a
            v-if="subscription.website_url"
            :href="normalizedUrl"
            class="text-accent no-underline hover:underline cursor-pointer"
            @click.stop.prevent="openExternalUrl"
          >{{ displayUrl }}</a>
          <span v-else class="text-text-muted">—</span>
        </div>
      </div>

      <!-- 到期时间 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM9 10H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.expiryDate') }}</span>
        </div>
        <div class="flex-1 text-[13px] truncate">
          <span :class="['inline-flex items-center gap-1', expiryStatusClass]">
            {{ formattedExpiryDate }}
            <span v-if="daysLeftText" class="text-[11px] opacity-80">({{ daysLeftText }})</span>
          </span>
        </div>
      </div>

      <!-- 费用 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.cost') }}</span>
        </div>
        <div class="flex-1 text-[13px] text-text truncate">
          <span v-if="subscription.cost" class="font-medium tabular-nums">{{ formattedCost }}</span>
          <span v-else class="text-text-muted">—</span>
        </div>
      </div>

      <!-- 标签 -->
      <div class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.63 5.84C17.27 5.33 16.67 5 16 5L5 5.01C3.9 5.01 3 5.9 3 7v10c0 1.1.9 1.99 2 1.99L16 19c.67 0 1.27-.33 1.63-.84L22 12l-4.37-6.16z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.tag') }}</span>
        </div>
        <div class="flex-1 text-[13px] truncate">
          <span
            v-if="subscription.tag"
            class="badge editable badge--sm"
            :style="{ '--tag-color': subscription.tag_color }"
          >{{ subscription.tag }}</span>
          <span v-else class="text-text-muted">—</span>
        </div>
      </div>

      <!-- 备注 (如果有) -->
      <div v-if="subscription.notes" class="flex items-start gap-2 min-h-6">
        <div class="flex items-center gap-1.5 w-[90px] shrink-0 text-text-muted text-xs">
          <svg class="w-3.5 h-3.5 shrink-0 opacity-70" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 18h12v-2H3v2zM3 6v2h18V6H3zm0 7h18v-2H3v2z"/>
          </svg>
          <span>{{ $t('subscriptions.fields.notes') }}</span>
        </div>
        <div class="flex-1 text-xs text-text-secondary leading-relaxed line-clamp-2">{{ subscription.notes }}</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t: $t } = useI18n()

const props = defineProps({
  subscription: {
    type: Object,
    required: true
  }
})

defineEmits(['edit', 'delete'])

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
  return date.toLocaleDateString()
})

// 剩余天数
const daysLeft = computed(() => {
  if (!props.subscription.expiry_date) return null
  const now = new Date()
  const expiry = new Date(props.subscription.expiry_date)
  return Math.ceil((expiry - now) / (1000 * 60 * 60 * 24))
})

// 剩余天数文案
const daysLeftText = computed(() => {
  if (daysLeft.value === null) return ''
  if (daysLeft.value < 0) return $t('subscriptions.expired')
  if (daysLeft.value === 0) return $t('subscriptions.expirestoday')
  return $t('subscriptions.daysLeft', { days: daysLeft.value })
})

// 过期状态样式 - 使用 Tailwind classes
const expiryStatusClass = computed(() => {
  if (daysLeft.value === null) return 'text-text-muted'
  if (daysLeft.value < 0) return 'text-danger'
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

// 获取对比色（用于标签文字）
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

