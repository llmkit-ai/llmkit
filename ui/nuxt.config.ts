// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false },
  runtimeConfig: {
    apiKey: process.env.API_KEY || 'default_api_key_for_development',
    public: {
      colorMode: {
        preference: 'system'
      }
    }
  },
  modules: [
    '@nuxtjs/tailwindcss',
    '@nuxtjs/color-mode',
    'nuxt-auth-utils'
  ],
  nitro: {
    // We're now handling proxying in our middleware
  },
  colorMode: {
    preference: 'system',
    fallback: 'light',
    storageKey: 'nuxt-color-mode',
    classSuffix: '',
    dataValue: 'theme'
  },
  auth: {
    provider: {
      type: 'local',
    },
    session: {
      // Enable session management
      enableRefreshOnWindowFocus: true,
      enableRefreshPeriodically: 5 * 60, // 5 minutes
    },
    globalAppMiddleware: {
      // Enable auth middleware globally
      isEnabled: true,
      // Exclude certain routes from authentication check
      exclude: ['/login', '/api/auth/**']
    }
  }
})