<template>
  <BaseModal
    :visible="true"
    :title="$t('platform.cursor.machineIdOption.title')"
    :show-close="true"
    :close-on-overlay="true"
    :close-on-esc="true"
    :body-scroll="false"
    modal-class="max-w-[450px]"
    @close="handleClose"
  >
    <div class="animate-fade-in">
      <!-- 提示信息 -->
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-accent/20 bg-accent/10 p-4">
        <svg class="mt-0.5 h-5 w-5 shrink-0 text-accent" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
        </svg>
        <div class="text-[13px] leading-relaxed text-text-secondary">
          <p>{{ $t('platform.cursor.machineIdOption.info') }}</p>
          <p class="mt-2 text-xs text-text-muted">{{ account.email }}</p>
        </div>
      </div>

      <!-- 选项 -->
      <div class="space-y-3">
        <!-- 使用绑定的机器码 -->
        <div
          class="flex items-start gap-3 rounded-lg border p-4 cursor-pointer transition-all"
          :class="selectedOption === 'bound' ? 'border-accent bg-accent/5' : 'border-border hover:border-accent/50'"
          @click="selectedOption = 'bound'"
        >
          <div
            class="mt-0.5 h-5 w-5 shrink-0 rounded-full border-2 flex items-center justify-center transition-colors"
            :class="selectedOption === 'bound' ? 'border-accent' : 'border-border'"
          >
            <div
              v-if="selectedOption === 'bound'"
              class="h-2.5 w-2.5 rounded-full bg-accent"
            ></div>
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium text-text">{{ $t('platform.cursor.machineIdOption.useBound') }}</div>
            <p class="mt-1 text-xs text-text-muted">{{ $t('platform.cursor.machineIdOption.useBoundDesc') }}</p>
            <!-- 机器码预览 -->
            <div class="mt-2 rounded bg-surface-alt p-2 text-[11px] text-text-muted font-mono overflow-hidden">
              <div class="truncate">machineId: {{ truncateId(machineInfo?.['telemetry.machineId']) }}</div>
              <div v-if="machineInfo?.['telemetry.devDeviceId']" class="truncate">devDeviceId: {{ truncateId(machineInfo?.['telemetry.devDeviceId']) }}</div>
            </div>
          </div>
        </div>

        <!-- 随机生成新机器码 -->
        <div
          class="flex items-start gap-3 rounded-lg border p-4 cursor-pointer transition-all"
          :class="selectedOption === 'random' ? 'border-accent bg-accent/5' : 'border-border hover:border-accent/50'"
          @click="selectedOption = 'random'"
        >
          <div
            class="mt-0.5 h-5 w-5 shrink-0 rounded-full border-2 flex items-center justify-center transition-colors"
            :class="selectedOption === 'random' ? 'border-accent' : 'border-border'"
          >
            <div
              v-if="selectedOption === 'random'"
              class="h-2.5 w-2.5 rounded-full bg-accent"
            ></div>
          </div>
          <div class="flex-1">
            <div class="text-sm font-medium text-text">{{ $t('platform.cursor.machineIdOption.useRandom') }}</div>
            <p class="mt-1 text-xs text-text-muted">{{ $t('platform.cursor.machineIdOption.useRandomDesc') }}</p>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <button @click="handleClose" class="btn btn--secondary">
        {{ $t('common.cancel') }}
      </button>
      <button
        @click="handleConfirm"
        class="btn btn--primary"
      >
        {{ $t('platform.cursor.machineIdOption.confirm') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'

const { t: $t } = useI18n()

const props = defineProps({
  account: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close', 'confirm'])

const selectedOption = ref('bound') // 'bound' | 'random'

const machineInfo = computed(() => props.account.machine_info)

const truncateId = (id) => {
  if (!id) return '-'
  if (id.length <= 20) return id
  return id.substring(0, 8) + '...' + id.substring(id.length - 8)
}

const handleClose = () => {
  emit('close')
}

const handleConfirm = () => {
  emit('confirm', selectedOption.value === 'bound')
}
</script>

<style scoped>
.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
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

















