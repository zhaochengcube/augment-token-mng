<template>
  <BaseModal
    :visible="true"
    :title="$t('tokenCard.suspensionDetails')"
    :close-on-overlay="true"
    :body-scroll="true"
    modal-class="!max-w-[600px]"
    @close="$emit('close')"
  >
    <!-- Suspensions List -->
    <div v-if="formattedSuspensions.length > 0" class="flex flex-col gap-3">
      <div 
        v-for="(suspension, index) in formattedSuspensions" 
        :key="index" 
        class="card hover:translate-y-0 p-4"
      >
        <div class="flex flex-col gap-2">
          <div class="flex items-start gap-2">
            <strong class="text-[13px] text-text-muted shrink-0">{{ $t('tokenCard.suspensionType') }}:</strong>
            <span class="text-[13px] text-text break-all">{{ suspension.type }}</span>
          </div>
          <div v-if="suspension.reason" class="flex items-start gap-2">
            <strong class="text-[13px] text-text-muted shrink-0">{{ $t('tokenCard.reason') }}:</strong>
            <span class="text-[13px] text-text break-all">{{ suspension.reason }}</span>
          </div>
          <div v-if="suspension.date" class="flex items-start gap-2">
            <strong class="text-[13px] text-text-muted shrink-0">{{ $t('tokenCard.date') }}:</strong>
            <span class="text-[13px] text-text break-all">{{ suspension.date }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- No Data -->
    <div v-else class="py-8 text-center text-text-muted text-[14px]">
      <p class="m-0">{{ $t('tokenCard.noSuspensionData') }}</p>
    </div>

    <!-- Raw JSON Data -->
    <details class="mt-4 rounded-lg border border-border bg-muted" open>
      <summary class="px-4 py-3 text-[13px] font-medium text-text-secondary cursor-pointer select-none hover:text-text">
        {{ $t('tokenCard.rawData') }}
      </summary>
      <pre class="m-0 p-4 pt-0 text-[12px] font-mono text-text-muted overflow-x-auto whitespace-pre-wrap break-all">{{ JSON.stringify(suspensions, null, 2) }}</pre>
    </details>
  </BaseModal>
</template>

<script setup>
import { computed } from 'vue'
import BaseModal from '../common/BaseModal.vue'

const props = defineProps({
  suspensions: {
    type: [Array, Object],
    default: null
  }
})

defineEmits(['close'])

const formattedSuspensions = computed(() => {
  if (!props.suspensions) return []
  
  // 如果是数组，直接处理
  if (Array.isArray(props.suspensions)) {
    return props.suspensions.map(s => ({
      type: s.suspension_type || s.type || 'Unknown',
      reason: s.reason || s.message || null,
      date: s.suspended_at || s.date || null
    }))
  }
  
  // 如果是对象，包装成数组
  if (typeof props.suspensions === 'object') {
    return [{
      type: props.suspensions.suspension_type || props.suspensions.type || 'Unknown',
      reason: props.suspensions.reason || props.suspensions.message || null,
      date: props.suspensions.suspended_at || props.suspensions.date || null
    }]
  }
  
  return []
})
</script>

