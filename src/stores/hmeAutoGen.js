import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useHmeAutoGenStore = defineStore('hmeAutoGen', () => {
  const isAutoGenerating = ref(false)
  const autoGenCountdownMs = ref(0)
  const lastGenTimestamp = ref(0)
  const generating = ref(false)

  let _intervalId = null
  let _genCount = 5
  let _genLabel = 'ATM HME'
  let _onGenerated = null

  const COOLDOWN_MS = 60 * 60 * 1000
  const STORAGE_KEY = 'hme_auto_gen_enabled'

  const getCooldownRemaining = () => {
    if (lastGenTimestamp.value === 0) return 0
    const elapsed = Date.now() - lastGenTimestamp.value
    const remaining = COOLDOWN_MS - elapsed
    return remaining > 0 ? remaining : 0
  }

  const doGenerate = async () => {
    if (getCooldownRemaining() > 0) return null

    generating.value = true
    try {
      const result = await invoke('hme_generate', {
        count: _genCount,
        label: _genLabel || null
      })
      if (result.generated?.length > 0) {
        lastGenTimestamp.value = Date.now()
      }
      if (_onGenerated) _onGenerated(result, null)
      return result
    } catch (e) {
      if (_onGenerated) _onGenerated(null, e)
      return null
    } finally {
      generating.value = false
    }
  }

  const _startInterval = () => {
    _intervalId = setInterval(() => {
      const remaining = getCooldownRemaining()
      autoGenCountdownMs.value = remaining
      if (remaining === 0 && !generating.value) {
        doGenerate()
      }
    }, 1000)
  }

  const startAutoGenerate = (count, label) => {
    _genCount = count
    _genLabel = label
    isAutoGenerating.value = true

    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ count, label }))
    } catch { /* quota exceeded etc. */ }

    doGenerate()
    _startInterval()
  }

  const stopAutoGenerate = () => {
    if (_intervalId) {
      clearInterval(_intervalId)
      _intervalId = null
    }
    isAutoGenerating.value = false
    autoGenCountdownMs.value = 0

    try { localStorage.removeItem(STORAGE_KEY) } catch {}
  }

  const restoreAutoGenerate = () => {
    if (isAutoGenerating.value) return null
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      if (!raw) return null
      const saved = JSON.parse(raw)
      _genCount = saved.count ?? 5
      _genLabel = saved.label ?? 'ATM HME'
      isAutoGenerating.value = true
      _startInterval()
      return saved
    } catch {
      return null
    }
  }

  const registerOnGenerated = (callback) => {
    _onGenerated = callback
  }

  const unregisterOnGenerated = () => {
    _onGenerated = null
  }

  const updateLastGenTimestamp = (ts) => {
    if (ts > lastGenTimestamp.value) {
      lastGenTimestamp.value = ts
    }
  }

  return {
    isAutoGenerating,
    autoGenCountdownMs,
    lastGenTimestamp,
    generating,
    COOLDOWN_MS,
    getCooldownRemaining,
    startAutoGenerate,
    stopAutoGenerate,
    restoreAutoGenerate,
    registerOnGenerated,
    unregisterOnGenerated,
    updateLastGenTimestamp
  }
})
