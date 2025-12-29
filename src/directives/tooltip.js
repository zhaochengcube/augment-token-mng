// Tooltip directive for Vue 3
let tooltipElement = null
let showTimer = null
let hideTimer = null

function createTooltipElement() {
  const el = document.createElement('div')
  el.className = 'v-tooltip'
  el.style.cssText = `
    position: fixed;
    z-index: 10000;
    padding: 6px 10px;
    background: var(--bg-surface-alt);
    color: var(--text);
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.15s ease;
    box-shadow: var(--shadow-elevated);
    border: 1px solid var(--border-strong);
  `
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

export const tooltip = {
  mounted(el, binding) {
    // 如果没有内容，不添加事件监听器
    if (!binding.value) return

    el._tooltipHandlers = {
      mouseenter: () => {
        if (hideTimer) {
          clearTimeout(hideTimer)
          hideTimer = null
        }

        showTimer = setTimeout(() => {
          showTooltip(el, binding)
        }, binding.modifiers.fast ? 100 : 200)
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
  },
  
  updated(el, binding) {
    // 如果内容改变了，更新 tooltip
    if (tooltipElement && tooltipElement.style.opacity === '1') {
      tooltipElement.textContent = binding.value
      updatePosition(el, tooltipElement, binding.arg || 'bottom')
    }
  },
  
  unmounted(el) {
    if (el._tooltipHandlers) {
      el.removeEventListener('mouseenter', el._tooltipHandlers.mouseenter)
      el.removeEventListener('mouseleave', el._tooltipHandlers.mouseleave)
    }
    
    if (showTimer) {
      clearTimeout(showTimer)
    }
    if (hideTimer) {
      clearTimeout(hideTimer)
    }
  }
}

export default tooltip

