<template>
  <div class="rounded-lg border border-border bg-muted/10 p-3 h-[280px] flex flex-col">
    <div class="flex justify-between items-center mb-2 shrink-0">
      <h4 class="text-[13px] font-semibold text-text-secondary m-0">{{ $t('platform.openai.codexDialog.monthlyTrend') }}</h4>
      <div class="flex items-center gap-3 text-[11px]">
        <button
          class="flex items-center gap-1 transition-opacity hover:opacity-70"
          :class="activeMetric === 'both' ? 'opacity-100' : 'opacity-40'"
          @click="activeMetric = 'both'"
        >
          <span class="flex items-center gap-1">
            <span class="w-2 h-2 rounded-full bg-[#4c6ef5]"></span>
            <span class="w-2 h-2 rounded-full bg-[#f783ac]"></span>
          </span>
          {{ $t('platform.openai.codexDialog.requests') }}/{{ $t('platform.openai.codexDialog.tokens') }}
        </button>
        <button
          class="flex items-center gap-1 transition-opacity hover:opacity-70"
          :class="activeMetric === 'requests' ? 'opacity-100' : 'opacity-40'"
          @click="activeMetric = 'requests'"
        >
          <span class="w-2.5 h-2.5 rounded-full bg-[#4c6ef5]"></span>
          {{ $t('platform.openai.codexDialog.requests') }}
        </button>
        <button
          class="flex items-center gap-1 transition-opacity hover:opacity-70"
          :class="activeMetric === 'tokens' ? 'opacity-100' : 'opacity-40'"
          @click="activeMetric = 'tokens'"
        >
          <span class="w-2.5 h-2.5 rounded-full bg-[#f783ac]"></span>
          {{ $t('platform.openai.codexDialog.tokens') }}
        </button>
      </div>
    </div>

    <div v-if="loading && !chartData" class="flex-1 flex flex-col items-center justify-center gap-2 text-text-muted">
      <span class="spinner spinner--sm"></span>
    </div>

    <div v-else-if="!chartData || chartData.length === 0" class="flex-1 flex flex-col items-center justify-center gap-2 text-text-muted">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor" class="opacity-50">
        <path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z"/>
      </svg>
      <p class="m-0 text-[12px]">{{ $t('platform.openai.codexDialog.noData') }}</p>
    </div>

    <div v-else class="flex-1 min-h-0 relative">
      <Line :key="chartKey" :data="lineChartData" :options="chartOptions" />
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { Line } from 'vue-chartjs'
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend } from 'chart.js'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend)

const props = defineProps({
  loading: {
    type: Boolean,
    default: false
  },
  chartData: {
    type: Array,
    default: () => []
  }
})

const activeMetric = ref('both') // 'requests' | 'tokens' | 'both'

const chartKey = ref(0)
const currentTheme = ref('light')

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
    gridColor: resolveCssVar('--border', isDark ? '#404040' : '#e5e5e5'),
    legendColor: resolveCssVar('--text-secondary', isDark ? '#a3a3a3' : '#525252'),
    tooltipBg: resolveCssVar('--surface-elevated', isDark ? '#1f1f1f' : '#ffffff'),
    tooltipTitle: resolveCssVar('--text', isDark ? '#fafafa' : '#171717'),
    tooltipBody: resolveCssVar('--text-secondary', isDark ? '#a3a3a3' : '#525252'),
    tooltipBorder: resolveCssVar('--border-strong', isDark ? '#404040' : '#d4d4d4')
  }
})

const formatNumber = (v) => {
  const n = Number(v || 0)
  if (n < 1000) return n.toLocaleString()
  if (n < 1000000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'K'
  if (n < 1000000000) return (n / 1000000).toFixed(2).replace(/\.00$/, '') + 'M'
  if (n < 1000000000000) return (n / 1000000000).toFixed(2).replace(/\.00$/, '') + 'B'
  return (n / 1000000000000).toFixed(2).replace(/\.00$/, '') + 'T'
}

const formatLabel = (dateStr) => {
  const date = new Date(dateStr)
  if (Number.isNaN(date.getTime())) return dateStr
  return `${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

const lineChartData = computed(() => {
  if (!props.chartData || props.chartData.length === 0) {
    return { labels: [], datasets: [] }
  }

  const labels = props.chartData.map(d => formatLabel(d.date))
  const requests = props.chartData.map(d => d.requests || 0)
  const tokens = props.chartData.map(d => d.tokens || 0)

  if (activeMetric.value === 'requests') {
    return {
      labels,
      datasets: [
        {
          label: 'Requests',
          data: requests,
          yAxisID: 'yRequests',
          borderColor: '#4c6ef5',
          backgroundColor: '#4c6ef5',
          borderWidth: 2,
          tension: 0.3,
          pointRadius: 2,
          pointHoverRadius: 5,
          fill: false
        }
      ]
    }
  }

  if (activeMetric.value === 'tokens') {
    return {
      labels,
      datasets: [
        {
          label: 'Tokens',
          data: tokens,
          yAxisID: 'yTokens',
          borderColor: '#f783ac',
          backgroundColor: '#f783ac',
          borderWidth: 2,
          tension: 0.3,
          pointRadius: 2,
          pointHoverRadius: 5,
          fill: false
        }
      ]
    }
  }

  return {
    labels,
    datasets: [
      {
        label: 'Requests',
        data: requests,
        yAxisID: 'yRequests',
        borderColor: '#4c6ef5',
        backgroundColor: '#4c6ef5',
        borderWidth: 2,
        tension: 0.3,
        pointRadius: 2,
        pointHoverRadius: 5,
        fill: false
      },
      {
        label: 'Tokens',
        data: tokens,
        yAxisID: 'yTokens',
        borderColor: '#f783ac',
        backgroundColor: '#f783ac',
        borderWidth: 2,
        tension: 0.3,
        pointRadius: 2,
        pointHoverRadius: 5,
        fill: false
      }
    ]
  }
})

const chartOptions = computed(() => {
  const palette = themePalette.value
  const isRequests = activeMetric.value === 'requests'
  const isTokens = activeMetric.value === 'tokens'
  const isBoth = activeMetric.value === 'both'

  const createAxis = ({ position = 'left', display = true, drawOnChartArea = true }) => ({
    display,
    position,
    grid: {
      color: palette.gridColor,
      drawBorder: false,
      drawOnChartArea
    },
    ticks: {
      color: palette.legendColor,
      font: {
        size: 10
      },
      precision: 0,
      callback: (value) => {
        if (value < 1) return ''
        return formatNumber(value)
      }
    },
    min: 0,
    beginAtZero: true
  })

  return {
    responsive: true,
    maintainAspectRatio: false,
    interaction: {
      mode: 'index',
      intersect: false
    },
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
        padding: 10,
        displayColors: true,
        callbacks: {
          title: (items) => {
            if (items.length > 0) {
              const index = items[0].dataIndex
              const data = props.chartData[index]
              if (data) {
                return data.date
              }
            }
            return ''
          },
          label: (context) => {
            const index = context.dataIndex
            const data = props.chartData[index]
            if (!data) return ''

            if (isRequests) {
              return `Requests: ${formatNumber(data.requests)}`
            }
            if (isTokens) {
              return `Tokens: ${formatNumber(data.tokens)}`
            }

            if (context.dataset?.label === 'Requests') {
              return `Requests: ${formatNumber(data.requests)}`
            }
            return `Tokens: ${formatNumber(data.tokens)}`
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
          color: palette.legendColor,
          font: {
            size: 10
          },
          maxRotation: 0,
          autoSkip: true,
          maxTicksLimit: 7
        }
      },
      yRequests: createAxis({
        position: 'left',
        display: isRequests || isBoth,
        drawOnChartArea: true
      }),
      yTokens: createAxis({
        position: 'right',
        display: isTokens || isBoth,
        drawOnChartArea: isBoth
      })
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

watch(activeMetric, () => {
  chartKey.value++
})
</script>
