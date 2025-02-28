export default defineNuxtRouteMiddleware(async (to, from) => {
  // Skip middleware for login and register pages
  if (to.path === '/login' || to.path === '/register') {
    return
  }

  try {
    const headers = useRequestHeaders(['cookie'])
    await $fetch('/v1/ui/auth/me', {
      headers
    })
  } catch(e) {
    console.error(e)
    return navigateTo("/login")
  }
})
