<template>
  <Teleport to="body">
    <!-- 遮罩层 - 点击关闭 -->
    <Transition
      enter-active-class="transition-opacity duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[899]"
        @click="handleClose"
      ></div>
    </Transition>

    <!-- 工具栏面板 -->
    <Transition
      enter-active-class="transition duration-300 ease-out"
      enter-from-class="opacity-0 -translate-y-5"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-300 ease-out"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-5"
    >
      <div
        v-if="visible"
        class="fixed top-[73px] left-1/2 -translate-x-1/2 z-[900] bg-surface border border-border rounded-xl shadow-lg"
        :style="{ maxWidth, width: 'calc(100% - 48px)' }"
        @click.stop
      >
        <!-- 工具栏内容 -->
        <div class="px-6 py-4 max-h-[calc(100vh-200px)] overflow-y-auto">
          <!-- 内容插槽 -->
          <slot></slot>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { watch, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  // 是否显示工具栏
  visible: {
    type: Boolean,
    default: false
  },
  // 工具栏标题
  title: {
    type: String,
    default: ''
  },
  // 是否显示关闭按钮
  showClose: {
    type: Boolean,
    default: true
  },
  // 最大宽度
  maxWidth: {
    type: String,
    default: '600px'
  }
})

const emit = defineEmits(['close', 'open'])

const handleClose = () => {
  emit('close')
}

// ESC 键关闭
const handleEscape = (e) => {
  if (e.key === 'Escape' && props.visible) {
    handleClose()
  }
}

watch(() => props.visible, (newVal) => {
  if (newVal) {
    emit('open')
  }
})

onMounted(() => {
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
})
</script>

