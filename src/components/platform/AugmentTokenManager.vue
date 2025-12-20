<template>
  <div class="augment-token-manager">
    <TokenList mode="page" />
  </div>
</template>

<script setup>
import TokenList from '../token/TokenList.vue'
</script>

<style scoped>
.augment-token-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
}
</style>

