<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="visible" class="tag-editor-overlay" @click="handleClose">
        <div class="tag-editor-modal" @click.stop>
          <div class="modal-header">
            <h3>{{ modalTitle }}</h3>
            <button @click="handleClose" class="modal-close">×</button>
          </div>
          <div class="modal-content">
            <div class="tag-group">
              <label>{{ $t('tokenForm.tagLabel') }}</label>
              <div class="tag-input-row">
                <div class="tag-autocomplete-wrapper" @click="showTagSuggestions = true">
                  <input
                    ref="tagNameInputRef"
                    v-model="editingTagName"
                    type="text"
                    class="tag-name-input"
                    :placeholder="$t('tokenForm.tagPlaceholder')"
                    maxlength="20"
                    @input="handleTagInput"
                    @focus="showTagSuggestions = true"
                    @blur="handleTagBlur"
                    @click.stop="showTagSuggestions = true"
                  />
                  <button
                    v-if="editingTagName"
                    type="button"
                    class="tag-clear-btn"
                    :title="$t('tokenForm.clearTag')"
                    @click="editingTagName = ''"
                  >
                    ×
                  </button>
                  <Transition name="dropdown">
                    <div v-if="showTagSuggestions && filteredTagSuggestions.length > 0" class="tag-suggestions">
                      <div
                        v-for="suggestion in filteredTagSuggestions"
                        :key="suggestion.name"
                        class="tag-suggestion-item"
                        @mousedown.prevent="selectTagSuggestion(suggestion)"
                      >
                        <span
                          class="tag-preview"
                          :style="{ backgroundColor: suggestion.color }"
                        ></span>
                        <span class="tag-suggestion-name">{{ suggestion.name }}</span>
                      </div>
                    </div>
                  </Transition>
                </div>
                <div class="tag-color-wrapper">
                  <button
                    type="button"
                    class="tag-color-display"
                    :style="{ backgroundColor: editingTagColor }"
                    :title="$t('tokenForm.tagColorPicker')"
                    @click="openTagColorPicker"
                  ></button>
                  <input
                    ref="tagColorInputRef"
                    type="color"
                    v-model="editingTagColor"
                    class="hidden-color-input"
                  />
                </div>
              </div>
              <div v-if="editingTagName" class="tag-preview-row">
                <span class="tag-badge-preview" :style="{ backgroundColor: editingTagColor, color: getContrastColor(editingTagColor) }">
                  {{ editingTagName }}
                </span>
              </div>
              <!-- 批量模式提示 -->
              <div v-if="isBatchMode" class="batch-hint">
                {{ $t('tokenList.batchTagHint', { count: tokens?.length || 0 }) }}
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <button @click="handleClear" class="btn-secondary" v-if="showClearButton">
              {{ $t('tokenForm.clearTag') }}
            </button>
            <button @click="handleClose" class="btn-secondary">
              {{ $t('common.cancel') }}
            </button>
            <button @click="handleSave" class="btn-primary">
              {{ $t('common.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  // 单个 token 编辑模式
  token: {
    type: Object,
    default: null
  },
  // 批量编辑模式
  tokens: {
    type: Array,
    default: null
  },
  // 所有 tokens，用于标签建议
  allTokens: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['update:visible', 'save', 'clear'])

// 编辑状态
const editingTagName = ref('')
const editingTagColor = ref('#f97316')
const showTagSuggestions = ref(false)
const tagNameInputRef = ref(null)
const tagColorInputRef = ref(null)

// 计算属性
const isBatchMode = computed(() => props.tokens && props.tokens.length > 0)

const modalTitle = computed(() => {
  if (isBatchMode.value) {
    return t('tokenList.batchEditTag')
  }
  return t('tokenList.editTag')
})

const showClearButton = computed(() => {
  if (isBatchMode.value) {
    // 批量模式：只要有任何一个 token 有标签就显示清除按钮
    return props.tokens?.some(token => token.tag_name)
  }
  // 单个模式：当前 token 有标签才显示
  return props.token?.tag_name
})

// 从所有 tokens 中提取已使用的标签
const existingTags = computed(() => {
  if (!props.allTokens) return []
  const tagMap = new Map()

  props.allTokens.forEach(token => {
    if (token.tag_name && token.tag_color) {
      if (!tagMap.has(token.tag_name)) {
        tagMap.set(token.tag_name, {
          name: token.tag_name,
          color: token.tag_color
        })
      }
    }
  })

  return Array.from(tagMap.values())
    .sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
})

// 根据输入过滤标签建议
const filteredTagSuggestions = computed(() => {
  const input = editingTagName.value.trim().toLowerCase()

  if (!input) {
    return existingTags.value
  }

  return existingTags.value.filter(tag =>
    tag.name.toLowerCase().includes(input)
  )
})

// 监听 visible 变化，初始化编辑状态
watch(() => props.visible, (newVal) => {
  if (newVal) {
    if (isBatchMode.value) {
      // 批量模式：使用默认值
      editingTagName.value = ''
      editingTagColor.value = '#f97316'
    } else if (props.token) {
      // 单个模式：使用当前 token 的值
      editingTagName.value = props.token.tag_name || ''
      editingTagColor.value = props.token.tag_color || '#f97316'
    }
    showTagSuggestions.value = false
  }
})

// 获取对比色
const getContrastColor = (hex) => {
  if (!/^#[0-9a-fA-F]{6}$/.test(hex)) return '#ffffff'
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  return luminance > 0.6 ? '#1f2937' : '#ffffff'
}

// 处理标签输入
const handleTagInput = () => {
  showTagSuggestions.value = true
}

// 处理标签输入框失焦
const handleTagBlur = () => {
  setTimeout(() => {
    showTagSuggestions.value = false
  }, 200)
}

// 选择标签建议
const selectTagSuggestion = (suggestion) => {
  editingTagName.value = suggestion.name
  editingTagColor.value = suggestion.color
  showTagSuggestions.value = false
}

// 打开颜色选择器
const openTagColorPicker = () => {
  tagColorInputRef.value?.click()
}

// 关闭模态框
const handleClose = () => {
  emit('update:visible', false)
}

// 保存标签
const handleSave = () => {
  emit('save', {
    tagName: editingTagName.value.trim(),
    tagColor: editingTagColor.value
  })
  handleClose()
}

// 清除标签
const handleClear = () => {
  emit('clear')
  handleClose()
}
</script>

<style scoped>
/* Modal 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .tag-editor-modal,
.modal-leave-active .tag-editor-modal {
  transition: transform 0.2s ease;
}

.modal-enter-from .tag-editor-modal,
.modal-leave-to .tag-editor-modal {
  transform: scale(0.95);
}

/* Dropdown 过渡动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* 标签编辑器样式 */
.tag-editor-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(2px);
}

.tag-editor-modal {
  background: var(--bg-surface);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 400px;
  width: 90%;
  overflow: visible;
}

.tag-editor-modal .modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--border);
}

.tag-editor-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
}

.tag-editor-modal .modal-close {
  background: none;
  border: none;
  font-size: 24px;
  color: var(--text-muted);
  cursor: pointer;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
  line-height: 1;
}

.tag-editor-modal .modal-close:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.tag-editor-modal .modal-content {
  padding: 20px 24px;
  overflow: visible;
}

.tag-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: visible;
}

.tag-group label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
}

.tag-input-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.tag-autocomplete-wrapper {
  flex: 1;
  position: relative;
}

.tag-name-input {
  width: 100%;
  padding: 10px 32px 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  color: var(--text);
  background: var(--bg-surface);
  outline: none;
  transition: border-color 0.15s ease;
  box-sizing: border-box;
}

.tag-name-input:focus {
  border-color: var(--accent);
}

.tag-clear-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  padding: 0;
  border: none;
  background: var(--bg-muted);
  color: var(--text-muted);
  border-radius: 50%;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.tag-clear-btn:hover {
  background: #ef4444;
  color: #ffffff;
}

.tag-color-wrapper {
  position: relative;
  flex-shrink: 0;
}

.tag-color-display {
  width: 40px;
  height: 40px;
  border: 2px solid var(--border);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tag-color-display:hover {
  border-color: var(--accent);
  transform: scale(1.08);
}

.hidden-color-input {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  width: 1px;
  height: 1px;
  pointer-events: none;
}

.tag-suggestions {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-height: 200px;
  overflow-y: auto;
  z-index: 1000;
  margin-top: 4px;
}

.tag-suggestion-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.tag-suggestion-item:hover,
.tag-suggestion-item.selected {
  background: var(--bg-hover);
}

.tag-suggestion-item .tag-preview {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  flex-shrink: 0;
}

.tag-suggestion-name {
  font-size: 14px;
  color: var(--text);
}

.tag-preview-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}

.tag-badge-preview {
  display: inline-block;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 12px;
}

.batch-hint {
  margin-top: 12px;
  padding: 10px 12px;
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  border-radius: 6px;
  font-size: 13px;
  color: var(--accent);
}

.tag-editor-modal .modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  background: var(--bg-muted);
  border-radius: 0 0 12px 12px;
}

.tag-editor-modal .btn-primary,
.tag-editor-modal .btn-secondary {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tag-editor-modal .btn-primary {
  background: var(--accent);
  color: var(--text-contrast);
  border: none;
}

.tag-editor-modal .btn-primary:hover {
  background: color-mix(in srgb, var(--accent) 90%, black);
}

.tag-editor-modal .btn-secondary {
  background: var(--bg-surface);
  color: var(--text);
  border: 1px solid var(--border);
}

.tag-editor-modal .btn-secondary:hover {
  background: var(--bg-hover);
}

</style>

