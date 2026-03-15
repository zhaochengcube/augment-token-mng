<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- Email List View -->
    <div v-if="!currentEmailManager" class="flex flex-col gap-8 p-[26px]">
      <div class="space-y-2">
        <h2 class="text-2xl font-semibold text-text">{{ $t('emails.title') }}</h2>
      </div>

      <div class="grid grid-cols-[repeat(auto-fill,minmax(130px,1fr))] gap-[18px] max-w-[520px]">
        <button
          v-for="service in availableServices"
          :key="service.id"
          type="button"
          class="card card--lift group relative flex aspect-square flex-col items-center justify-center gap-3 p-[18px] text-left cursor-pointer"
          @click="openManager(service.id)"
        >
          <span class="flex h-[52px] w-[52px] items-center justify-center text-accent transition-transform duration-300 group-hover:scale-110">
            <svg viewBox="0 0 24 24" fill="currentColor" class="h-full w-full">
              <path :d="service.icon" />
            </svg>
          </span>
          <span class="text-[15px] font-semibold text-text group-hover:text-accent text-center">{{ $t(service.nameKey) }}</span>
        </button>
      </div>
    </div>

    <!-- Email Manager Detail View -->
    <div v-else class="flex h-full flex-col min-h-0">
      <component :is="currentServiceComponent" @close="backToList" />
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import OutlookManager from '../email/OutlookManager.vue'
import GPTMailManager from '../email/GPTMailManager.vue'
import ICloudHMEManager from '../email/ICloudHMEManager.vue'

// Email service configuration
const services = [
  {
    id: 'icloud-hme',
    nameKey: 'hmeManager.title',
    icon: 'M13.762 4.29a6.51 6.51 0 0 0-5.669 3.332 3.571 3.571 0 0 0-1.558-.36 3.571 3.571 0 0 0-3.516 3A4.918 4.918 0 0 0 0 14.796a4.918 4.918 0 0 0 4.92 4.914 4.93 4.93 0 0 0 .617-.045h14.42c2.305-.272 4.041-2.258 4.043-4.589v-.009a4.594 4.594 0 0 0-3.727-4.508 6.51 6.51 0 0 0-6.511-6.27z',
    component: ICloudHMEManager,
    enabled: true
  },
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
    enabled: true
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

defineExpose({ clearSelection: backToList })
</script>
