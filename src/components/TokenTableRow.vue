<template>
  <tr 
    :class="['token-table-row', { 'selected': isSelected, 'highlighted': isHighlighted }]"
    @click="handleRowClick"
  >
    <!-- 多选框 -->
    <td class="cell-checkbox">
      <div 
        class="checkbox-wrapper"
        @click.stop="toggleSelection"
      >
        <div class="checkbox-inner" :class="{ 'checked': isSelected }">
          <svg v-if="isSelected" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
          </svg>
        </div>
      </div>
    </td>

    <!-- Tag -->
    <td class="cell-tag">
      <div class="tag-cell-wrapper" @click.stop="openTagEditor">
        <span
          v-if="hasTag"
          class="tag-badge editable"
          :style="tagBadgeStyle"
          :title="$t('tokenList.clickToEditTag')"
        >
          {{ tagDisplayName }}
        </span>
        <span v-else class="no-tag add-tag" :title="$t('tokenList.clickToAddTag')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
        </span>
      </div>
    </td>

    <!-- 状态 -->
    <td class="cell-status">
      <span 
        :class="['status-badge', getStatusClass(token.ban_status)]"
        :title="token.ban_status === 'SUSPENDED' ? $t('tokenCard.clickToViewDetails') : ''"
      >
        {{ getStatusText(token.ban_status) }}
      </span>
    </td>

    <!-- 邮箱 -->
    <td class="cell-email">
      <div 
        v-if="token.email_note" 
        class="email-wrapper"
        @click.stop="copyEmailNote"
        :title="token.email_note"
      >
        <span class="email-text email-masked">{{ maskedEmail }}</span>
        <span class="email-text email-full">{{ token.email_note }}</span>
        <span class="copy-hint">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
          </svg>
        </span>
      </div>
      <span v-else class="no-email">-</span>
    </td>

    <!-- 剩余次数 -->
    <td class="cell-balance">
      <span :class="balanceClasses">
        {{ balanceDisplay }}
      </span>
    </td>

    <!-- 过期时间 -->
    <td class="cell-expiry">
      <span v-if="expiryDate" class="expiry-text">
        {{ formatExpiryDate(expiryDate) }}
      </span>
      <span v-else class="no-expiry">-</span>
    </td>

    <!-- 操作 -->
    <td class="cell-actions">
      <div class="actions-wrapper">
        <!-- 编辑器选择 -->
        <button @click.stop="showEditorModal = true" class="action-btn editor" :title="$t('tokenCard.selectEditor')">
          <img src="/icons/vscode.svg" alt="Editor" width="16" height="16" />
        </button>
        
        <!-- 导出JSON -->
        <button @click.stop="exportTokenAsJson" class="action-btn export" :title="$t('tokenCard.exportJson')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92 1.61 0 2.92-1.31 2.92-2.92s-1.31-2.92-2.92-2.92z"/>
          </svg>
        </button>

        <!-- 复制菜单 -->
        <div class="copy-menu-wrapper" @click.stop>
          <button @click.stop="toggleCopyMenu" class="action-btn copy" :title="$t('tokenCard.copyMenu')">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
            </svg>
          </button>
          <Transition name="dropdown">
            <div v-if="showCopyMenu" class="copy-dropdown" @click.stop>
              <button @click="handleCopyMenuClick('token')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>
                </svg>
                <span>{{ $t('tokenCard.copyToken') }}</span>
              </button>
              <button @click="handleCopyMenuClick('url')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"/>
                </svg>
                <span>{{ $t('tokenCard.copyTenantUrl') }}</span>
              </button>
              <button v-if="token.portal_url" @click="handleCopyMenuClick('portal')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
                </svg>
                <span>{{ $t('tokenCard.copyPortalUrl') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('session')" class="copy-menu-item">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                </svg>
                <span>{{ $t('tokenCard.copyAuthSession') }}</span>
              </button>
              <button v-if="token.auth_session" @click="handleCopyMenuClick('payment')" class="copy-menu-item" :disabled="isFetchingPaymentLink">
                <div v-if="isFetchingPaymentLink" class="loading-spinner-small"></div>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z"/>
                </svg>
                <span>{{ isFetchingPaymentLink ? $t('tokenCard.fetchingPaymentLink') : $t('tokenCard.copyPaymentLink') }}</span>
              </button>
            </div>
          </Transition>
        </div>

        <!-- 刷新状态 -->
        <button 
          @click.stop="checkAccountStatus"
          :class="['action-btn', 'refresh', { 'loading': isCheckingStatus || (isBatchChecking && !token.skip_check) }]"
          :disabled="isCheckingStatus || (isBatchChecking && !token.skip_check) || token.skip_check"
          :title="token.skip_check ? $t('tokenCard.checkDisabled') : $t('tokenCard.checkAccountStatus')"
        >
          <svg v-if="!isCheckingStatus && !(isBatchChecking && !token.skip_check)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
          <div v-else class="loading-spinner"></div>
        </button>

        <!-- Portal -->
        <button 
          v-if="token.portal_url" 
          @click.stop="showPortalDialog = true" 
          class="action-btn portal" 
          :title="$t('tokenCard.openPortal')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.11 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
          </svg>
        </button>

        <!-- 编辑 -->
        <button @click.stop="editToken" class="action-btn edit" :title="$t('tokenCard.editToken')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/>
          </svg>
        </button>

        <!-- 删除 -->
        <button @click.stop="deleteToken" class="action-btn delete" :title="$t('tokenCard.deleteToken')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
        </button>
      </div>
    </td>
  </tr>

  <!-- 编辑器选择模态框 -->
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="showEditorModal" class="editor-modal-overlay">
        <div class="editor-modal" @click.stop>
          <div class="modal-header">
            <h3>{{ $t('tokenCard.selectEditor') }}</h3>
            <button @click.stop="showEditorModal = false" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="modal-content">
            <!-- VSCode 系编辑器区域 -->
            <div class="editor-section">
              <div class="editor-options jetbrains-grid">
                <button @click="handleEditorClick('vscode')" class="editor-option vscode-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.vscode" alt="VS Code" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">VS Code</span>
                  </div>
                </button>
                <button @click="handleEditorClick('cursor')" class="editor-option cursor-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.cursor" alt="Cursor" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Cursor</span>
                  </div>
                </button>
                <button @click="handleEditorClick('kiro')" class="editor-option kiro-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.kiro" alt="Kiro" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Kiro</span>
                  </div>
                </button>
                <button @click="handleEditorClick('trae')" class="editor-option trae-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.trae" alt="Trae" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Trae</span>
                  </div>
                </button>
                <button @click="handleEditorClick('windsurf')" class="editor-option windsurf-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.windsurf" alt="Windsurf" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Windsurf</span>
                  </div>
                </button>
                <button @click="handleEditorClick('qoder')" class="editor-option qoder-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.qoder" alt="Qoder" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Qoder</span>
                  </div>
                </button>
                <button @click="handleEditorClick('vscodium')" class="editor-option vscodium-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.vscodium" alt="VSCodium" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">VSCodium</span>
                  </div>
                </button>
                <button @click="handleEditorClick('codebuddy')" class="editor-option codebuddy-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.codebuddy" alt="CodeBuddy" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">CodeBuddy</span>
                  </div>
                </button>
                <button @click="handleEditorClick('vim')" class="editor-option vim-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.vim" alt="Vim" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Vim</span>
                  </div>
                </button>
                <button @click="handleEditorClick('auggie')" class="editor-option auggie-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.auggie" alt="Auggie" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Auggie</span>
                  </div>
                </button>
                <button @click="handleEditorClick('antigravity')" class="editor-option antigravity-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.antigravity" alt="Antigravity" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Antigravity</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- JetBrains 系编辑器区域 -->
            <div class="editor-section">
              <div class="editor-options jetbrains-grid">
                <button @click="handleEditorClick('idea')" class="editor-option idea-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.idea" alt="IntelliJ IDEA" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">IntelliJ IDEA</span>
                  </div>
                </button>
                <button @click="handleEditorClick('pycharm')" class="editor-option pycharm-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.pycharm" alt="PyCharm" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">PyCharm</span>
                  </div>
                </button>
                <button @click="handleEditorClick('goland')" class="editor-option goland-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.goland" alt="GoLand" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">GoLand</span>
                  </div>
                </button>
                <button @click="handleEditorClick('rustrover')" class="editor-option rustrover-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.rustrover" alt="RustRover" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">RustRover</span>
                  </div>
                </button>
                <button @click="handleEditorClick('webstorm')" class="editor-option webstorm-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.webstorm" alt="WebStorm" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">WebStorm</span>
                  </div>
                </button>
                <button @click="handleEditorClick('clion')" class="editor-option clion-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.clion" alt="CLion" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">CLion</span>
                  </div>
                </button>
                <button @click="handleEditorClick('rider')" class="editor-option rider-option">
                  <div class="editor-icon">
                    <img :src="editorIcons.rider" alt="Rider" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">Rider</span>
                  </div>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- 标签编辑模态框 -->
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="showTagEditor" class="tag-editor-overlay" @click="showTagEditor = false">
        <div class="tag-editor-modal" @click.stop>
          <div class="modal-header">
            <h3>{{ $t('tokenList.editTag') }}</h3>
            <button @click="showTagEditor = false" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
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
              <div v-if="editingTagName" class="tag-preview-row">
                <span class="tag-badge-preview" :style="{ backgroundColor: editingTagColor, color: getContrastColor(editingTagColor) }">
                  {{ editingTagName }}
                </span>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <button @click="clearTag" class="btn-secondary" v-if="hasTag">
              {{ $t('tokenForm.clearTag') }}
            </button>
            <button @click="showTagEditor = false" class="btn-secondary">
              {{ $t('common.cancel') }}
            </button>
            <button @click="saveTag" class="btn-primary">
              {{ $t('common.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- Portal 对话框 -->
  <ExternalLinkDialog
    :show="showPortalDialog"
    :title="$t('dialogs.selectOpenMethod')"
    :url="token.portal_url || ''"
    :browser-title="getPortalBrowserTitle()"
    @close="showPortalDialog = false"
  />
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useTokenActions } from '@/composables/useTokenActions'
import ExternalLinkDialog from './ExternalLinkDialog.vue'

const { t } = useI18n()

// 获取当前主题
const currentTheme = computed(() => {
  return document.documentElement.getAttribute('data-theme') || 'light'
})

// 编辑器图标
const editorIcons = computed(() => ({
  vscode: '/icons/vscode.svg',
  cursor: '/icons/cursor.svg',
  kiro: '/icons/kiro.svg',
  trae: '/icons/trae.svg',
  windsurf: '/icons/windsurf.svg',
  qoder: '/icons/qoder.svg',
  vscodium: '/icons/vscodium.svg',
  codebuddy: '/icons/codebuddy.svg',
  vim: '/icons/vim.svg',
  auggie: currentTheme.value === 'dark' ? '/icons/auggie_dark.svg' : '/icons/auggie.svg',
  antigravity: '/icons/antigravity.png',
  idea: '/icons/idea.svg',
  pycharm: '/icons/pycharm.svg',
  goland: '/icons/goland.svg',
  rustrover: '/icons/rustrover.svg',
  webstorm: '/icons/webstorm.svg',
  clion: '/icons/clion.svg',
  rider: '/icons/rider.svg'
}))

// Props
const props = defineProps({
  token: {
    type: Object,
    required: true
  },
  isBatchChecking: {
    type: Boolean,
    default: false
  },
  isHighlighted: {
    type: Boolean,
    default: false
  },
  isSelected: {
    type: Boolean,
    default: false
  },
  selectionMode: {
    type: Boolean,
    default: false
  },
  isSelectedRefreshing: {
    type: Boolean,
    default: false
  },
  cachedPaymentLink: {
    type: String,
    default: null
  },
  allTokens: {
    type: Array,
    default: () => []
  }
})

// Emits
const emit = defineEmits([
  'delete', 
  'edit', 
  'token-updated', 
  'select',
  'open-editor',
  'open-portal',
  'payment-link-fetched',
  'edit-tag'
])

// 使用共享的 token actions
const {
  tagDisplayName,
  hasTag,
  tagBadgeStyle,
  displayUrl,
  maskedEmail,
  formatDate,
  formatExpiryDate,
  getStatusClass,
  getStatusText,
  copyToken,
  copyTenantUrl,
  copyEmailNote,
  copyPortalUrl,
  copyAuthSession,
  exportTokenAsJson,
  deleteToken,
  editToken,
  toggleSelection
} = useTokenActions(props, emit)

// 本地状态
const showCopyMenu = ref(false)
const isCheckingStatus = ref(false)
const isFetchingPaymentLink = ref(false)
const showEditorModal = ref(false)
const showPortalDialog = ref(false)
const showTagEditor = ref(false)
const editingTagName = ref('')
const editingTagColor = ref('#f97316')
const showTagSuggestions = ref(false)
const tagNameInputRef = ref(null)
const tagColorInputRef = ref(null)

// 计算属性
const expiryDate = computed(() => {
  return props.token.portal_info?.expiry_date || null
})

const balanceClasses = computed(() => {
  const hasError = !props.token.portal_info
  const exhausted = (
    props.token.ban_status === 'EXPIRED' ||
    props.token.ban_status === 'SUSPENDED'
  )

  if (hasError || exhausted) {
    return ['balance-text', 'exhausted']
  }

  const colorMode = props.token.balance_color_mode || 'green'
  return ['balance-text', `color-${colorMode}`]
})

const balanceDisplay = computed(() => {
  if (!props.token.portal_info) return '-'

  const status = props.token.ban_status
  if (status === 'EXPIRED') return t('tokenCard.expired')
  if (status === 'SUSPENDED') return t('tokenCard.banned')
  
  const credits = props.token.portal_info.credits_balance
  return credits !== undefined ? credits : '-'
})

// 方法
const handleRowClick = () => {
  if (showCopyMenu.value) {
    showCopyMenu.value = false
  }
}

const toggleCopyMenu = () => {
  showCopyMenu.value = !showCopyMenu.value
}

const handleCopyMenuClick = async (type) => {
  if (type !== 'payment') {
    showCopyMenu.value = false
  }
  
  switch (type) {
    case 'token':
      copyToken()
      break
    case 'url':
      copyTenantUrl()
      break
    case 'portal':
      copyPortalUrl()
      break
    case 'session':
      copyAuthSession()
      break
    case 'payment':
      await copyPaymentMethodLink()
      break
  }
}

const copyPaymentMethodLink = async () => {
  if (!props.token.auth_session) {
    window.$notify.warning(t('messages.noAuthSession'))
    showCopyMenu.value = false
    return
  }

  if (props.cachedPaymentLink) {
    try {
      await invoke('copy_to_clipboard', { text: props.cachedPaymentLink })
      window.$notify.success(t('messages.paymentLinkCopied'))
    } catch (error) {
      console.error('Failed to copy cached payment link:', error)
      window.$notify.error(t('messages.copyPaymentLinkFailed') + ': ' + error)
    } finally {
      showCopyMenu.value = false
    }
    return
  }

  isFetchingPaymentLink.value = true
  try {
    const result = await invoke('fetch_payment_method_link_command', {
      authSession: props.token.auth_session
    })

    const paymentLink = result.payment_method_link
    if (!paymentLink) {
      window.$notify.error(t('messages.copyPaymentLinkFailed') + ': 链接为空')
      return
    }

    await invoke('copy_to_clipboard', { text: paymentLink })
    window.$notify.success(t('messages.paymentLinkCopied'))
    emit('payment-link-fetched', { tokenId: props.token.id, link: paymentLink })
  } catch (error) {
    console.error('Failed to fetch or copy payment link:', error)
    window.$notify.error(t('messages.copyPaymentLinkFailed') + ': ' + error)
  } finally {
    isFetchingPaymentLink.value = false
    showCopyMenu.value = false
  }
}

const checkAccountStatus = async () => {
  if (props.token.skip_check) return
  if (isCheckingStatus.value || (props.isBatchChecking && !props.token.skip_check)) return

  isCheckingStatus.value = true

  try {
    const batchResults = await invoke('batch_check_tokens_status', {
      tokens: [{
        id: props.token.id,
        access_token: props.token.access_token,
        tenant_url: props.token.tenant_url,
        portal_url: props.token.portal_url || null,
        auth_session: props.token.auth_session || null,
        email_note: props.token.email_note || null
      }]
    })

    if (batchResults && batchResults.length > 0) {
      const result = batchResults[0]
      const statusResult = result.status_result
      const banStatus = statusResult.status
      let hasChanges = false

      if (props.token.ban_status !== banStatus) {
        props.token.ban_status = banStatus
        hasChanges = true
      }

      if (result.portal_info) {
        props.token.portal_info = {
          credits_balance: result.portal_info.credits_balance,
          expiry_date: result.portal_info.expiry_date
        }
        hasChanges = true
      }

      if (hasChanges) {
        props.token.updated_at = new Date().toISOString()
        emit('token-updated')
      }

      // 显示状态消息
      let statusMessage = ''
      let statusType = 'info'
      
      switch (banStatus) {
        case 'SUSPENDED':
          statusMessage = t('messages.accountBanned')
          statusType = 'error'
          break
        case 'EXPIRED':
          statusMessage = t('tokenCard.expired')
          statusType = 'warning'
          break
        case 'ACTIVE':
          statusMessage = t('messages.accountStatusNormal')
          statusType = 'success'
          break
        default:
          statusMessage = `${t('messages.accountStatus')}: ${banStatus}`
      }

      window.$notify[statusType](`${t('messages.checkComplete')}: ${statusMessage}`)
    }
  } catch (error) {
    window.$notify.error(`${t('messages.checkFailed')}: ${error}`)
  } finally {
    isCheckingStatus.value = false
  }
}

// 点击外部关闭菜单
const handleClickOutside = (event) => {
  if (showCopyMenu.value) {
    const copyWrapper = document.querySelector('.copy-menu-wrapper')
    if (copyWrapper && !copyWrapper.contains(event.target)) {
      showCopyMenu.value = false
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 编辑器名称映射
const editorNames = {
  'cursor': 'Cursor',
  'vscode': 'VS Code',
  'kiro': 'Kiro',
  'trae': 'Trae',
  'windsurf': 'Windsurf',
  'qoder': 'Qoder',
  'vscodium': 'VSCodium',
  'codebuddy': 'CodeBuddy',
  'vim': 'Vim',
  'auggie': 'Auggie',
  'antigravity': 'Antigravity',
  'idea': 'IntelliJ IDEA',
  'pycharm': 'PyCharm',
  'goland': 'GoLand',
  'rustrover': 'RustRover',
  'webstorm': 'WebStorm',
  'clion': 'CLion',
  'rider': 'Rider'
}

// VSCode 协议映射
const vscodeSchemes = {
  'cursor': 'cursor',
  'vscode': 'vscode',
  'kiro': 'kiro',
  'trae': 'trae',
  'windsurf': 'windsurf',
  'qoder': 'qoder',
  'vscodium': 'vscodium',
  'codebuddy': 'codebuddy',
  'antigravity': 'antigravity'
}

// JetBrains 编辑器集合
const jetbrainsEditors = new Set([
  'idea', 'pycharm', 'goland', 'rustrover', 'webstorm', 'clion', 'rider'
])

// 创建 VSCode 协议 URL
const createVSCodeProtocolUrl = (scheme) => {
  const token = encodeURIComponent(props.token.access_token)
  const url = encodeURIComponent(props.token.tenant_url)
  const portalUrl = encodeURIComponent(props.token.portal_url || "")
  return `${scheme}://Augment.vscode-augment/autoAuth?token=${token}&url=${url}&portal=${portalUrl}`
}

// 为 JetBrains 编辑器创建 JSON 文件
const createJetBrainsTokenFile = async (editorType) => {
  try {
    const tokenData = {
      url: props.token.tenant_url,
      token: props.token.access_token,
      timestamp: Date.now(),
      ide: editorType
    }
    await invoke('create_jetbrains_token_file', {
      editorType,
      tokenData: JSON.stringify(tokenData)
    })
    return { success: true }
  } catch (error) {
    return { success: false, error: String(error) }
  }
}

// 配置 Vim
const configureVimAugment = async () => {
  try {
    await invoke('configure_vim_augment', {
      tenantUrl: props.token.tenant_url,
      accessToken: props.token.access_token
    })
    return { success: true }
  } catch (error) {
    return { success: false, error: String(error) }
  }
}

// 配置 Auggie
const configureAuggie = async () => {
  try {
    await invoke('configure_auggie', {
      tenantUrl: props.token.tenant_url,
      accessToken: props.token.access_token
    })
    return { success: true }
  } catch (error) {
    return { success: false, error: String(error) }
  }
}

// 处理编辑器点击
const handleEditorClick = async (editorType) => {
  try {
    const editorName = editorNames[editorType] || editorType

    if (editorType === 'vim') {
      const result = await configureVimAugment()
      if (result.success) {
        window.$notify.success(t('messages.vimConfigSuccess'))
      } else {
        window.$notify.error(t('messages.vimConfigFailed') + ': ' + result.error)
      }
    } else if (editorType === 'auggie') {
      const result = await configureAuggie()
      if (result.success) {
        window.$notify.success(t('messages.auggieConfigSuccess'))
      } else {
        window.$notify.error(t('messages.auggieConfigFailed') + ': ' + result.error)
      }
    } else if (jetbrainsEditors.has(editorType)) {
      const result = await createJetBrainsTokenFile(editorType)
      if (result.success) {
        window.$notify.success(t('messages.editorTokenFileCreated', { editor: editorName }))
      } else {
        window.$notify.error(t('messages.createEditorTokenFileFailed', { editor: editorName, error: result.error }))
      }
    } else if (vscodeSchemes[editorType]) {
      const protocolUrl = createVSCodeProtocolUrl(vscodeSchemes[editorType])
      await invoke('open_editor_with_protocol', { protocolUrl })
      window.$notify.success(t('messages.openingEditor', { editor: editorName }))
    }
  } catch (error) {
    window.$notify.error(t('messages.operationFailed'))
  } finally {
    showEditorModal.value = false
  }
}

// 打开标签编辑器
const openTagEditor = () => {
  editingTagName.value = props.token.tag_name || ''
  editingTagColor.value = props.token.tag_color || '#f97316'
  showTagEditor.value = true
}

// 获取对比色
const getContrastColor = (hex) => {
  if (!/^#[0-9a-fA-F]{6}$/.test(hex)) return '#ffffff'
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  return luminance > 0.6 ? '#1f2937' : '#ffffff'
}

// 从所有 tokens 中提取已使用的标签
const existingTags = computed(() => {
  if (!props.allTokens) return []
  const tagMap = new Map()
  
  props.allTokens.forEach(token => {
    if (token.tag_name && token.tag_color) {
      // 使用标签名称作为key，避免重复
      if (!tagMap.has(token.tag_name)) {
        tagMap.set(token.tag_name, {
          name: token.tag_name,
          color: token.tag_color
        })
      }
    }
  })
  
  // 转换为数组并按名称排序
  return Array.from(tagMap.values())
    .sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
})

// 根据输入过滤标签建议
const filteredTagSuggestions = computed(() => {
  const input = editingTagName.value.trim().toLowerCase()
  
  // 如果没有输入,显示所有标签
  if (!input) {
    return existingTags.value
  }
  
  // 过滤匹配的标签
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
  editingTagName.value = suggestion.name
  editingTagColor.value = suggestion.color
  showTagSuggestions.value = false
}

// 打开颜色选择器
const openTagColorPicker = () => {
  tagColorInputRef.value?.click()
}

// 保存标签
const saveTag = () => {
  props.token.tag_name = editingTagName.value.trim()
  props.token.tag_color = editingTagColor.value
  props.token.updated_at = new Date().toISOString()
  emit('token-updated')
  showTagEditor.value = false
  window.$notify.success(t('messages.tagUpdated'))
}

// 清除标签
const clearTag = () => {
  props.token.tag_name = ''
  props.token.tag_color = ''
  props.token.updated_at = new Date().toISOString()
  emit('token-updated')
  showTagEditor.value = false
  window.$notify.success(t('messages.tagCleared'))
}

// 获取 Portal 浏览器标题
const getPortalBrowserTitle = () => {
  const email = props.token.email_note || ''
  const tag = props.token.tag_name || ''
  if (email && tag) return `${tag} - ${email}`
  if (email) return email
  if (tag) return tag
  return 'Portal'
}

// 暴露方法给父组件
defineExpose({
  refreshAccountStatus: checkAccountStatus
})
</script>

<style scoped>
.token-table-row {
  /* 不添加 transition 避免快速移动鼠标时的闪烁 */
  background: transparent;
}

.token-table-row:hover {
  background-color: var(--color-surface-hover, #f8fafc);
}

.token-table-row.selected {
  background-color: var(--color-accent-surface, rgba(59, 130, 246, 0.08));
}

.token-table-row.highlighted {
  animation: row-highlight 1s ease-in-out 2;
}

@keyframes row-highlight {
  0%, 100% { background-color: transparent; }
  50% { background-color: rgba(251, 191, 36, 0.2); }
}

.token-table-row td {
  padding: 12px 8px;
  /* 使用透明边框占位，避免边框闪烁 */
  border-bottom: 1px solid var(--color-divider, #e5e7eb);
  vertical-align: middle;
  white-space: nowrap;
  /* 确保单元格背景继承行背景 */
  background: inherit;
}

.cell-checkbox {
  width: 40px;
  text-align: center;
}

.checkbox-wrapper {
  display: inline-flex;
  cursor: pointer;
}

.checkbox-inner {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 2px solid var(--color-divider, #d1d5db);
  background: var(--color-surface, #ffffff);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.checkbox-inner:hover {
  border-color: var(--color-accent, #3b82f6);
}

.checkbox-inner.checked {
  background: var(--color-accent, #3b82f6);
  border-color: var(--color-accent, #3b82f6);
  color: white;
}

.cell-tag {
  width: 100px;
  max-width: 120px;
}

.tag-cell-wrapper {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 2px;
  border-radius: 6px;
  transition: background-color 0.15s ease;
}

.tag-cell-wrapper:hover {
  background-color: var(--color-surface-hover, #f1f5f9);
}

.tag-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.tag-badge.editable:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.no-tag {
  color: var(--color-text-muted, #9ca3af);
}

.no-tag.add-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px dashed var(--color-divider, #d1d5db);
  transition: all 0.15s ease;
  opacity: 0.5;
}

.token-table-row:hover .no-tag.add-tag {
  opacity: 1;
}

.no-tag.add-tag:hover {
  border-color: var(--color-accent, #3b82f6);
  color: var(--color-accent, #3b82f6);
  background: var(--color-accent-surface, rgba(59, 130, 246, 0.1));
}

.cell-status {
  width: 70px;
}

.status-badge {
  display: inline-block;
  font-size: 10px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 10px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.status-badge.active {
  background: var(--color-success-surface, #d4edda);
  color: var(--color-success-text, #155724);
  border: 1px solid var(--color-success-border, #c3e6cb);
}

.status-badge.banned {
  background: var(--color-danger-surface, #f8d7da);
  color: var(--color-danger-text, #721c24);
  border: 1px solid var(--color-danger-border, #f5c6cb);
}

.status-badge.inactive {
  background: var(--color-danger-surface, #f8d7da);
  color: var(--color-danger-text, #721c24);
  border: 1px solid var(--color-danger-border, #f5c6cb);
}

.status-badge.invalid {
  background: var(--color-warning-surface, #fff3cd);
  color: var(--color-warning-text, #856404);
  border: 1px solid var(--color-warning-border, #ffeaa7);
}

.cell-email {
  min-width: 180px;
  max-width: 220px;
}

.email-wrapper {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  width: 160px;
  position: relative;
}

.email-text {
  font-size: 12px;
  color: var(--color-link-visited, #4f46e5);
  background: var(--color-info-surface, #f0f9ff);
  padding: 2px 6px;
  border-radius: 4px;
  width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 默认显示脱敏邮箱，隐藏完整邮箱 */
.email-masked {
  display: inline-block;
}

.email-full {
  display: none;
}

/* 悬浮时显示完整邮箱，隐藏脱敏邮箱 */
.email-wrapper:hover .email-masked {
  display: none;
}

.email-wrapper:hover .email-full {
  display: inline-block;
  background: var(--color-info-border, #bae6fd);
}

.copy-hint {
  color: var(--color-text-muted, #6b7280);
  opacity: 0;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.email-wrapper:hover .copy-hint {
  opacity: 1;
}

.no-email {
  color: var(--color-text-muted, #9ca3af);
}

.cell-balance {
  width: 80px;
  text-align: center;
}

.balance-text {
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
}

.balance-text.color-green {
  color: var(--color-success-text, #166534);
  background: var(--color-success-surface, #dcfce7);
}

.balance-text.color-blue {
  color: #db2777;
  background: rgba(236, 72, 153, 0.1);
}

.balance-text.exhausted {
  color: var(--color-danger-text, #dc2626);
  background: var(--color-danger-surface, #fee2e2);
}

.cell-expiry {
  width: 110px;
}

.expiry-text {
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
}

.no-expiry {
  color: var(--color-text-muted, #9ca3af);
}

.cell-actions {
  width: 200px;
}

.actions-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 6px;
  background: var(--color-surface, #ffffff);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  color: var(--color-text-secondary, #6b7280);
}

.action-btn:hover:not(:disabled) {
  border-color: var(--color-accent, #3b82f6);
  background: var(--color-surface-hover, #f8fafc);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.loading {
  pointer-events: none;
}

.action-btn.delete:hover {
  border-color: var(--color-danger, #dc2626);
  color: var(--color-danger, #dc2626);
  background: var(--color-danger-surface, #fef2f2);
}

.action-btn.edit:hover {
  border-color: var(--color-success, #16a34a);
  color: var(--color-success, #16a34a);
  background: var(--color-success-surface, #f0fdf4);
}

.action-btn.portal:hover {
  border-color: var(--color-info, #0ea5e9);
  color: var(--color-info, #0ea5e9);
  background: var(--color-info-surface, #f0f9ff);
}

.action-btn img {
  width: 16px;
  height: 16px;
}

/* 复制菜单 */
.copy-menu-wrapper {
  position: relative;
}

.copy-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  z-index: 1000;
  padding: 4px 0;
}

.copy-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-primary, #374151);
  transition: background-color 0.15s ease;
}

.copy-menu-item:hover:not(:disabled) {
  background: var(--color-surface-hover, #f3f4f6);
}

.copy-menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 加载动画 */
.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-divider, #e5e7eb);
  border-top-color: var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.loading-spinner-small {
  width: 12px;
  height: 12px;
  border: 2px solid var(--color-divider, #e5e7eb);
  border-top-color: var(--color-accent, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 下拉动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* 暗黑模式 */
[data-theme='dark'] .token-table-row:hover {
  background-color: rgba(55, 65, 81, 0.4);
}

[data-theme='dark'] .token-table-row.selected {
  background-color: rgba(59, 130, 246, 0.15);
}

[data-theme='dark'] .email-text {
  background: rgba(56, 189, 248, 0.2);
  color: #93c5fd;
}

[data-theme='dark'] .action-btn {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(71, 85, 105, 0.6);
  color: #cbd5e1;
}

[data-theme='dark'] .action-btn:hover:not(:disabled) {
  background: rgba(71, 85, 105, 0.6);
  border-color: rgba(100, 116, 139, 0.7);
}

[data-theme='dark'] .copy-dropdown {
  background: var(--color-surface, #1f2937);
  border-color: rgba(75, 85, 99, 0.6);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .copy-menu-item {
  color: var(--color-text-primary, #e5e7eb);
}

[data-theme='dark'] .copy-menu-item:hover:not(:disabled) {
  background: rgba(55, 65, 81, 0.6);
}

[data-theme='dark'] .balance-text.color-green {
  color: #86efac;
  background: rgba(34, 197, 94, 0.2);
}

[data-theme='dark'] .balance-text.color-blue {
  color: #f9a8d4;
  background: rgba(236, 72, 153, 0.2);
}

[data-theme='dark'] .balance-text.exhausted {
  color: #fca5a5;
  background: rgba(220, 38, 38, 0.2);
}

[data-theme='dark'] .tag-cell-wrapper:hover {
  background-color: rgba(55, 65, 81, 0.5);
}

[data-theme='dark'] .no-tag.add-tag {
  border-color: rgba(75, 85, 99, 0.6);
  color: #9ca3af;
}

[data-theme='dark'] .no-tag.add-tag:hover {
  border-color: var(--color-accent, #3b82f6);
  color: var(--color-accent, #3b82f6);
  background: rgba(59, 130, 246, 0.2);
}

[data-theme='dark'] .email-text {
  color: #93c5fd;
  background: rgba(56, 189, 248, 0.15);
}

[data-theme='dark'] .email-wrapper:hover .email-full {
  background: rgba(56, 189, 248, 0.25);
}

/* 编辑器模态框样式 - 与 TokenCard 一致 */
.editor-modal-overlay {
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
  z-index: 4000;
  backdrop-filter: blur(2px);
  pointer-events: auto;
}

.editor-modal {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 700px;
  width: 90%;
  max-height: 95vh;
  overflow: hidden;
  transition: transform 0.3s ease;
  position: relative;
  pointer-events: auto;
  margin: auto;
}

.editor-modal .modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.editor-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

.modal-close {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted, #666);
  border-radius: 4px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-close:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-heading, #333);
}

.editor-modal .modal-content {
  padding: 20px 24px 24px;
  max-height: calc(95vh - 80px);
  overflow-y: auto;
}

.editor-section {
  margin-bottom: 24px;
  padding-bottom: 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.editor-section:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.editor-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.jetbrains-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 12px;
}

.editor-option {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border: 2px solid var(--color-divider, #e1e5e9);
  border-radius: 8px;
  background: var(--color-surface, #ffffff);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
  width: 100%;
  position: relative;
  font-family: inherit;
  font-size: inherit;
  text-decoration: none;
  color: inherit;
  box-sizing: border-box;
}

.editor-option:hover {
  border-color: var(--color-accent, #3b82f6);
  background: var(--color-surface-soft, #f8fafc);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.12);
}

.editor-option:active {
  background: var(--color-surface-soft, #f1f5f9);
  box-shadow: 0 1px 4px rgba(59, 130, 246, 0.08);
}

.editor-option:focus {
  outline: 2px solid var(--color-accent, #3b82f6);
  outline-offset: 2px;
}

.editor-icon {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--color-surface-muted, #f8f9fa);
  border: 1px solid var(--color-surface-muted, #e9ecef);
}

.editor-icon img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.vscode-option .editor-icon,
.cursor-option .editor-icon,
.kiro-option .editor-icon,
.trae-option .editor-icon,
.windsurf-option .editor-icon,
.qoder-option .editor-icon,
.vscodium-option .editor-icon,
.codebuddy-option .editor-icon,
.vim-option .editor-icon,
.auggie-option .editor-icon,
.antigravity-option .editor-icon,
.idea-option .editor-icon,
.pycharm-option .editor-icon,
.goland-option .editor-icon,
.rustrover-option .editor-icon,
.webstorm-option .editor-icon,
.clion-option .editor-icon,
.rider-option .editor-icon {
  background: var(--color-info-surface, #f0f9ff);
  border-color: var(--color-info-surface, #e0f2fe);
}

.editor-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.editor-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

/* 标签编辑器样式 - 复用 TokenForm 样式 */
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
  z-index: 4000;
  backdrop-filter: blur(2px);
}

.tag-editor-modal {
  background: var(--color-surface, #ffffff);
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
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.tag-editor-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
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
  color: var(--color-text-primary, #1f2937);
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
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 6px;
  font-size: 14px;
  color: var(--color-text-primary, #1f2937);
  background: var(--color-surface, #ffffff);
  outline: none;
  transition: border-color 0.15s ease;
  box-sizing: border-box;
}

.tag-name-input:focus {
  border-color: var(--color-accent, #3b82f6);
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
  background: var(--color-surface-hover, #e5e7eb);
  color: var(--color-text-muted, #6b7280);
  border-radius: 50%;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.tag-clear-btn:hover {
  background: var(--color-error, #ef4444);
  color: #ffffff;
}

.tag-color-display {
  width: 40px;
  height: 40px;
  border: 2px solid var(--color-divider, #e5e7eb);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
  position: relative;
}

.tag-color-display:hover {
  border-color: var(--color-accent, #3b82f6);
  transform: scale(1.08);
}

.hidden-color-input {
  position: absolute;
  top: 100%;
  left: 0;
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
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-divider, #e5e7eb);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-height: 200px;
  overflow-y: auto;
  z-index: 100;
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
  background: var(--color-surface-hover, #f3f4f6);
}

.tag-suggestion-item .tag-preview {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  flex-shrink: 0;
}

.tag-suggestion-name {
  font-size: 14px;
  color: var(--color-text-primary, #1f2937);
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

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-divider, #e5e7eb);
  background: var(--color-surface-secondary, #f9fafb);
}

.btn-primary,
.btn-secondary {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary {
  background: var(--color-accent, #3b82f6);
  color: white;
  border: none;
}

.btn-primary:hover {
  background: var(--color-accent-hover, #2563eb);
}

.btn-secondary {
  background: var(--color-surface, #ffffff);
  color: var(--color-text-primary, #1f2937);
  border: 1px solid var(--color-divider, #e5e7eb);
}

.btn-secondary:hover {
  background: var(--color-surface-hover, #f3f4f6);
}

/* 模态框动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .editor-modal,
.modal-enter-active .tag-editor-modal,
.modal-leave-active .editor-modal,
.modal-leave-active .tag-editor-modal {
  transition: transform 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .editor-modal,
.modal-enter-from .tag-editor-modal {
  transform: scale(0.95) translateY(-10px);
}

.modal-leave-to .editor-modal,
.modal-leave-to .tag-editor-modal {
  transform: scale(0.95) translateY(-10px);
}

/* 下拉动画 */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* 暗黑模式 - 模态框 */
[data-theme='dark'] .editor-modal,
[data-theme='dark'] .tag-editor-modal {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .editor-modal .modal-header,
[data-theme='dark'] .tag-editor-modal .modal-header {
  border-bottom-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .editor-modal .modal-header h3,
[data-theme='dark'] .tag-editor-modal .modal-header h3 {
  color: #f9fafb;
}

[data-theme='dark'] .modal-close:hover {
  background: rgba(55, 65, 81, 0.6);
  color: #f9fafb;
}

[data-theme='dark'] .editor-option {
  background: rgba(55, 65, 81, 0.3);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .editor-option:hover {
  background: rgba(59, 130, 246, 0.15);
  border-color: var(--color-accent, #3b82f6);
}

[data-theme='dark'] .editor-icon {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .editor-info {
  color: #f9fafb;
}

[data-theme='dark'] .editor-name {
  color: #f9fafb;
}

[data-theme='dark'] .tag-group label {
  color: #f9fafb;
}

[data-theme='dark'] .tag-name-input {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
  color: #f9fafb;
}

[data-theme='dark'] .tag-name-input:focus {
  border-color: var(--color-accent, #3b82f6);
}

[data-theme='dark'] .tag-color-display {
  border-color: rgba(75, 85, 99, 0.8);
}

[data-theme='dark'] .tag-color-display:hover {
  border-color: var(--color-accent, #3b82f6);
}

[data-theme='dark'] .tag-editor-modal {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .tag-clear-btn {
  background: rgba(75, 85, 99, 0.6);
  color: #9ca3af;
}

[data-theme='dark'] .tag-clear-btn:hover {
  background: var(--color-error, #ef4444);
  color: #ffffff;
}

[data-theme='dark'] .tag-suggestions {
  background: var(--color-surface, #1f2937);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .tag-suggestion-item:hover,
[data-theme='dark'] .tag-suggestion-item.selected {
  background: rgba(55, 65, 81, 0.6);
}

[data-theme='dark'] .tag-suggestion-name {
  color: #f9fafb;
}

[data-theme='dark'] .modal-footer {
  background: rgba(17, 24, 39, 0.5);
  border-top-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .btn-secondary {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
  color: #f9fafb;
}

[data-theme='dark'] .btn-secondary:hover {
  background: rgba(75, 85, 99, 0.6);
}
</style>
