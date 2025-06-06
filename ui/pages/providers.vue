<template>
  <div class="font-mono pl-12">
    <div class="flex w-full items-center justify-between mb-6">
      <h1 class="text-xl font-semibold text-neutral-900 dark:text-white">Providers</h1>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="mt-4 animate-pulse space-y-2">
      <div v-for="i in 3" :key="i" class="h-16 w-full rounded bg-neutral-200 dark:bg-neutral-800"></div>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="mt-4 rounded border-2 border-red-500 bg-red-100 p-4 text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {{ error }}
    </div>

    <!-- Providers list -->
    <div v-else class="mt-4">
      <div class="border border-neutral-200 dark:border-neutral-700">
        <table class="min-w-full divide-y divide-neutral-100 dark:divide-neutral-700">
          <thead class="bg-neutral-100 dark:bg-neutral-800">
            <tr>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Provider
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Base URL
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Status
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white dark:bg-neutral-900 divide-y divide-neutral-100 dark:divide-neutral-700">
            <tr v-for="provider in providers" :key="provider.id" class="hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 font-medium text-neutral-700 dark:text-neutral-300">
                {{ provider.name.toUpperCase() }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 text-neutral-500 dark:text-neutral-400">
                <span v-if="provider.base_url" class="font-mono text-xs">{{ provider.base_url }}</span>
                <span v-else class="text-neutral-400 dark:text-neutral-600 italic">Not configured</span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span v-if="provider.is_available" class="inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-800 dark:bg-green-900/20 dark:text-green-400">
                  Available
                </span>
                <span v-else class="inline-flex items-center rounded-full bg-red-100 px-2 py-0.5 text-xs text-red-800 dark:bg-red-900/20 dark:text-red-400">
                  Not Available
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <PrimaryButton 
                  @click="editProvider(provider)" 
                  buttonType="secondary"
                  size="xs"
                >
                  Configure
                </PrimaryButton>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Configuration help -->
      <div class="mt-6 bg-teal-50 dark:bg-teal-900/20 border border-teal-200 dark:border-teal-800 p-4">
        <h3 class="text-sm font-semibold text-teal-900 dark:text-teal-200 mb-2">Configuration Requirements</h3>
        <ul class="space-y-1 text-sm text-teal-800 dark:text-teal-300">
          <li>• <strong>OpenAI:</strong> Set <code class="px-1 py-0.5 bg-teal-100 dark:bg-teal-800">OPENAI_API_KEY</code> environment variable</li>
          <li>• <strong>OpenRouter:</strong> Set <code class="px-1 py-0.5 bg-teal-100 dark:bg-teal-800">OPENROUTER_API_KEY</code> environment variable</li>
          <li>• <strong>Azure:</strong> Set <code class="px-1 py-0.5 bg-teal-100 dark:bg-teal-800">AZURE_API_KEY</code> environment variable and configure base URL</li>
        </ul>
      </div>
    </div>

    <!-- Edit Provider Modal -->
    <div 
      v-if="showEditModal" 
      class="fixed inset-0 z-50 overflow-y-auto" 
      aria-labelledby="modal-title" 
      role="dialog" 
      aria-modal="true"
    >
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-neutral-500 bg-opacity-75 transition-opacity" aria-hidden="true"></div>
        
        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>
        <div class="inline-block align-bottom bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-6 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div>
            <div class="mt-3 text-center sm:mt-0 sm:text-left">
              <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white" id="modal-title">
                Configure {{ selectedProvider?.name.toUpperCase() }} Provider
              </h3>
              <div class="mt-4">
                <form @submit.prevent="handleSubmit" class="space-y-4">
                  <div>
                    <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
                      Base URL
                      <span v-if="selectedProvider?.name === 'azure'" class="ml-1 text-red-500">*</span>
                    </label>
                    <input 
                      v-model="baseUrl" 
                      :required="selectedProvider?.name === 'azure'"
                      :placeholder="getPlaceholder()"
                      class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white font-mono text-sm"
                      type="text"
                    />
                    <p v-if="selectedProvider?.name === 'azure'" class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
                      Example: https://your-resource.openai.azure.com/
                    </p>
                  </div>
                  
                  <!-- Form error message -->
                  <div v-if="formError" class="mt-4 rounded border-2 border-red-500 bg-red-100 p-3 text-red-700 dark:bg-red-900/20 dark:text-red-400">
                    {{ formError }}
                  </div>
                  
                  <div class="mt-6 flex justify-end space-x-3">
                    <PrimaryButton 
                      type="button"
                      @click="closeModal"
                      buttonType="secondary"
                      size="sm"
                    >
                      Cancel
                    </PrimaryButton>
                    <PrimaryButton 
                      htmlType="submit" 
                      :loading="formLoading"
                      :disabled="formLoading"
                      buttonType="primary"
                      size="sm"
                    >
                      {{ formLoading ? 'Saving...' : 'Save' }}
                    </PrimaryButton>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useProviders } from '~/composables/useProviders'
import type { Provider } from '~/types/response/providers'
import PrimaryButton from '~/components/global/primary-button.vue'

definePageMeta({
  middleware: ['auth'],
  layout: 'logged-in'
})

const { providers, loading, error, fetchProviders, updateProvider } = useProviders()

const showEditModal = ref(false)
const selectedProvider = ref<Provider | null>(null)
const baseUrl = ref('')
const formLoading = ref(false)
const formError = ref<string | null>(null)

onMounted(async () => {
  await fetchProviders()
})

function editProvider(provider: Provider) {
  selectedProvider.value = provider
  baseUrl.value = provider.base_url || ''
  showEditModal.value = true
}

function closeModal() {
  showEditModal.value = false
  selectedProvider.value = null
  baseUrl.value = ''
  formError.value = null
}

function getPlaceholder() {
  if (!selectedProvider.value) return ''
  
  switch (selectedProvider.value.name) {
    case 'openai':
      return 'https://api.openai.com/v1'
    case 'openrouter':
      return 'https://openrouter.ai/api/v1'
    case 'azure':
      return 'https://your-resource.openai.azure.com/'
    default:
      return ''
  }
}

async function handleSubmit() {
  console.log('handleSubmit called')
  if (!selectedProvider.value) return
  
  formLoading.value = true
  formError.value = null
  
  try {
    // Azure requires base URL, others can be null/empty
    const finalBaseUrl = baseUrl.value.trim() || null
    
    if (selectedProvider.value.name === 'azure' && !finalBaseUrl) {
      formError.value = 'Base URL is required for Azure provider'
      return
    }
    
    await updateProvider(selectedProvider.value.id, finalBaseUrl)
    await fetchProviders() // Refresh the list
    closeModal()
  } catch (err: any) {
    console.error('Error updating provider:', err)
    formError.value = err?.data?.message || err?.message || 'Failed to update provider. Please try again.'
  } finally {
    formLoading.value = false
  }
}
</script>
