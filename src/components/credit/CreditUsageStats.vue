<template>
  <div class="credit-stats-container">
    <div class="stats-header">
      <h3>{{ $t('credit.usageStats') }}</h3>
    </div>

    <div v-if="loading && !statsData" class="loading">
      <div class="spinner"></div>
      <p>{{ $t('credit.loading') }}</p>
    </div>

    <div v-else-if="error" class="error">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <p>{{ error }}</p>
    </div>

    <div v-else-if="!statsData || !statsData.data_points || statsData.data_points.length === 0" class="no-data">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
        <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
      </svg>
      <p>{{ $t('credit.noData') }}</p>
    </div>

    <div v-else class="stats-content">
      <!-- 今日消耗卡片 -->
      <div class="stat-card">
        <div class="stat-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3.5 18.49l6-6.01 4 4L22 6.92l-1.41-1.41-7.09 7.97-4-4L2 16.99z"/>
          </svg>
        </div>
        <div class="stat-content">
          <div class="stat-metrics">
            <div class="metric-block">
              <div class="metric-label">{{ $t('credit.today') }}</div>
              <div class="metric-value">{{ todayCreditsDisplay }}</div>
              <div class="metric-unit">{{ $t('credit.credits') }}</div>
            </div>
            <div class="metric-divider" aria-hidden="true"></div>
            <div class="metric-block">
              <div class="metric-label">{{ $t('credit.total') }}</div>
              <div class="metric-value">{{ totalCreditsDisplay }}</div>
              <div class="metric-unit">{{ $t('credit.credits') }}</div>
            </div>
            <div class="metric-divider" aria-hidden="true"></div>
            <div class="metric-block">
              <div class="metric-label">{{ $t('credit.remaining') }}</div>
              <div class="metric-value">{{ remainingCreditsDisplay }}</div>
              <div class="metric-unit">{{ $t('credit.credits') }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 本周期趋势图 -->
      <div class="chart-card">
        <div class="chart-title">{{ $t('credit.cycleTrend') }}</div>
        <div class="chart-wrapper">
          <Line :key="chartKey" :data="lineChartData" :options="lineChartOptions" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { Line } from 'vue-chartjs'
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler } from 'chart.js'

// 注册 Chart.js 组件
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler)

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
  statsData: {
    type: Object,
    default: null
  },
  remainingCredits: {
    type: [Number, String],
    default: null
  }
})

const todayCredits = computed(() => {
  if (!props.statsData?.data_points?.length) return '0'
  // 获取最新的一天数据
  const latestPoint = props.statsData.data_points[props.statsData.data_points.length - 1]
  return latestPoint?.credits_consumed || '0'
})

const totalCredits = computed(() => {
  if (!props.statsData?.data_points?.length) return '0'
  // 计算所有数据点的总消耗
  const total = props.statsData.data_points.reduce((sum, point) => {
    const consumed = parseInt(point.credits_consumed) || 0
    return sum + consumed
  }, 0)
  return total.toString()
})

const formatCredits = (value) => {
  if (value === null || value === undefined || value === '') {
    return '--'
  }
  const numeric = Number(value)
  if (Number.isNaN(numeric)) {
    return `${value}`
  }
  return numeric.toLocaleString()
}

const todayCreditsDisplay = computed(() => formatCredits(todayCredits.value))

const totalCreditsDisplay = computed(() => formatCredits(totalCredits.value))

const remainingCreditsDisplay = computed(() => formatCredits(props.remainingCredits))

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

  const primary = resolveCssVar('--color-btn-primary-bg', isDark ? '#a5b4fc' : '#667eea')
  const primaryHover = resolveCssVar('--color-btn-primary-bg-hover', isDark ? '#c7d2fe' : '#5a6fd8')
  const surface = resolveCssVar('--color-surface', isDark ? '#0f172a' : '#ffffff')
  const tooltipSurface = resolveCssVar('--color-surface-alt', isDark ? '#111827' : '#ffffff')

  return {
    isDark,
    primary,
    primaryHover,
    surface,
    tooltipBg: tooltipSurface,
    tooltipTitle: resolveCssVar('--color-text-heading', isDark ? '#f9fafb' : '#1f2937'),
    tooltipBody: resolveCssVar('--color-text-primary', isDark ? '#e2e8f0' : '#374151'),
    tooltipBorder: resolveCssVar('--color-border-strong', isDark ? 'rgba(148, 163, 184, 0.45)' : '#e5e7eb'),
    tickColor: resolveCssVar('--color-text-muted', isDark ? '#cbd5f5' : '#6b7280'),
    gridColor: resolveCssVar('--color-border-muted', isDark ? 'rgba(148, 163, 184, 0.3)' : '#e5e7eb'),
    pointBorder: surface
  }
})

const withAlpha = (color, alpha, fallback = '#667eea') => {
  const base = color || fallback
  if (base.startsWith('#')) {
    const hex = base.slice(1)
    const normalize = hex.length === 3
      ? hex.split('').map(ch => ch + ch).join('')
      : hex
    const intVal = parseInt(normalize, 16)
    const r = (intVal >> 16) & 255
    const g = (intVal >> 8) & 255
    const b = intVal & 255
    return `rgba(${r}, ${g}, ${b}, ${alpha})`
  }
  if (base.startsWith('rgb')) {
    const values = base.replace(/rgba?\(/, '').replace(')', '').split(',').map(part => parseFloat(part.trim()))
    if (values.length >= 3) {
      const [r, g, b] = values
      return `rgba(${r}, ${g}, ${b}, ${alpha})`
    }
  }
  return fallback
}

// 折线图数据
const lineChartData = computed(() => {
  if (!props.statsData?.data_points?.length) {
    return { labels: [], datasets: [] }
  }

  const labels = props.statsData.data_points.map(point => {
    if (!point.date_range?.start_date_iso) return ''
    const date = new Date(point.date_range.start_date_iso)
    return `${date.getMonth() + 1}/${date.getDate()}`
  })

  const data = props.statsData.data_points.map(point => parseInt(point.credits_consumed) || 0)

  const palette = themePalette.value

  return {
    labels,
    datasets: [{
      label: 'Credits',
      data,
      borderColor: palette.primary,
      backgroundColor: withAlpha(palette.primary, palette.isDark ? 0.24 : 0.12),
      borderWidth: 3,
      fill: true,
      tension: 0.4,
      pointRadius: 5,
      pointHoverRadius: 7,
      pointBackgroundColor: palette.primaryHover,
      pointBorderColor: palette.pointBorder,
      pointBorderWidth: 2,
      pointHoverBackgroundColor: palette.primary,
      pointHoverBorderColor: palette.pointBorder,
      pointHoverBorderWidth: 3
    }]
  }
})

// 折线图配置
const lineChartOptions = computed(() => {
  const palette = themePalette.value

  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        display: false
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
            return `${context.parsed.y} credits`
          }
        }
      }
    },
    scales: {
      x: {
        grid: {
          display: false
        },
        ticks: {
          color: palette.tickColor,
          font: {
            size: 11
          }
        }
      },
      y: {
        beginAtZero: true,
        grid: {
          color: palette.gridColor,
          drawBorder: false
        },
        ticks: {
          color: palette.tickColor,
          font: {
            size: 11
          },
          callback: (value) => {
            return value.toLocaleString()
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
.credit-stats-container {
  background: var(--color-surface);
  border-radius: 12px;
  padding: 20px;
  border: 1px solid var(--color-border);
  margin-bottom: 20px;
  box-shadow: var(--color-shadow-elevated);
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.stats-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-heading);
  margin: 0;
}



.loading, .error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px 20px;
  color: var(--color-text-muted);
}

.error {
  color: var(--color-danger-text);
}

.error svg {
  color: var(--color-danger-text);
  opacity: 0.6;
}

.loading p, .error p {
  margin: 0;
  font-size: 14px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-btn-primary-bg);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.no-data {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px 20px;
  color: var(--text-muted);
}

.no-data svg {
  opacity: 0.4;
}

.no-data p {
  margin: 0;
  font-size: 14px;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stat-card {
  background: linear-gradient(135deg, var(--bg-muted) 0%, var(--bg-surface) 100%);
  border-radius: 10px;
  padding: 20px;
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.3s ease;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-color: var(--border-strong);
}

.chart-card {
  background: linear-gradient(135deg, var(--bg-surface) 0%, var(--bg-muted) 100%);
  border-radius: 10px;
  padding: 20px;
  border: 1px solid var(--border);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 16px;
}

.chart-wrapper {
  height: 200px;
  position: relative;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--accent) 0%, var(--accent-strong) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-icon svg {
  color: white;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-metrics {
  display: flex;
  align-items: stretch;
  gap: 24px;
}

.metric-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.metric-label {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 500;
  text-transform: none;
}

.metric-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--accent);
  line-height: 1;
  letter-spacing: -0.01em;
}

.metric-unit {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
  text-transform: uppercase;
}

.metric-divider {
  width: 1px;
  background: var(--border);
  border-radius: 2px;
  opacity: 0.6;
}


/* 响应式设计 */
@media (max-width: 768px) {
  .credit-stats-container {
    padding: 16px;
  }

  .stat-card {
    padding: 16px;
  }

  .stat-icon {
    width: 40px;
    height: 40px;
  }

  .stat-icon svg {
    width: 20px;
    height: 20px;
  }

  .stat-metrics {
    flex-direction: column;
    gap: 12px;
  }

  .metric-divider {
    width: 100%;
    height: 1px;
  }

  .metric-value {
    font-size: 24px;
  }

  .stats-header h3 {
    font-size: 14px;
  }

  .chart-card {
    padding: 16px;
  }

  .chart-title {
    font-size: 13px;
  }

  .chart-wrapper {
    height: 180px;
  }
}
</style>
