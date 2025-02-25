import type { ApiLogReponse } from '../types/response/logs'

export const useLogs = () => {
  const logs = ref<ApiLogReponse[]>([])
  const log = ref<ApiLogReponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const totalLogs = ref<number>(0) // Add totalLogs ref

  const fetchLogs = async (page: number = 1, pageSize: number = 20) => {
    try {
      loading.value = true
      logs.value = await $fetch<ApiLogReponse[]>('/api/v1/ui/logs', {
        query: {
          page: page.toString(),
          page_size: pageSize.toString(),
        },
      })
    } catch (err) {
      console.error(err)
      error.value = 'failed to fetch logs'
    } finally {
      loading.value = false
    }
  }

  const fetchLogById = async (id: number) => {
    try {
      loading.value = true
      log.value = await $fetch<ApiLogReponse>(`/api/v1/ui/logs/${id}`)
    } catch (err) {
      console.error(err)
      error.value = `failed to fetch log with id ${id}`
    } finally {
      loading.value = false
    }
  }

  const fetchLogsCount = async () => {
    try {
      loading.value = true
      const response = await $fetch<{ count: number }>('/api/v1/ui/logs/count')
      totalLogs.value = response.count
    } catch (err) {
      console.error(err)
      error.value = 'failed to fetch logs count'
    } finally {
      loading.value = false
    }
  }

  return {
    logs,
    log,
    loading,
    error,
    fetchLogs,
    fetchLogById,
    fetchLogsCount,
    totalLogs,
  }
}
