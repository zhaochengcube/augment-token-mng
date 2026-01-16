<template>
  <div class="flex h-full flex-col bg-page text-text">
    <div v-if="!activePlatform" class="flex flex-col gap-8" style="padding: 3rem;">
      <div class="space-y-2">
        <h2 class="text-2xl font-semibold text-text">{{ $t('platforms.title') }}</h2>
        <p class="text-sm text-text-muted">{{ $t('platforms.description') }}</p>
      </div>

      <div class="grid max-w-3xl grid-cols-5 gap-4">
        <button
          v-for="platform in availablePlatforms"
          :key="platform.id"
          type="button"
          class="card card--lift group relative flex aspect-square flex-col items-center justify-center gap-2 p-3 text-left"
          @click="handleSelectPlatform(platform)"
        >
          <span class="flex h-10 w-10 items-center justify-center">
            <img
              v-if="platform.icon.endsWith('.svg') || platform.icon.endsWith('.png')"
              :src="platform.icon"
              :alt="platform.name"
              class="h-10 w-10 object-contain transition group-hover:scale-105"
            />
            <span
              v-else
              class="flex h-10 w-10 items-center justify-center rounded-lg bg-accent text-base font-bold text-white shadow-sm"
            >
              {{ platform.icon }}
            </span>
          </span>
          <span class="text-sm font-semibold text-text group-hover:text-accent">{{ platform.name }}</span>
        </button>
      </div>
    </div>

    <div v-else class="flex h-full flex-col min-h-0">
      <div class="flex-1 min-h-0 overflow-hidden">
        <component
          v-if="activePlatform.component"
          :is="activePlatform.component"
          :is-sidebar-collapsed="isSidebarCollapsed"
        />
        <div v-else class="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
          <h2 class="text-xl font-semibold text-text">{{ activePlatform.name || $t('messages.comingSoon') }}</h2>
          <p class="text-sm text-text-muted">{{ $t('messages.comingSoon') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import AugmentTokenManager from '../platform/AugmentTokenManager.vue'
import AntigravityAccountManager from '../platform/AntigravityAccountManager.vue'

const props = defineProps({
  isDarkTheme: {
    type: Boolean,
    default: false
  },
  isSidebarCollapsed: {
    type: Boolean,
    default: false
  }
})

const platforms = computed(() => [
  {
    id: 'augment',
    name: 'Augment',
    icon: props.isDarkTheme ? '/icons/auggie_dark.svg' : '/icons/auggie.svg',
    component: AugmentTokenManager,
    enabled: true
  },
  {
    id: 'antigravity',
    name: 'Antigravity',
    icon: '/icons/antigravity.png',
    component: AntigravityAccountManager,
    enabled: true
  },
  {
    id: 'windsurf',
    name: 'Windsurf',
    icon: '/icons/windsurf.svg',
    component: null,
    enabled: false
  }
])

const availablePlatforms = computed(() => platforms.value.filter((platform) => platform.enabled))

const activePlatformId = ref(null)
const activePlatform = computed(() =>
  platforms.value.find((platform) => platform.id === activePlatformId.value)
)

const handleSelectPlatform = (platform) => {
  activePlatformId.value = platform.id
}

const clearSelection = () => {
  activePlatformId.value = null
}

const selectPlatformById = (platformId) => {
  const targetPlatform = platforms.value.find(
    (platform) => platform.id === platformId && platform.enabled
  )
  if (!targetPlatform) {
    return null
  }
  activePlatformId.value = targetPlatform.id
  return targetPlatform
}

defineExpose({ selectPlatformById, clearSelection })
</script>
