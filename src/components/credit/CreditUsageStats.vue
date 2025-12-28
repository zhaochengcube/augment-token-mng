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

  const primary = resolveCssVar('--accent', isDark ? '#a5b4fc' : '#667eea')
  const primaryHover = resolveCssVar('--accent-hover', isDark ? '#c7d2fe' : '#5a6fd8')
  const surface = resolveCssVar('--bg-surface', isDark ? '#0f172a' : '#ffffff')
  const tooltipSurface = resolveCssVar('--bg-surface-alt', isDark ? '#111827' : '#ffffff')

  return {
    isDark,
    primary,
    primaryHover,
    surface,
    tooltipBg: tooltipSurface,
    tooltipTitle: resolveCssVar('--text-strong', isDark ? '#f9fafb' : '#1f2937'),
    tooltipBody: resolveCssVar('--text', isDark ? '#e2e8f0' : '#374151'),
    tooltipBorder: resolveCssVar('--border-strong', isDark ? 'rgba(148, 163, 184, 0.45)' : '#e5e7eb'),
    tickColor: resolveCssVar('--text-muted', isDark ? '#cbd5f5' : '#6b7280'),
    gridColor: resolveCssVar('--border-muted', isDark ? 'rgba(148, 163, 184, 0.3)' : '#e5e7eb'),
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
/* ============================================
   CreditUsageStats - Modern Tech Style
   ============================================ */

.credit-stats-container {
  background: var(--tech-glass-bg);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--tech-glass-border);
  border-radius: 16px;
  padding: 22px;
  margin-bottom: 22px;
  box-shadow: var(--tech-border-glow);
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 22px;
}

.stats-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-strong);
  margin: 0;
}

.loading, .error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 45px 22px;
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

.loading p, .error p {
  margin: 0;
  font-size: 14px;
}

/* 加载动画 - 科技风 */
.spinner {
  width: 42px;
  height: 42px;
  border: 3px solid var(--tech-glass-border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  box-shadow: 0 0 15px var(--tech-glow-primary);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.no-data {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 45px 22px;
  color: var(--text-muted);
}

.no-data svg {
  opacity: 0.5;
}

.no-data p {
  margin: 0;
  font-size: 14px;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

/* 统计卡片 - 科技风 */
.stat-card {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  padding: 22px;
  display: flex;
  align-items: center;
  gap: 18px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.stat-card:hover {
  transform: translateY(-3px);
  border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  box-shadow: 0 0 20px var(--tech-glow-primary);
}

/* 图表卡片 - 科技风 */
.chart-card {
  background: color-mix(in srgb, var(--bg-muted) 50%, transparent);
  border: 1px solid var(--tech-glass-border);
  border-radius: 14px;
  padding: 22px;
}

.chart-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 18px;
}

.chart-wrapper {
  height: 200px;
  position: relative;
}

/* 统计图标 - 科技风渐变 */
.stat-icon {
  width: 50px;
  height: 50px;
  border-radius: 12px;
  background: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 0 15px var(--tech-glow-primary);
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
  gap: 26px;
}

.metric-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.metric-label {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.metric-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--accent);
  line-height: 1;
  letter-spacing: -0.01em;
  text-shadow: 0 0 10px var(--tech-glow-primary);
  font-family: var(--tech-mono-font);
}

.metric-unit {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 600;
  text-transform: uppercase;
}

.metric-divider {
  width: 1px;
  background: var(--tech-glass-border);
  border-radius: 2px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .credit-stats-container {
    padding: 18px;
  }

  .stat-card {
    padding: 18px;
  }

  .stat-icon {
    width: 42px;
    height: 42px;
  }

  .stat-icon svg {
    width: 20px;
    height: 20px;
  }

  .stat-metrics {
    flex-direction: column;
    gap: 14px;
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
    padding: 18px;
  }

  .chart-title {
    font-size: 13px;
  }

  .chart-wrapper {
    height: 180px;
  }
}
</style>
