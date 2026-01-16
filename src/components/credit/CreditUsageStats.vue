<template>
  <div class="card hover:translate-y-0 p-5 mb-5 md:p-4">
    <div class="flex justify-between items-center mb-5">
      <h3 class="text-[16px] font-semibold text-text-strong m-0 md:text-[14px]">{{ $t('credit.usageStats') }}</h3>
    </div>

    <div v-if="loading && !statsData" class="flex flex-col items-center justify-center gap-3.5 py-11 px-5 text-text-muted">
      <span class="spinner"></span>
      <p class="m-0 text-[14px]">{{ $t('credit.loading') }}</p>
    </div>

    <div v-else-if="error" class="flex flex-col items-center justify-center gap-3.5 py-11 px-5 text-danger">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" class="opacity-70">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <p class="m-0 text-[14px]">{{ error }}</p>
    </div>

    <div v-else-if="!statsData || !statsData.data_points || statsData.data_points.length === 0" class="flex flex-col items-center justify-center gap-3.5 py-11 px-5 text-text-muted">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" class="opacity-50">
        <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
      </svg>
      <p class="m-0 text-[14px]">{{ $t('credit.noData') }}</p>
    </div>

    <div v-else class="flex flex-col gap-4">
      <!-- 今日消耗卡片 -->
      <div class="card hover:translate-y-0 p-5 flex items-center gap-4">
        <div class="w-[50px] h-[50px] rounded-xl bg-accent flex items-center justify-center shrink-0 max-md:w-[42px] max-md:h-[42px]">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" class="text-white max-md:w-5 max-md:h-5">
            <path d="M3.5 18.49l6-6.01 4 4L22 6.92l-1.41-1.41-7.09 7.97-4-4L2 16.99z"/>
          </svg>
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-stretch gap-6 max-md:flex-col max-md:gap-3.5">
            <div class="flex flex-col gap-2 flex-1 min-w-0">
              <div class="text-[12px] text-text-muted font-semibold uppercase tracking-wide">{{ $t('credit.today') }}</div>
              <div class="text-[28px] font-bold text-accent leading-none max-md:text-[24px]">{{ todayCreditsDisplay }}</div>
              <div class="text-[11px] text-text-muted font-semibold uppercase">{{ $t('credit.credits') }}</div>
            </div>
            <div class="w-px bg-border rounded max-md:w-full max-md:h-px" aria-hidden="true"></div>
            <div class="flex flex-col gap-2 flex-1 min-w-0">
              <div class="text-[12px] text-text-muted font-semibold uppercase tracking-wide">{{ $t('credit.total') }}</div>
              <div class="text-[28px] font-bold text-accent leading-none max-md:text-[24px]">{{ totalCreditsDisplay }}</div>
              <div class="text-[11px] text-text-muted font-semibold uppercase">{{ $t('credit.credits') }}</div>
            </div>
            <div class="w-px bg-border rounded max-md:w-full max-md:h-px" aria-hidden="true"></div>
            <div class="flex flex-col gap-2 flex-1 min-w-0">
              <div class="text-[12px] text-text-muted font-semibold uppercase tracking-wide">{{ $t('credit.remaining') }}</div>
              <div class="text-[28px] font-bold text-accent leading-none max-md:text-[24px]">{{ remainingCreditsDisplay }}</div>
              <div class="text-[11px] text-text-muted font-semibold uppercase">{{ $t('credit.credits') }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 本周期趋势图 -->
      <div class="card hover:translate-y-0 p-5 md:p-4">
        <div class="text-[14px] font-semibold text-text-strong mb-4 md:text-[13px]">{{ $t('credit.cycleTrend') }}</div>
        <div class="h-[200px] relative md:h-[180px]">
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

  const primary = resolveCssVar('--accent', isDark ? '#60a5fa' : '#2563eb')
  const primaryHover = resolveCssVar('--accent-hover', isDark ? '#93c5fd' : '#1d4ed8')
  const surface = resolveCssVar('--surface', isDark ? '#171717' : '#ffffff')
  const tooltipSurface = resolveCssVar('--surface-elevated', isDark ? '#1f1f1f' : '#ffffff')

  return {
    isDark,
    primary,
    primaryHover,
    surface,
    tooltipBg: tooltipSurface,
    tooltipTitle: resolveCssVar('--text', isDark ? '#fafafa' : '#171717'),
    tooltipBody: resolveCssVar('--text-secondary', isDark ? '#a3a3a3' : '#525252'),
    tooltipBorder: resolveCssVar('--border-strong', isDark ? '#404040' : '#d4d4d4'),
    tickColor: resolveCssVar('--text-muted', isDark ? '#737373' : '#a3a3a3'),
    gridColor: resolveCssVar('--border-muted', isDark ? '#262626' : '#f0f0f0'),
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
