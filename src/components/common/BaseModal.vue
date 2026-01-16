<template>
  <Teleport v-if="teleport" to="body">
    <Transition name="modal" appear>
      <div v-if="visible" class="modal-overlay" @click.self="onOverlayClick">
        <div
          ref="modalRef"
          :class="['modal', modalClass]"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="computedLabelledby"
          :aria-describedby="ariaDescribedby || undefined"
          tabindex="-1"
          @click.stop
        >
          <div v-if="hasHeader" class="modal-header">
            <slot name="header">
              <h3 v-if="title" :id="titleId" class="modal-title">{{ title }}</h3>
            </slot>
            <button
              v-if="showClose"
              type="button"
              class="btn btn--ghost btn--icon"
              aria-label="Close"
              @click="emitClose"
            >
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
          <div :class="['modal-body', bodyScroll ? 'modal-body-scroll' : null]">
            <slot />
          </div>
          <div v-if="$slots.footer" class="modal-footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
  <Transition v-else name="modal" appear>
    <div v-if="visible" class="modal-overlay" @click.self="onOverlayClick">
      <div
        ref="modalRef"
        :class="['modal', modalClass]"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="computedLabelledby"
        :aria-describedby="ariaDescribedby || undefined"
        tabindex="-1"
        @click.stop
      >
        <div v-if="hasHeader" class="modal-header">
          <slot name="header">
            <h3 v-if="title" :id="titleId" class="modal-title">{{ title }}</h3>
          </slot>
          <button
            v-if="showClose"
            type="button"
            class="btn btn--ghost btn--icon"
            aria-label="Close"
            @click="emitClose"
          >
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div :class="['modal-body', bodyScroll ? 'modal-body-scroll' : null]">
          <slot />
        </div>
        <div v-if="$slots.footer" class="modal-footer">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { computed, getCurrentInstance, onBeforeUnmount, ref, watch, useSlots } from 'vue'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: ''
  },
  closeOnOverlay: {
    type: Boolean,
    default: true
  },
  closeOnEsc: {
    type: Boolean,
    default: true
  },
  showClose: {
    type: Boolean,
    default: true
  },
  bodyScroll: {
    type: Boolean,
    default: true
  },
  trapFocus: {
    type: Boolean,
    default: true
  },
  lockScroll: {
    type: Boolean,
    default: true
  },
  teleport: {
    type: Boolean,
    default: true
  },
  ariaLabelledby: {
    type: String,
    default: ''
  },
  ariaDescribedby: {
    type: String,
    default: ''
  },
  modalClass: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['close'])
const slots = useSlots()
const modalRef = ref(null)
const titleId = `modal-title-${getCurrentInstance()?.uid ?? Math.random().toString(36).slice(2)}`
const hasHeader = computed(() => Boolean(slots.header || props.title || props.showClose))
const computedLabelledby = computed(() => props.ariaLabelledby || (props.title ? titleId : undefined))

let lastFocused = null
let activeKeyHandler = null

const lockBodyScroll = () => {
  if (typeof document === 'undefined') return
  if (!props.lockScroll) return
  const count = Number(document.body.dataset.modalLockCount || 0)
  if (count === 0) {
    document.body.dataset.modalPrevOverflow = document.body.style.overflow || ''
    document.body.style.overflow = 'hidden'
  }
  document.body.dataset.modalLockCount = String(count + 1)
}

const unlockBodyScroll = () => {
  if (typeof document === 'undefined') return
  if (!props.lockScroll) return
  const count = Number(document.body.dataset.modalLockCount || 0)
  const nextCount = Math.max(0, count - 1)
  document.body.dataset.modalLockCount = String(nextCount)
  if (nextCount === 0) {
    document.body.style.overflow = document.body.dataset.modalPrevOverflow || ''
    delete document.body.dataset.modalPrevOverflow
    delete document.body.dataset.modalLockCount
  }
}

const getFocusable = () => {
  if (!modalRef.value) return []
  return Array.from(
    modalRef.value.querySelectorAll(
      'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
    )
  )
}

const handleKeydown = (event) => {
  if (!props.visible) return
  if (event.key === 'Escape' && props.closeOnEsc) {
    event.stopPropagation()
    emitClose()
    return
  }
  if (!props.trapFocus || event.key !== 'Tab') return

  const focusable = getFocusable()
  if (focusable.length === 0) {
    event.preventDefault()
    return
  }
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  const active = document.activeElement

  if (event.shiftKey && active === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && active === last) {
    event.preventDefault()
    first.focus()
  }
}

const emitClose = () => {
  emit('close')
}

const onOverlayClick = () => {
  if (props.closeOnOverlay) {
    emitClose()
  }
}

const addKeyListener = () => {
  if (typeof document === 'undefined') return
  if (activeKeyHandler) return
  activeKeyHandler = handleKeydown
  document.addEventListener('keydown', activeKeyHandler)
}

const removeKeyListener = () => {
  if (typeof document === 'undefined') return
  if (!activeKeyHandler) return
  document.removeEventListener('keydown', activeKeyHandler)
  activeKeyHandler = null
}

watch(
  () => props.visible,
  (isVisible) => {
    if (typeof document === 'undefined') return
    if (isVisible) {
      lastFocused = document.activeElement
      lockBodyScroll()
      addKeyListener()
    } else {
      removeKeyListener()
      unlockBodyScroll()
      if (lastFocused && typeof lastFocused.focus === 'function') {
        lastFocused.focus()
      }
      lastFocused = null
    }
  },
  { immediate: true }
)

onBeforeUnmount(() => {
  removeKeyListener()
  if (props.visible) {
    unlockBodyScroll()
  }
})
</script>
