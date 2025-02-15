import type { 
  PromptSampleResponse 
} from '~/types/response/prompt_samples'

import type { 
  CreatePromptSampleRequest, 
  UpdatePromptSampleRequest 
} from '~/types/request/prompt_samples'

export const usePromptSamples = () => {
  const samples = ref<PromptSampleResponse[]>([])
  const currentSample = ref<PromptSampleResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchSampleById = async (id: number) => {
    try {
      loading.value = true
      currentSample.value = await $fetch<PromptSampleResponse>(`/api/v1/prompt-samples/${id}`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch sample'
    } finally {
      loading.value = false
    }
  }

  const fetchSamplesByPrompt = async (promptId: number) => {
    try {
      loading.value = true
      samples.value = await $fetch<PromptSampleResponse[]>(`/api/v1/prompts/${promptId}/prompt-samples`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch samples'
    } finally {
      loading.value = false
    }
  }

  const createSample = async (sampleData: CreatePromptSampleRequest) => {
    try {
      const newSample = await $fetch<PromptSampleResponse>('/api/v1/prompt-samples', {
        method: 'POST',
        body: sampleData
      })
      samples.value.push(newSample)
      return newSample
    } catch (err) {
      error.value = 'Failed to create sample'
      throw err
    }
  }

  const updateSample = async (id: number, updates: UpdatePromptSampleRequest) => {
    try {
      const updatedSample = await $fetch<PromptSampleResponse>(`/api/v1/prompt-samples/${id}`, {
        method: 'PUT',
        body: updates
      })
      
      const index = samples.value.findIndex(s => s.id === id)
      if (index !== -1) samples.value[index] = updatedSample
      if (currentSample.value?.id === id) currentSample.value = updatedSample
      
      return updatedSample
    } catch (err) {
      error.value = 'Failed to update sample'
      throw err
    }
  }

  const deleteSample = async (id: number) => {
    try {
      await $fetch(`/api/v1/prompt-samples/${id}`, { method: 'DELETE' })
      samples.value = samples.value.filter(s => s.id !== id)
      if (currentSample.value?.id === id) currentSample.value = null
    } catch (err) {
      error.value = 'Failed to delete sample'
      throw err
    }
  }

  return {
    samples,
    currentSample,
    loading,
    error,
    fetchSampleById,
    fetchSamplesByPrompt,
    createSample,
    updateSample,
    deleteSample
  }
}
