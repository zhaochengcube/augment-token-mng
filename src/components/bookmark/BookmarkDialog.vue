<template>
  <BaseModal
    :visible="true"
    :title="isEditing ? $t('bookmarks.editTitle') : $t('bookmarks.addTitle')"
    :show-close="true"
    :close-on-overlay="true"
    :close-on-esc="true"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="$emit('close')"
  >
    <div class="animate-fade-in">
      <!-- 书签名称 -->
      <div class="form-group">
        <label class="label">{{ $t('bookmarks.fields.name') }} *</label>
        <input
          v-model="formData.name"
          type="text"
          class="input"
          :placeholder="$t('bookmarks.placeholders.name')"
        />
      </div>

      <!-- 网站地址 -->
      <div class="form-group">
        <label class="label">{{ $t('bookmarks.fields.url') }}</label>
        <input
          v-model="formData.url"
          type="url"
          class="input"
          :placeholder="$t('bookmarks.placeholders.url')"
        />
      </div>

      <!-- 标签 -->
      <div class="form-group">
        <label class="label">{{ $t('bookmarks.fields.tag') }}</label>
        <div class="flex gap-2.5 items-center">
          <!-- 标签名称输入 + 下拉建议 -->
          <div class="dropdown flex-1" @click="showTagSuggestions = true">
            <input
              ref="tagNameInputRef"
              v-model="formData.tag"
              type="text"
              class="input !pr-9"
              :placeholder="$t('bookmarks.placeholders.tag')"
              maxlength="20"
              @input="handleTagInput"
              @focus="showTagSuggestions = true"
              @blur="handleTagBlur"
              @click.stop="showTagSuggestions = true"
            />
            <button
              v-if="formData.tag"
              type="button"
              class="btn btn--ghost btn--icon-sm absolute right-1.5 top-1/2 -translate-y-1/2"
              :title="$t('common.clear')"
              @click="formData.tag = ''"
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
          <!-- 圆形颜色选择器 -->
          <div class="relative shrink-0">
            <div
              class="w-[42px] h-[42px] border-2 border-border rounded-full shadow-sm"
              :style="{ backgroundColor: formData.tag_color }"
            ></div>
            <input
              type="color"
              v-model="formData.tag_color"
              :title="$t('bookmarks.fields.tagColor')"
              class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />
          </div>
        </div>
      </div>

      <!-- 描述 -->
      <div class="form-group mb-0">
        <label class="label">{{ $t('bookmarks.fields.description') }}</label>
        <textarea
          v-model="formData.description"
          class="input resize-none"
          rows="3"
          :placeholder="$t('bookmarks.placeholders.description')"
        ></textarea>
      </div>
    </div>

    <template #footer>
      <button class="btn btn--secondary" @click="$emit('close')">
        {{ $t('common.cancel') }}
      </button>
      <button class="btn btn--primary" @click="handleSubmit" :disabled="!canSubmit">
        {{ isEditing ? $t('common.save') : $t('common.add') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()

const props = defineProps({
  bookmark: {
    type: Object,
    default: null
  },
  // 所有书签，用于标签建议
  allBookmarks: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['save', 'close'])

const isEditing = computed(() => !!props.bookmark)

// 表单数据
const formData = ref({
  name: '',
  url: '',
  tag: '',
  tag_color: '#f97316',
  description: ''
})

// 标签建议相关
const showTagSuggestions = ref(false)
const tagNameInputRef = ref(null)

// 从所有书签中提取已使用的标签
const existingTags = computed(() => {
  if (!props.allBookmarks) return []
  const tagMap = new Map()

  props.allBookmarks.forEach(b => {
    if (b.tag) {
      if (!tagMap.has(b.tag)) {
        tagMap.set(b.tag, {
          name: b.tag,
          color: b.tag_color || '#f97316'
        })
      }
    }
  })

  return Array.from(tagMap.values())
    .sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
})

// 根据输入过滤标签建议
const filteredTagSuggestions = computed(() => {
  const input = formData.value.tag.trim().toLowerCase()

  if (!input) {
    return existingTags.value
  }

  return existingTags.value.filter(tag =>
    tag.name.toLowerCase().includes(input)
  )
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
  formData.value.tag = suggestion.name
  formData.value.tag_color = suggestion.color
  showTagSuggestions.value = false
}

// 是否可以提交
const canSubmit = computed(() => formData.value.name?.trim())

// 提交表单
const handleSubmit = () => {
  if (!canSubmit.value) return

  const result = {
    ...formData.value
  }

  emit('save', result)
}

// 初始化表单数据
onMounted(() => {
  if (props.bookmark) {
    formData.value = {
      name: props.bookmark.name || '',
      url: props.bookmark.url || '',
      tag: props.bookmark.tag || '',
      tag_color: props.bookmark.tag_color || '#f97316',
      description: props.bookmark.description || ''
    }
  }
})
</script>

<style scoped>
/* Fade-in animation for form content */
.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
