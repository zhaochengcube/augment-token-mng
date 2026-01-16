// Tooltip directive for Vue 3
let tooltipElement = null
let showTimer = null
let hideTimer = null

function createTooltipElement() {
  const el = document.createElement('div')
  el.className = 'v-tooltip'
  document.body.appendChild(el)
  return el
}

function updatePosition(el, tooltipEl, placement = 'bottom') {
  const triggerRect = el.getBoundingClientRect()
  const tooltipRect = tooltipEl.getBoundingClientRect()
  
  let top = 0
  let left = 0

  switch (placement) {
    case 'top':
      top = triggerRect.top - tooltipRect.height - 8
      left = triggerRect.left + (triggerRect.width - tooltipRect.width) / 2
      break
    case 'bottom':
      top = triggerRect.bottom + 8
      left = triggerRect.left + (triggerRect.width - tooltipRect.width) / 2
      break
    case 'left':
      top = triggerRect.top + (triggerRect.height - tooltipRect.height) / 2
      left = triggerRect.left - tooltipRect.width - 8
      break
    case 'right':
      top = triggerRect.top + (triggerRect.height - tooltipRect.height) / 2
      left = triggerRect.right + 8
      break
  }

  // 确保 tooltip 不会超出视口
  const padding = 8
  if (left < padding) left = padding
  if (left + tooltipRect.width > window.innerWidth - padding) {
    left = window.innerWidth - tooltipRect.width - padding
  }
  if (top < padding) top = padding
  if (top + tooltipRect.height > window.innerHeight - padding) {
    top = window.innerHeight - tooltipRect.height - padding
  }

  tooltipEl.style.top = `${top}px`
  tooltipEl.style.left = `${left}px`
}

function showTooltip(el, binding) {
  if (!binding.value) return
  
  if (!tooltipElement) {
    tooltipElement = createTooltipElement()
  }
  
  tooltipElement.textContent = binding.value
  tooltipElement.style.opacity = '0'
  
  // 等待下一帧以获取正确的尺寸
  requestAnimationFrame(() => {
    updatePosition(el, tooltipElement, binding.arg || 'bottom')
    tooltipElement.style.opacity = '1'
  })
}

function hideTooltip() {
  if (tooltipElement) {
    tooltipElement.style.opacity = '0'
  }
}

function addTooltipListeners(el, binding) {
  // 如果已经添加过，先移除
  removeTooltipListeners(el)

  el._tooltipBinding = binding
  el._tooltipHandlers = {
    mouseenter: () => {
      if (hideTimer) {
        clearTimeout(hideTimer)
        hideTimer = null
      }

      showTimer = setTimeout(() => {
        showTooltip(el, el._tooltipBinding)
      }, 500)
    },
    mouseleave: () => {
      if (showTimer) {
        clearTimeout(showTimer)
        showTimer = null
      }

      hideTimer = setTimeout(() => {
        hideTooltip()
      }, 0)
    }
  }

  el.addEventListener('mouseenter', el._tooltipHandlers.mouseenter)
  el.addEventListener('mouseleave', el._tooltipHandlers.mouseleave)
}

function removeTooltipListeners(el) {
  if (el._tooltipHandlers) {
    el.removeEventListener('mouseenter', el._tooltipHandlers.mouseenter)
    el.removeEventListener('mouseleave', el._tooltipHandlers.mouseleave)
    el._tooltipHandlers = null
  }
}

export const tooltip = {
  mounted(el, binding) {
    // 如果有内容，添加事件监听器
    if (binding.value) {
      addTooltipListeners(el, binding)
    }
  },

  updated(el, binding) {
    // 保存最新的 binding
    el._tooltipBinding = binding

    if (binding.value) {
      // 有值：确保监听器已添加
      if (!el._tooltipHandlers) {
        addTooltipListeners(el, binding)
      }
      // 如果 tooltip 正在显示，更新内容
      if (tooltipElement && tooltipElement.style.opacity === '1') {
        tooltipElement.textContent = binding.value
        updatePosition(el, tooltipElement, binding.arg || 'bottom')
      }
    } else {
      // 无值：移除监听器并隐藏 tooltip
      removeTooltipListeners(el)
      hideTooltip()
    }
  },

  unmounted(el) {
    removeTooltipListeners(el)

    if (showTimer) {
      clearTimeout(showTimer)
    }
    if (hideTimer) {
      clearTimeout(hideTimer)
    }
  }
}

export default tooltip
