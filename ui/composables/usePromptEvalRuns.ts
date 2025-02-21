import type { PromptEvalExecutionRunResponse, PromptEvalRunResponse } from '~/types/response/prompt_eval_runs'

export const usePromptEvalRuns = () => {
  const evalRuns = ref<PromptEvalRunResponse[]>([])
  const currentEvalRun = ref<PromptEvalRunResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchEvalRunById = async (id: number) => {
    try {
      loading.value = true
      currentEvalRun.value = await $fetch<PromptEvalRunResponse>(`/api/v1/prompt-eval-runs/${id}`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch sample'
    } finally {
      loading.value = false
    }
  }

  const fetchEvalRunsByPromptVersion = async (promptId: number, promptVersionId: number) => {
    try {
      loading.value = true
      evalRuns.value = await $fetch<PromptEvalRunResponse[]>(`/api/v1/prompt-eval-runs/${promptId}/version/${promptVersionId}`)
    } catch (err) {
      console.error(err)
      error.value = 'Failed to fetch samples'
    } finally {
      loading.value = false
    }
  }

  // const fetchEvalRunsByRunId = async (promptId: number, promptVersionId: number) => {
  //   try {
  //     loading.value = true
  //     evals.value = await $fetch<PromptEvalRunResponse[]>(`/api/v1/prompt-eval-runs/${promptId}/version/${promptVersionId}`)
  //   } catch (err) {
  //     console.error(err)
  //     error.value = 'Failed to fetch samples'
  //   } finally {
  //     loading.value = false
  //   }
  // }

  const createEvalRun = async (promptId: number, promptVersionId: number) => {
    try {
      const newEval = await $fetch<PromptEvalExecutionRunResponse>(`/api/v1/prompt-eval-runs/${promptId}/version/${promptVersionId}`, {
        method: 'POST',
      })

      newEval.runs.forEach(r => {
        evalRuns.value.push(r)
      })

      loading.value = false
      return newEval
    } catch (err) {
      error.value = 'Failed to create sample'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateEvalRunScore = async (id: number, score: number) => {
    try {
      const updatedEval = await $fetch<PromptEvalRunResponse>(`/api/v1/prompt-eval-runs/${id}`, {
        method: 'PUT',
        body: {
          id,
          score
        }
      })
      
      return updatedEval
    } catch (err) {
      error.value = 'Failed to update sample'
      throw err
    }
  }

  return {
    evalRuns,
    currentEvalRun,
    loading,
    error,
    fetchEvalRunById,
    fetchEvalRunByPromptVersion: fetchEvalRunsByPromptVersion,
    createEvalRun,
    updateEvalRunScore
  }
}
