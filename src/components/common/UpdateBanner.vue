<template>
  <div v-if="visible" class="update-banner" :class="{ expanded }">
    <div class="update-content">
      <!-- 简化视图 -->
      <div class="update-header" @click="toggleExpand">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="update-icon">
          <circle cx="12" cy="12" r="10"></circle>
          <polyline points="12 6 12 12 16 14"></polyline>
        </svg>
        <h3>{{ $t('update.newVersionAvailable') }}: v{{ updateInfo.latest_version }}</h3>
        <button class="modal-close" @click.stop="close" :aria-label="$t('update.close')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- 展开内容 -->
      <div v-if="expanded" class="expanded-content">
        <div class="version-info">
          <div class="version-item">
            <span class="version-label">{{ $t('update.currentVersion') }}:</span>
            <span class="version-value">v{{ updateInfo.current_version }}</span>
          </div>
          <div class="version-item">
            <span class="version-label">{{ $t('update.latestVersion') }}:</span>
            <span class="version-value highlight">v{{ updateInfo.latest_version }}</span>
          </div>
        </div>

        <div class="install-methods">
          <h4>{{ $t('update.installMethods') }}:</h4>

          <div class="method-item">
            <button class="method-btn" @click="openGitHubRelease">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              {{ $t('update.githubRelease') }}
            </button>
          </div>

          <div class="method-item">
            <span class="method-label">{{ $t('update.homebrewCommand') }}</span>
            <div class="command-box">
              <code>brew upgrade atm</code>
              <button class="copy-btn" @click="copyCommand('brew upgrade atm')" :title="$t('update.copyCommand')">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </button>
            </div>
          </div>

          <div class="method-item">
            <span class="method-label">{{ $t('update.scoopCommand') }}</span>
            <div class="command-box">
              <code>scoop update atm</code>
              <button class="copy-btn" @click="copyCommand('scoop update atm')" :title="$t('update.copyCommand')">
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
/* ============================================
   UpdateBanner - Modern Tech Style
   ============================================ */

.update-banner {
  position: fixed;
  bottom: 22px;
  left: 22px;
  max-width: 360px;
  width: auto;
  background: var(--tech-glass-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  box-shadow: 0 15px 35px rgba(0, 0, 0, 0.25), var(--tech-border-glow);
  z-index: 999;
  animation: slideIn 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.update-banner.expanded {
  width: 420px;
  max-width: 420px;
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

.update-content {
  padding: 22px;
  position: relative;
}

.update-banner:not(.expanded) .update-content {
  padding: 14px 18px;
}


.update-header {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  padding: 6px;
  margin: -6px;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.update-header:hover {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.update-icon {
  color: var(--accent);
  flex-shrink: 0;
  filter: drop-shadow(0 0 6px var(--tech-glow-primary));
}

.update-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  flex: 1;
  white-space: nowrap;
}

.expanded-content {
  margin-top: 18px;
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

/* 版本信息 - 科技风 */
.version-info {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 22px;
  padding: 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 12px;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.version-label {
  font-size: 14px;
  color: var(--text-muted);
}

.version-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
  font-family: var(--tech-mono-font);
}

.version-value.highlight {
  color: var(--accent);
  text-shadow: 0 0 8px var(--tech-glow-primary);
}

.install-methods h4 {
  margin: 0 0 14px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.method-item {
  margin-bottom: 18px;
}

.method-item:last-child {
  margin-bottom: 0;
}

.method-label {
  display: block;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 8px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* 方法按钮 - 科技风 */
.method-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 12px 16px;
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
  border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.method-btn:hover {
  background: color-mix(in srgb, var(--accent) 22%, transparent);
  border-color: color-mix(in srgb, var(--accent) 55%, transparent);
  transform: translateY(-2px);
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

/* 命令框 - 科技风 */
.command-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 10px;
}

.command-box code {
  font-family: var(--tech-mono-font);
  font-size: 13px;
  color: var(--text);
  flex: 1;
}

.copy-btn {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  cursor: pointer;
  padding: 6px;
  border-radius: 8px;
  color: var(--text-muted);
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.copy-btn:hover {
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: var(--accent);
  color: var(--accent);
}
</style>
