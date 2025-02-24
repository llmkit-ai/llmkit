import type { PromptCreateDTO, PromptUpdateDTO } from '~/types/components/prompt'
import type { Message, Prompt, PromptEvalVersionPerformanceResponse, PromptExecutionResponse } from '../types/response/prompts'

export const usePrompts = () => {
  const prompts = ref<Prompt[]>([])
  const promptPerformance = ref<PromptEvalVersionPerformanceResponse[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchPrompts = async () => {
    try {
      loading.value = true
      prompts.value = await $fetch<Prompt[]>('/api/v1/prompts')
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch prompts'
    } finally {
      loading.value = false
    }
  }

  const getPromptPerformance = async (promptId: number) => {
    try {
      promptPerformance.value = await $fetch<PromptEvalVersionPerformanceResponse[]>(`/api/v1/prompts/${promptId}/performance`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch performance'
    } finally {
    }
  }

  const createPrompt = async (prompt: PromptCreateDTO) => {
    try {
      const newPrompt = await $fetch<Prompt>('/api/v1/prompts', {
        method: 'POST',
        body: {
          key: prompt.key,
          system: prompt.system,
          user: prompt.user,
          model_id: prompt.model_id,
          max_tokens: prompt.max_tokens,
          temperature: prompt.temperature,
          json_mode: prompt.json_mode,
          prompt_type: prompt.prompt_type,
          is_chat: prompt.is_chat
        }
      })
      prompts.value.push(newPrompt)
      return newPrompt
    } catch (err) {
      error.value = 'Failed to create prompt'
      throw err
    }
  }

  const updatePrompt = async (id: number, prompt: PromptUpdateDTO) => {
    try {
      const updatedPrompt = await $fetch<Prompt>(`/api/v1/prompts/${id}`, {
        method: 'PUT',
        body: {
          key: prompt.key,
          system: prompt.system,
          user: prompt.user,
          model_id: prompt.model_id,
          max_tokens: prompt.max_tokens,
          temperature: prompt.temperature,
          json_mode: prompt.json_mode,
          prompt_type: prompt.prompt_type,
          is_chat: prompt.is_chat
        }
      })
      const index = prompts.value.findIndex(p => p.id === id)
      if (index !== -1) prompts.value[index] = updatedPrompt
      return updatedPrompt
    } catch (err) {
      error.value = 'Failed to update prompt'
      throw err
    }
  }

  const deletePrompt = async (id: number) => {
    try {
      await $fetch(`/api/v1/prompts/${id}`, { method: 'DELETE' })
      prompts.value = prompts.value.filter(p => p.id !== id)
    } catch (err) {
      error.value = 'Failed to delete prompt'
      throw err
    }
  }

  const executePrompt = async (id: number, body: any) => {
    try {
      return await $fetch<PromptExecutionResponse>(`/api/v1/prompts/execute/${id}`, { 
        method: 'POST',
        body 
      })
    } catch (err) {
      error.value = 'Failed to execute prompt'
      throw err
    }
  }

  const executePromptStream = async (id: number, body: any) => {
    try {
      return await $fetch<string>(`/api/v1/prompts/execute/${id}/stream`, { 
        method: 'POST',
        body 
      })
    } catch (err) {
      error.value = 'Failed to execute prompt'
      throw err
    }
  }

  // Chat execution (non-streaming)
  const executeChat = async (id: number, context: Record<string, any> = {}, messages: Message[]) => {
    try {
      return await $fetch<PromptExecutionResponse>(`/api/v1/prompts/execute/${id}/chat`, { 
        method: 'POST',
        body: {
          context,
          messages
        }
      })
    } catch (err) {
      error.value = 'Failed to execute chat prompt'
      throw err
    }
  }

  return {
    prompts,
    promptPerformance,
    loading,
    error,
    fetchPrompts,
    getPromptPerformance,
    createPrompt,
    updatePrompt,
    deletePrompt,
    executePrompt,
    executePromptStream,
    executeChat
  }
}
