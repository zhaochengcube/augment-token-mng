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
  emit('select', platform.id)
}
</script>

<style scoped>
/* ============================================
   PlatformSelector - Modern Tech Style
   ============================================ */

.platform-selector {
  max-width: 1200px;
  margin: 0 auto;
  padding: 32px;
}

.page-header {
  margin-bottom: 32px;
}

.page-header h2 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-strong);
  margin: 0 0 10px 0;
  letter-spacing: -0.5px;
}

.page-header p {
  font-size: 15px;
  color: var(--text-muted);
  margin: 0;
  opacity: 0.8;
}

.platform-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 20px;
  max-width: 550px;
}

/* 平台卡片 - 科技风 */
.platform-card {
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  aspect-ratio: 1;
  position: relative;
  overflow: hidden;
}

/* 顶部发光边线 */
.platform-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 20%;
  right: 20%;
  height: 2px;
  background: var(--accent);
  opacity: 0;
  transition: all 0.3s ease;
  border-radius: 0 0 2px 2px;
}

.platform-card:hover::before {
  opacity: 1;
  left: 10%;
  right: 10%;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.platform-card:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 30%, transparent),
              0 8px 32px -8px rgba(0, 0, 0, 0.2),
              0 0 30px -10px var(--tech-glow-primary);
  transform: translateY(-4px);
}

.platform-icon {
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: transform 0.3s ease;
}

.platform-card:hover .platform-icon {
  transform: scale(1.1);
}

.platform-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.1));
}

.icon-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  font-weight: 800;
  background: var(--accent);
  border-radius: 12px;
  color: #fff;
  box-shadow: 0 4px 15px var(--tech-glow-primary);
}

.platform-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
  text-align: center;
  transition: color 0.2s ease;
}

.platform-card:hover .platform-name {
  color: var(--accent);
}
</style>

