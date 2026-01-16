<template>
  <BaseModal
    :visible="visible"
    :title="modalTitle"
    :close-on-overlay="true"
    :body-scroll="false"
    modal-class="!max-w-[360px]"
    @close="handleClose"
  >
    <div class="flex flex-col gap-2.5 overflow-visible">
      <label class="label">{{ $t('tokenForm.tagLabel') }}</label>
      <div class="flex gap-2.5 items-center">
        <div class="dropdown flex-1" @click="showTagSuggestions = true">
          <input
            ref="tagNameInputRef"
            v-model="editingTagName"
            type="text"
            class="input !pr-9"
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
            class="btn btn--ghost btn--icon-sm absolute right-1.5 top-1/2 -translate-y-1/2"
            :title="$t('tokenForm.clearTag')"
            @click="editingTagName = ''"
          >
            ×
          </button>
          <Transition name="dropdown">
            <div
              v-if="showTagSuggestions && filteredTagSuggestions.length > 0"
              class="dropdown-menu dropdown-menu--left w-full max-h-[200px] overflow-y-auto"
              @mousedown.prevent
            >
              <div
                v-for="suggestion in filteredTagSuggestions"
                :key="suggestion.name"
                class="dropdown-item"
                @mousedown.prevent="selectTagSuggestion(suggestion)"
              >
                <span
                  class="w-4.5 h-4.5 rounded-md shrink-0 shadow-sm"
                  :style="{ backgroundColor: suggestion.color }"
                ></span>
                <span class="text-[14px] text-text">{{ suggestion.name }}</span>
              </div>
            </div>
          </Transition>
        </div>
        <div class="relative shrink-0">
          <div
            class="w-[42px] h-[42px] border-2 border-border rounded-full shadow-sm"
            :style="{ backgroundColor: editingTagColor }"
          ></div>
          <input
            ref="tagColorInputRef"
            type="color"
            v-model="editingTagColor"
            :title="$t('tokenForm.tagColorPicker')"
            class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
          />
        </div>
      </div>
      <div v-if="editingTagName" class="flex items-center gap-2.5 mt-3">
        <span class="badge editable" :style="{ '--tag-color': editingTagColor }">
          {{ editingTagName }}
        </span>
      </div>
      <!-- 批量模式提示 -->
      <div v-if="isBatchMode" class="mt-3 px-3.5 py-3 bg-accent-tech border border-border-accent-tech rounded-lg text-[13px] text-accent">
        {{ $t('tokenList.batchTagHint', { count: tokens?.length || 0 }) }}
      </div>
    </div>

    <template #footer>
      <button @click="handleClear" class="btn btn--secondary btn--md" v-if="showClearButton">
        {{ $t('tokenForm.clearTag') }}
      </button>
      <button @click="handleClose" class="btn btn--ghost btn--md">
        {{ $t('common.cancel') }}
      </button>
      <button @click="handleSave" class="btn btn--primary btn--md">
        {{ $t('common.confirm') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

const { t } = useI18n()

const resolveCssVar = (name, fallback) => {
  if (typeof window === 'undefined') return fallback
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  return value || fallback
}

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
const defaultTagColor = resolveCssVar('--tag-default', '#f97316')
const editingTagName = ref('')
const editingTagColor = ref(defaultTagColor)
const showTagSuggestions = ref(false)
const tagNameInputRef = ref(null)

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
      editingTagColor.value = defaultTagColor
    } else if (props.token) {
      // 单个模式：使用当前 token 的值
      editingTagName.value = props.token.tag_name || ''
      editingTagColor.value = props.token.tag_color || defaultTagColor
    }
    showTagSuggestions.value = false
  }
})

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
