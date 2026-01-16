<template>
  <div class="flex h-full flex-col min-h-0">
    <div v-if="$slots.header" class="shrink-0">
      <slot name="header" />
    </div>
    <div class="flex-1 min-h-0 relative">
      <div
        ref="pageBodyRef"
        class="absolute inset-x-0 top-0 overflow-y-auto bg-transparent"
        :class="bodyClass"
        :style="{ bottom: `${paginationHeight}px` }"
      >
        <slot />
      </div>
    </div>
    <div v-if="showPagination" ref="paginationRef" :class="paginationClass">
      <slot name="pagination" />
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  showPagination: {
    type: Boolean,
    default: false
  },
  bodyClass: {
    type: [String, Array, Object],
    default: ''
  },
  paginationClass: {
    type: [String, Array, Object],
    default: ''
  },
  scrollKey: {
    type: [String, Number, Boolean, Object, Array, Date],
    default: null
  },
  scrollBehavior: {
    type: String,
    default: 'auto'
  }
})

const pageBodyRef = ref(null)
const paginationRef = ref(null)
const paginationHeight = ref(0)
let paginationResizeObserver = null

const updatePaginationHeight = () => {
  if (!props.showPagination || !paginationRef.value) {
    paginationHeight.value = 0
    return
  }
  paginationHeight.value = paginationRef.value.offsetHeight || 0
}

const startPaginationObserver = () => {
  if (typeof ResizeObserver === 'undefined' || !paginationRef.value) return
  if (!paginationResizeObserver) {
    paginationResizeObserver = new ResizeObserver(() => {
      updatePaginationHeight()
    })
  }
  paginationResizeObserver.disconnect()
  paginationResizeObserver.observe(paginationRef.value)
}

const scrollToTop = async () => {
  await nextTick()
  if (pageBodyRef.value) {
    pageBodyRef.value.scrollTo({ top: 0, behavior: props.scrollBehavior })
  }
}

watch(
  () => props.showPagination,
  async (showing) => {
    if (!showing) {
      paginationHeight.value = 0
      return
    }
    await nextTick()
    updatePaginationHeight()
    startPaginationObserver()
  },
  { immediate: true }
)

watch(
  () => props.scrollKey,
  async () => {
    await scrollToTop()
  }
)

onMounted(async () => {
  await nextTick()
  updatePaginationHeight()
  startPaginationObserver()
})

onUnmounted(() => {
  if (paginationResizeObserver) {
    paginationResizeObserver.disconnect()
    paginationResizeObserver = null
  }
})
</script>
