<template>
  <BaseModal
    :visible="true"
    :title="isEditing ? $t('platform.claude.dialog.editTitle') : $t('platform.claude.dialog.addTitle')"
    :show-close="true"
    :close-on-overlay="true"
    :close-on-esc="true"
    :body-scroll="false"
    modal-class="max-w-[500px]"
    @close="$emit('close')"
  >
    <div class="animate-fade-in">
      <!-- Tabs -->
      <div class="flex border-b border-border mb-4 gap-1">
        <button
          class="px-4 py-2 text-xs font-medium border border-transparent rounded-t-lg cursor-pointer transition-all duration-200 text-text-secondary hover:text-text hover:bg-bg-secondary"
          :class="{'text-accent border-border border-b-transparent bg-accent/10 -mb-px': activeTab === 'basic'}"
          @click="activeTab = 'basic'"
        >
          {{ $t('platform.claude.dialog.tabs.basic') }}
        </button>
        <button
          class="px-4 py-2 text-xs font-medium border border-transparent rounded-t-lg cursor-pointer transition-all duration-200 text-text-secondary hover:text-text hover:bg-bg-secondary"
          :class="{'text-accent border-border border-b-transparent bg-accent/10 -mb-px': activeTab === 'api'}"
          @click="activeTab = 'api'"
        >
          {{ $t('platform.claude.dialog.tabs.api') }}
        </button>
      </div>

      <!-- 基础信息 Tab -->
      <div v-show="activeTab === 'basic'" class="animate-fade-in">
        <!-- 服务名称 -->
        <div class="form-group">
          <label class="label">{{ $t('platform.claude.dialog.fields.serviceName') }} *</label>
          <input
            v-model="formData.service_name"
            type="text"
            class="input"
            :placeholder="$t('platform.claude.dialog.placeholders.serviceName')"
          />
        </div>

        <!-- 网站地址 -->
        <div class="form-group">
          <label class="label">{{ $t('platform.claude.dialog.fields.websiteUrl') }}</label>
          <input
            v-model="formData.website_url"
            type="url"
            class="input"
            :placeholder="$t('platform.claude.dialog.placeholders.websiteUrl')"
          />
        </div>

        <!-- 订阅时间：开始时间 + 时长 + 到期时间 -->
        <div class="form-group">
          <label class="label">{{ $t('platform.claude.dialog.fields.subscription') }}</label>
          <div class="grid grid-cols-3 gap-2">
            <!-- 开始时间 -->
            <div>
              <label class="text-xs text-text-secondary mb-1 block">{{ $t('platform.claude.dialog.fields.startDate') }}</label>
              <input
                v-model="formData.start_date"
                type="date"
                class="input"
                @change="calculateExpiryDate"
              />
            </div>
            <!-- 时长 -->
            <div>
              <label class="text-xs text-text-secondary mb-1 block">{{ $t('platform.claude.dialog.fields.duration') }}</label>
              <div class="flex items-center gap-1">
                <input
                  v-model.number="formData.duration_value"
                  type="number"
                  min="1"
                  class="input w-20"
                  placeholder="1"
                  @input="calculateExpiryDate"
                />
                <!-- 单位下拉选择 -->
                <FloatingDropdown placement="bottom-end" :offset="4">
                  <template #trigger="{ isOpen }">
                    <button
                      type="button"
                      class="input !pr-3 !py-2 text-[13px] min-w-[60px] text-left"
                    >
                      {{ selectedDurationUnitLabel }}
                    </button>
                  </template>
                  <template #default>
                    <button
                      v-for="unit in durationUnits"
                      :key="unit.value"
                      type="button"
                      class="dropdown-item"
                      :class="{ 'bg-accent/10': formData.duration_unit === unit.value }"
                      @click="selectDurationUnit(unit.value)"
                    >
                      {{ unit.label }}
                    </button>
                  </template>
                </FloatingDropdown>
              </div>
            </div>
            <!-- 到期时间 -->
            <div>
              <label class="text-xs text-text-secondary mb-1 block">{{ $t('platform.claude.dialog.fields.expiryDate') }}</label>
              <input
                v-model="formData.expiry_date"
                type="date"
                class="input"
              />
            </div>
          </div>
          <p class="text-xs text-text-tertiary mt-1">{{ $t('platform.claude.dialog.expiryHint') }}</p>
        </div>

        <!-- 标签 -->
        <div class="form-group">
          <label class="label">{{ $t('platform.claude.dialog.fields.tag') }}</label>
          <div class="flex gap-2.5 items-center">
            <!-- 标签名称输入 + 下拉建议 -->
            <div class="dropdown flex-1 relative" @click="showTagSuggestions = true">
              <input
                ref="tagNameInputRef"
                v-model="formData.tag"
                type="text"
                class="input !pr-9"
                :placeholder="$t('platform.claude.dialog.placeholders.tag')"
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
                      class="w-[18px] h-[18px] rounded-md shrink-0 shadow-sm"
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
                :title="$t('platform.claude.dialog.fields.tagColor')"
                class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
              />
            </div>
          </div>
        </div>

        <!-- 备注 -->
        <div class="form-group mb-0">
          <label class="label">{{ $t('platform.claude.dialog.fields.notes') }}</label>
          <textarea
            v-model="formData.notes"
            class="input resize-none"
            rows="2"
            :placeholder="$t('platform.claude.dialog.placeholders.notes')"
          ></textarea>
        </div>
      </div>

      <!-- API 配置 Tab -->
      <div v-show="activeTab === 'api'" class="animate-fade-in">
        <!-- Base URL -->
        <div class="form-group">
          <label class="label">{{ $t('platform.claude.dialog.fields.baseUrl') }} *</label>
          <input
            v-model="formData.base_url"
            type="url"
            class="input"
            :placeholder="$t('platform.claude.dialog.placeholders.baseUrl')"
          />
        </div>

        <!-- Auth Token -->
        <div class="form-group">
          <label class="label">{{ $t('platform.claude.dialog.fields.authToken') }} *</label>
          <div class="relative">
            <input
              v-model="formData.auth_token"
              :type="showToken ? 'text' : 'password'"
              class="input !pr-9"
              :placeholder="$t('platform.claude.dialog.placeholders.authToken')"
            />
            <button
              type="button"
              class="btn btn--ghost btn--icon-sm absolute right-1.5 top-1/2 -translate-y-1/2"
              :title="showToken ? $t('common.hide') : $t('common.show')"
              @click="showToken = !showToken"
            >
              <svg v-if="showToken" width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 0zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 7c2.76 0 5 2.24 5 5 0 .65-.13 1.26-.36 1.83l2.92 2.92c1.51-1.26 2.7-2.89 3.43-4.75-1.73-4.39-6-7.5-11-7.5-1.4 0-2.74.25-3.98.7l2.16 2.16C10.74 7.13 11.35 7 12 7zM2 4.27l2.28 2.28.46.46C3.08 8.3 1.78 10.02 1 12c1.73 4.39 6 7.5 11 7.5 1.55 0 3.03-.3 4.38-.84l.42.42L19.73 22 21 20.73 3.27 3 2 4.27zM7.53 9.8l1.55 1.55c-.05.21-.08.43-.08.65 0 1.66 1.34 3 3 3 .22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53-2.76 0-5-2.24-5-5 0-.79.2-1.53.53-2.2zm4.31-.78l3.15 3.15.02-.16c0-1.66-1.34-3-3-3l-.17.01z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- 默认模型配置 -->
        <div class="form-group mb-0">
          <label class="label">{{ $t('platform.claude.dialog.fields.defaultModels') }}</label>
          <div class="grid grid-cols-3 gap-2">
            <div>
              <label class="text-xs text-text-secondary mb-1 block">{{ $t('platform.claude.dialog.fields.opusModel') }}</label>
              <input
                v-model="formData.default_opus_model"
                type="text"
                class="input"
                placeholder="claude-4-5-opus"
              />
            </div>
            <div>
              <label class="text-xs text-text-secondary mb-1 block">{{ $t('platform.claude.dialog.fields.sonnetModel') }}</label>
              <input
                v-model="formData.default_sonnet_model"
                type="text"
                class="input"
                placeholder="claude-4-5-sonnet"
              />
            </div>
            <div>
              <label class="text-xs text-text-secondary mb-1 block">{{ $t('platform.claude.dialog.fields.haikuModel') }}</label>
              <input
                v-model="formData.default_haiku_model"
                type="text"
                class="input"
                placeholder="claude-4-5-haiku"
              />
            </div>
          </div>
        </div>

        <!-- 使用模型选择 -->
        <div class="form-group mt-3">
          <label class="label">{{ $t('platform.claude.dialog.fields.useModel') }}</label>
          <FloatingDropdown placement="bottom-end" :offset="4">
            <template #trigger="{ isOpen }">
              <button type="button" class="input !pr-3 !py-2 text-[13px] text-left">
                {{ selectedUseModelLabel }}
              </button>
            </template>
            <template #default>
              <button
                v-for="option in useModelOptions"
                :key="option.value"
                type="button"
                class="dropdown-item"
                :class="{ 'bg-accent/10': formData.use_model === option.value }"
                @click="formData.use_model = option.value"
              >
                {{ option.label }}
              </button>
            </template>
          </FloatingDropdown>
          <p class="text-xs text-text-tertiary mt-1">{{ $t('platform.claude.dialog.useModelHint') }}</p>
        </div>
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
import FloatingDropdown from '@/components/common/FloatingDropdown.vue'

const { t: $t } = useI18n()

const props = defineProps({
  account: {
    type: Object,
    default: null
  },
  // 所有账户，用于标签建议
  allAccounts: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['save', 'close'])

const isEditing = computed(() => !!props.account)
const activeTab = ref('basic')
const showToken = ref(false)

// 时长单位选项
const durationUnits = computed(() => [
  { label: $t('platform.claude.dialog.durationUnits.days'), value: 'days' },
  { label: $t('platform.claude.dialog.durationUnits.weeks'), value: 'weeks' },
  { label: $t('platform.claude.dialog.durationUnits.months'), value: 'months' },
  { label: $t('platform.claude.dialog.durationUnits.years'), value: 'years' }
])

// 当前选中的单位标签
const selectedDurationUnitLabel = computed(() => {
  const unit = durationUnits.value.find(u => u.value === formData.value.duration_unit)
  return unit?.label || $t('platform.claude.dialog.durationUnits.days')
})

// 格式化日期为 YYYY-MM-DD 格式（用于 date input）
const formatDateForInput = (timestamp) => {
  if (!timestamp) return ''
  try {
    const date = new Date(timestamp * 1000)
    if (isNaN(date.getTime())) return ''
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    return `${year}-${month}-${day}`
  } catch {
    return ''
  }
}

// 将日期字符串转换为时间戳
const dateToTimestamp = (dateStr) => {
  if (!dateStr) return null
  const date = new Date(dateStr)
  return Math.floor(date.getTime() / 1000)
}

// 表单数据
const formData = ref({
  service_name: '',
  website_url: '',
  start_date: '',
  duration_value: null,
  duration_unit: 'days',
  expiry_date: '',
  tag: '',
  tag_color: '#f97316',
  notes: '',
  base_url: '',
  auth_token: '',
  default_opus_model: '',
  default_sonnet_model: '',
  default_haiku_model: '',
  use_model: 'default'
})

// 使用模型选项
const useModelOptions = computed(() => [
  { label: $t('platform.claude.dialog.useModels.default'), value: 'default' },
  { label: $t('platform.claude.dialog.useModels.opus'), value: 'opus' },
  { label: $t('platform.claude.dialog.useModels.haiku'), value: 'haiku' }
])

// 当前选中的使用模型标签
const selectedUseModelLabel = computed(() => {
  const option = useModelOptions.value.find(o => o.value === formData.value.use_model)
  return option?.label || $t('platform.claude.dialog.useModels.default')
})

// 根据开始时间和时长计算到期时间
const calculateExpiryDate = () => {
  if (formData.value.start_date && formData.value.duration_value) {
    const startDate = new Date(formData.value.start_date)
    const value = formData.value.duration_value

    switch (formData.value.duration_unit) {
      case 'days':
        startDate.setDate(startDate.getDate() + value)
        break
      case 'weeks':
        startDate.setDate(startDate.getDate() + (value * 7))
        break
      case 'months':
        startDate.setMonth(startDate.getMonth() + value)
        break
      case 'years':
        startDate.setFullYear(startDate.getFullYear() + value)
        break
    }

    formData.value.expiry_date = startDate.toISOString().split('T')[0]
  }
}

// 选择时长单位
const selectDurationUnit = (unit) => {
  formData.value.duration_unit = unit
  calculateExpiryDate()
}

// 标签建议相关
const showTagSuggestions = ref(false)
const tagNameInputRef = ref(null)

// 从所有账户中提取已使用的标签
const existingTags = computed(() => {
  if (!props.allAccounts) return []
  const tagMap = new Map()

  props.allAccounts.forEach(acc => {
    if (acc.tag && acc.tag_color) {
      if (!tagMap.has(acc.tag)) {
        tagMap.set(acc.tag, {
          name: acc.tag,
          color: acc.tag_color
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
const canSubmit = computed(() => {
  return formData.value.service_name?.trim() &&
         formData.value.base_url?.trim() &&
         formData.value.auth_token?.trim()
})

// 提交表单
const handleSubmit = () => {
  if (!canSubmit.value) return

  // 将时长转换为天数
  let durationDays = 0
  if (formData.value.duration_value) {
    switch (formData.value.duration_unit) {
      case 'days':
        durationDays = formData.value.duration_value
        break
      case 'weeks':
        durationDays = formData.value.duration_value * 7
        break
      case 'months':
        durationDays = formData.value.duration_value * 30
        break
      case 'years':
        durationDays = formData.value.duration_value * 365
        break
    }
  }

  const result = {
    service_name: formData.value.service_name.trim(),
    website_url: formData.value.website_url?.trim() || null,
    start_date: dateToTimestamp(formData.value.start_date) || 0,
    duration_days: durationDays,
    expiry_date: dateToTimestamp(formData.value.expiry_date) || 0,
    tag: formData.value.tag?.trim() || '',
    tag_color: formData.value.tag_color || '#f97316',
    notes: formData.value.notes?.trim() || '',
    base_url: formData.value.base_url.trim(),
    auth_token: formData.value.auth_token.trim(),
    default_opus_model: formData.value.default_opus_model?.trim() || '',
    default_sonnet_model: formData.value.default_sonnet_model?.trim() || '',
    default_haiku_model: formData.value.default_haiku_model?.trim() || '',
    use_model: formData.value.use_model || 'default'
  }

  emit('save', result)
}

// 初始化表单数据
onMounted(() => {
  if (props.account) {
    formData.value = {
      service_name: props.account.service_name || '',
      website_url: props.account.website_url || '',
      start_date: formatDateForInput(props.account.start_date),
      duration_value: props.account.duration_days || null,
      duration_unit: 'days',
      expiry_date: formatDateForInput(props.account.expiry_date),
      tag: props.account.tag || '',
      tag_color: props.account.tag_color || '#f97316',
      notes: props.account.notes || '',
      base_url: props.account.base_url || '',
      auth_token: props.account.auth_token || '',
      default_opus_model: props.account.default_opus_model || '',
      default_sonnet_model: props.account.default_sonnet_model || '',
      default_haiku_model: props.account.default_haiku_model || '',
      use_model: props.account.use_model || 'default'
    }
  }
})
</script>

<style scoped>
/* Animations only */
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
