import type { ApiTraceResponse } from '../types/response/logs';

export const useLogs = () => {
  const logs = ref<ApiTraceResponse[]>([]);
  const log = ref<ApiTraceResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const fetchLogs = async () => {
    try {
      loading.value = true;
      logs.value = await $fetch<ApiTraceResponse[]>('/api/v1/logs');
    } catch (err) {
      console.error(err);
      error.value = 'failed to fetch logs';
    } finally {
      loading.value = false;
    }
  };

  const fetchLogById = async (id: number) => {
    try {
      loading.value = true;
      log.value = await $fetch<ApiTraceResponse>(`/api/v1/logs/${id}`);
    } catch (err) {
      console.error(err);
      error.value = `failed to fetch log with id ${id}`;
    } finally {
      loading.value = false;
    }
  };

  return {
    logs,
    log,
    loading,
    error,
    fetchLogs,
    fetchLogById,
  };
};
