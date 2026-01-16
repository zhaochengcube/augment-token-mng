<template>
  <BaseModal
    :visible="true"
    :title="$t('fontConfig.title')"
    :body-scroll="false"
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-5">
      <!-- 字体输入 -->
      <div class="form-group !mb-0">
        <label for="fontName" class="label">
          {{ $t('fontConfig.fontName') }}:
        </label>
        <input
          id="fontName"
          v-model="fontName"
          type="text"
          class="input"
          :placeholder="$t('fontConfig.placeholder')"
        >
        <small class="text-[12px] text-text-muted mt-1 block">{{ $t('fontConfig.help') }}</small>
      </div>

      <!-- 预览区域 -->
      <div class="form-group !mb-0">
        <label class="label">{{ $t('fontConfig.preview') }}:</label>
        <div 
          class="p-4 rounded-lg border border-border bg-muted"
          :style="{ fontFamily: previewFontFamily }"
        >
          <p class="text-base text-text mb-2">The quick brown fox jumps over the lazy dog.</p>
          <p class="text-base text-text">敏捷的棕色狐狸跳过懒狗。 1234567890</p>
        </div>
      </div>
    </div>

    <template #footer>
      <button
        @click="resetFont"
        class="btn btn--secondary mr-auto"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        {{ $t('fontConfig.reset') }}
      </button>

      <button
        @click="saveFont"
        class="btn btn--primary"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"/>
        </svg>
        {{ $t('fontConfig.save') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

const emit = defineEmits(['close'])
const { t } = useI18n()

const STORAGE_KEY = 'user-font-sans'
const DEFAULT_FALLBACK = 'system-ui, sans-serif'

const fontName = ref('')

// 预览字体
const previewFontFamily = computed(() => {
  return fontName.value ? `"${fontName.value}", ${DEFAULT_FALLBACK}` : DEFAULT_FALLBACK
})

// 应用字体到文档
const applyFont = (name) => {
  const fontValue = name ? `"${name}", ${DEFAULT_FALLBACK}` : DEFAULT_FALLBACK
  document.documentElement.style.setProperty('--font-sans', fontValue)
}

// 保存字体
const saveFont = () => {
  const name = fontName.value.trim()
  applyFont(name)
  if (name) {
    localStorage.setItem(STORAGE_KEY, name)
  } else {
    localStorage.removeItem(STORAGE_KEY)
  }
  window.$notify.success(t('fontConfig.saveSuccess'))
  emit('close')
}

// 重置字体
const resetFont = () => {
  fontName.value = ''
  applyFont('')
  localStorage.removeItem(STORAGE_KEY)
  window.$notify.success(t('fontConfig.resetSuccess'))
}

// 初始化
onMounted(() => {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved) {
    fontName.value = saved
  }
})
</script>

