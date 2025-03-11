import type { Tool } from '../types/response/tools'

export interface CreateToolPayload {
  name: string
  tool_name: string
  description: string
  parameters: string
  strict: boolean
}

export interface UpdateToolPayload {
  name: string
  tool_name: string
  description: string
  parameters: string
  strict: boolean
}

export interface AssociateToolPromptPayload {
  tool_id: number
  prompt_version_id: number
}

export const useTools = () => {
  const tools = ref<Tool[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchTools = async () => {
    try {
      loading.value = true
      tools.value = await $fetch<Tool[]>('/v1/ui/tools')
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch tools'
    } finally {
      loading.value = false
    }
  }

  const getToolById = async (id: number) => {
    try {
      loading.value = true
      return await $fetch<Tool>(`/v1/ui/tools/${id}`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch tool'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createTool = async (payload: CreateToolPayload) => {
    try {
      loading.value = true
      const newTool = await $fetch<Tool>('/v1/ui/tools', {
        method: 'POST',
        body: payload,
      })
      tools.value = [...tools.value, newTool]
      error.value = null
      return newTool
    } catch (err: any) {
      console.error('Failed to create tool:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to create tool'
      error.value = errorMessage
      throw err // Rethrow for component level handling
    } finally {
      loading.value = false
    }
  }

  const updateTool = async (id: number, payload: UpdateToolPayload) => {
    try {
      loading.value = true
      const updatedTool = await $fetch<Tool>(`/v1/ui/tools/${id}`, {
        method: 'PUT',
        body: payload,
      })
      tools.value = tools.value.map(tool => tool.id === id ? updatedTool : tool)
      error.value = null
      return updatedTool
    } catch (err: any) {
      console.error('Failed to update tool:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to update tool'
      error.value = errorMessage
      throw err // Rethrow for component level handling
    } finally {
      loading.value = false
    }
  }

  const deleteTool = async (id: number) => {
    try {
      loading.value = true
      await $fetch(`/v1/ui/tools/${id}`, {
        method: 'DELETE',
      })
      tools.value = tools.value.filter(tool => tool.id !== id)
      error.value = null
    } catch (err: any) {
      console.error('Failed to delete tool:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to delete tool'
      error.value = errorMessage
      throw err
    } finally {
      loading.value = false
    }
  }

  const associateToolWithPromptVersion = async (payload: AssociateToolPromptPayload) => {
    try {
      loading.value = true
      await $fetch('/v1/ui/tools/associate', {
        method: 'POST',
        body: payload,
      })
      error.value = null
    } catch (err: any) {
      console.error('Failed to associate tool with prompt version:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to associate tool with prompt version'
      error.value = errorMessage
      throw err
    } finally {
      loading.value = false
    }
  }

  const removeToolPromptAssociation = async (payload: AssociateToolPromptPayload) => {
    try {
      loading.value = true
      await $fetch('/v1/ui/tools/disassociate', {
        method: 'POST',
        body: payload,
      })
      error.value = null
    } catch (err: any) {
      console.error('Failed to remove tool prompt association:', err)
      const errorMessage = err?.data?.message || err?.message || 'Failed to remove tool prompt association'
      error.value = errorMessage
      throw err
    } finally {
      loading.value = false
    }
  }

  const getPromptVersionsByTool = async (toolId: number) => {
    try {
      loading.value = true
      return await $fetch<number[]>(`/v1/ui/tools/${toolId}/prompts`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch prompt versions'
      throw err
    } finally {
      loading.value = false
    }
  }

  const getToolsByPromptVersion = async (promptVersionId: number) => {
    try {
      loading.value = true
      return await $fetch<Tool[]>(`/v1/ui/prompts/versions/${promptVersionId}/tools`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch tools'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    tools,
    loading,
    error,
    fetchTools,
    getToolById,
    createTool,
    updateTool,
    deleteTool,
    associateToolWithPromptVersion,
    removeToolPromptAssociation,
    getPromptVersionsByTool,
    getToolsByPromptVersion
  }
}