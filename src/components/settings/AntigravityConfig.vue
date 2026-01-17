<template>
  <BaseModal
    :visible="true"
    :title="$t('antigravityConfig.title')"
    :body-scroll="false"
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-5">
      <!-- 启用自定义路径开关 -->
      <div class="flex flex-col gap-2">
        <label class="flex items-center cursor-pointer font-medium text-text">
          <input
            type="checkbox"
            v-model="config.useCustomPath"
            :disabled="isLoading"
            class="mr-2.5 w-[18px] h-[18px] cursor-pointer accent-accent disabled:cursor-not-allowed"
          >
          <span>{{ $t('antigravityConfig.useCustomPath') }}</span>
        </label>
        <small class="text-[12px] text-text-muted ml-[26px]">
          {{ $t('antigravityConfig.useCustomPathHelp') }}
        </small>
      </div>

      <!-- 自定义路径输入 -->
      <div v-if="config.useCustomPath" class="form-group !mb-0">
        <label for="customPath" class="label">
          {{ $t('antigravityConfig.customPath') }}:
        </label>
        <div class="flex gap-2">
          <input
            id="customPath"
            v-model="config.customExecutablePath"
            type="text"
            class="input flex-1"
            :placeholder="$t('antigravityConfig.pathPlaceholder')"
            :disabled="isLoading"
          >
          <button
            class="btn btn--secondary"
            @click="selectFile"
            :disabled="isLoading"
          >
            {{ $t('antigravityConfig.browse') }}
          </button>
        </div>
        <small class="text-[12px] text-text-muted mt-1 block">
          {{ $t('antigravityConfig.pathHelp') }}
        </small>
      </div>

      <!-- 当前路径显示 -->
      <div v-if="currentPath" class="p-3 bg-surface-secondary rounded-lg">
        <div class="text-[12px] font-medium text-text-muted mb-1">
          {{ $t('antigravityConfig.currentPath') }}:
        </div>
        <div class="text-[13px] text-text font-mono break-all">
          {{ currentPath }}
        </div>
      </div>

      <!-- 默认路径提示 -->
      <div v-if="!config.useCustomPath" class="p-3 bg-surface-secondary rounded-lg">
        <div class="text-[12px] font-medium text-text-muted mb-2">
          {{ $t('antigravityConfig.defaultPaths') }}:
        </div>
        <ul class="text-[12px] text-text-muted space-y-1 font-mono">
          <li v-for="path in defaultPaths" :key="path">• {{ path }}</li>
        </ul>
      </div>

      <!-- 操作按钮 -->
      <div class="flex gap-3 pt-2">
        <button
          class="btn btn--primary flex-1"
          @click="saveConfig"
          :disabled="isLoading || isSaving"
        >
          <span v-if="isSaving">{{ $t('antigravityConfig.saving') }}</span>
          <span v-else>{{ $t('antigravityConfig.save') }}</span>
        </button>
        
        <button
          v-if="config.useCustomPath && config.customExecutablePath"
          class="btn btn--secondary"
          @click="validatePath"
          :disabled="isLoading || isValidating"
        >
          <span v-if="isValidating">{{ $t('antigravityConfig.validating') }}</span>
          <span v-else>{{ $t('antigravityConfig.validate') }}</span>
        </button>
        
        <button
          class="btn btn--secondary"
          @click="testLaunch"
          :disabled="isLoading || isTesting"
        >
          <span v-if="isTesting">{{ $t('antigravityConfig.testing') }}</span>
          <span v-else>{{ $t('antigravityConfig.test') }}</span>
        </button>
      </div>

      <!-- 重置按钮 -->
      <div v-if="configExists" class="pt-2 border-t border-border">
        <button
          class="btn btn--danger w-full"
          @click="deleteConfig"
          :disabled="isLoading || isDeleting"
        >
          <span v-if="isDeleting">{{ $t('antigravityConfig.deleting') }}</span>
          <span v-else>{{ $t('antigravityConfig.reset') }}</span>
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'
import BaseModal from '../common/BaseModal.vue'

// Emits
const emit = defineEmits(['close', 'config-saved'])

// i18n
const { t } = useI18n()

// Reactive data
const config = ref({
  useCustomPath: false,
  customExecutablePath: ''
})

const isLoading = ref(false)
const isSaving = ref(false)
const isValidating = ref(false)
const isTesting = ref(false)
const isDeleting = ref(false)
const configExists = ref(false)
const currentPath = ref('')

// 默认路径列表（根据平台）
const defaultPaths = computed(() => {
  const platform = navigator.platform.toLowerCase()
  
  if (platform.includes('mac')) {
    return ['/Applications/Antigravity.app/Contents/MacOS/Antigravity']
  } else if (platform.includes('win')) {
    return [
      '%LOCALAPPDATA%\\Programs\\Antigravity\\Antigravity.exe',
      'C:\\Program Files\\Antigravity\\Antigravity.exe'
    ]
  } else {
    return [
      '/usr/bin/antigravity',
      '/usr/local/bin/antigravity',
      '/opt/Antigravity/antigravity'
    ]
  }
})

// 加载配置
const loadConfig = async () => {
  isLoading.value = true
  try {
    const loadedConfig = await invoke('antigravity_load_config')
    config.value = {
      useCustomPath: loadedConfig.useCustomPath || false,
      customExecutablePath: loadedConfig.customExecutablePath || ''
    }
    
    // 检查配置是否存在
    configExists.value = await invoke('antigravity_config_exists')
    
    // 获取当前使用的路径
    try {
      currentPath.value = await invoke('antigravity_get_executable_path')
    } catch (error) {
      console.warn('无法获取当前路径:', error)
      currentPath.value = ''
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    window.$notify.error(t('antigravityConfig.messages.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

// 选择文件
const selectFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      title: t('antigravityConfig.selectExecutable')
    })
    
    if (selected) {
      config.value.customExecutablePath = selected
    }
  } catch (error) {
    console.error('选择文件失败:', error)
    window.$notify.error(t('antigravityConfig.messages.selectFailed', { error }))
  }
}

// 验证路径
const validatePath = async () => {
  if (!config.value.customExecutablePath) {
    window.$notify.warning(t('antigravityConfig.messages.pathRequired'))
    return
  }
  
  isValidating.value = true
  try {
    await invoke('antigravity_validate_executable_path', {
      path: config.value.customExecutablePath
    })
    window.$notify.success(t('antigravityConfig.messages.pathValid'))
  } catch (error) {
    console.error('路径验证失败:', error)
    window.$notify.error(t('antigravityConfig.messages.pathInvalid', { error }))
  } finally {
    isValidating.value = false
  }
}

// 保存配置
const saveConfig = async () => {
  // 如果启用了自定义路径但没有填写路径
  if (config.value.useCustomPath && !config.value.customExecutablePath) {
    window.$notify.warning(t('antigravityConfig.messages.pathRequired'))
    return
  }
  
  isSaving.value = true
  try {
    await invoke('antigravity_save_config', {
      useCustomPath: config.value.useCustomPath,
      customExecutablePath: config.value.useCustomPath ? config.value.customExecutablePath : null
    })
    
    window.$notify.success(t('antigravityConfig.messages.saveSuccess'))
    emit('config-saved')
    
    // 重新加载配置以更新当前路径
    await loadConfig()
  } catch (error) {
    console.error('保存配置失败:', error)
    window.$notify.error(t('antigravityConfig.messages.saveFailed', { error }))
  } finally {
    isSaving.value = false
  }
}

// 测试启动
const testLaunch = async () => {
  isTesting.value = true
  try {
    await invoke('antigravity_launch')
    window.$notify.success(t('antigravityConfig.messages.launchSuccess'))
  } catch (error) {
    console.error('启动失败:', error)
    window.$notify.error(t('antigravityConfig.messages.launchFailed', { error }))
  } finally {
    isTesting.value = false
  }
}

// 删除配置
const deleteConfig = async () => {
  if (!confirm(t('antigravityConfig.messages.confirmDelete'))) {
    return
  }
  
  isDeleting.value = true
  try {
    await invoke('antigravity_delete_config')
    window.$notify.success(t('antigravityConfig.messages.deleteSuccess'))
    
    // 重置为默认值
    config.value = {
      useCustomPath: false,
      customExecutablePath: ''
    }
    configExists.value = false
    
    // 重新加载配置
    await loadConfig()
  } catch (error) {
    console.error('删除配置失败:', error)
    window.$notify.error(t('antigravityConfig.messages.deleteFailed', { error }))
  } finally {
    isDeleting.value = false
  }
}

// 初始化
onMounted(() => {
  loadConfig()
})
</script>

