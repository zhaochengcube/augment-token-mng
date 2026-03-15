// Tooltip directive for Vue 3
let tooltipElement = null
let showTimer = null
let hideTimer = null
let currentTriggerEl = null

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

  currentTriggerEl = el
  tooltipElement.textContent = binding.value
  tooltipElement.style.opacity = '0'
  tooltipElement.style.top = '-9999px'
  tooltipElement.style.left = '-9999px'

  requestAnimationFrame(() => {
    if (currentTriggerEl !== el) return
    updatePosition(el, tooltipElement, binding.arg || 'bottom')
    tooltipElement.style.opacity = '1'
  })
}

function hideTooltip() {
  if (tooltipElement) {
    tooltipElement.style.opacity = '0'
  }
  currentTriggerEl = null
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
    el._tooltipBinding = binding

    if (binding.value) {
      if (!el._tooltipHandlers) {
        addTooltipListeners(el, binding)
      }
      if (tooltipElement && tooltipElement.style.opacity === '1' && currentTriggerEl === el) {
        tooltipElement.textContent = binding.value
        updatePosition(el, tooltipElement, binding.arg || 'bottom')
      }
    } else {
      removeTooltipListeners(el)
      if (currentTriggerEl === el) {
        hideTooltip()
      }
    }
  },

  unmounted(el) {
    removeTooltipListeners(el)

    if (currentTriggerEl === el) {
      hideTooltip()
      if (showTimer) {
        clearTimeout(showTimer)
        showTimer = null
      }
      if (hideTimer) {
        clearTimeout(hideTimer)
        hideTimer = null
      }
    }
  }
}

export default tooltip
