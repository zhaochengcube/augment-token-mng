<template>
  <div class="email-page">
    <!-- Email List View -->
    <div v-if="!currentEmailManager" class="email-list-view">
      <div class="page-header">
        <h2>{{ $t('emails.title') }}</h2>
      </div>

      <div class="email-grid">
        <!-- Outlook Manager Card -->
        <div class="email-card" @click="openOutlookManager">
          <div class="email-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
            </svg>
          </div>
          <h3 class="email-name">{{ $t('outlookManager.title') }}</h3>
        </div>

        <!-- GPTMail Manager Card -->
        <div class="email-card" @click="openGPTMailManager">
          <div class="email-icon">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z"/>
            </svg>
          </div>
          <h3 class="email-name">{{ $t('gptMailManager.title') }}</h3>
        </div>
      </div>
    </div>

    <!-- Email Manager Detail View -->
    <div v-else class="email-detail-view">
      <OutlookManager v-if="currentEmailManager === 'outlook'" @close="backToList" :is-page-mode="true" />
      <GPTMailManager v-if="currentEmailManager === 'gptmail'" @close="backToList" :is-page-mode="true" />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import OutlookManager from '../email/OutlookManager.vue'
import GPTMailManager from '../email/GPTMailManager.vue'

const currentEmailManager = ref(null)

const openOutlookManager = () => {
  currentEmailManager.value = 'outlook'
}

const openGPTMailManager = () => {
  currentEmailManager.value = 'gptmail'
}

const backToList = () => {
  currentEmailManager.value = null
}
</script>

<style scoped>
.email-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.email-list-view {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
  width: 100%;
}

.email-detail-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.page-header {
  margin-bottom: 24px;
  flex-shrink: 0;
}

.page-header h2 {
  font-size: 28px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
}

.email-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 16px;
  max-width: 500px;
}

.email-card {
  position: relative;
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

.email-card:hover {
  border-color: var(--accent);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.15);
  transform: translateY(-2px);
}

.email-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--primary);
}

.email-icon svg {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.email-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
  text-align: center;
}
</style>

