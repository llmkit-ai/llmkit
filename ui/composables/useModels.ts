import type { Model } from '../types/response/models'

export const useModels = () => {
  const models = ref<Model[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchModels = async () => {
    try {
      loading.value = true
      models.value = await $fetch<Model[]>('/v1/ui/models')
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch models'
    } finally {
      loading.value = false
    }
  }

  return {
    models,
    loading,
    error,
    fetchModels,
  }
}
