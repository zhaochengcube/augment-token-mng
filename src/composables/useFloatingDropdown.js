import { ref, computed } from 'vue'
import { useFloating, offset, flip, shift, autoUpdate } from '@floating-ui/vue'

/**
 * Floating UI 下拉菜单组合式函数
 * 自动处理定位、方向翻转和边界检测
 * 
 * @param {Object} options - 配置选项
 * @param {string} options.placement - 默认位置 ('bottom-start' | 'bottom-end' | 'top-start' | 'top-end')
 * @param {number} options.offsetDistance - 偏移距离，默认 4px
 */
export function useFloatingDropdown(options = {}) {
  const {
    placement = 'bottom-end',
    offsetDistance = 4
  } = options

  // 触发元素和浮动元素的引用
  const triggerRef = ref(null)
  const floatingRef = ref(null)
  
  // 菜单开关状态
  const isOpen = ref(false)

  // Floating UI 核心
  const { floatingStyles, placement: actualPlacement } = useFloating(triggerRef, floatingRef, {
    placement,
    whileElementsMounted: autoUpdate,
    middleware: [
      offset(offsetDistance),
      flip({
        fallbackAxisSideDirection: 'start',
        padding: 8
      }),
      shift({ padding: 8 })
    ]
  })

  // 计算是否向上展开（用于动画方向等）
  const isFlipped = computed(() => {
    return actualPlacement.value?.startsWith('top')
  })

  // 切换菜单
  const toggle = () => {
    isOpen.value = !isOpen.value
  }

  // 打开菜单
  const open = () => {
    isOpen.value = true
  }

  // 关闭菜单
  const close = () => {
    isOpen.value = false
  }

  // 点击外部关闭
  const handleClickOutside = (event) => {
    if (!isOpen.value) return
    
    const trigger = triggerRef.value?.$el || triggerRef.value
    const floating = floatingRef.value?.$el || floatingRef.value
    
    if (trigger?.contains(event.target)) return
    if (floating?.contains(event.target)) return
    
    close()
  }

  return {
    // 模板引用
    triggerRef,
    floatingRef,
    
    // 状态
    isOpen,
    isFlipped,
    actualPlacement,
    
    // 样式（直接绑定到浮动元素）
    floatingStyles,
    
    // 方法
    toggle,
    open,
    close,
    handleClickOutside
  }
}

