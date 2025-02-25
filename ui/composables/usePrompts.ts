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
      prompts.value = await $fetch<Prompt[]>('/api/v1/ui/prompts')
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch prompts'
    } finally {
      loading.value = false
    }
  }

  const getPromptPerformance = async (promptId: number) => {
    try {
      promptPerformance.value = await $fetch<PromptEvalVersionPerformanceResponse[]>(`/api/v1/ui/prompts/${promptId}/performance`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch performance'
    } finally {
    }
  }

  const createPrompt = async (prompt: PromptCreateDTO) => {
    try {
      const newPrompt = await $fetch<Prompt>('/api/v1/ui/prompts', {
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
      const updatedPrompt = await $fetch<Prompt>(`/api/v1/ui/prompts/${id}`, {
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
      await $fetch(`/api/v1/ui/prompts/${id}`, { method: 'DELETE' })
      prompts.value = prompts.value.filter(p => p.id !== id)
    } catch (err) {
      error.value = 'Failed to delete prompt'
      throw err
    }
  }

  const executePrompt = async (id: number, body: any) => {
    try {
      return await $fetch<PromptExecutionResponse>(`/api/v1/ui/prompts/execute/${id}`, { 
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
      return await $fetch<string>(`/api/v1/ui/prompts/execute/${id}/stream`, { 
        method: 'POST',
        body 
      })
    } catch (err) {
      error.value = 'Failed to execute prompt'
      throw err
    }
  }

  // Chat execution (non-streaming) - using OpenAI compatible API
  const executeChat = async (id: number, context: Record<string, any> = {}, messages: Message[]) => {
    try {
      // Get the prompt to retrieve its key
      const prompt = prompts.value.find(p => p.id === id)
      if (!prompt) {
        throw new Error(`Prompt with ID ${id} not found`)
      }
      
      // Prepare system message with context if present
      let messagesWithContext = [...messages]
      if (Object.keys(context).length > 0) {
        // Check if there's already a system message
        const hasSystemMessage = messagesWithContext.some(msg => msg.role === 'system')
        
        if (!hasSystemMessage) {
          messagesWithContext.unshift({
            role: 'system',
            content: JSON.stringify(context)
          })
        } else {
          // Replace the existing system message with context
          const systemIndex = messagesWithContext.findIndex(msg => msg.role === 'system')
          if (systemIndex !== -1) {
            messagesWithContext[systemIndex] = {
              role: 'system',
              content: JSON.stringify(context)
            }
          }
        }
      }
      
      // Call the OpenAI compatible API
      const response = await $fetch(`/api/v1/ui/prompts/execute/chat`, { 
        method: 'POST',
        body: {
          model: prompt.key,
          messages: messagesWithContext
        }
      })
      
      // Extract and format the response to match the legacy PromptExecutionResponse format
      if (response.choices && response.choices.length > 0) {
        const content = response.choices[0].message.content
        
        // Construct a compatible response
        return {
          content,
          log: {
            id: parseInt(response.id.split('-')[1]) || 0,
            prompt_id: id,
            model_id: prompt.model_id,
            model_name: prompt.model,
            status_code: 200,
            input_tokens: response.usage?.prompt_tokens || 0,
            output_tokens: response.usage?.completion_tokens || 0,
            created_at: new Date().toISOString()
          }
        } as PromptExecutionResponse
      }
      
      throw new Error('Invalid response from chat API')
    } catch (err) {
      error.value = 'Failed to execute chat prompt'
      throw err
    }
  }

  // OpenAI-compatible API execution
  const executeApiCompletion = async (
    modelKey: string, 
    messages: Message[], 
    jsonMode: boolean = false
  ) => {
    try {
      const requestBody: any = {
        model: modelKey,
        messages
      };
      
      // Add JSON mode if required
      if (jsonMode) {
        requestBody.response_format = {
          type: "json_object"
        };
      }
      
      // Use the OpenAI-compatible API endpoint
      return await $fetch<ApiCompletionResponse>('/api/v1/ui/prompts/execute', {
        method: 'POST',
        body: requestBody
      });
    } catch (err) {
      error.value = 'Failed to execute API completion'
      throw err
    }
  }

  // OpenAI-compatible API streaming (no direct return value since it streams)
  const executeApiCompletionStream = async (
    modelKey: string, 
    messages: Message[], 
    jsonMode: boolean = false,
    onChunk: (chunk: string) => void,
    onError: (err: any) => void
  ) => {
    try {
      const requestBody: any = {
        model: modelKey,
        messages,
        stream: true
      };
      
      // Add JSON mode if required
      if (jsonMode) {
        requestBody.response_format = {
          type: "json_object"
        };
      }
      
      // Use the SSE implementation for streaming
      const { startStream } = useSSE();
      await startStream(
        requestBody,
        `/api/v1/ui/prompts/execute/stream`,
        {
          onMessage: onChunk,
          onError
        }
      );
    } catch (err) {
      error.value = 'Failed to execute API streaming'
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
    executeChat,
    executeApiCompletion,
    executeApiCompletionStream
  }
}
