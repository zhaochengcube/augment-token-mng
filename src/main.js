import { createApp } from "vue"
import { createPinia } from "pinia"
import "./style.css"
import App from "./App.vue"
import i18n from "./locales"

const THEME_STORAGE_KEY = "atm-theme"
const prefersDarkMedia = window.matchMedia("(prefers-color-scheme: dark)")

const applyTheme = (theme) => {
  const normalized = theme === "dark" ? "dark" : "light"
  const root = document.documentElement
  root.dataset.theme = normalized
  root.style.colorScheme = normalized
}

const getStoredTheme = () => {
  try {
    return localStorage.getItem(THEME_STORAGE_KEY)
  } catch (error) {
    console.warn("Failed to read stored theme preference", error)
    return null
  }
}

const resolveInitialTheme = () => {
  const stored = getStoredTheme()
  if (stored === "dark" || stored === "light") {
    return stored
  }
  return prefersDarkMedia.matches ? "dark" : "light"
}

const initialTheme = resolveInitialTheme()
applyTheme(initialTheme)

const themeManager = {
  storageKey: THEME_STORAGE_KEY,
  mediaQuery: prefersDarkMedia,
  applyTheme,
  getStoredTheme,
  initialTheme
}

const pinia = createPinia()
const app = createApp(App)
app.use(pinia)
app.use(i18n)
app.provide("themeManager", themeManager)
app.mount("#app")
