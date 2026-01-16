<template>
  <div
    :class="[
      'notification-item',
      'relative flex items-start gap-3.5 min-w-[340px] max-w-[500px] p-[18px] rounded-[14px]',
      'bg-surface/95 backdrop-blur-[16px] border border-border',
      'overflow-hidden pointer-events-auto',
      'transition-all duration-250 ease-out hover:-translate-y-[3px] hover:shadow-[0_12px_32px_rgba(0,0,0,0.3)]',
      'max-sm:min-w-0 max-sm:max-w-none max-sm:w-full max-sm:mx-2.5 max-sm:rounded-xl',
      typeClasses
    ]"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- 图标 -->
    <div :class="['shrink-0 mt-0.5', iconClasses]">
      <svg v-if="notification.type === 'success'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
      </svg>
      <svg v-else-if="notification.type === 'error'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.47 2 2 6.47 2 12s4.47 10 10 10 10-4.47 10-10S17.53 2 12 2zm5 13.59L15.59 17 12 13.41 8.41 17 7 15.59 10.59 12 7 8.41 8.41 7 12 10.59 15.59 7 17 8.41 13.41 12 17 15.59z"/>
      </svg>
      <svg v-else-if="notification.type === 'warning'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z"/>
      </svg>
      <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
      </svg>
    </div>

    <!-- 消息内容 -->
    <div class="flex-1 min-w-0">
      <div class="text-sm leading-relaxed text-text break-words">{{ notification.message }}</div>
    </div>

    <!-- 关闭按钮 -->
    <button
      class="shrink-0 flex items-center justify-center w-[26px] h-[26px] border border-border bg-muted/50 text-text-muted rounded-lg cursor-pointer transition-all duration-200 hover:bg-accent/15 hover:border-accent hover:text-accent"
      @click="handleClose"
      :aria-label="$t ? $t('common.close') : 'Close'"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
      </svg>
    </button>

    <!-- 进度条 -->
    <div v-if="notification.duration > 0" class="absolute bottom-0 left-0 right-0 h-[3px] bg-muted/50">
      <div
        class="progress-bar h-full bg-current opacity-80 shadow-[0_0_8px_currentColor]"
        :style="{
          animationDuration: `${notification.duration}ms`,
          animationPlayState: isHovering ? 'paused' : 'running'
        }"
      ></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

// Props
const props = defineProps({
  notification: {
    type: Object,
    required: true
  }
})

// Emits
const emit = defineEmits(['close', 'hover-start', 'hover-end'])

// 状态管理
const isHovering = ref(false)

// 类型样式映射 - 使用 style.css 中的颜色变量
const typeClasses = computed(() => {
  const typeMap = {
    success: 'border-l-4 border-l-success',
    error: 'border-l-4 border-l-danger',
    warning: 'border-l-4 border-l-warning',
    info: 'border-l-4 border-l-accent'
  }
  return typeMap[props.notification.type] || typeMap.info
})

// 图标样式映射 - 使用 style.css 中的颜色变量
const iconClasses = computed(() => {
  const iconMap = {
    success: 'text-success drop-shadow-[0_0_6px_var(--success)]',
    error: 'text-danger drop-shadow-[0_0_6px_var(--danger)]',
    warning: 'text-warning drop-shadow-[0_0_6px_var(--warning)]',
    info: 'text-accent drop-shadow-[0_0_6px_var(--accent)]'
  }
  return iconMap[props.notification.type] || iconMap.info
})

// 事件处理
const handleClose = () => {
  emit('close', props.notification.id)
}

const handleMouseEnter = () => {
  isHovering.value = true
  emit('hover-start', props.notification.id)
}

const handleMouseLeave = () => {
  isHovering.value = false
  emit('hover-end', props.notification.id)
}
</script>

<style scoped>
/* 进度条动画 - 必须保留 (无法用 Tailwind 实现) */
.progress-bar {
  animation: progress linear forwards;
}

@keyframes progress {
  from { width: 100%; }
  to { width: 0%; }
}
</style>
