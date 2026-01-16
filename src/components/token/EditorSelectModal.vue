<template>
  <!-- 主编辑器选择模态框 -->
  <BaseModal
    :visible="show"
    :title="$t('tokenCard.selectEditor')"
    :close-on-overlay="true"
    :body-scroll="true"
    modal-class="!max-w-[720px]"
    @close="closeModal"
  >
    <div class="flex flex-col gap-6">
      <!-- VSCode 系编辑器区域 -->
      <div class="pb-6 border-b border-border last:pb-0 last:border-b-0">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            v-for="editor in vscodeEditors"
            :key="editor.type"
            class="card card--lift flex items-center gap-4 !p-4 text-left"
            @click="handleEditorClick(editor.type)"
          >
            <div class="shrink-0 w-12 h-12 flex items-center justify-center">
              <img :src="editorIcons[editor.type]" :alt="editor.name" class="w-8 h-8 object-contain" />
            </div>
            <span class="text-base font-semibold text-text">{{ editor.name }}</span>
          </button>
        </div>
      </div>

      <!-- JetBrains 系编辑器区域 -->
      <div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            v-for="editor in jetbrainsEditorsList"
            :key="editor.type"
            class="card card--lift flex items-center gap-4 !p-4 text-left"
            @click="handleEditorClick(editor.type)"
          >
            <div class="shrink-0 w-12 h-12 flex items-center justify-center">
              <img :src="editorIcons[editor.type]" :alt="editor.name" class="w-8 h-8 object-contain" />
            </div>
            <span class="text-base font-semibold text-text">{{ editor.name }}</span>
          </button>
        </div>
      </div>
    </div>
  </BaseModal>

  <!-- Trae 版本选择对话框 -->
  <BaseModal
    :visible="showTraeVersionDialog"
    title="选择 Trae 版本"
    :close-on-overlay="true"
    :body-scroll="false"
    modal-class="!max-w-[520px] !z-[10000]"
    @close="showTraeVersionDialog = false"
  >
    <div class="flex flex-col gap-4">
      <button
        class="card card--lift flex items-center gap-4 !p-5 text-left"
        @click="handleTraeVersionSelect('global')"
      >
        <div class="text-[32px] shrink-0 w-12 h-12 flex items-center justify-center">🌍</div>
        <span class="text-base font-semibold text-text">Trae 国际版</span>
      </button>
      <button
        class="card card--lift flex items-center gap-4 !p-5 text-left"
        @click="handleTraeVersionSelect('cn')"
      >
        <div class="text-[32px] shrink-0 w-12 h-12 flex items-center justify-center">🇨🇳</div>
        <span class="text-base font-semibold text-text">Trae 国内版</span>
      </button>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

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
