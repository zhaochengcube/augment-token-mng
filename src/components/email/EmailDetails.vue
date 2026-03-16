<template>
  <BaseModal
    :visible="true"
    title="邮件详情"
    modal-class="max-w-[920px]"
    @close="$emit('close')"
  >
        <div v-if="isLoading" class="text-center py-16 text-text-muted">
          <span class="btn-spinner btn-spinner--lg inline-block mb-3" aria-hidden="true"></span>
          <p>加载邮件详情中...</p>
        </div>

        <div v-else-if="error" class="text-center py-16 text-text-muted">
          <p>{{ error }}</p>
          <button @click="loadEmailDetails" class="btn btn--primary">重新加载</button>
        </div>

        <div v-else-if="emailDetails">
          <!-- 邮件头部信息 -->
          <div class="mb-6 pb-5 border-b border-border">
            <h2 class="m-0 mb-4 text-2xl font-bold text-text leading-snug">{{ emailDetails.subject }}</h2>
            
            <div class="flex flex-col gap-2.5">
              <div class="flex items-start gap-3.5">
                <span class="font-semibold text-text-muted min-w-[65px] shrink-0 text-[13px] uppercase tracking-wide">发件人:</span>
                <span class="text-text break-all">{{ emailDetails.from_email }}</span>
              </div>
              <div class="flex items-start gap-3.5">
                <span class="font-semibold text-text-muted min-w-[65px] shrink-0 text-[13px] uppercase tracking-wide">收件人:</span>
                <span class="text-text break-all">{{ emailDetails.to_email }}</span>
              </div>
              <div v-if="emailDetails.cc_email" class="flex items-start gap-3.5">
                <span class="font-semibold text-text-muted min-w-[65px] shrink-0 text-[13px] uppercase tracking-wide">抄送:</span>
                <span class="text-text break-all">{{ emailDetails.cc_email }}</span>
              </div>
              <div class="flex items-start gap-3.5">
                <span class="font-semibold text-text-muted min-w-[65px] shrink-0 text-[13px] uppercase tracking-wide">日期:</span>
                <span class="text-text">{{ formatDate(emailDetails.date) }}</span>
              </div>
            </div>
          </div>

          <!-- 邮件正文 -->
          <div class="leading-relaxed">
            <div v-if="emailDetails.body_html" class="mb-6">
              <h4 class="m-0 mb-3.5 text-base font-semibold text-text">HTML 内容:</h4>
              <div class="bg-muted/50 border border-border rounded-xl p-4 max-h-[400px] overflow-y-auto break-words" v-html="emailDetails.body_html"></div>
            </div>
            
            <div v-if="emailDetails.body_plain" class="mb-6">
              <h4 class="m-0 mb-3.5 text-base font-semibold text-text">纯文本内容:</h4>
              <pre class="bg-muted/50 border border-border rounded-xl p-4 text-sm whitespace-pre-wrap break-words max-h-[400px] overflow-y-auto m-0">{{ emailDetails.body_plain }}</pre>
            </div>

            <div v-if="!emailDetails.body_html && !emailDetails.body_plain" class="text-center py-11 text-text-muted italic">
              <p>此邮件没有可显示的内容</p>
              <p class="text-xs mt-2.5 opacity-70">可能是特殊格式或编码问题</p>
            </div>
          </div>
        </div>
  </BaseModal>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import BaseModal from '@/components/common/BaseModal.vue'

const props = defineProps({
  email: {
    type: String,
    required: true
  },
  messageId: {
    type: String,
    required: true
  },
  method: {
    type: String,
    default: 'graph'
  }
})

const emit = defineEmits(['close'])

// 响应式数据
const emailDetails = ref(null)
const isLoading = ref(false)
const error = ref('')

// 方法
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const loadEmailDetails = async () => {
  isLoading.value = true
  error.value = ''
  
  try {
    emailDetails.value = await invoke('outlook_get_email_details', {
      email: props.email,
      messageId: props.messageId,
      method: props.method
    })
  } catch (err) {
    error.value = `加载邮件详情失败: ${err}`
    showStatus(error.value, 'error')
  } finally {
    isLoading.value = false
  }
}

const formatDate = (dateString) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  } catch {
    return dateString
  }
}

// 生命周期
onMounted(() => {
  loadEmailDetails()
})
</script>
