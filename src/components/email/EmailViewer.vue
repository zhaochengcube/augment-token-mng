<template>
  <BaseModal
    :visible="true"
    :title="$t('emailViewer.title') + ' - ' + email"
    modal-class="max-w-[1020px]"
    @close="$emit('close')"
  >
        <!-- 文件夹选择和控制 -->
        <div class="flex justify-between items-center mb-5 p-4 bg-muted/30 border border-border rounded-xl flex-wrap gap-3.5">
          <div class="flex items-center gap-2.5">
            <label class="font-semibold text-text whitespace-nowrap">{{ $t('emailViewer.folder') }}:</label>
            <select v-model="selectedFolder" @change="loadEmails" class="input py-1.5 px-3 min-w-[100px]">
              <option value="inbox">{{ $t('emailViewer.inbox') }}</option>
              <option value="junk">{{ $t('emailViewer.junk') }}</option>
            </select>
          </div>

          <div class="flex items-center gap-3.5">
            <button
              @click="previousPage"
              :disabled="currentPage <= 1 || isLoading"
              class="btn btn--secondary btn--sm disabled:opacity-40"
            >
              ← {{ $t('emailViewer.previousPage') }}
            </button>
            <span class="text-sm text-text-secondary whitespace-nowrap">
              {{ $t('emailViewer.pageInfo', { current: currentPage, total: totalPages }) }}
            </span>
            <button
              @click="nextPage"
              :disabled="currentPage >= totalPages || isLoading"
              class="btn btn--secondary btn--sm disabled:opacity-40"
            >
              {{ $t('emailViewer.nextPage') }} →
            </button>
          </div>

          <div class="flex items-center gap-2.5">
            <button
              v-if="selectedIds.length > 0"
              @click="deleteSelectedEmails"
              :disabled="isDeleting"
              class="btn btn-tech-danger btn--sm"
            >
              <span v-if="isDeleting" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
              {{ isDeleting ? '删除中...' : `删除 (${selectedIds.length})` }}
            </button>
            <button
              @click="refreshEmails"
              :disabled="isLoading"
              class="btn btn--primary btn--sm"
            >
              <span v-if="isLoading" class="btn-spinner btn-spinner--xs" aria-hidden="true"></span>
              {{ isLoading ? $t('emailViewer.loading') : $t('emailViewer.reload') }}
            </button>
          </div>

        </div>

        <!-- 邮件列表 -->
        <div class="min-h-[400px]">
          <div v-if="isLoading" class="text-center py-16 text-text-muted">
            <span class="btn-spinner btn-spinner--lg inline-block mb-3" aria-hidden="true"></span>
            <p>{{ $t('emailViewer.loading') }}</p>
          </div>

          <div v-else-if="emails.length === 0" class="text-center py-16 text-text-muted">
            <p>{{ $t('emailViewer.noEmails') }}</p>
          </div>

          <div v-else class="flex flex-col gap-2.5">
            <!-- 全选 -->
            <div class="flex items-center gap-2 px-1 text-xs text-text-muted">
              <input type="checkbox" :checked="isAllSelected" @change="toggleSelectAll" class="accent-accent" />
              <span>{{ isAllSelected ? '取消全选' : '全选当页' }}</span>
            </div>
            <div
              v-for="emailItem in emails"
              :key="emailItem.message_id"
              class="flex items-center p-4 bg-muted/50 border border-border rounded-xl cursor-pointer select-none transition-all hover:border-accent/50 hover:translate-x-1"
              :class="selectedIds.includes(emailItem.message_id) ? 'border-accent bg-accent/15 shadow-accent' : ''"
              @click="viewEmailDetails(emailItem)"
            >
              <input
                type="checkbox"
                :checked="selectedIds.includes(emailItem.message_id)"
                @click.stop="toggleSelect(emailItem.message_id)"
                class="mr-3 accent-accent shrink-0"
              />
              <div class="flex items-center gap-3.5 min-w-[200px]">
                <div class="w-10 h-10 rounded-full bg-accent text-white flex items-center justify-center font-bold text-base shadow-accent">
                  {{ emailItem.sender_initial }}
                </div>
                <div class="flex-1 min-w-0">
                  <div class="font-semibold text-text text-sm truncate">{{ emailItem.from_email }}</div>
                  <div class="text-xs text-text-muted">{{ formatDate(emailItem.date) }}</div>
                </div>
              </div>
              <div class="flex-1 mx-4 min-w-0">
                <div class="font-semibold text-text mb-1.5 truncate">{{ emailItem.subject }}</div>
                <span class="text-[11px] text-accent bg-accent/15 border border-accent/30 px-2.5 py-0.5 rounded-full font-semibold inline-block">
                  {{ emailItem.folder }}
                </span>
              </div>
              <div class="flex items-center gap-2.5">
                <span v-if="emailItem.has_attachments" class="text-sm text-text-muted">📎</span>
              </div>
            </div>
          </div>

          <!-- 分页信息 -->
          <div v-if="emails.length > 0" class="text-center mt-5 p-4 bg-muted/50 border border-border rounded-xl">
            <p class="m-0 text-sm text-text-muted">
              {{ $t('emailViewer.totalEmails', { count: totalEmails }) }}
            </p>
          </div>
        </div>
  </BaseModal>

  <!-- 邮件详情查看器 -->
  <EmailDetails
    v-if="showEmailDetails"
    :email="email"
    :message-id="selectedEmailForDetails"
    :method="fetchMethod"
    @close="showEmailDetails = false"
  />
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import EmailDetails from './EmailDetails.vue'

const props = defineProps({
  email: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

// i18n
const { t } = useI18n()

// 响应式数据
const emails = ref([])
const selectedFolder = ref('inbox')
const currentPage = ref(1)
const pageSize = ref(20)
const totalEmails = ref(0)
const isLoading = ref(false)
const selectedEmailId = ref('')
const selectedIds = ref([])
const isDeleting = ref(false)
const showEmailDetails = ref(false)
const selectedEmailForDetails = ref('')
const fetchMethod = ref('graph')

// 计算属性
const totalPages = computed(() => {
  return Math.ceil(totalEmails.value / pageSize.value)
})

// 方法
const showStatus = (message, type = 'info') => {
  window.$notify[type](message)
}

const loadEmails = async () => {
  isLoading.value = true
  try {
    const response = await invoke('outlook_get_emails', {
      email: props.email,
      folder: selectedFolder.value,
      page: currentPage.value,
      pageSize: pageSize.value
    })
    
    emails.value = response.emails
    totalEmails.value = response.total_emails
    fetchMethod.value = response.method || 'graph'
    
    if (emails.value.length === 0 && currentPage.value > 1) {
      // 如果当前页没有邮件且不是第一页，回到第一页
      currentPage.value = 1
      await loadEmails()
    }
  } catch (error) {
    showStatus(`加载邮件失败: ${error}`, 'error')
    emails.value = []
    totalEmails.value = 0
  } finally {
    isLoading.value = false
  }
}

const refreshEmails = async () => {
  currentPage.value = 1
  await loadEmails()
  showStatus('邮件列表已重新加载', 'success')
}



const previousPage = async () => {
  if (currentPage.value > 1) {
    currentPage.value--
    await loadEmails()
  }
}

const nextPage = async () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
    await loadEmails()
  }
}

const viewEmailDetails = (emailItem) => {
  selectedEmailId.value = emailItem.message_id
  selectedEmailForDetails.value = emailItem.message_id
  showEmailDetails.value = true
}

const isAllSelected = computed(() => {
  return emails.value.length > 0 && emails.value.every(e => selectedIds.value.includes(e.message_id))
})

const toggleSelect = (id) => {
  const idx = selectedIds.value.indexOf(id)
  if (idx === -1) {
    selectedIds.value.push(id)
  } else {
    selectedIds.value.splice(idx, 1)
  }
}

const toggleSelectAll = () => {
  if (isAllSelected.value) {
    selectedIds.value = []
  } else {
    selectedIds.value = emails.value.map(e => e.message_id)
  }
}

const deleteSelectedEmails = async () => {
  if (selectedIds.value.length === 0) return
  isDeleting.value = true
  try {
    const result = await invoke('outlook_delete_emails', {
      email: props.email,
      messageIds: selectedIds.value
    })
    if (result.failed_count === 0) {
      showStatus(`成功删除 ${result.success_count} 封邮件`, 'success')
    } else {
      showStatus(`删除 ${result.success_count} 成功，${result.failed_count} 失败`, 'warning')
    }
    selectedIds.value = []
    await loadEmails()
  } catch (error) {
    showStatus(`删除失败: ${error}`, 'error')
  } finally {
    isDeleting.value = false
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
      minute: '2-digit'
    })
  } catch {
    return dateString
  }
}

// 生命周期
onMounted(() => {
  loadEmails()
})
</script>
