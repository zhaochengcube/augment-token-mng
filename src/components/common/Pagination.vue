<template>
  <div v-if="totalPages > 0" class="flex items-center justify-between gap-5.5 px-5.5 py-4.5">
    <!-- 左侧：分页信息 -->
    <div class="shrink-0">
      <span class="text-sm text-text-muted font-mono">
        {{ $t('pagination.showing') }}
        <strong class="text-accent font-bold">{{ startItem }}-{{ endItem }}</strong>
        {{ $t('pagination.of') }}
        <strong class="text-accent font-bold">{{ totalItems }}</strong>
      </span>
    </div>

    <!-- 中间：页码导航 -->
    <div class="flex items-center gap-2 flex-1 justify-center">
      <!-- 首页 -->
      <button
        class="btn btn--secondary h-[34px] w-[34px] p-0 rounded-[10px] disabled:opacity-30 disabled:cursor-not-allowed disabled:pointer-events-none"
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
        class="btn btn--secondary h-[34px] w-[34px] p-0 rounded-[10px] disabled:opacity-30 disabled:cursor-not-allowed disabled:pointer-events-none"
        :disabled="currentPage === 1"
        @click="goToPage(currentPage - 1)"
        :title="$t('pagination.prev')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
        </svg>
      </button>

      <!-- 页码显示 -->
      <div class="flex items-center gap-1.5 mx-2.5">
        <template v-for="page in visiblePages" :key="page">
          <span v-if="page === '...'" class="px-1.5 text-text-muted text-sm">...</span>
          <button
            v-else
            :class="[
              'btn min-w-[34px] h-[34px] px-2.5 rounded-[10px] text-sm font-semibold font-mono',
              page === currentPage
                ? 'bg-accent border-transparent text-white'
                : 'btn--secondary'
            ]"
            @click="goToPage(page)"
          >
            {{ page }}
          </button>
        </template>
      </div>

      <!-- 下一页 -->
      <button
        class="btn btn--secondary h-[34px] w-[34px] p-0 rounded-[10px] disabled:opacity-30 disabled:cursor-not-allowed disabled:pointer-events-none"
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
        class="btn btn--secondary h-[34px] w-[34px] p-0 rounded-[10px] disabled:opacity-30 disabled:cursor-not-allowed disabled:pointer-events-none"
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
    <div class="flex items-center gap-2.5 shrink-0">
      <label class="text-sm text-text-muted whitespace-nowrap">{{ $t('pagination.perPage') }}</label>
      <select
        :value="pageSize"
        @change="handlePageSizeChange"
        class="input px-3 py-2 rounded-[10px] text-sm font-mono cursor-pointer"
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

