// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: true },
  modules: ['@nuxtjs/tailwindcss'],
  nitro: {
    routeRules: {
      "/api/v1/**": { proxy: 'http://127.0.0.1:8000/api/v1/**' },
    }
  },
})
