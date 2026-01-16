<template>
  <div class="inline-block" ref="containerRef">
    <!-- 触发器插槽 -->
    <div ref="triggerRef" @click="toggle">
      <slot name="trigger" :is-open="isOpen" :toggle="toggle" />
    </div>

    <!-- 浮动菜单 -->
    <Teleport to="body">
      <Transition :name="transitionName">
        <div
          v-if="isOpen"
          ref="floatingRef"
          :style="[floatingStyles, { transformOrigin }]"
          :class="menuClasses"
          @click="closeOnSelect ? close() : null"
        >
          <slot :close="close" :is-flipped="isFlipped" />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useFloatingDropdown } from '@/composables/useFloatingDropdown'

const props = defineProps({
  // 初始位置
  placement: {
    type: String,
    default: 'bottom-end',
    validator: (v) => [
      'bottom', 'bottom-start', 'bottom-end',
      'top', 'top-start', 'top-end',
      'left', 'left-start', 'left-end',
      'right', 'right-start', 'right-end'
    ].includes(v)
  },
  // 偏移距离
  offset: {
    type: Number,
    default: 4
  },
  // 菜单宽度变体
  width: {
    type: String,
    default: 'default',
    validator: (v) => ['default', 'medium', 'wide', 'xwide', 'full'].includes(v)
  },
  // 点击选项后是否关闭
  closeOnSelect: {
    type: Boolean,
    default: true
  },
  // 是否禁用
  disabled: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['open', 'close'])

const {
  triggerRef,
  floatingRef,
  isOpen,
  isFlipped,
  actualPlacement,
  floatingStyles,
  toggle: baseToggle,
  close,
  handleClickOutside
} = useFloatingDropdown({
  placement: props.placement,
  offsetDistance: props.offset
})

// 禁用时不允许切换
const toggle = () => {
  if (props.disabled) return
  baseToggle()
}

// 过渡动画方向
const transitionName = computed(() => {
  return isFlipped.value ? 'dropdown-up' : 'dropdown'
})

// 根据实际展开位置计算 transform-origin
const transformOrigin = computed(() => {
  const placement = actualPlacement.value || props.placement

  // 垂直方向
  const isTop = placement.startsWith('top')
  const vertical = isTop ? 'bottom' : 'top'

  // 水平方向
  let horizontal = 'right'
  if (placement.endsWith('-start')) {
    horizontal = 'left'
  } else if (placement.endsWith('-end')) {
    horizontal = 'right'
  } else {
    horizontal = 'center'
  }

  return `${vertical} ${horizontal}`
})

// 菜单样式类
const menuClasses = computed(() => {
  const classes = ['dropdown-menu', 'dropdown-menu--floating']

  if (props.width === 'medium') classes.push('dropdown-menu--medium')
  else if (props.width === 'wide') classes.push('dropdown-menu--wide')
  else if (props.width === 'xwide') classes.push('dropdown-menu--xwide')

  return classes
})

// 监听开关状态，触发事件
watch(isOpen, (val) => {
  emit(val ? 'open' : 'close')
})

// 全局点击监听
onMounted(() => {
  document.addEventListener('click', handleClickOutside, true)
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside, true)
  document.removeEventListener('keydown', handleEscape)
})

// ESC 关闭
const handleEscape = (e) => {
  if (e.key === 'Escape' && isOpen.value) {
    close()
  }
}

// 暴露方法给父组件
defineExpose({
  open: () => { if (!props.disabled) isOpen.value = true },
  close,
  toggle,
  isOpen
})
</script>

<style>
/* Floating UI 定位覆盖样式 */
.dropdown-menu--floating {
  /* 覆盖 .dropdown-menu 的定位样式，由 Floating UI 控制 */
  position: fixed !important;
  right: auto !important;
  margin-top: 0 !important;
  /* 让宽度自适应内容 */
  min-width: auto !important;
  width: max-content;
}

/* 下拉动画 - Fade Only */
.dropdown-enter-active,
.dropdown-leave-active,
.dropdown-up-enter-active,
.dropdown-up-leave-active {
  transition: opacity 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to,
.dropdown-up-enter-from,
.dropdown-up-leave-to {
  opacity: 0;
}
</style>

