<template>
  <div class="notification-manager">
    <transition-group name="notification" tag="div" class="notification-container">
      <NotificationItem
        v-for="notification in notifications"
        :key="notification.id"
        :notification="notification"
        @close="removeNotification"
        @hover-start="pauseTimer"
        @hover-end="resumeTimer"
      />
    </transition-group>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import NotificationItem from './NotificationItem.vue'

// 通知列表
const notifications = ref([])

// 通知定时器映射
const timers = ref(new Map())

// 通知管理方法
const addNotification = (message, type = 'info', duration = 3000) => {
  const notification = {
    id: Date.now() + Math.random(),
    message,
    type,
    duration,
    createdAt: Date.now(),
    remainingTime: duration,
    isHovered: false
  }
  
  notifications.value.unshift(notification) // 新通知显示在顶部
  
  // 自动移除通知
  if (duration > 0) {
    startTimer(notification.id, duration)
  }
  
  return notification.id
}

// 启动定时器
const startTimer = (id, remainingTime) => {
  const timer = setTimeout(() => {
    removeNotification(id)
  }, remainingTime)
  timers.value.set(id, { timer, remainingTime, startTime: Date.now() })
}

// 暂停定时器（悬浮时）
const pauseTimer = (id) => {
  const timerData = timers.value.get(id)
  if (timerData) {
    clearTimeout(timerData.timer)
    const elapsed = Date.now() - timerData.startTime
    const remaining = Math.max(0, timerData.remainingTime - elapsed)
    timers.value.set(id, { ...timerData, remainingTime: remaining })
  }
}

// 恢复定时器（离开悬浮时）
const resumeTimer = (id) => {
  const timerData = timers.value.get(id)
  if (timerData && timerData.remainingTime > 0) {
    startTimer(id, timerData.remainingTime)
  }
}

const removeNotification = (id) => {
  const index = notifications.value.findIndex(n => n.id === id)
  if (index > -1) {
    notifications.value.splice(index, 1)
  }
  
  // 清理定时器
  const timerData = timers.value.get(id)
  if (timerData) {
    clearTimeout(timerData.timer)
    timers.value.delete(id)
  }
}

const clearAllNotifications = () => {
  // 清理所有定时器
  timers.value.forEach(timerData => {
    clearTimeout(timerData.timer)
  })
  timers.value.clear()
  
  notifications.value = []
}


// 注册全局服务
onMounted(() => {
  if (typeof window !== 'undefined') {
    window.$notify = {
      success: (message, duration = 1500) => addNotification(message, 'success', duration),
      error: (message, duration = 2500) => addNotification(message, 'error', duration),
      warning: (message, duration = 2000) => addNotification(message, 'warning', duration),
      info: (message, duration = 1500) => addNotification(message, 'info', duration),
      remove: removeNotification,
      clear: clearAllNotifications
    }
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    delete window.$notify
  }
})

// 暴露方法给父组件
defineExpose({
  addNotification,
  removeNotification,
  clearAllNotifications,
  notifications
})
</script>

<style scoped>
/* ============================================
   NotificationManager - Modern Tech Style
   ============================================ */

.notification-manager {
  position: fixed;
  top: 22px;
  left: 22px;
  z-index: 9999;
  pointer-events: none;
}

.notification-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-height: 80vh;
  overflow: visible;
}

/* 通知动画 - 科技风 */
.notification-enter-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.notification-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(-100%) scale(0.9);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(-100%) scale(0.9);
}

.notification-move {
  transition: transform 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 响应式设计 */
@media (max-width: 640px) {
  .notification-manager {
    top: 16px;
    left: 12px;
    right: 12px;
  }

  .notification-container {
    max-height: 70vh;
  }
}
</style>
