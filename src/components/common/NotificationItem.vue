<template>
  <div
    :class="['notification-item', notification.type, { 'hovering': isHovering }]"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- 图标 -->
    <div class="notification-icon">
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
    <div class="notification-content">
      <div class="notification-message">{{ notification.message }}</div>
    </div>

    <!-- 关闭按钮 -->
    <button 
      class="notification-close" 
      @click="handleClose"
      :aria-label="$t ? $t('common.close') : 'Close'"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
      </svg>
    </button>

    <!-- 进度条 -->
    <div v-if="notification.duration > 0" class="notification-progress">
      <div 
        class="progress-bar" 
        :style="{ 
          animationDuration: `${notification.duration}ms`,
          animationPlayState: isHovering ? 'paused' : 'running'
        }"
      ></div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

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
.notification-item {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  min-width: 320px;
  max-width: 480px;
  padding: 16px;
  border-radius: 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  overflow: hidden;
  pointer-events: auto;
  transition: all 0.2s ease;
}

.notification-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

/* 通知类型样式 */
.notification-item.success {
  border-left: 4px solid #10b981;
}

.notification-item.success .notification-icon {
  color: #10b981;
}

.notification-item.error {
  border-left: 4px solid #ef4444;
}

.notification-item.error .notification-icon {
  color: #ef4444;
}

.notification-item.warning {
  border-left: 4px solid #f59e0b;
}

.notification-item.warning .notification-icon {
  color: #f59e0b;
}

.notification-item.info {
  border-left: 4px solid #3b82f6;
}

.notification-item.info .notification-icon {
  color: #3b82f6;
}

/* 图标样式 */
.notification-icon {
  flex-shrink: 0;
  margin-top: 2px;
}

/* 内容样式 */
.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-message {
  font-size: 14px;
  line-height: 1.5;
  color: var(--text);
  word-wrap: break-word;
}

/* 关闭按钮 */
.notification-close {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: none;
  color: var(--text-muted);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.notification-close:hover {
  background: var(--bg-hover);
  color: var(--text);
}

/* 进度条 */
.notification-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: rgba(0, 0, 0, 0.1);
}

.progress-bar {
  height: 100%;
  background: currentColor;
  opacity: 0.6;
  animation: progress linear forwards;
}

@keyframes progress {
  from {
    width: 100%;
  }
  to {
    width: 0%;
  }
}

/* 响应式设计 */
@media (max-width: 640px) {
  .notification-item {
    min-width: unset;
    max-width: unset;
    width: 100%;
    margin: 0 10px;
  }
}
</style>
