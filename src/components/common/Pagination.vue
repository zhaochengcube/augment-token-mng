<template>
  <div v-if="totalPages > 0" class="pagination-container">
    <!-- 左侧：分页信息 -->
    <div class="pagination-info">
      <span class="info-text">
        {{ $t('pagination.showing') }} 
        <strong>{{ startItem }}-{{ endItem }}</strong> 
        {{ $t('pagination.of') }} 
        <strong>{{ totalItems }}</strong>
      </span>
    </div>

    <!-- 中间：页码导航 -->
    <div class="pagination-nav">
      <!-- 首页 -->
      <button 
        class="nav-btn" 
        :disabled="currentPage === 1"
        @click="goToPage(1)"
        :title="$t('pagination.first')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M18.41 16.59L13.82 12l4.59-4.59L17 6l-6 6 6 6zM6 6h2v12H6z"/>
        </svg>
      </button>

      <!-- 上一页 -->
      <button 
        class="nav-btn" 
        :disabled="currentPage === 1"
        @click="goToPage(currentPage - 1)"
        :title="$t('pagination.prev')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
        </svg>
      </button>

      <!-- 页码显示 -->
      <div class="page-numbers">
        <template v-for="page in visiblePages" :key="page">
          <span v-if="page === '...'" class="page-ellipsis">...</span>
          <button 
            v-else
            :class="['page-btn', { active: page === currentPage }]"
            @click="goToPage(page)"
          >
            {{ page }}
          </button>
        </template>
      </div>

      <!-- 下一页 -->
      <button 
        class="nav-btn" 
        :disabled="currentPage === totalPages"
        @click="goToPage(currentPage + 1)"
        :title="$t('pagination.next')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
        </svg>
      </button>

      <!-- 末页 -->
      <button 
        class="nav-btn" 
        :disabled="currentPage === totalPages"
        @click="goToPage(totalPages)"
        :title="$t('pagination.last')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M5.59 7.41L10.18 12l-4.59 4.59L7 18l6-6-6-6zM16 6h2v12h-2z"/>
        </svg>
      </button>
    </div>

    <!-- 右侧：每页数量选择 -->
    <div class="pagination-size">
      <label>{{ $t('pagination.perPage') }}</label>
      <select 
        :value="pageSize" 
        @change="handlePageSizeChange"
        class="size-select"
      >
        <option v-for="size in pageSizeOptions" :key="size" :value="size">
          {{ size }}
        </option>
      </select>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps({
  currentPage: {
    type: Number,
    required: true
  },
  totalPages: {
    type: Number,
    required: true
  },
  totalItems: {
    type: Number,
    required: true
  },
  pageSize: {
    type: Number,
    required: true
  },
  pageSizeOptions: {
    type: Array,
    default: () => [10, 20, 50, 100, 200]
  }
})

const emit = defineEmits(['update:currentPage', 'update:pageSize', 'page-change', 'size-change'])

// 计算当前页显示的项目范围
const startItem = computed(() => {
  if (props.totalItems === 0) return 0
  return (props.currentPage - 1) * props.pageSize + 1
})

const endItem = computed(() => {
  return Math.min(props.currentPage * props.pageSize, props.totalItems)
})

// 计算可见的页码（智能省略）
const visiblePages = computed(() => {
  const pages = []
  const total = props.totalPages
  const current = props.currentPage
  
  if (total <= 7) {
    // 总页数少于7页，全部显示
    for (let i = 1; i <= total; i++) {
      pages.push(i)
    }
  } else {
    // 总页数多于7页，智能省略
    pages.push(1)
    
    if (current <= 3) {
      // 当前页在前面
      pages.push(2, 3, 4, '...', total)
    } else if (current >= total - 2) {
      // 当前页在后面
      pages.push('...', total - 3, total - 2, total - 1, total)
    } else {
      // 当前页在中间
      pages.push('...', current - 1, current, current + 1, '...', total)
    }
  }
  
  return pages
})

// 跳转到指定页
const goToPage = (page) => {
  if (page < 1 || page > props.totalPages || page === props.currentPage) return
  emit('update:currentPage', page)
  emit('page-change', page)
}

// 修改每页数量
const handlePageSizeChange = (event) => {
  const newSize = parseInt(event.target.value)
  emit('update:pageSize', newSize)
  emit('size-change', newSize)
}
</script>

<style scoped>
.pagination-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 16px 20px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border);
  border-radius: 0 0 8px 8px;
}

.pagination-info {
  flex-shrink: 0;
}

.info-text {
  font-size: 14px;
  color: var(--text-muted);
}

.info-text strong {
  color: var(--text);
  font-weight: 600;
}

.pagination-nav {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  justify-content: center;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--border);
  background: var(--bg-surface);
  color: var(--text);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.nav-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--accent);
}

.nav-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-numbers {
  display: flex;
  align-items: center;
  gap: 4px;
  margin: 0 8px;
}

.page-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 8px;
  border: 1px solid var(--border);
  background: var(--bg-surface);
  color: var(--text);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 500;
}

.page-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--accent);
}

.page-btn.active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--text-contrast);
  font-weight: 600;
}

.page-ellipsis {
  padding: 0 4px;
  color: var(--text-muted);
  font-size: 14px;
}

.pagination-size {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.pagination-size label {
  font-size: 14px;
  color: var(--text-muted);
  white-space: nowrap;
}

.size-select {
  padding: 6px 10px;
  border: 1px solid var(--border);
  background: var(--bg-surface);
  color: var(--text);
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.size-select:hover {
  border-color: var(--accent);
}

.size-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 10%, transparent);
}
</style>

