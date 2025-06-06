import type { Provider } from '~/types/response/providers';

export const useProviders = () => {
  const providers = ref<Provider[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const fetchProviders = async () => {
    loading.value = true;
    error.value = null;
    
    try {
      const data = await $fetch<Provider[]>('/v1/ui/providers', {
        method: 'GET',
      });
      
      providers.value = data;
    } catch (err: any) {
      error.value = err.data?.message || 'Failed to fetch providers';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const updateProvider = async (id: number, baseUrl: string | null) => {
    loading.value = true;
    error.value = null;
    
    try {
      const data = await $fetch<Provider>(`/v1/ui/providers/${id}`, {
        method: 'PUT',
        body: {
          base_url: baseUrl,
        },
      });
      
      // Update the provider in the local list
      const index = providers.value.findIndex(p => p.id === id);
      if (index !== -1) {
        providers.value[index] = data;
      }
      
      return data;
    } catch (err: any) {
      error.value = err.data?.message || 'Failed to update provider';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  return {
    providers,
    loading,
    error,
    fetchProviders,
    updateProvider,
  };
};