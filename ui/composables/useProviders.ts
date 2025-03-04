import type { Provider } from '../types/response/providers'

export const useProviders = () => {
  const providers = ref<Provider[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchProviders = async () => {
    try {
      loading.value = true
      providers.value = await $fetch<Provider[]>('/v1/ui/providers')
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch providers'
    } finally {
      loading.value = false
    }
  }

  return {
    providers,
    loading,
    error,
    fetchProviders,
  }
}