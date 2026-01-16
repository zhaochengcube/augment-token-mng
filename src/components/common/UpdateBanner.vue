<template>
  <div v-if="visible" :class="[
    'update-banner',
    'fixed bottom-[22px] left-[22px] max-w-[360px] w-auto',
    'bg-surface/95 backdrop-blur-[16px] border border-border rounded-2xl',
    'shadow-[0_15px_35px_rgba(0,0,0,0.25)] shadow-muted z-[999]',
    expanded ? 'w-[420px] max-w-[420px]' : ''
  ]">
    <div :class="['relative', expanded ? 'p-[22px]' : 'px-[18px] py-[14px]']">
      <!-- 简化视图 -->
      <div
        class="flex items-center gap-3 cursor-pointer p-1.5 -m-1.5 rounded-[10px] transition-all duration-200 hover:bg-accent/10"
        @click="toggleExpand"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-accent shrink-0 drop-shadow-[0_0_6px_var(--accent)]">
          <circle cx="12" cy="12" r="10"></circle>
          <polyline points="12 6 12 12 16 14"></polyline>
        </svg>
        <h3 class="m-0 text-sm font-semibold text-text flex-1 whitespace-nowrap">{{ $t('update.newVersionAvailable') }}: v{{ updateInfo.latest_version }}</h3>
        <button
          class="flex items-center justify-center w-6 h-6 rounded-lg border border-border bg-muted/50 text-text-muted transition-all duration-200 hover:bg-accent/15 hover:border-accent hover:text-accent"
          @click.stop="close"
          :aria-label="$t('update.close')"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- 展开内容 -->
      <div v-if="expanded" class="expanded-content mt-[18px]">
        <!-- 版本信息 -->
        <div class="flex flex-col gap-2.5 mb-[22px] p-3.5 bg-muted/50 border border-border rounded-xl">
          <div class="flex justify-between items-center">
            <span class="text-sm text-text-muted">{{ $t('update.currentVersion') }}:</span>
            <span class="text-sm font-bold text-text font-mono">v{{ updateInfo.current_version }}</span>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-sm text-text-muted">{{ $t('update.latestVersion') }}:</span>
            <span class="text-sm font-bold text-accent font-mono drop-shadow-[0_0_8px_var(--accent)]">v{{ updateInfo.latest_version }}</span>
          </div>
        </div>

        <div>
          <h4 class="m-0 mb-3.5 text-sm font-semibold text-text">{{ $t('update.installMethods') }}:</h4>

          <div class="mb-[18px]">
            <button
              class="flex items-center gap-2.5 w-full px-4 py-3 bg-accent-tech text-accent border border-border-accent-tech rounded-[10px] text-sm font-semibold cursor-pointer transition-all duration-200 hover:bg-accent-tech-hover hover:border-border-accent-tech-hover hover:-translate-y-0.5 hover:shadow-accent"
              @click="openGitHubRelease"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              {{ $t('update.githubRelease') }}
            </button>
          </div>

          <div class="mb-[18px]">
            <span class="block text-xs text-text-muted mb-2 font-semibold uppercase tracking-[0.5px]">{{ $t('update.homebrewCommand') }}</span>
            <div class="flex items-center justify-between px-3.5 py-2.5 bg-muted/50 border border-border rounded-[10px]">
              <code class="text-[13px] text-text flex-1 font-mono">brew upgrade atm</code>
              <button
                class="shrink-0 p-1.5 bg-muted/50 border border-border rounded-lg text-text-muted cursor-pointer transition-all duration-200 hover:bg-accent/15 hover:border-accent hover:text-accent"
                @click="copyCommand('brew upgrade atm')"
                :title="$t('update.copyCommand')"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>

          <div>
            <span class="block text-xs text-text-muted mb-2 font-semibold uppercase tracking-[0.5px]">{{ $t('update.scoopCommand') }}</span>
            <div class="flex items-center justify-between px-3.5 py-2.5 bg-muted/50 border border-border rounded-[10px]">
              <code class="text-[13px] text-text flex-1 font-mono">scoop update atm</code>
              <button
                class="shrink-0 p-1.5 bg-muted/50 border border-border rounded-lg text-text-muted cursor-pointer transition-all duration-200 hover:bg-accent/15 hover:border-accent hover:text-accent"
                @click="copyCommand('scoop update atm')"
                :title="$t('update.copyCommand')"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps({
  updateInfo: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close'])

const visible = ref(true)
const expanded = ref(false)

const close = () => {
  visible.value = false
  emit('close')
}

const toggleExpand = () => {
  expanded.value = !expanded.value
}

const openGitHubRelease = async () => {
  try {
    await invoke('open_internal_browser', {
      url: props.updateInfo.download_url,
      title: 'GitHub Release'
    })
  } catch (error) {
    console.error('Failed to open GitHub release:', error)
    window.$notify.error(`${t('update.checkFailed')}: ${error}`)
  }
}

const copyCommand = async (command) => {
  try {
    await navigator.clipboard.writeText(command)
    window.$notify.success(t('update.commandCopied'))
  } catch (error) {
    console.error('Failed to copy command:', error)
    window.$notify.error(`${t('messages.copyFailed')}: ${error}`)
  }
}
</script>

<style scoped>
/* 入场动画 - 必须保留 */
.update-banner {
  animation: slideIn 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* 展开内容动画 */
.expanded-content {
  animation: slideDown 0.3s ease;
}

@keyframes slideDown {
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
