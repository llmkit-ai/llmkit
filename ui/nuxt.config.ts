// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false },
  runtimeConfig: {
    public: {
      colorMode: {
        preference: 'system'
      }
    }
  },
  modules: [
    '@nuxtjs/tailwindcss',
    '@nuxtjs/color-mode',
    '@vueuse/nuxt',
  ],
  nitro: {
    routeRules: {
      "/v1/**": { 
        proxy: process.env.API_BASE_URL ? 
          `${process.env.API_BASE_URL}/v1/**` : 
          'http://127.0.0.1:8000/v1/**' 
      },
    }
  },
  colorMode: {
    preference: 'system',
    fallback: 'light',
    storageKey: 'nuxt-color-mode',
    classSuffix: '',
    dataValue: 'theme'
  },
})