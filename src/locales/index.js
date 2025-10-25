import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN.js'
import enUS from './en-US.js'

const messages = {
  'zh-CN': zhCN,
  'en-US': enUS
}

// 从localStorage读取保存的语言偏好，默认为中文
const getInitialLocale = () => {
  const saved = localStorage.getItem('preferred-language')
  if (saved && (saved === 'zh-CN' || saved === 'en-US')) {
    return saved
  }
  return 'zh-CN'
}

const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: 'en-US',
  messages
})

export default i18n
