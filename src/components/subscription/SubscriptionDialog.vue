<template>
  <BaseModal
    :visible="true"
    :title="isEditing ? $t('subscriptions.editTitle') : $t('subscriptions.addTitle')"
    :show-close="true"
    :close-on-overlay="true"
    :close-on-esc="true"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="$emit('close')"
  >
    <div class="animate-fade-in">
      <!-- 网站名称 -->
      <div class="form-group">
        <label class="label">{{ $t('subscriptions.fields.website') }} *</label>
        <input
          v-model="formData.website"
          type="text"
          class="input"
          :placeholder="$t('subscriptions.placeholders.website')"
        />
      </div>

      <!-- 网站地址 -->
      <div class="form-group">
        <label class="label">{{ $t('subscriptions.fields.websiteUrl') }}</label>
        <input
          v-model="formData.website_url"
          type="url"
          class="input"
          :placeholder="$t('subscriptions.placeholders.websiteUrl')"
        />
      </div>

      <!-- 订阅时间：开始时间 + 时长 + 到期时间 -->
      <div class="form-group">
        <label class="label">{{ $t('subscriptions.fields.subscription') }}</label>
        <div class="grid grid-cols-3 gap-2">
          <!-- 开始时间 -->
          <div>
            <label class="text-xs text-text-secondary mb-1 block">{{ $t('subscriptions.fields.startDate') }}</label>
            <input
              v-model="formData.start_date"
              type="date"
              class="input"
              @change="calculateExpiryDate"
            />
          </div>
          <!-- 时长（月） -->
          <div>
            <label class="text-xs text-text-secondary mb-1 block">{{ $t('subscriptions.fields.duration') }}</label>
            <div class="flex items-center gap-1">
              <input
                v-model.number="formData.duration_months"
                type="number"
                min="1"
                class="input"
                :placeholder="$t('subscriptions.placeholders.duration')"
                @input="calculateExpiryDate"
              />
              <span class="text-text-secondary text-sm whitespace-nowrap">{{ $t('subscriptions.fields.months') }}</span>
            </div>
          </div>
          <!-- 到期时间 -->
          <div>
            <label class="text-xs text-text-secondary mb-1 block">{{ $t('subscriptions.fields.expiryDate') }}</label>
            <input
              v-model="formData.expiry_date"
              type="date"
              class="input"
            />
          </div>
        </div>
        <p class="text-xs text-text-tertiary mt-1">{{ $t('subscriptions.expiryHint') }}</p>
      </div>

      <!-- 费用 -->
      <div class="form-group">
        <label class="label">{{ $t('subscriptions.fields.cost') }}</label>
        <input
          v-model.number="formData.cost"
          type="number"
          step="0.01"
          min="0"
          class="input"
          :placeholder="$t('subscriptions.placeholders.cost')"
        />
      </div>

      <!-- 标签 -->
      <div class="form-group">
        <label class="label">{{ $t('subscriptions.fields.tag') }}</label>
        <div class="flex gap-2.5 items-center">
          <!-- 标签名称输入 + 下拉建议 -->
          <div class="dropdown flex-1" @click="showTagSuggestions = true">
            <input
              ref="tagNameInputRef"
              v-model="formData.tag"
              type="text"
              class="input !pr-9"
              :placeholder="$t('subscriptions.placeholders.tag')"
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
              :title="$t('subscriptions.fields.tagColor')"
              class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />
          </div>
        </div>
      </div>

      <!-- 备注 -->
      <div class="form-group mb-0">
        <label class="label">{{ $t('subscriptions.fields.notes') }}</label>
        <textarea
          v-model="formData.notes"
          class="input resize-none"
          rows="3"
          :placeholder="$t('subscriptions.placeholders.notes')"
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
  subscription: {
    type: Object,
    default: null
  },
  // 所有订阅，用于标签建议
  allSubscriptions: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['save', 'close'])

const isEditing = computed(() => !!props.subscription)

// 表单数据
const formData = ref({
  website: '',
  website_url: '',
  start_date: '',
  duration_months: null,
  expiry_date: '',
  cost: null,
  tag: '',
  tag_color: '#3b82f6',
  notes: ''
})

// 根据开始时间和时长计算到期时间
const calculateExpiryDate = () => {
  if (formData.value.start_date && formData.value.duration_months) {
    const startDate = new Date(formData.value.start_date)
    startDate.setMonth(startDate.getMonth() + formData.value.duration_months)
    formData.value.expiry_date = startDate.toISOString().split('T')[0]
  }
}

// 标签建议相关
const showTagSuggestions = ref(false)
const tagNameInputRef = ref(null)

// 从所有订阅中提取已使用的标签
const existingTags = computed(() => {
  if (!props.allSubscriptions) return []
  const tagMap = new Map()

  props.allSubscriptions.forEach(sub => {
    if (sub.tag && sub.tag_color) {
      if (!tagMap.has(sub.tag)) {
        tagMap.set(sub.tag, {
          name: sub.tag,
          color: sub.tag_color
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
const canSubmit = computed(() => formData.value.website?.trim())

// 提交表单
const handleSubmit = () => {
  if (!canSubmit.value) return
  emit('save', { ...formData.value })
}

// 初始化表单数据
onMounted(() => {
  if (props.subscription) {
    formData.value = {
      website: props.subscription.website || '',
      website_url: props.subscription.website_url || '',
      start_date: props.subscription.start_date || '',
      duration_months: props.subscription.duration_months || null,
      expiry_date: props.subscription.expiry_date || '',
      cost: props.subscription.cost || null,
      tag: props.subscription.tag || '',
      tag_color: props.subscription.tag_color || '#3b82f6',
      notes: props.subscription.notes || ''
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

