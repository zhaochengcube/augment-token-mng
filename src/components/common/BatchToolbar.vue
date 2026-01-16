<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0 translate-y-5"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-300 ease-out"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-5"
    >
      <div
        v-if="visible"
        class="fixed bottom-18 left-1/2 -translate-x-1/2 z-[1000] bg-surface border border-border rounded-xl shadow-lg px-6 py-3"
      >
        <div class="flex items-center gap-6">
          <!-- 左侧：选中数量（不换行） -->
          <div class="flex items-center gap-3 shrink-0 whitespace-nowrap">
            <span class="font-semibold text-sm text-accent">
              {{ $t('common.selected', { count: selectedCount }) }}
            </span>
            <button
              @click="$emit('select-all')"
              class="btn btn--ghost btn--sm"
            >
              {{ $t('common.selectAllPage') }}
            </button>
          </div>

          <!-- 右侧：操作按钮（自适应宽度） -->
          <div class="flex items-center gap-2 shrink-0">
            <slot name="actions"></slot>

            <!-- 关闭按钮（始终显示） -->
            <button
              @click="$emit('clear')"
              class="btn btn--icon btn--ghost"
              v-tooltip="$t('common.cancelSelection')"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  selectedCount: {
    type: Number,
    default: 0
  }
})

defineEmits(['select-all', 'clear'])
</script>

