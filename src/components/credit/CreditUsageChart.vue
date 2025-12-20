<template>
  <div class="credit-chart-container">
    <div class="chart-header">
      <h3>{{ $t('credit.modelDistribution') }}</h3>
    </div>

    <div v-if="loading && !chartData" class="loading">
      <div class="spinner"></div>
      <p>{{ $t('credit.loading') }}</p>
    </div>

    <div v-else-if="error" class="error">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <p>{{ error }}</p>
    </div>

    <div v-else-if="!chartData || !chartData.data_points || chartData.data_points.length === 0" class="no-data">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
      </svg>
      <p>{{ $t('credit.noData') }}</p>
    </div>

    <div v-else class="chart-wrapper">
      <Pie :key="chartKey" :data="pieChartData" :options="chartOptions" />
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { Pie } from 'vue-chartjs'
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js'

// 注册 Chart.js 组件
ChartJS.register(ArcElement, Tooltip, Legend)

// 用于强制重新渲染图表
const chartKey = ref(0)
const currentTheme = ref('light')

const props = defineProps({
  loading: {
    type: Boolean,
    default: false
  },
  error: {
    type: String,
    default: null
  },
  chartData: {
    type: Object,
    default: null
  }
})

const readTheme = () => {
  if (typeof document === 'undefined') {
    return 'light'
  }
  return document.documentElement.dataset.theme || document.documentElement.getAttribute('data-theme') || 'light'
}

const resolveCssVar = (name, fallback) => {
  if (typeof document === 'undefined') {
    return fallback
  }
  const value = getComputedStyle(document.documentElement).getPropertyValue(name)?.trim()
  return value || fallback
}

const themePalette = computed(() => {
  const theme = currentTheme.value
  const isDark = theme === 'dark'

  return {
    isDark,
    surface: resolveCssVar('--color-surface', isDark ? '#0f172a' : '#ffffff'),
    legendColor: resolveCssVar('--color-text-primary', isDark ? '#e2e8f0' : '#374151'),
    tooltipBg: resolveCssVar('--color-surface-alt', isDark ? '#111827' : '#ffffff'),
    tooltipTitle: resolveCssVar('--color-text-heading', isDark ? '#f9fafb' : '#1f2937'),
    tooltipBody: resolveCssVar('--color-text-primary', isDark ? '#e2e8f0' : '#374151'),
    tooltipBorder: resolveCssVar('--color-border-strong', isDark ? 'rgba(148, 163, 184, 0.45)' : '#e5e7eb')
  }
})

// 图表数据
const pieChartData = computed(() => {
  if (!props.chartData || !props.chartData.data_points || props.chartData.data_points.length === 0) {
    return { labels: [], datasets: [] }
  }

  const labels = props.chartData.data_points.map(p => p.group_key || 'Unknown')
  const data = props.chartData.data_points.map(p => parseInt(p.credits_consumed) || 0)
  const palette = themePalette.value

  const lightColors = ['#4c6ef5', '#f783ac', '#38bdf8', '#43e97b', '#ff922b', '#845ef7', '#12b886', '#fd7e14']
  const darkColors = ['#c7d2fe', '#f9a8d4', '#7dd3fc', '#6ee7b7', '#facc15', '#fca5a5', '#fcd34d', '#a5b4fc']

  return {
    labels,
    datasets: [{
      data,
      backgroundColor: palette.isDark ? darkColors : lightColors,
      borderWidth: 2,
      borderColor: palette.surface,
      hoverOffset: 14
    }]
  }
})

// 图表配置
const chartOptions = computed(() => {
  const palette = themePalette.value

  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'bottom',
        labels: {
          padding: 15,
          font: {
            size: 12,
            family: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
          },
          color: palette.legendColor,
          usePointStyle: true,
          pointStyle: 'circle'
        }
      },
      tooltip: {
        backgroundColor: palette.tooltipBg,
        titleColor: palette.tooltipTitle,
        bodyColor: palette.tooltipBody,
        borderColor: palette.tooltipBorder,
        borderWidth: 1,
        padding: 12,
        boxPadding: 6,
        usePointStyle: true,
        callbacks: {
          label: (context) => {
            const label = context.label || ''
            const value = context.parsed || 0
            const total = context.dataset.data.reduce((a, b) => a + b, 0)
            const percentage = total > 0 ? ((value / total) * 100).toFixed(1) : '0.0'
            return `${label}: ${value} credits (${percentage}%)`
          }
        }
      }
    }
  }
})

const updateTheme = () => {
  currentTheme.value = readTheme()
}

let observer

onMounted(() => {
  updateTheme()
  if (typeof document === 'undefined') {
    return
  }
  observer = new MutationObserver(updateTheme)
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme']
  })
})

onUnmounted(() => {
  observer?.disconnect()
})

watch(currentTheme, () => {
  chartKey.value++
})
</script>

<style scoped>
.credit-chart-container {
  background: linear-gradient(135deg, var(--bg-surface) 0%, var(--bg-muted) 100%);
  border-radius: 12px;
  padding: 20px;
  border: 1px solid var(--border);
  height: 380px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  flex-shrink: 0;
}

.chart-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
}



.chart-wrapper {
  flex: 1;
  min-height: 0;
  position: relative;
}

.loading, .error, .no-data {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-muted);
}

.error {
  color: #ef4444;
}

.error svg {
  color: #ef4444;
  opacity: 0.6;
}

.no-data svg {
  opacity: 0.4;
}

.loading p, .error p, .no-data p {
  margin: 0;
  font-size: 14px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .credit-chart-container {
    height: 320px;
    padding: 16px;
  }

  .chart-header h3 {
    font-size: 14px;
  }
}
</style>
