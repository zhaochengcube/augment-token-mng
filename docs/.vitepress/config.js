export default {
  base: '/',
  title: 'AI Tools Manager',
  description: 'ATM 文档',
  head: [['meta', { name: 'theme-color', content: '#646cff' }]],
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      themeConfig: {
        nav: [
          { text: '首页', link: '/' },
          { text: '安装与构建', link: '/install' },
          { text: '平台说明', link: '/platforms/augment' },
          { text: '订阅管理', link: '/subscription' },
          { text: '邮箱管理', link: '/email' }
        ],
        sidebar: [
          { text: '首页', link: '/' },
          { text: '安装与构建', link: '/install' },
          {
            text: '平台说明',
            collapsed: false,
            items: [
              { text: 'Augment', link: '/platforms/augment' },
              { text: 'Antigravity', link: '/platforms/antigravity' },
              { text: 'Windsurf', link: '/platforms/windsurf' },
              { text: 'Cursor', link: '/platforms/cursor' },
              { text: 'OpenAI', link: '/platforms/openai' },
              { text: 'Claude Code', link: '/platforms/claude' }
            ]
          },
          { text: '订阅管理', link: '/subscription' },
          { text: '邮箱管理', link: '/email' }
        ]
      }
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Install', link: '/en/install' },
          { text: 'Platforms', link: '/en/platforms/augment' },
          { text: 'Subscriptions', link: '/en/subscription' },
          { text: 'Email', link: '/en/email' }
        ],
        sidebar: [
          { text: 'Home', link: '/en/' },
          { text: 'Install', link: '/en/install' },
          {
            text: 'Platforms',
            collapsed: false,
            items: [
              { text: 'Augment', link: '/en/platforms/augment' },
              { text: 'Antigravity', link: '/en/platforms/antigravity' },
              { text: 'Windsurf', link: '/en/platforms/windsurf' },
              { text: 'Cursor', link: '/en/platforms/cursor' },
              { text: 'OpenAI', link: '/en/platforms/openai' },
              { text: 'Claude Code', link: '/en/platforms/claude' }
            ]
          },
          { text: 'Subscriptions', link: '/en/subscription' },
          { text: 'Email', link: '/en/email' }
        ]
      }
    }
  },
  themeConfig: {
    i18nRouting: true,
    socialLinks: [{ icon: 'github', link: 'https://github.com/zhaochengcube/augment-token-mng' }]
  }
}
