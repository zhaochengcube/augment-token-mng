<template>
  <div class="relative h-8 w-full overflow-hidden rounded-md bg-muted">
    <div
      class="h-full opacity-30 transition-[width] duration-200 ease-[cubic-bezier(0.4,0,0.2,1)]"
      :class="fillClass"
      :style="{ width: `${percent}%` }"
    />

    <div class="absolute inset-0 flex items-center justify-between px-3 text-xs font-medium font-mono leading-none gap-2">
      <span v-if="label" class="pl-1 text-text flex items-center min-w-8 flex-1 truncate leading-normal" v-tooltip="tooltipLabel || label">
        {{ label }}
      </span>
      <div class="flex items-center gap-2 pr-1 shrink min-w-0">
        <span v-if="refresh" class="text-text-muted flex items-center">R: {{ refresh }}</span>
        <span v-if="showPercent" class="font-semibold flex items-center" :class="percentClass">
          {{ percent }}%
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  /** 当前值 */
  value: {
    type: Number,
    default: 0
  },
  /** 最大值 */
  max: {
    type: Number,
    default: 100
  },
  /** 直接传入百分比 (0-100)，优先于 value/max 计算 */
  percent: {
    type: Number,
    default: null
  },
  /** 左侧标签文字 */
  label: {
    type: String,
    default: ''
  },
  /** 标签悬浮提示文字，默认使用 label */
  tooltipLabel: {
    type: String,
    default: ''
  },
  /** 刷新间隔显示，如 2h15m */
  refresh: {
    type: String,
    default: ''
  },
  /** 是否显示百分比 */
  showPercent: {
    type: Boolean,
    default: true
  },
  /** 状态阈值：低于此值为 low */
  lowThreshold: {
    type: Number,
    default: 30
  },
  /** 状态阈值：低于此值为 medium */
  mediumThreshold: {
    type: Number,
    default: 60
  }
})

const percent = computed(() => {
  if (props.percent !== null) {
    return Math.max(0, Math.min(100, props.percent))
  }
  if (props.max <= 0) return 0
  return Math.max(0, Math.min(100, Math.round((props.value / props.max) * 100)))
})

const level = computed(() => {
  if (percent.value < props.lowThreshold) return 'low'
  if (percent.value < props.mediumThreshold) return 'medium'
  return 'high'
})

const fillClass = computed(() => {
  if (level.value === 'low') return 'bg-danger'
  if (level.value === 'medium') return 'bg-warning'
  return 'bg-success'
})

const percentClass = computed(() => {
  if (level.value === 'low') return 'text-danger'
  if (level.value === 'medium') return 'text-warning'
  return 'text-success'
})
</script>
