<template>
  <div class="h-full w-full min-h-0">
    <TokenList :is-sidebar-collapsed="isSidebarCollapsed" />
  </div>
</template>

<script setup>
import TokenList from '../token/TokenList.vue'

const props = defineProps({
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})
</script>