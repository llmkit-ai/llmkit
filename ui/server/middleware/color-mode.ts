export default defineEventHandler((event) => {
  const colorMode = parseCookies(event)['nuxt-color-mode'] || 'system'
  event.context.$colorMode = colorMode
})
