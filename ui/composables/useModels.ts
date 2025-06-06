import type { Model } from '../types/response/models'

export interface CreateModelPayload {
  name: string
  provider_id: number
  supports_json: boolean
  supports_json_schema: boolean
  supports_tools: boolean
  is_reasoning: boolean
}

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

  const createModel = async (payload: CreateModelPayload) => {
    try {
      loading.value = true
      const newModel = await $fetch<Model>('/v1/ui/models', {
        method: 'POST',
        body: payload,
      })
      models.value = [...models.value, newModel]
      error.value = null
      return newModel
    } catch (err: any) {
      console.error('Failed to create model:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to create model'
      error.value = errorMessage
      throw err // Rethrow for component level handling
    } finally {
      loading.value = false
    }
  }

  const updateModel = async (id: number, payload: CreateModelPayload) => {
    try {
      loading.value = true
      const updatedModel = await $fetch<Model>(`/v1/ui/models/${id}`, {
        method: 'PUT',
        body: payload,
      })
      models.value = models.value.map(model => model.id === id ? updatedModel : model)
      error.value = null
      return updatedModel
    } catch (err: any) {
      console.error('Failed to update model:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to update model'
      error.value = errorMessage
      throw err // Rethrow for component level handling
    } finally {
      loading.value = false
    }
  }

  return {
    models,
    loading,
    error,
    fetchModels,
    createModel,
    updateModel
  }
}
