<template>
  <div class="platform-selector">
    <div class="page-header">
      <h2>{{ $t('platforms.title') }}</h2>
      <p>{{ $t('platforms.description') }}</p>
    </div>

    <div class="platform-grid">
      <div
        v-for="platform in platforms"
        :key="platform.id"
        class="platform-card"
        :class="{ disabled: platform.status === 'coming_soon' }"
        @click="handleSelectPlatform(platform)"
      >
        <div class="platform-icon">
          <img v-if="platform.icon.endsWith('.svg') || platform.icon.endsWith('.png')" :src="platform.icon" :alt="platform.name" />
          <div v-else class="icon-placeholder">{{ platform.icon }}</div>
        </div>
        <h3 class="platform-name">{{ platform.name }}</h3>
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  platforms: {
    type: Array,
    required: true
  }
})

const emit = defineEmits(['select'])

const handleSelectPlatform = (platform) => {
  if (platform.status === 'ready') {
    emit('select', platform.id)
  }
}
</script>

<style scoped>
.platform-selector {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

.page-header {
  margin-bottom: 24px;
}

.page-header h2 {
  font-size: 28px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0 0 8px 0;
}

.page-header p {
  font-size: 16px;
  color: var(--text-muted);
  margin: 0;
}

.platform-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 16px;
  max-width: 500px;
}

.platform-card {
  background: var(--bg-surface);
  border: 2px solid var(--border);
  border-radius: 10px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  aspect-ratio: 1;
}

.platform-card:not(.disabled):hover {
  border-color: var(--accent);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
  transform: translateY(-2px);
}

.platform-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.platform-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.platform-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.icon-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  font-weight: 700;
  background: var(--bg-muted);
  border-radius: 6px;
  color: var(--accent);
}

.platform-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
  text-align: center;
}
</style>

