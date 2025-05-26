import type { PromptEvalExecutionRunResponse, PromptEvalRunResponse } from '~/types/response/prompt_eval_runs'

export const usePromptEvalRuns = () => {
  const evalRuns = ref<PromptEvalRunResponse[]>([])
  const currentEvalRun = ref<PromptEvalRunResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchEvalRunById = async (id: number) => {
    try {
      loading.value = true
      currentEvalRun.value = await $fetch<PromptEvalRunResponse>(`/v1/ui/prompt-eval-runs/${id}`)
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
      const runs = await $fetch<PromptEvalRunResponse[]>(`/v1/ui/prompt-eval-runs/${promptId}/version/${promptVersionId}`)
      
      // Group by run_id and assign round numbers
      const runIdGroups = new Map<string, PromptEvalRunResponse[]>()
      runs.forEach(run => {
        if (!runIdGroups.has(run.run_id)) {
          runIdGroups.set(run.run_id, [])
        }
        runIdGroups.get(run.run_id)!.push(run)
      })
      
      // Assign round numbers based on creation time of run_ids
      const sortedRunIds = Array.from(runIdGroups.keys()).sort((a, b) => {
        const aFirstRun = runIdGroups.get(a)![0]
        const bFirstRun = runIdGroups.get(b)![0]
        return new Date(aFirstRun.created_at).getTime() - new Date(bFirstRun.created_at).getTime()
      })
      
      evalRuns.value = []
      sortedRunIds.forEach((runId, roundIndex) => {
        const runsInGroup = runIdGroups.get(runId)!
        runsInGroup.forEach(run => {
          evalRuns.value.push({
            ...run,
            round_number: roundIndex + 1
          })
        })
      })
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
  //     evals.value = await $fetch<PromptEvalRunResponse[]>(`/v1/ui/prompt-eval-runs/${promptId}/version/${promptVersionId}`)
  //   } catch (err) {
  //     console.error(err)
  //     error.value = 'Failed to fetch samples'
  //   } finally {
  //     loading.value = false
  //   }
  // }

  const createEvalRun = async (promptId: number, promptVersionId: number, rounds: number) => {
    try {
      const newEval = await $fetch<PromptEvalExecutionRunResponse[]>(`/v1/ui/prompt-eval-runs/${promptId}/version/${promptVersionId}?rounds=${rounds}`, {
        method: 'POST',
      })

      newEval.forEach((group, roundIndex) => {
        group.runs.forEach(r => {
          // Add round information to each run
          const runWithRound = {
            ...r,
            round_number: roundIndex + 1,
            run_id: group.run_id
          }
          evalRuns.value.push(runWithRound)
        })
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
      const updatedEval = await $fetch<PromptEvalRunResponse>(`/v1/ui/prompt-eval-runs/${id}`, {
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
