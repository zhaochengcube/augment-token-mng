<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="show" class="editor-modal-overlay">
        <div class="editor-modal" @click.stop>
          <div class="modal-header">
            <h3>{{ $t('tokenCard.selectEditor') }}</h3>
            <button @click.stop="closeModal" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="modal-content">
            <!-- VSCode 系编辑器区域 -->
            <div class="editor-section">
              <div class="editor-options jetbrains-grid">
                <button v-for="editor in vscodeEditors" :key="editor.type" @click="handleEditorClick(editor.type)" :class="['editor-option', `${editor.type}-option`]">
                  <div class="editor-icon">
                    <img :src="editorIcons[editor.type]" :alt="editor.name" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">{{ editor.name }}</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- JetBrains 系编辑器区域 -->
            <div class="editor-section">
              <div class="editor-options jetbrains-grid">
                <button v-for="editor in jetbrainsEditorsList" :key="editor.type" @click="handleEditorClick(editor.type)" :class="['editor-option', `${editor.type}-option`]">
                  <div class="editor-icon">
                    <img :src="editorIcons[editor.type]" :alt="editor.name" width="32" height="32" />
                  </div>
                  <div class="editor-info">
                    <span class="editor-name">{{ editor.name }}</span>
                  </div>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Trae 版本选择对话框 -->
    <Transition name="modal" appear>
      <div v-if="showTraeVersionDialog" class="trae-version-modal-overlay" @click="showTraeVersionDialog = false">
        <div class="trae-version-modal" @click.stop>
          <div class="modal-header">
            <h3>选择 Trae 版本</h3>
            <button @click="showTraeVersionDialog = false" class="modal-close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="version-options">
              <button @click="handleTraeVersionSelect('global')" class="version-option">
                <div class="version-icon">🌍</div>
                <div class="version-name">Trae 国际版</div>
              </button>
              <button @click="handleTraeVersionSelect('cn')" class="version-option">
                <div class="version-icon">🇨🇳</div>
                <div class="version-name">Trae 国内版</div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Props
const props = defineProps({
  show: {
    type: Boolean,
    default: false
  },
  token: {
    type: Object,
    required: true
  }
})

// Emits
const emit = defineEmits(['close', 'success', 'error'])

// 本地状态
const showTraeVersionDialog = ref(false)

// 当前主题
const currentTheme = ref('light')

// 监听主题变化
const updateTheme = () => {
  currentTheme.value = document.documentElement.dataset.theme || 'light'
}

let observer = null
onMounted(() => {
  updateTheme()
  observer = new MutationObserver(() => updateTheme())
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme']
  })
})

onUnmounted(() => {
  if (observer) observer.disconnect()
})

// 编辑器图标
const editorIcons = computed(() => ({
  vscode: '/icons/vscode.svg',
  'vscode-insiders': '/icons/vscode-insiders.svg',
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
  phpstorm: '/icons/phpstorm.svg',
  clion: '/icons/clion.svg',
  datagrip: '/icons/datagrip.svg',
  rider: '/icons/rider.svg',
  rubymine: '/icons/rubymine.svg',
  aqua: '/icons/aqua.svg',
  androidstudio: '/icons/androidstudio.svg'
}))

// VSCode 系编辑器列表
const vscodeEditors = [
  { type: 'vscode', name: 'VS Code' },
  { type: 'vscode-insiders', name: 'VS Code - Insiders' },
  { type: 'cursor', name: 'Cursor' },
  { type: 'kiro', name: 'Kiro' },
  { type: 'trae', name: 'Trae' },
  { type: 'windsurf', name: 'Windsurf' },
  { type: 'qoder', name: 'Qoder' },
  { type: 'vscodium', name: 'VSCodium' },
  { type: 'codebuddy', name: 'CodeBuddy' },
  { type: 'vim', name: 'Vim' },
  { type: 'auggie', name: 'Auggie' },
  { type: 'antigravity', name: 'Antigravity' }
]

// JetBrains 系编辑器列表
const jetbrainsEditorsList = [
  { type: 'idea', name: 'IntelliJ IDEA' },
  { type: 'pycharm', name: 'PyCharm' },
  { type: 'goland', name: 'GoLand' },
  { type: 'rustrover', name: 'RustRover' },
  { type: 'webstorm', name: 'WebStorm' },
  { type: 'phpstorm', name: 'PhpStorm' },
  { type: 'androidstudio', name: 'Android Studio' },
  { type: 'clion', name: 'CLion' },
  { type: 'datagrip', name: 'DataGrip' },
  { type: 'rider', name: 'Rider' },
  { type: 'rubymine', name: 'RubyMine' },
  { type: 'aqua', name: 'Aqua' }
]

// 编辑器名称映射
const editorNames = {
  'cursor': 'Cursor',
  'vscode': 'VS Code',
  'vscode-insiders': 'VS Code - Insiders',
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
  'phpstorm': 'PhpStorm',
  'androidstudio': 'Android Studio',
  'clion': 'CLion',
  'datagrip': 'DataGrip',
  'rider': 'Rider',
  'rubymine': 'RubyMine',
  'aqua': 'Aqua'
}

// VSCode 协议映射
const vscodeSchemes = {
  'cursor': 'cursor',
  'vscode': 'vscode',
  'vscode-insiders': 'vscode-insiders',
  'kiro': 'kiro',
  'trae': 'trae',
  'trae-cn': 'trae-cn',
  'windsurf': 'windsurf',
  'qoder': 'qoder',
  'vscodium': 'vscodium',
  'codebuddy': 'codebuddy',
  'antigravity': 'antigravity'
}

// JetBrains 编辑器集合
const jetbrainsEditors = new Set([
  'idea', 'pycharm', 'goland', 'rustrover', 'webstorm',
  'phpstorm', 'androidstudio', 'clion', 'datagrip', 'rider', 'rubymine', 'aqua'
])

// 关闭模态框
const closeModal = () => {
  emit('close')
}

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
      tokenData: JSON.stringify(tokenData, null, 2)
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
      accessToken: props.token.access_token,
      tenantUrl: props.token.tenant_url
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
      accessToken: props.token.access_token,
      tenantUrl: props.token.tenant_url
    })
    return { success: true }
  } catch (error) {
    return { success: false, error: String(error) }
  }
}

// 处理编辑器点击
const handleEditorClick = async (editorType) => {
  // 如果是 Trae，显示版本选择对话框
  if (editorType === 'trae') {
    showTraeVersionDialog.value = true
    return
  }

  try {
    const editorName = editorNames[editorType] || editorType

    if (editorType === 'vim') {
      const result = await configureVimAugment()
      if (result.success) {
        emit('success', t('messages.vimConfigSuccess'))
      } else {
        emit('error', t('messages.vimConfigFailed') + ': ' + result.error)
      }
    } else if (editorType === 'auggie') {
      const result = await configureAuggie()
      if (result.success) {
        emit('success', t('messages.auggieConfigSuccess'))
      } else {
        emit('error', t('messages.auggieConfigFailed') + ': ' + result.error)
      }
    } else if (jetbrainsEditors.has(editorType)) {
      const result = await createJetBrainsTokenFile(editorType)
      if (result.success) {
        emit('success', t('messages.editorTokenFileCreated', { editor: editorName }))
      } else {
        emit('error', t('messages.createEditorTokenFileFailed', { editor: editorName, error: result.error }))
      }
    } else if (vscodeSchemes[editorType]) {
      const protocolUrl = createVSCodeProtocolUrl(vscodeSchemes[editorType])
      await invoke('open_editor_with_protocol', { protocolUrl })
      emit('success', t('messages.openingEditor', { editor: editorName }))
    }
  } catch (error) {
    emit('error', t('messages.operationFailed'))
  } finally {
    closeModal()
  }
}

// 处理 Trae 版本选择
const handleTraeVersionSelect = async (version) => {
  showTraeVersionDialog.value = false

  try {
    const editorType = version === 'global' ? 'trae' : 'trae-cn'
    const protocolUrl = createVSCodeProtocolUrl(vscodeSchemes[editorType])
    await invoke('open_editor_with_protocol', { protocolUrl })
    emit('success', t('messages.openingEditor', { editor: 'Trae' }))
  } catch (error) {
    emit('error', t('messages.operationFailed'))
  } finally {
    closeModal()
  }
}
</script>

<style scoped>
/* Vue 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .editor-modal,
.modal-leave-to .editor-modal {
  transform: translateY(-20px) scale(0.95);
}

.modal-enter-to .editor-modal,
.modal-leave-from .editor-modal {
  transform: translateY(0) scale(1);
}

/* 编辑器选择模态框样式 */
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

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.modal-header h3 {
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

.modal-content {
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
.phpstorm-option .editor-icon,
.androidstudio-option .editor-icon,
.clion-option .editor-icon,
.datagrip-option .editor-icon,
.rider-option .editor-icon,
.rubymine-option .editor-icon,
.aqua-option .editor-icon {
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

/* Trae 版本选择对话框样式 */
.trae-version-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
}

.trae-version-modal {
  background: var(--color-surface, #ffffff);
  border-radius: 12px;
  max-width: 500px;
  width: 100%;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.trae-version-modal .modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-divider, #e1e5e9);
}

.trae-version-modal .modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.trae-version-modal .modal-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.version-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.version-option {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  border: 2px solid var(--color-divider, #e1e5e9);
  border-radius: 12px;
  background: var(--color-surface, #ffffff);
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  text-align: left;
  font-family: inherit;
}

.version-option:hover {
  border-color: var(--color-accent, #3b82f6);
  background: var(--color-surface-soft, #f8fafc);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
  transform: translateY(-2px);
}

.version-option:active {
  transform: translateY(0);
}

.version-icon {
  font-size: 32px;
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface-muted, #f8f9fa);
  border-radius: 8px;
}

.version-name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading, #333);
}

/* 响应式处理 */
@media (max-width: 480px) {
  .editor-modal {
    width: 95%;
    margin: 16px;
    max-height: 90vh;
  }

  .modal-header {
    padding: 16px 20px 12px;
  }

  .modal-header h3 {
    font-size: 16px;
  }

  .modal-content {
    padding: 16px 20px 20px;
  }

  .editor-section {
    margin-bottom: 20px;
    padding-bottom: 20px;
  }

  .jetbrains-grid {
    grid-template-columns: 1fr;
  }

  .editor-option {
    padding: 12px;
    gap: 12px;
  }

  .editor-icon {
    width: 40px;
    height: 40px;
  }

  .editor-icon img {
    width: 28px;
    height: 28px;
  }

  .editor-name {
    font-size: 15px;
  }
}

/* 黑暗模式 */
[data-theme='dark'] .editor-modal {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .trae-version-modal {
  background: var(--color-surface, #1f2937);
}

[data-theme='dark'] .version-option {
  background: rgba(55, 65, 81, 0.5);
  border-color: rgba(75, 85, 99, 0.6);
}

[data-theme='dark'] .version-option:hover {
  background: rgba(55, 65, 81, 0.7);
  border-color: rgba(59, 130, 246, 0.6);
}

[data-theme='dark'] .version-icon {
  background: rgba(55, 65, 81, 0.8);
}

[data-theme='dark'] .editor-option {
  background: rgba(51, 65, 85, 0.5);
  border-color: rgba(71, 85, 105, 0.6);
}

[data-theme='dark'] .editor-option:hover {
  background: rgba(71, 85, 105, 0.6);
  border-color: var(--color-accent, #3b82f6);
}
</style>

