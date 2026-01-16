<template>
  <div class="card hover:translate-y-0 p-5 h-[400px] flex flex-col md:h-[340px] md:p-4">
    <div class="flex justify-between items-center mb-4 shrink-0">
      <h3 class="text-[16px] font-semibold text-text-strong m-0 md:text-[14px]">{{ $t('credit.modelDistribution') }}</h3>
    </div>

    <div v-if="loading && !chartData" class="flex-1 flex flex-col items-center justify-center gap-3.5 text-text-muted">
      <span class="spinner"></span>
      <p class="m-0 text-[14px]">{{ $t('credit.loading') }}</p>
    </div>

    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center gap-3.5 text-danger">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" class="opacity-70">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <p class="m-0 text-[14px]">{{ error }}</p>
    </div>

    <div v-else-if="!chartData || !chartData.data_points || chartData.data_points.length === 0" class="flex-1 flex flex-col items-center justify-center gap-3.5 text-text-muted">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor" class="opacity-50">
        <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
      </svg>
      <p class="m-0 text-[14px]">{{ $t('credit.noData') }}</p>
    </div>

    <div v-else class="flex-1 min-h-0 relative">
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
    surface: resolveCssVar('--surface', isDark ? '#171717' : '#ffffff'),
    legendColor: resolveCssVar('--text', isDark ? '#fafafa' : '#171717'),
    tooltipBg: resolveCssVar('--surface-elevated', isDark ? '#1f1f1f' : '#ffffff'),
    tooltipTitle: resolveCssVar('--text', isDark ? '#fafafa' : '#171717'),
    tooltipBody: resolveCssVar('--text-secondary', isDark ? '#a3a3a3' : '#525252'),
    tooltipBorder: resolveCssVar('--border-strong', isDark ? '#404040' : '#d4d4d4')
  }
})

const chartColors = computed(() => ([
  '#4c6ef5',
  '#f783ac',
  '#38bdf8',
  '#43e97b',
  '#ff922b',
  '#845ef7',
  '#12b886',
  '#fd7e14'
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
