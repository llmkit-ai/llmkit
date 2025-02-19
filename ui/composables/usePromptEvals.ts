import type { 
  PromptEvalResponse 
} from '~/types/response/prompt_eval'

import type { 
  CreatePromptEvalRequest, 
  UpdatePromptEvalRequest 
} from '~/types/request/prompt_eval'

export const usePromptEvals = () => {
  const evals = ref<PromptEvalResponse[]>([])
  const currentEval = ref<PromptEvalResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchEvalById = async (id: number) => {
    try {
      loading.value = true
      currentEval.value = await $fetch<PromptEvalResponse>(`/api/v1/prompt-evals/${id}`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch sample'
    } finally {
      loading.value = false
    }
  }

  const fetchEvalByPrompt = async (promptId: number) => {
    try {
      loading.value = true
      evals.value = await $fetch<PromptEvalResponse[]>(`/api/v1/prompts/${promptId}/prompt-evals`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch samples'
    } finally {
      loading.value = false
    }
  }

  const createEval = async (sampleData: CreatePromptEvalRequest) => {
    try {
      const newEval = await $fetch<PromptEvalResponse>('/api/v1/prompt-evals', {
        method: 'POST',
        body: sampleData
      })
      evals.value.push(newEval)
      return newEval
    } catch (err) {
      error.value = 'Failed to create sample'
      throw err
    }
  }

  const updateEval = async (id: number, updates: UpdatePromptEvalRequest) => {
    try {
      const updatedEval = await $fetch<PromptEvalResponse>(`/api/v1/prompt-evals/${id}`, {
        method: 'PUT',
        body: updates
      })
      
      const index = evals.value.findIndex(s => s.id === id)
      if (index !== -1) evals.value[index] = updatedEval
      if (currentEval.value?.id === id) currentEval.value = updatedEval
      
      return updatedEval
    } catch (err) {
      error.value = 'Failed to update sample'
      throw err
    }
  }

  const deleteEval = async (id: number) => {
    try {
      await $fetch(`/api/v1/prompt-evals/${id}`, { method: 'DELETE' })
      evals.value = evals.value.filter(s => s.id !== id)
      if (currentEval.value?.id === id) currentEval.value = null
    } catch (err) {
      error.value = 'Failed to delete sample'
      throw err
    }
  }

  return {
    evals,
    currentEval,
    loading,
    error,
    fetchEvalById,
    fetchEvalByPrompt,
    createEval,
    updateEval,
    deleteEval
  }
}
