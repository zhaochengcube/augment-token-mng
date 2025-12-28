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
/* ============================================
   NotificationItem - Modern Tech Style
   ============================================ */

.notification-item {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 14px;
  min-width: 340px;
  max-width: 500px;
  padding: 18px;
  border-radius: 14px;
  background: var(--tech-glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--tech-glass-border);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  overflow: hidden;
  pointer-events: auto;
  transition: all 0.25s ease;
}

.notification-item:hover {
  transform: translateY(-3px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3), var(--tech-border-glow);
}

/* 通知类型样式 - 科技风发光边框 */
.notification-item.success {
  border-left: 4px solid #10b981;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25), 0 0 15px rgba(16, 185, 129, 0.3);
}

.notification-item.success .notification-icon {
  color: #10b981;
  filter: drop-shadow(0 0 6px rgba(16, 185, 129, 0.5));
}

.notification-item.error {
  border-left: 4px solid #ef4444;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25), 0 0 15px var(--tech-glow-danger);
}

.notification-item.error .notification-icon {
  color: #ef4444;
  filter: drop-shadow(0 0 6px rgba(239, 68, 68, 0.5));
}

.notification-item.warning {
  border-left: 4px solid #f59e0b;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25), 0 0 15px var(--tech-glow-warning);
}

.notification-item.warning .notification-icon {
  color: #f59e0b;
  filter: drop-shadow(0 0 6px rgba(245, 158, 11, 0.5));
}

.notification-item.info {
  border-left: 4px solid #3b82f6;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25), 0 0 15px rgba(59, 130, 246, 0.3);
}

.notification-item.info .notification-icon {
  color: #3b82f6;
  filter: drop-shadow(0 0 6px rgba(59, 130, 246, 0.5));
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
  line-height: 1.6;
  color: var(--text);
  word-wrap: break-word;
}

/* 关闭按钮 - 科技风 */
.notification-close {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--tech-glass-border);
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  color: var(--text-muted);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.notification-close:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}

/* 进度条 - 科技风发光 */
.notification-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
}

.progress-bar {
  height: 100%;
  background: currentColor;
  opacity: 0.8;
  animation: progress linear forwards;
  box-shadow: 0 0 8px currentColor;
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
    border-radius: 12px;
  }
}
</style>
