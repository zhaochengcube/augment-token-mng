<template>
  <div
    ref="wrapperRef"
    class="inline-flex max-w-full"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <slot></slot>
    <Teleport to="body">
      <Transition name="tooltip-fade">
        <div
          v-if="isVisible"
          ref="tooltipRef"
          class="fixed z-[10000] px-2.5 py-1.5 bg-surface text-text rounded-md text-xs font-medium whitespace-nowrap pointer-events-none shadow-md border border-border-strong"
          :style="tooltipStyle"
        >
          {{ content }}
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, nextTick } from 'vue'

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  delay: {
    type: Number,
    default: 200 // 默认 200ms 延迟
  },
  placement: {
    type: String,
    default: 'bottom', // top, bottom, left, right
    validator: (value) => ['top', 'bottom', 'left', 'right'].includes(value)
  }
})

const isVisible = ref(false)
const wrapperRef = ref(null)
const tooltipRef = ref(null)
const tooltipStyle = ref({})
let showTimer = null
let hideTimer = null

const handleMouseEnter = () => {
  // 清除隐藏定时器
  if (hideTimer) {
    clearTimeout(hideTimer)
    hideTimer = null
  }

  // 设置显示定时器
  showTimer = setTimeout(async () => {
    isVisible.value = true
    await nextTick()
    updatePosition()
  }, props.delay)
}

const handleMouseLeave = () => {
  // 清除显示定时器
  if (showTimer) {
    clearTimeout(showTimer)
    showTimer = null
  }

  // 设置隐藏定时器（立即隐藏）
  hideTimer = setTimeout(() => {
    isVisible.value = false
  }, 0)
}

const updatePosition = () => {
  if (!tooltipRef.value || !wrapperRef.value) return

  const triggerRect = wrapperRef.value.getBoundingClientRect()
  const tooltipRect = tooltipRef.value.getBoundingClientRect()

  let top = 0
  let left = 0

  switch (props.placement) {
    case 'top':
      top = triggerRect.top - tooltipRect.height - 8
      left = triggerRect.left + (triggerRect.width - tooltipRect.width) / 2
      break
    case 'bottom':
      top = triggerRect.bottom + 8
      left = triggerRect.left + (triggerRect.width - tooltipRect.width) / 2
      break
    case 'left':
      top = triggerRect.top + (triggerRect.height - tooltipRect.height) / 2
      left = triggerRect.left - tooltipRect.width - 8
      break
    case 'right':
      top = triggerRect.top + (triggerRect.height - tooltipRect.height) / 2
      left = triggerRect.right + 8
      break
  }

  // 确保 tooltip 不会超出视口
  const padding = 8
  if (left < padding) left = padding
  if (left + tooltipRect.width > window.innerWidth - padding) {
    left = window.innerWidth - tooltipRect.width - padding
  }
  if (top < padding) top = padding
  if (top + tooltipRect.height > window.innerHeight - padding) {
    top = window.innerHeight - tooltipRect.height - padding
  }

  tooltipStyle.value = {
    top: `${top}px`,
    left: `${left}px`
  }
}
</script>

<style scoped>
/* Vue transition 动画类 - 必须保留 */
.tooltip-fade-enter-active,
.tooltip-fade-leave-active {
  @apply transition-opacity duration-150 ease-out;
}

.tooltip-fade-enter-from,
.tooltip-fade-leave-to {
  @apply opacity-0;
}
</style>

