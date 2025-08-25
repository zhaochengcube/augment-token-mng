<template>
  <div class="token-list-modal">
    <div class="modal-overlay">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <div class="header-title">
            <h2>已保存Token</h2>
            <div :class="['status-badge', hasUnsavedChanges ? 'unsaved' : 'saved']">
              <span :class="['status-dot', hasUnsavedChanges ? 'unsaved' : 'saved']"></span>
              <span class="status-text">{{ hasUnsavedChanges ? '未保存' : '正常' }}</span>
            </div>
          </div>
          <div class="header-actions">
            <button @click="$emit('save')" class="btn success small" :disabled="!hasUnsavedChanges">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"/>
              </svg>
              保存
            </button>
            <button @click="$emit('add-token')" class="btn primary small">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
              </svg>
              添加
            </button>
            <button @click="handleRefresh" class="btn secondary small">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              刷新
            </button>
            <button class="close-btn" @click="handleClose">×</button>
          </div>
        </div>
        
        <div class="modal-body">
          <!-- Empty State -->
          <div v-if="tokens.length === 0 && !isLoading" class="empty-state">
            <div class="empty-icon">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
              </svg>
            </div>
            <h3>还没有保存的Token</h3>
            <p>关闭此窗口，在主页面生成你的第一个Token</p>
          </div>

          <!-- Loading State -->
          <div v-if="isLoading" class="loading-state">
            <div class="spinner"></div>
            <p>正在加载Token...</p>
          </div>

          <!-- Token List -->
          <div v-if="tokens.length > 0" class="token-list">
            <div class="list-header">
              <h3>Token列表 ({{ tokens.length }})</h3>
            </div>

            <div class="token-grid">
              <TokenCard
                v-for="token in tokens"
                :key="token.id"
                :ref="el => setTokenCardRef(el, token.id)"
                :token="token"
                @delete="$emit('delete', $event)"
                @copy-success="$emit('copy-success', $event)"
              @open-portal="$emit('open-portal', $event)"
              @edit="$emit('edit', $event)"
              @token-updated="$emit('token-updated')"
              />
            </div>


          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import TokenCard from './TokenCard.vue'

// Props
const props = defineProps({
  tokens: {
    type: Array,
    default: () => []
  },
  isLoading: {
    type: Boolean,
    default: false
  },
  hasUnsavedChanges: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['close', 'delete', 'copy-success', 'add-token', 'refresh', 'open-portal', 'edit', 'save', 'token-updated'])

// Token card refs for accessing child methods
const tokenCardRefs = ref({})



// 设置ref的函数
const setTokenCardRef = (el, tokenId) => {
  if (el) {
    tokenCardRefs.value[tokenId] = el
  } else {
    // 当组件被移除时，清理引用
    delete tokenCardRefs.value[tokenId]
  }
}

// 刷新所有Portal信息
const refreshAllPortalInfo = async () => {
  await nextTick() // 确保DOM已更新

  const portalTokens = Object.entries(tokenCardRefs.value).filter(([tokenId, cardRef]) => {
    const token = props.tokens.find(t => t.id === tokenId)
    return token && token.portal_url && cardRef && typeof cardRef.refreshPortalInfo === 'function'
  })

  if (portalTokens.length === 0) {
    return { success: true, message: '没有需要刷新的 Portal 信息' }
  }

  try {
    // 并行刷新所有Portal信息
    const refreshPromises = portalTokens.map(async ([tokenId, cardRef]) => {
      try {
        await cardRef.refreshPortalInfo()
        return { tokenId, success: true }
      } catch (error) {
        console.error(`Failed to refresh portal info for token ${tokenId}:`, error)
        return { tokenId, success: false, error: error.message }
      }
    })

    const results = await Promise.allSettled(refreshPromises)

    // 统计成功和失败的数量
    let successCount = 0
    let failureCount = 0

    results.forEach(result => {
      if (result.status === 'fulfilled' && result.value.success) {
        successCount++
      } else {
        failureCount++
      }
    })

    if (failureCount === 0) {
      return {
        success: true,
        message: `Portal 信息刷新完成 (${successCount}/${portalTokens.length})`
      }
    } else if (successCount === 0) {
      return {
        success: false,
        message: `Portal 信息刷新失败 (${failureCount}/${portalTokens.length})`
      }
    } else {
      return {
        success: true,
        message: `Portal 信息部分刷新成功 (${successCount}/${portalTokens.length})`
      }
    }
  } catch (error) {
    return {
      success: false,
      message: `刷新 Portal 信息时发生错误: ${error.message}`
    }
  }
}

// 检查所有Token的账号状态
const checkAllAccountStatus = async () => {
  await nextTick() // 确保DOM已更新

  const allTokens = Object.entries(tokenCardRefs.value).filter(([tokenId, cardRef]) => {
    // 确保 token 还存在于当前列表中
    const tokenExists = props.tokens.find(t => t.id === tokenId)
    return tokenExists && cardRef && typeof cardRef.refreshAccountStatus === 'function'
  })

  if (allTokens.length === 0) {
    return { success: true, message: '没有可检查的 Token' }
  }

  try {
    // 并行检查所有Token的账号状态
    const checkPromises = allTokens.map(async ([tokenId, cardRef]) => {
      try {
        await cardRef.refreshAccountStatus()
        return { tokenId, success: true }
      } catch (error) {
        console.error(`Failed to check account status for token ${tokenId}:`, error)
        return { tokenId, success: false, error: error.message }
      }
    })

    const results = await Promise.allSettled(checkPromises)

    // 统计成功和失败的数量
    let successCount = 0
    let failureCount = 0

    results.forEach(result => {
      if (result.status === 'fulfilled' && result.value.success) {
        successCount++
      } else {
        failureCount++
      }
    })

    if (failureCount === 0) {
      return {
        success: true,
        message: `账号状态检查完成 (${successCount}/${allTokens.length})`
      }
    } else if (successCount === 0) {
      return {
        success: false,
        message: `账号状态检查失败 (${failureCount}/${allTokens.length})`
      }
    } else {
      return {
        success: true,
        message: `账号状态部分检查成功 (${successCount}/${allTokens.length})`
      }
    }
  } catch (error) {
    return {
      success: false,
      message: `检查账号状态时发生错误: ${error.message}`
    }
  }
}

// 处理关闭事件
const handleClose = () => {
  // 如果有未保存的更改，显示提示并阻止关闭
  if (props.hasUnsavedChanges) {
    emit('copy-success', '检测到未保存的更改，请先保存后再关闭', 'error')
    return
  }

  // 没有未保存更改，正常关闭
  emit('close')
}

// 处理刷新事件
const handleRefresh = async () => {
  // 如果有未保存的更改，警告用户
  if (props.hasUnsavedChanges) {
    emit('copy-success', '检测到未保存的更改，请先保存后再刷新', 'error')
    return
  }

  // 显示开始刷新的通知
  emit('copy-success', '正在刷新 Token 状态和 Portal 信息...', 'info')

  try {
    // 先刷新token列表
    emit('refresh')
    await nextTick() // 等待DOM更新

    // 并行执行账号状态检查和Portal信息刷新
    const [statusResult, portalResult] = await Promise.allSettled([
      checkAllAccountStatus(),
      refreshAllPortalInfo()
    ])

    // 处理结果
    const messages = []
    let hasError = false

    if (statusResult.status === 'fulfilled') {
      messages.push(statusResult.value.message)
      if (!statusResult.value.success) hasError = true
    } else {
      messages.push(`账号状态检查失败: ${statusResult.reason}`)
      hasError = true
    }

    if (portalResult.status === 'fulfilled') {
      messages.push(portalResult.value.message)
      if (!portalResult.value.success) hasError = true
    } else {
      messages.push(`Portal 信息刷新失败: ${portalResult.reason}`)
      hasError = true
    }

    // 显示刷新结果通知
    const finalMessage = messages.join('; ')
    emit('copy-success', finalMessage, hasError ? 'error' : 'success')
    emit('save')
  } catch (error) {
    // 显示错误通知
    emit('copy-success', `刷新失败: ${error.message}`, 'error')
  }
}



// 组件挂载时只在没有未保存更改时才刷新
onMounted(async () => {
  // 如果有未保存的更改，不要重新加载文件数据，避免覆盖内存中的新token
  if (!props.hasUnsavedChanges) {
    emit('refresh', true) // 传递 true 表示显示成功消息
  }
})

// 暴露方法给父组件
defineExpose({
  refreshAllPortalInfo
})
</script>

<style scoped>
.token-list-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
}

.modal-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 12px;
  width: 100%;
  max-width: 900px;
  height: 90vh; /* 固定高度，更大 */
  overflow: hidden;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  margin-top: -60px; /* 向上移动到红框位置 */
}

/* 移除旧的 modal-header 样式，使用新的样式 */

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #6b7280;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #374151;
}

.modal-body {
  padding: 24px;
  height: calc(90vh - 80px); /* 固定高度 */
  overflow-y: auto;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
}

.empty-icon {
  color: #d1d5db;
  margin-bottom: 16px;
}

.empty-state h3 {
  color: #374151;
  margin: 0 0 8px 0;
  font-size: 1.25rem;
}

.empty-state p {
  color: #6b7280;
  margin: 0;
}

.loading-state {
  text-align: center;
  padding: 40px 20px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e7eb;
  border-top: 3px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-state p {
  color: #6b7280;
  margin: 0;
}

.token-list {
  /* 移除内部滚动，使用外层 modal-body 的滚动 */
}

.token-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
  padding: 4px;
}

/* 响应式处理 */
@media (max-width: 768px) {
  .modal-content {
    margin: 10px;
    max-width: calc(100vw - 20px);
  }

  .modal-header {
    padding: 16px;
  }

  .modal-body {
    padding: 16px;
  }

  .token-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .list-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .list-header h3 {
    font-size: 1rem;
  }
}

@media (max-width: 480px) {
  .modal-overlay {
    padding: 10px;
  }

  .modal-content {
    max-height: 95vh;
  }

  .modal-header h2 {
    font-size: 1.25rem;
  }

  .empty-state {
    padding: 20px 10px;
  }

  .empty-state h3 {
    font-size: 1.125rem;
  }

  .btn.small {
    padding: 4px 8px;
    font-size: 11px;
  }

  .sync-actions {
    flex-direction: column;
  }

  .btn.sync-btn {
    min-width: auto;
  }
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e5e7eb;
}

.list-header h3 {
  margin: 0;
  color: #374151;
  font-size: 1.125rem;
  font-weight: 600;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 36px;
  box-sizing: border-box;
}

.btn.secondary {
  background: #f3f4f6;
  color: #374151;
}

.btn.secondary:hover {
  background: #e5e7eb;
}

.btn.success {
  background: #10b981;
  color: white;
  border: 1px solid #10b981;
}

.btn.success:hover:not(:disabled) {
  background: #059669;
  border-color: #059669;
}

.btn.success:disabled {
  background: #d1d5db;
  color: #9ca3af;
  border-color: #d1d5db;
  cursor: not-allowed;
}

.btn.small {
  padding: 6px 12px;
  font-size: 12px;
  height: 32px;
}

/* Header layout */
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
  background: #f9fafb;
  min-height: 60px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.header-title h2 {
  margin: 0;
  color: #111827;
  font-size: 1.25rem;
  font-weight: 600;
  line-height: 1.2;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* Status badge styles */
.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  height: fit-content;
}

.status-badge.saved {
  background-color: #d1fae5;
  color: #065f46;
}

.status-badge.unsaved {
  background-color: #fef3c7;
  color: #92400e;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}

.status-dot.saved {
  background-color: #10b981;
}

.status-dot.unsaved {
  background-color: #f59e0b;
}

.status-text {
  font-size: 11px;
  font-weight: 500;
}



@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.btn.loading {
  opacity: 0.7;
  pointer-events: none;
}


</style>
