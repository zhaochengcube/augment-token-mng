<template>
  <div class="flex flex-col gap-4">
    <!-- 趋势折线图 (费用 + Token 双轴) -->
    <div class="rounded-lg border border-border bg-muted/10 p-3 h-[260px] flex flex-col">
      <div class="flex justify-between items-center mb-2 shrink-0">
        <div class="flex items-center gap-2">
          <h4 class="text-[13px] font-semibold text-text-secondary m-0">{{ $t('cursorUsage.chartTrend') }}</h4>
          <button
            class="px-1.5 py-0.5 text-[10px] rounded border transition-colors"
            :class="trendByModel
              ? 'bg-accent/15 text-accent border-accent/30'
              : 'bg-transparent text-text-muted border-border hover:text-text-strong'"
            @click="toggleByModel"
          >
            {{ $t('cursorUsage.byModel') }}
          </button>
        </div>
        <div class="flex items-center gap-3 text-[11px]">
          <template v-if="!trendByModel">
            <button
              class="flex items-center gap-1 transition-opacity hover:opacity-70"
              :class="trendMetric === 'both' ? 'opacity-100' : 'opacity-40'"
              @click="trendMetric = 'both'"
            >
              <span class="flex items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-[#4c6ef5]"></span>
                <span class="w-2 h-2 rounded-full bg-[#f783ac]"></span>
              </span>
              {{ $t('cursorUsage.cost') }}/Tokens
            </button>
          </template>
          <button
            class="flex items-center gap-1 transition-opacity hover:opacity-70"
            :class="trendMetric === 'cost' ? 'opacity-100' : 'opacity-40'"
            @click="trendMetric = 'cost'"
          >
            <span class="w-2.5 h-2.5 rounded-full bg-[#4c6ef5]"></span>
            {{ $t('cursorUsage.cost') }}
          </button>
          <button
            class="flex items-center gap-1 transition-opacity hover:opacity-70"
            :class="trendMetric === 'tokens' ? 'opacity-100' : 'opacity-40'"
            @click="trendMetric = 'tokens'"
          >
            <span class="w-2.5 h-2.5 rounded-full bg-[#f783ac]"></span>
            Tokens
          </button>
        </div>
      </div>
      <div v-if="!hasData" class="flex-1 flex items-center justify-center text-text-muted text-[12px]">
        {{ $t('cursorUsage.noEvents') }}
      </div>
      <div v-else class="flex-1 min-h-0 relative">
        <Line :key="chartKey + '_trend'" :data="trendChartData" :options="trendChartOptions" />
      </div>
    </div>

    <!-- 下方两个图表并排 -->
    <div class="grid grid-cols-2 gap-4">
      <!-- 饼图: 模型使用占比 -->
      <div class="rounded-lg border border-border bg-muted/10 p-3 h-[260px] flex flex-col">
        <h4 class="text-[13px] font-semibold text-text-secondary m-0 mb-2 shrink-0">{{ $t('cursorUsage.chartModelDist') }}</h4>
        <div v-if="!hasData" class="flex-1 flex items-center justify-center text-text-muted text-[12px]">
          {{ $t('cursorUsage.noEvents') }}
        </div>
        <div v-else class="flex-1 min-h-0 relative">
          <Pie :key="chartKey + '_pie'" :data="pieChartData" :options="pieChartOptions" />
        </div>
      </div>

      <!-- 柱状图: 模型费用对比 -->
      <div class="rounded-lg border border-border bg-muted/10 p-3 h-[260px] flex flex-col">
        <h4 class="text-[13px] font-semibold text-text-secondary m-0 mb-2 shrink-0">{{ $t('cursorUsage.chartModelCost') }}</h4>
        <div v-if="!hasData" class="flex-1 flex items-center justify-center text-text-muted text-[12px]">
          {{ $t('cursorUsage.noEvents') }}
        </div>
        <div v-else class="flex-1 min-h-0 relative">
          <Bar :key="chartKey + '_bar'" :data="barChartData" :options="barChartOptions" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Line, Pie, Bar } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Tooltip,
  Legend
} from 'chart.js'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, BarElement, ArcElement, Tooltip, Legend)

const { t: $t } = useI18n()

const props = defineProps({
  usageEvents: {
    type: Array,
    default: () => []
  }
})

const chartKey = ref(0)
const currentTheme = ref('light')
const trendMetric = ref('both')
const trendByModel = ref(false)

const toggleByModel = () => {
  trendByModel.value = !trendByModel.value
  if (trendByModel.value) {
    // 按模型模式不支持双轴，自动切换到单指标
    if (trendMetric.value === 'both') {
      trendMetric.value = 'cost'
    }
  } else {
    // 取消按模型时恢复默认双指标
    trendMetric.value = 'both'
  }
}

const hasData = computed(() => props.usageEvents && props.usageEvents.length > 0)

// --- Theme ---

const readTheme = () => {
  if (typeof document === 'undefined') return 'light'
  return document.documentElement.dataset.theme || document.documentElement.getAttribute('data-theme') || 'light'
}

const resolveCssVar = (name, fallback) => {
  if (typeof document === 'undefined') return fallback
  const value = getComputedStyle(document.documentElement).getPropertyValue(name)?.trim()
  return value || fallback
}

const themePalette = computed(() => {
  const isDark = currentTheme.value === 'dark'
  return {
    isDark,
    surface: resolveCssVar('--surface', isDark ? '#171717' : '#ffffff'),
    gridColor: resolveCssVar('--border', isDark ? '#404040' : '#e5e5e5'),
    legendColor: resolveCssVar('--text-secondary', isDark ? '#a3a3a3' : '#525252'),
    tooltipBg: resolveCssVar('--surface-elevated', isDark ? '#1f1f1f' : '#ffffff'),
    tooltipTitle: resolveCssVar('--text', isDark ? '#fafafa' : '#171717'),
    tooltipBody: resolveCssVar('--text-secondary', isDark ? '#a3a3a3' : '#525252'),
    tooltipBorder: resolveCssVar('--border-strong', isDark ? '#404040' : '#d4d4d4')
  }
})

const chartColors = [
  '#4c6ef5', '#f783ac', '#38bdf8', '#43e97b',
  '#ff922b', '#845ef7', '#12b886', '#fd7e14'
]

// --- Data processing ---

const parseTimestamp = (timestamp) => {
  if (!timestamp) return null
  if (typeof timestamp === 'number') return new Date(timestamp)
  const num = Number(timestamp)
  if (!isNaN(num) && num > 1000000000000) return new Date(num)
  const d = new Date(timestamp)
  return isNaN(d.getTime()) ? null : d
}

const formatDateKey = (date) => {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

const formatDateLabel = (dateKey) => {
  return dateKey.slice(5) // MM-DD
}

const formatNumber = (v) => {
  const n = Number(v || 0)
  if (n < 1000) return n.toLocaleString()
  if (n < 1e6) return (n / 1e3).toFixed(1).replace(/\.0$/, '') + 'K'
  if (n < 1e9) return (n / 1e6).toFixed(2).replace(/\.00$/, '') + 'M'
  return (n / 1e9).toFixed(2).replace(/\.00$/, '') + 'B'
}

// Trend data: group by date
const trendData = computed(() => {
  if (!hasData.value) return { labels: [], costs: [], tokens: [] }

  const grouped = {}
  for (const event of props.usageEvents) {
    const date = parseTimestamp(event.timestamp)
    if (!date) continue
    const key = formatDateKey(date)
    if (!grouped[key]) grouped[key] = { cost: 0, tokens: 0 }
    if (event.tokenUsage) {
      grouped[key].cost += (event.tokenUsage.totalCents || 0) / 100
      grouped[key].tokens += (event.tokenUsage.inputTokens || 0) + (event.tokenUsage.outputTokens || 0)
    }
  }

  const sorted = Object.entries(grouped).sort((a, b) => a[0].localeCompare(b[0]))
  return {
    labels: sorted.map(([k]) => formatDateLabel(k)),
    costs: sorted.map(([, d]) => Math.round(d.cost * 10000) / 10000),
    tokens: sorted.map(([, d]) => d.tokens)
  }
})

// Trend data by model: group by date + model
const trendByModelData = computed(() => {
  if (!hasData.value) return { labels: [], models: [], modelDayData: {} }

  const dayModelMap = {} // { dateKey: { model: { cost, tokens } } }
  const modelSet = new Set()

  for (const event of props.usageEvents) {
    const date = parseTimestamp(event.timestamp)
    if (!date) continue
    const key = formatDateKey(date)
    const model = event.model || 'Unknown'
    modelSet.add(model)
    if (!dayModelMap[key]) dayModelMap[key] = {}
    if (!dayModelMap[key][model]) dayModelMap[key][model] = { cost: 0, tokens: 0 }
    if (event.tokenUsage) {
      dayModelMap[key][model].cost += (event.tokenUsage.totalCents || 0) / 100
      dayModelMap[key][model].tokens += (event.tokenUsage.inputTokens || 0) + (event.tokenUsage.outputTokens || 0)
    }
  }

  const sortedDays = Object.keys(dayModelMap).sort()
  // 按总费用排序模型，取前8
  const modelTotals = {}
  for (const model of modelSet) {
    modelTotals[model] = 0
    for (const day of sortedDays) {
      modelTotals[model] += dayModelMap[day]?.[model]?.cost || 0
    }
  }
  const models = [...modelSet].sort((a, b) => modelTotals[b] - modelTotals[a]).slice(0, 8)

  return { labels: sortedDays.map(formatDateLabel), models, dayModelMap, sortedDays }
})

// Model data: group by model
const modelData = computed(() => {
  if (!hasData.value) return { labels: [], costs: [], counts: [], inputTokens: [], outputTokens: [] }

  const grouped = {}
  for (const event of props.usageEvents) {
    const model = event.model || 'Unknown'
    if (!grouped[model]) grouped[model] = { cost: 0, count: 0, inputTokens: 0, outputTokens: 0 }
    if (event.tokenUsage) {
      grouped[model].cost += (event.tokenUsage.totalCents || 0) / 100
      grouped[model].inputTokens += event.tokenUsage.inputTokens || 0
      grouped[model].outputTokens += event.tokenUsage.outputTokens || 0
    }
    grouped[model].count++
  }

  const sorted = Object.entries(grouped).sort((a, b) => b[1].cost - a[1].cost)
  return {
    labels: sorted.map(([m]) => m),
    costs: sorted.map(([, d]) => Math.round(d.cost * 10000) / 10000),
    counts: sorted.map(([, d]) => d.count),
    inputTokens: sorted.map(([, d]) => d.inputTokens),
    outputTokens: sorted.map(([, d]) => d.outputTokens)
  }
})

// --- Chart 1: Trend Line ---

const trendChartData = computed(() => {
  // 按模型模式
  if (trendByModel.value) {
    const bm = trendByModelData.value
    const isCost = trendMetric.value === 'cost'
    const field = isCost ? 'cost' : 'tokens'

    const datasets = bm.models.map((model, i) => ({
      label: model,
      data: bm.sortedDays.map(day => {
        const v = bm.dayModelMap[day]?.[model]?.[field] || 0
        return isCost ? Math.round(v * 10000) / 10000 : v
      }),
      borderColor: chartColors[i % chartColors.length],
      backgroundColor: chartColors[i % chartColors.length],
      borderWidth: 2,
      tension: 0.3,
      pointRadius: 2,
      pointHoverRadius: 5,
      fill: false
    }))

    return { labels: bm.labels, datasets }
  }

  // 聚合模式
  const d = trendData.value
  const datasets = []

  if (trendMetric.value === 'cost' || trendMetric.value === 'both') {
    datasets.push({
      label: $t('cursorUsage.cost'),
      data: d.costs,
      yAxisID: 'yCost',
      borderColor: '#4c6ef5',
      backgroundColor: '#4c6ef5',
      borderWidth: 2,
      tension: 0.3,
      pointRadius: 2,
      pointHoverRadius: 5,
      fill: false
    })
  }

  if (trendMetric.value === 'tokens' || trendMetric.value === 'both') {
    datasets.push({
      label: 'Tokens',
      data: d.tokens,
      yAxisID: 'yTokens',
      borderColor: '#f783ac',
      backgroundColor: '#f783ac',
      borderWidth: 2,
      tension: 0.3,
      pointRadius: 2,
      pointHoverRadius: 5,
      fill: false
    })
  }

  return { labels: d.labels, datasets }
})

const trendChartOptions = computed(() => {
  const p = themePalette.value
  const isCost = trendMetric.value === 'cost'
  const isTokens = trendMetric.value === 'tokens'
  const isBoth = trendMetric.value === 'both'

  const createAxis = ({ position = 'left', display = true, drawOnChartArea = true, prefix = '' }) => ({
    display,
    position,
    grid: { color: p.gridColor, drawBorder: false, drawOnChartArea },
    ticks: {
      color: p.legendColor,
      font: { size: 10 },
      precision: 0,
      callback: (value) => {
        if (value < 0) return ''
        return prefix + formatNumber(value)
      }
    },
    min: 0,
    beginAtZero: true
  })

  // 按模型模式：单 Y 轴 + 显示图例
  if (trendByModel.value) {
    const prefix = isCost ? '$' : ''
    return {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: {
        legend: {
          display: true,
          position: 'bottom',
          labels: { padding: 8, font: { size: 10 }, color: p.legendColor, usePointStyle: true, pointStyle: 'circle', boxWidth: 6 }
        },
        tooltip: {
          backgroundColor: p.tooltipBg,
          titleColor: p.tooltipTitle,
          bodyColor: p.tooltipBody,
          borderColor: p.tooltipBorder,
          borderWidth: 1,
          padding: 10,
          displayColors: true,
          filter: (item) => item.parsed.y !== 0,
          callbacks: {
            label: (ctx) => {
              if (isCost) return `${ctx.dataset.label}: $${ctx.parsed.y.toFixed(4)}`
              return `${ctx.dataset.label}: ${formatNumber(ctx.parsed.y)}`
            }
          }
        }
      },
      scales: {
        x: {
          grid: { display: false },
          ticks: { color: p.legendColor, font: { size: 10 }, maxRotation: 0, autoSkip: true, maxTicksLimit: 10 }
        },
        y: createAxis({ position: 'left', display: true, drawOnChartArea: true, prefix })
      }
    }
  }

  // 聚合模式：双 Y 轴 + 隐藏图例
  return {
    responsive: true,
    maintainAspectRatio: false,
    interaction: { mode: 'index', intersect: false },
    plugins: {
      legend: { display: false },
      tooltip: {
        backgroundColor: p.tooltipBg,
        titleColor: p.tooltipTitle,
        bodyColor: p.tooltipBody,
        borderColor: p.tooltipBorder,
        borderWidth: 1,
        padding: 10,
        displayColors: true,
        callbacks: {
          label: (ctx) => {
            if (ctx.dataset.yAxisID === 'yCost') {
              return `${$t('cursorUsage.cost')}: $${ctx.parsed.y.toFixed(4)}`
            }
            return `Tokens: ${formatNumber(ctx.parsed.y)}`
          }
        }
      }
    },
    scales: {
      x: {
        grid: { display: false },
        ticks: { color: p.legendColor, font: { size: 10 }, maxRotation: 0, autoSkip: true, maxTicksLimit: 10 }
      },
      yCost: createAxis({ position: 'left', display: isCost || isBoth, drawOnChartArea: true, prefix: '$' }),
      yTokens: createAxis({ position: 'right', display: isTokens || isBoth, drawOnChartArea: isBoth })
    }
  }
})

// --- Chart 2: Pie ---

const pieChartData = computed(() => {
  const d = modelData.value
  const p = themePalette.value
  return {
    labels: d.labels,
    datasets: [{
      data: d.costs,
      backgroundColor: chartColors.slice(0, d.labels.length),
      borderWidth: 2,
      borderColor: p.surface,
      hoverOffset: 10
    }]
  }
})

const pieChartOptions = computed(() => {
  const p = themePalette.value
  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'right',
        labels: {
          padding: 10,
          font: { size: 11 },
          color: p.legendColor,
          usePointStyle: true,
          pointStyle: 'circle',
          boxWidth: 8
        }
      },
      tooltip: {
        backgroundColor: p.tooltipBg,
        titleColor: p.tooltipTitle,
        bodyColor: p.tooltipBody,
        borderColor: p.tooltipBorder,
        borderWidth: 1,
        padding: 10,
        callbacks: {
          label: (ctx) => {
            const total = ctx.dataset.data.reduce((a, b) => a + b, 0)
            const pct = total > 0 ? ((ctx.parsed / total) * 100).toFixed(1) : '0.0'
            return `${ctx.label}: $${ctx.parsed.toFixed(4)} (${pct}%)`
          }
        }
      }
    }
  }
})

// --- Chart 3: Bar ---

const barChartData = computed(() => {
  const d = modelData.value
  return {
    labels: d.labels,
    datasets: [
      {
        label: $t('cursorUsage.inputTokens'),
        data: d.inputTokens,
        backgroundColor: '#4c6ef5',
        borderRadius: 3
      },
      {
        label: $t('cursorUsage.outputTokens'),
        data: d.outputTokens,
        backgroundColor: '#f783ac',
        borderRadius: 3
      }
    ]
  }
})

const barChartOptions = computed(() => {
  const p = themePalette.value
  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'bottom',
        labels: {
          padding: 10,
          font: { size: 11 },
          color: p.legendColor,
          usePointStyle: true,
          pointStyle: 'rectRounded',
          boxWidth: 10
        }
      },
      tooltip: {
        backgroundColor: p.tooltipBg,
        titleColor: p.tooltipTitle,
        bodyColor: p.tooltipBody,
        borderColor: p.tooltipBorder,
        borderWidth: 1,
        padding: 10,
        callbacks: {
          label: (ctx) => `${ctx.dataset.label}: ${formatNumber(ctx.parsed.y)}`
        }
      }
    },
    scales: {
      x: {
        stacked: true,
        grid: { display: false },
        ticks: { color: p.legendColor, font: { size: 10 }, maxRotation: 45 }
      },
      y: {
        stacked: true,
        grid: { color: p.gridColor, drawBorder: false },
        ticks: {
          color: p.legendColor,
          font: { size: 10 },
          callback: (value) => formatNumber(value)
        },
        beginAtZero: true
      }
    }
  }
})

// --- Theme observer ---

let observer

const updateTheme = () => {
  currentTheme.value = readTheme()
}

onMounted(() => {
  updateTheme()
  if (typeof document === 'undefined') return
  observer = new MutationObserver(updateTheme)
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] })
})

onUnmounted(() => {
  observer?.disconnect()
})

watch(currentTheme, () => { chartKey.value++ })
watch(trendMetric, () => { chartKey.value++ })
watch(trendByModel, () => { chartKey.value++ })
</script>
