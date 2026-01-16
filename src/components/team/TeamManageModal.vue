<template>
  <BaseModal
    :visible="true"
    :title="$t('team.title')"
    :close-on-overlay="true"
    :body-scroll="true"
    modal-class="!max-w-[720px]"
    @close="handleClose"
  >
    <template #header>
      <div class="flex items-center gap-3 flex-1">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" class="text-accent">
          <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
        </svg>
        <h3 class="modal-title">{{ $t('team.title') }}</h3>
      </div>
      <button @click="refresh" class="btn btn--ghost btn--icon" :disabled="loading">
        <svg v-if="!loading" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        <span v-else class="spinner-sm"></span>
      </button>
    </template>

    <!-- Loading State -->
    <div v-if="loading && !teamInfo" class="flex flex-col items-center justify-center py-16 gap-4">
      <span class="spinner"></span>
      <p class="text-text-muted">{{ $t('team.loading') }}</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="flex flex-col items-center justify-center py-16 gap-4">
      <p class="text-danger">{{ error }}</p>
      <button @click="refresh" class="btn btn--primary btn--md">{{ $t('common.refresh') }}</button>
    </div>

    <!-- Team Content -->
    <div v-else-if="teamInfo" class="flex flex-col gap-6">
      <!-- Members Section -->
      <div class="card p-5 hover:translate-y-0">
        <!-- Tabs -->
        <div class="flex gap-2 mb-5 border-b border-border">
          <button
            :class="['px-5 py-2.5 border-0 bg-transparent text-[14px] font-medium cursor-pointer border-b-2 transition-all', activeTab === 'members' ? 'text-accent border-accent' : 'text-text-muted border-transparent hover:text-text']"
            @click="activeTab = 'members'"
          >
            {{ $t('team.members') }} ({{ teamInfo.team?.users?.length || 0 }})
          </button>
          <button
            :class="['px-5 py-2.5 border-0 bg-transparent text-[14px] font-medium cursor-pointer border-b-2 transition-all', activeTab === 'invitations' ? 'text-accent border-accent' : 'text-text-muted border-transparent hover:text-text']"
            @click="activeTab = 'invitations'"
          >
            {{ $t('team.invitations') }} ({{ teamInfo.team?.invitations?.length || 0 }})
          </button>
        </div>

        <!-- Invite Form -->
        <div v-if="activeTab === 'members'" class="mb-5">
          <h4 class="label mb-3">{{ $t('team.inviteMembers') }}</h4>
          <textarea
            v-model="inviteEmails"
            class="input w-full resize-y mb-3"
            :placeholder="$t('team.emailPlaceholder')"
            rows="3"
          ></textarea>
          <button
            @click="handleInviteMembers"
            class="btn btn--primary btn--md"
            :disabled="inviting || !inviteEmails.trim()"
          >
            <span v-if="inviting" class="spinner-sm"></span>
            <span v-else>{{ $t('team.invite') }}</span>
          </button>
        </div>

        <!-- Members List -->
        <div v-if="activeTab === 'members'" class="flex flex-col gap-2.5">
          <div v-if="!teamInfo.team?.users || teamInfo.team.users.length === 0" class="py-10 text-center text-text-muted text-[14px]">
            {{ $t('team.noMembers') }}
          </div>
          <div v-else v-for="member in teamInfo.team.users" :key="member.id" class="card flex items-center justify-between p-3">
            <div class="flex flex-col gap-1 flex-1">
              <span class="text-[14px] font-medium text-text-strong">{{ member.email }}</span>
              <span class="text-[12px] text-text-muted capitalize">{{ member.role }}</span>
            </div>
            <button
              @click="confirmDeleteMember(member)"
              class="btn btn--ghost btn--icon-sm text-text-muted hover:text-danger hover:bg-danger-hover"
              :title="$t('team.deleteMember')"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Invitations List -->
        <div v-if="activeTab === 'invitations'" class="flex flex-col gap-2.5">
          <div v-if="!teamInfo.team?.invitations || teamInfo.team.invitations.length === 0" class="py-10 text-center text-text-muted text-[14px]">
            {{ $t('team.noInvitations') }}
          </div>
          <div v-else v-for="invitation in teamInfo.team.invitations" :key="invitation.id" class="card flex items-center justify-between p-3">
            <div class="flex flex-col gap-1 flex-1">
              <span class="text-[14px] font-medium text-text-strong">{{ invitation.email }}</span>
              <span class="text-[12px] text-text-muted capitalize">{{ invitation.status || 'Pending' }}</span>
            </div>
            <button
              @click="confirmDeleteInvitation(invitation)"
              class="btn btn--ghost btn--icon-sm text-text-muted hover:text-danger hover:bg-danger-hover"
              :title="$t('team.deleteInvitation')"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

const { t } = useI18n()

const props = defineProps({
  authSession: {
    type: String,
    required: true
  }
})

const emit = defineEmits(['close'])

// 响应式状态
const loading = ref(false)
const error = ref(null)
const teamInfo = ref(null)
const activeTab = ref('members')
const inviteEmails = ref('')
const inviting = ref(false)
const deleteTarget = ref(null)
const deleteType = ref('member') // 'member' or 'invitation'
const deleting = ref(false)

// 获取团队信息
const fetchTeamInfo = async () => {
  loading.value = true
  error.value = null

  try {
    const result = await invoke('fetch_team_info', {
      authSession: props.authSession
    })

    teamInfo.value = result
  } catch (e) {
    error.value = e.toString()
    console.error('Failed to fetch team info:', e)
  } finally {
    loading.value = false
  }
}

// 刷新
const refresh = () => {
  if (!loading.value) {
    fetchTeamInfo()
  }
}

// 邀请成员
const handleInviteMembers = async () => {
  const emailsText = inviteEmails.value.trim()
  if (!emailsText) return

  // 解析邮箱（支持逗号和换行分隔）
  const emails = emailsText
    .split(/[,\n]/)
    .map(e => e.trim())
    .filter(e => e.length > 0)

  if (emails.length === 0) return

  inviting.value = true
  try {
    await invoke('invite_team_members', {
      authSession: props.authSession,
      emails
    })

    window.$notify?.success(t('team.invitationsSent') || 'Invitations sent successfully')
    inviteEmails.value = ''
    // 在后台刷新，不阻塞 UI
    fetchTeamInfo()
  } catch (e) {
    window.$notify?.error(e.toString())
    console.error('Failed to invite members:', e)
  } finally {
    inviting.value = false
  }
}

// 确认删除成员
const confirmDeleteMember = async (member) => {
  const confirmed = await window.$confirm({
    title: t('team.confirmDelete'),
    message: t('team.confirmDeleteMember', { email: member.email }),
    confirmText: t('common.confirm'),
    cancelText: t('common.cancel'),
    variant: 'danger'
  })

  if (!confirmed) return

  deleteTarget.value = member
  deleteType.value = 'member'
  await handleDelete()
}

// 确认删除邀请
const confirmDeleteInvitation = async (invitation) => {
  const confirmed = await window.$confirm({
    title: t('team.confirmDelete'),
    message: t('team.confirmDeleteInvitation', { email: invitation.email }),
    confirmText: t('common.confirm'),
    cancelText: t('common.cancel'),
    variant: 'danger'
  })

  if (!confirmed) return

  deleteTarget.value = invitation
  deleteType.value = 'invitation'
  await handleDelete()
}

// 执行删除
const handleDelete = async () => {
  if (!deleteTarget.value) return

  deleting.value = true
  try {
    if (deleteType.value === 'member') {
      await invoke('delete_team_member', {
        authSession: props.authSession,
        userId: deleteTarget.value.id
      })
      window.$notify?.success(t('team.memberDeleted') || 'Member deleted successfully')
    } else {
      await invoke('delete_team_invitation', {
        authSession: props.authSession,
        invitationId: deleteTarget.value.id
      })
      window.$notify?.success(t('team.invitationDeleted') || 'Invitation deleted successfully')
    }

    deleteTarget.value = null
    // 在后台刷新，不阻塞 UI
    fetchTeamInfo()
  } catch (e) {
    window.$notify?.error(e.toString())
    console.error('Failed to delete:', e)
  } finally {
    deleting.value = false
  }
}

// 关闭模态框
const handleClose = () => {
  emit('close')
}

// 初始化
onMounted(() => {
  fetchTeamInfo()
})
</script>
