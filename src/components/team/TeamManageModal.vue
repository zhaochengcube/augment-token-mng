<template>
  <div class="modal-overlay team-manage-overlay" @click="handleClose">
    <div class="modal-content team-manage-modal" @click.stop>
      <div class="modal-header">
        <div class="header-title">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>
          </svg>
          <h2>{{ $t('team.title') }}</h2>
        </div>
        <div class="header-actions">
          <button @click="refresh" class="refresh-btn" :disabled="loading">
            <svg v-if="!loading" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            <div v-else class="mini-spinner"></div>
          </button>
          <button class="modal-close" @click="handleClose">×</button>
        </div>
      </div>

      <div class="modal-body">
        <div v-if="loading && !teamInfo" class="loading-container">
          <div class="spinner"></div>
          <p>{{ $t('team.loading') }}</p>
        </div>

        <div v-else-if="error" class="error-container">
          <p>{{ error }}</p>
          <button @click="refresh" class="btn primary">{{ $t('common.refresh') }}</button>
        </div>

        <div v-else-if="teamInfo" class="team-content">
          <!-- 成员管理区域 -->
          <div class="members-section">
            <div class="tabs">
              <button
                :class="['tab', { active: activeTab === 'members' }]"
                @click="activeTab = 'members'"
              >
                {{ $t('team.members') }} ({{ teamInfo.team?.users?.length || 0 }})
              </button>
              <button
                :class="['tab', { active: activeTab === 'invitations' }]"
                @click="activeTab = 'invitations'"
              >
                {{ $t('team.invitations') }} ({{ teamInfo.team?.invitations?.length || 0 }})
              </button>
            </div>

            <!-- 邀请新成员表单 -->
            <div v-if="activeTab === 'members'" class="invite-form">
              <h4>{{ $t('team.inviteMembers') }}</h4>
              <textarea 
                v-model="inviteEmails" 
                class="invite-textarea"
                :placeholder="$t('team.emailPlaceholder')"
                rows="3"
              ></textarea>
              <button
                @click="handleInviteMembers"
                class="btn primary"
                :disabled="inviting || !inviteEmails.trim()"
              >
                <div v-if="inviting" class="mini-spinner"></div>
                <span v-else>{{ $t('team.invite') }}</span>
              </button>
            </div>

            <!-- 成员列表 -->
            <div v-if="activeTab === 'members'" class="members-list">
              <div v-if="!teamInfo.team?.users || teamInfo.team.users.length === 0" class="empty-state">
                {{ $t('team.noMembers') }}
              </div>
              <div v-else class="member-item" v-for="member in teamInfo.team.users" :key="member.id">
                <div class="member-info">
                  <span class="member-email">{{ member.email }}</span>
                  <span class="member-role">{{ member.role }}</span>
                </div>
                <button
                  @click="confirmDeleteMember(member)"
                  class="btn-delete"
                  :title="$t('team.deleteMember')"
                >
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                  </svg>
                </button>
              </div>
            </div>

            <!-- 邀请列表 -->
            <div v-if="activeTab === 'invitations'" class="invitations-list">
              <div v-if="!teamInfo.team?.invitations || teamInfo.team.invitations.length === 0" class="empty-state">
                {{ $t('team.noInvitations') }}
              </div>
              <div v-else class="invitation-item" v-for="invitation in teamInfo.team.invitations" :key="invitation.id">
                <div class="invitation-info">
                  <span class="invitation-email">{{ invitation.email }}</span>
                  <span class="invitation-status">{{ invitation.status || 'Pending' }}</span>
                </div>
                <button
                  @click="confirmDeleteInvitation(invitation)"
                  class="btn-delete"
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
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <Teleport to="body">
      <Transition name="modal" appear>
        <div v-if="showDeleteConfirm" class="confirm-overlay" @click="showDeleteConfirm = false">
          <div class="confirm-dialog" @click.stop>
            <h3>{{ $t('team.confirmDelete') }}</h3>
            <p v-if="deleteTarget?.email">
              {{ deleteType === 'member'
                ? $t('team.confirmDeleteMember', { email: deleteTarget.email })
                : $t('team.confirmDeleteInvitation', { email: deleteTarget.email })
              }}
            </p>
            <div class="confirm-actions">
              <button @click="showDeleteConfirm = false" class="btn secondary">
                {{ $t('common.cancel') }}
              </button>
              <button @click="handleDelete" class="btn danger" :disabled="deleting">
                <div v-if="deleting" class="mini-spinner"></div>
                <span v-else>{{ $t('common.confirm') }}</span>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

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
const showDeleteConfirm = ref(false)
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
const confirmDeleteMember = (member) => {
  deleteTarget.value = member
  deleteType.value = 'member'
  showDeleteConfirm.value = true
}

// 确认删除邀请
const confirmDeleteInvitation = (invitation) => {
  deleteTarget.value = invitation
  deleteType.value = 'invitation'
  showDeleteConfirm.value = true
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

    showDeleteConfirm.value = false
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

<style scoped>
/* ============================================
   TeamManageModal - Modern Tech Style
   ============================================ */

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  padding: 20px;
  backdrop-filter: blur(4px);
}

/* 模态框布局 */
.team-manage-modal {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 18px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35), var(--tech-border-glow);
  max-width: 720px;
  width: 100%;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideUp {
  from {
    transform: translateY(20px) scale(0.95);
    opacity: 0;
  }
  to {
    transform: translateY(0) scale(1);
    opacity: 1;
  }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 1px solid var(--tech-glass-border);
  flex-shrink: 0;
  background: color-mix(in srgb, var(--bg-muted) 30%, transparent);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-strong);
}

.header-title svg {
  color: var(--accent);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  border: 1px solid var(--tech-glass-border);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.refresh-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.modal-body {
  padding: 24px 28px;
  overflow-y: auto;
  flex: 1;
}

/* Loading & Error */
.loading-container,
.error-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  gap: 16px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--tech-glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.mini-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

/* Team Content */
.team-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* Members Section */
.members-section {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
  padding: 20px;
}

/* Tabs */
.tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
  border-bottom: 1px solid var(--tech-glass-border);
}

.tab {
  padding: 10px 20px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s ease;
}

.tab:hover {
  color: var(--text);
}

.tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

/* Invite Form */
.invite-form {
  margin-bottom: 20px;
}

.invite-form h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-strong);
}

.invite-textarea {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--tech-glass-border);
  border-radius: 8px;
  background: var(--bg);
  color: var(--text);
  font-size: 14px;
  font-family: inherit;
  resize: vertical;
  margin-bottom: 12px;
  transition: all 0.2s ease;
}

.invite-textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 10%, transparent);
}

/* Members & Invitations List */
.members-list,
.invitations-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.empty-state {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 14px;
}

.member-item,
.invitation-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg);
  border: 1px solid var(--tech-glass-border);
  border-radius: 8px;
  transition: all 0.2s ease;
}

.member-item:hover,
.invitation-item:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  background: color-mix(in srgb, var(--accent) 5%, transparent);
}

.member-info,
.invitation-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.member-email,
.invitation-email {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-strong);
}

.member-role,
.invitation-status {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: capitalize;
}

.btn-delete {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px solid var(--tech-glass-border);
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-delete:hover {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  border-color: #ef4444;
  color: #ef4444;
}

/* Confirm Dialog */
.confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 4000;
  backdrop-filter: blur(4px);
}

.confirm-dialog {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35);
  max-width: 420px;
  width: 90%;
  padding: 24px;
}

.confirm-dialog h3 {
  margin: 0 0 12px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
}

.confirm-dialog p {
  margin: 0 0 24px 0;
  font-size: 14px;
  color: var(--text);
  line-height: 1.5;
}

.confirm-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

/* Modal Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.25s ease;
}

.modal-enter-active .team-manage-modal,
.modal-leave-active .team-manage-modal,
.modal-enter-active .confirm-dialog,
.modal-leave-active .confirm-dialog {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .team-manage-modal,
.modal-leave-to .team-manage-modal,
.modal-enter-from .confirm-dialog,
.modal-leave-to .confirm-dialog {
  transform: scale(0.92) translateY(10px);
  opacity: 0;
}
</style>

