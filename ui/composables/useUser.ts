import type { User } from '../types/response/user'

export const useUser = () => {
  const user = ref<User | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const isLoggedIn = ref(false)
  const router = useRouter()

  const fetchCurrentUser = async () => {
    try {
      loading.value = true
      error.value = null
      const data = await $fetch<User>('/v1/ui/auth/me', {
        credentials: 'include' // Important for cookies
      })
      user.value = data
      isLoggedIn.value = true
    } catch (err) {
      console.error('Failed to fetch current user:', err)
      error.value = 'Failed to fetch user information'
      isLoggedIn.value = false
      user.value = null
    } finally {
      loading.value = false
    }
  }

  const logout = async () => {
    try {
      loading.value = true
      await $fetch('/api/auth/logout', { 
        method: 'POST',
        credentials: 'include' // Important for cookies
      })
      // Clear user data
      user.value = null
      isLoggedIn.value = false
      // Redirect to login page
      router.push('/login')
    } catch (err) {
      console.error('Failed to logout:', err)
      error.value = 'Failed to logout'
    } finally {
      loading.value = false
    }
  }

  // Initialize on first load
  onMounted(async () => {
    await fetchCurrentUser()
  })

  return {
    user,
    loading,
    error,
    isLoggedIn,
    fetchCurrentUser,
    logout
  }
}