<template>
  <div
    ref="wrapperRef"
    class="tooltip-wrapper"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <slot></slot>
    <Teleport to="body">
      <Transition name="tooltip-fade">
        <div
          v-if="isVisible"
          ref="tooltipRef"
          class="tooltip"
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
.tooltip-wrapper {
  display: inline-flex;
  max-width: 100%;
}

.tooltip {
  position: fixed;
  z-index: 10000;
  padding: 6px 10px;
  background: var(--bg-surface-alt, #1f2937);
  color: var(--text-inverse, #ffffff);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  pointer-events: none;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15),
              0 0 0 1px var(--tech-glass-border);
}

.tooltip-fade-enter-active,
.tooltip-fade-leave-active {
  transition: opacity 0.15s ease;
}

.tooltip-fade-enter-from,
.tooltip-fade-leave-to {
  opacity: 0;
}
</style>

