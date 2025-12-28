<template>
  <div class="email-page">
    <!-- Email List View -->
    <div v-if="!currentEmailManager" class="email-list-view">
      <div class="page-header">
        <h2>{{ $t('emails.title') }}</h2>
      </div>

      <div class="email-grid">
        <div 
          v-for="service in availableServices" 
          :key="service.id"
          class="email-card" 
          @click="openManager(service.id)"
        >
          <div class="email-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path :d="service.icon" />
            </svg>
          </div>
          <h3 class="email-name">{{ $t(service.nameKey) }}</h3>
        </div>
      </div>
    </div>

    <!-- Email Manager Detail View -->
    <div v-else class="email-detail-view">
      <component 
        :is="currentServiceComponent" 
        @close="backToList" 
        :is-page-mode="true" 
      />
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import OutlookManager from '../email/OutlookManager.vue'
import GPTMailManager from '../email/GPTMailManager.vue'

// Email service configuration
const services = [
  {
    id: 'outlook',
    nameKey: 'outlookManager.title',
    icon: 'M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z',
    component: OutlookManager,
    enabled: false
  },
  {
    id: 'gptmail',
    nameKey: 'gptMailManager.title',
    icon: 'M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z',
    component: GPTMailManager,
    enabled: false
  }
]

const currentEmailManager = ref(null)

const availableServices = computed(() => {
  return services.filter(service => service.enabled)
})

const currentServiceComponent = computed(() => {
  const service = services.find(s => s.id === currentEmailManager.value)
  return service ? service.component : null
})

const openManager = (id) => {
  currentEmailManager.value = id
}

const backToList = () => {
  currentEmailManager.value = null
}
</script>

<style scoped>
/* ============================================
   EmailPage - Modern Tech Style
   ============================================ */

.email-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.email-list-view {
  max-width: 1200px;
  margin: 0 auto;
  padding: 26px;
  width: 100%;
}

.email-detail-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.page-header {
  margin-bottom: 28px;
  flex-shrink: 0;
}

.page-header h2 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-strong);
  margin: 0;
  letter-spacing: -0.5px;
}

.email-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 18px;
  max-width: 520px;
}

/* 邮箱卡片 - 科技风 */
.email-card {
  position: relative;
  background: var(--tech-card-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  padding: 18px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  aspect-ratio: 1;
  overflow: hidden;
}

/* 顶部发光边线 */
.email-card::before {
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

.email-card:hover::before {
  opacity: 1;
  left: 10%;
  right: 10%;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

.email-card:hover {
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 30%, transparent),
              0 8px 32px -8px rgba(0, 0, 0, 0.2),
              0 0 30px -10px var(--tech-glow-primary);
  transform: translateY(-4px);
}

.email-icon {
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--accent);
  transition: transform 0.3s ease;
}

.email-card:hover .email-icon {
  transform: scale(1.1);
}

.email-icon svg {
  width: 100%;
  height: 100%;
  object-fit: contain;
  filter: drop-shadow(0 0 8px var(--tech-glow-primary));
}

.email-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
  text-align: center;
}
</style>

