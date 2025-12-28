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
    surface: resolveCssVar('--bg-surface', isDark ? '#0f172a' : '#ffffff'),
    legendColor: resolveCssVar('--text', isDark ? '#e2e8f0' : '#374151'),
    tooltipBg: resolveCssVar('--bg-surface-alt', isDark ? '#111827' : '#ffffff'),
    tooltipTitle: resolveCssVar('--text-strong', isDark ? '#f9fafb' : '#1f2937'),
    tooltipBody: resolveCssVar('--text', isDark ? '#e2e8f0' : '#374151'),
    tooltipBorder: resolveCssVar('--border-strong', isDark ? 'rgba(148, 163, 184, 0.45)' : '#e5e7eb')
  }
})

const chartColors = computed(() => ([
  resolveCssVar('--chart-1', '#4c6ef5'),
  resolveCssVar('--chart-2', '#f783ac'),
  resolveCssVar('--chart-3', '#38bdf8'),
  resolveCssVar('--chart-4', '#43e97b'),
  resolveCssVar('--chart-5', '#ff922b'),
  resolveCssVar('--chart-6', '#845ef7'),
  resolveCssVar('--chart-7', '#12b886'),
  resolveCssVar('--chart-8', '#fd7e14')
]))

// 图表数据
const pieChartData = computed(() => {
  if (!props.chartData || !props.chartData.data_points || props.chartData.data_points.length === 0) {
    return { labels: [], datasets: [] }
  }

  const labels = props.chartData.data_points.map(p => p.group_key || 'Unknown')
  const data = props.chartData.data_points.map(p => parseInt(p.credits_consumed) || 0)
  const palette = themePalette.value

  return {
    labels,
    datasets: [{
      data,
      backgroundColor: chartColors.value,
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
/* ============================================
   CreditUsageChart - Modern Tech Style
   ============================================ */

.credit-chart-container {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  padding: 22px;
  height: 400px;
  display: flex;
  flex-direction: column;
  box-shadow: var(--tech-border-glow);
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 18px;
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
  gap: 14px;
  color: var(--text-muted);
}

.error {
  color: #ef4444;
}

.error svg {
  color: #ef4444;
  opacity: 0.7;
  filter: drop-shadow(0 0 6px var(--tech-glow-danger));
}

.no-data svg {
  opacity: 0.5;
  color: var(--text-muted);
}

.loading p, .error p, .no-data p {
  margin: 0;
  font-size: 14px;
}

/* 加载动画 - 科技风 */
.spinner {
  width: 44px;
  height: 44px;
  border: 3px solid var(--tech-glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  box-shadow: 0 0 12px var(--tech-glow-primary);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .credit-chart-container {
    height: 340px;
    padding: 18px;
  }

  .chart-header h3 {
    font-size: 14px;
  }
}
</style>
