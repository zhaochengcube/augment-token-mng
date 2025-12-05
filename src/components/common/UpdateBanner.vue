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
        <button class="close-btn" @click.stop="close" :aria-label="$t('update.close')">
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
.update-banner {
  position: fixed;
  bottom: 20px;
  left: 20px;
  max-width: 350px;
  width: auto;
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 12px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
  z-index: 999;
  animation: slideIn 0.3s ease;
}

.update-banner.expanded {
  width: 400px;
  max-width: 400px;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.update-content {
  padding: 20px;
  position: relative;
}

.update-banner:not(.expanded) .update-content {
  padding: 12px 16px;
}

.close-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  color: var(--color-text-muted, #6b7280);
  transition: all 0.2s;
  margin-left: auto;
  flex-shrink: 0;
}

.close-btn:hover {
  background: var(--color-surface-hover, #f3f4f6);
  color: var(--color-text-primary, #374151);
}

.update-header {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  padding: 4px;
  margin: -4px;
  border-radius: 6px;
  transition: background 0.2s;
}

.update-header:hover {
  background: var(--color-surface-hover, #f3f4f6);
}

.update-icon {
  color: var(--color-primary, #3b82f6);
  flex-shrink: 0;
}

.update-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
  flex: 1;
  white-space: nowrap;
}

.expanded-content {
  margin-top: 16px;
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

.version-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
  padding: 12px;
  background: var(--color-surface-secondary, #f9fafb);
  border-radius: 8px;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.version-label {
  font-size: 14px;
  color: var(--color-text-secondary, #6b7280);
}

.version-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
  font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
}

.version-value.highlight {
  color: var(--color-primary, #3b82f6);
}

.install-methods h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, #374151);
}

.method-item {
  margin-bottom: 16px;
}

.method-item:last-child {
  margin-bottom: 0;
}

.method-label {
  display: block;
  font-size: 12px;
  color: var(--color-text-muted, #9ca3af);
  margin-bottom: 6px;
  font-weight: 500;
}

.method-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  background: var(--color-blue-soft-bg, #e3f2fd);
  color: var(--color-blue-soft-text, #1976d2);
  border: 1px solid var(--color-blue-soft-border, #90caf9);
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.method-btn:hover {
  background: var(--color-blue-soft-bg, #bbdefb);
  border-color: var(--color-blue-soft-hover, #64b5f6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(25, 118, 210, 0.4);
}

.command-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--color-surface-secondary, #f3f4f6);
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 6px;
}

.command-box code {
  font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
  font-size: 13px;
  color: var(--color-text-primary, #374151);
  flex: 1;
}

.copy-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  color: var(--color-text-muted, #6b7280);
  transition: all 0.2s;
  flex-shrink: 0;
}

.copy-btn:hover {
  background: var(--color-surface-hover, #e5e7eb);
  color: var(--color-text-primary, #374151);
}

/* Dark mode */
[data-theme='dark'] .update-banner {
  background: var(--color-surface, #1e293b);
  border-color: rgba(148, 163, 184, 0.2);
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
}

[data-theme='dark'] .update-header:hover {
  background: rgba(148, 163, 184, 0.05);
}

[data-theme='dark'] .close-btn:hover {
  background: rgba(148, 163, 184, 0.1);
}

[data-theme='dark'] .version-info {
  background: rgba(148, 163, 184, 0.05);
}

[data-theme='dark'] .command-box {
  background: #0f172a;
}

[data-theme='dark'] .method-btn {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fd;
  border: 1px solid rgba(147, 197, 253, 0.4);
}

[data-theme='dark'] .method-btn:hover {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(96, 165, 250, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(59, 130, 246, 0.4);
}
</style>

