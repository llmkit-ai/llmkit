<template>
  <div class="font-mono">
    <div class="flex w-full items-center justify-between mb-6">
      <h1 class="text-xl font-semibold text-neutral-900 dark:text-white">Models</h1>
      <PrimaryButton @click="showAddModelModal = true" buttonType="primary" size="sm">
        Add Model
      </PrimaryButton>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="mt-4 animate-pulse space-y-2">
      <div v-for="i in 3" :key="i" class="h-16 w-full rounded bg-neutral-200 dark:bg-neutral-800"></div>
    </div>

    <!-- Error state -->
    <div v-else-if="modelsError" class="mt-4 rounded border-2 border-red-500 bg-red-100 p-4 text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {{ modelsError }}
    </div>

    <!-- Empty state -->
    <div v-else-if="!models.length" class="mt-4 flex flex-col items-center justify-center rounded border-2 border-dashed border-neutral-400 bg-neutral-100 p-8 text-center dark:border-neutral-700 dark:bg-neutral-800/50">
      <svg class="mb-2 size-8 text-neutral-500 dark:text-neutral-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 6C20.0681 6 19.1629 5.59203 18.5 4.9L18 4.5C17.3371 3.80797 16.4319 3.5 15.5 3.5H8.5C7.56812 3.5 6.66286 3.80797 6 4.5L5.5 4.9C4.83714 5.59203 3.93188 6 3 6" />
        <path d="M21 9V15C21 16.8856 21 17.8284 20.4142 18.4142C19.8284 19 18.8856 19 17 19H7C5.11438 19 4.17157 19 3.58579 18.4142C3 17.8284 3 16.8856 3 15V9C3 8.06812 3.40797 7.16286 4.1 6.5L4.5 6C5.19203 5.33714 6.09728 5 7.02916 5H16.9708C17.9027 5 18.808 5.33714 19.5 6L19.9 6.5C20.592 7.16286 21 8.06812 21 9Z" />
        <path d="M7 9H17V15C17 16.1046 16.1046 17 15 17H9C7.89543 17 7 16.1046 7 15V9Z" />
      </svg>
      <p class="text-neutral-600 dark:text-neutral-400">No models added yet</p>
      <p class="mt-2 text-sm text-neutral-500 dark:text-neutral-500">Add a new model by clicking the button above.</p>
    </div>

    <!-- Models list -->
    <div v-else class="mt-4">
      <div class="border border-neutral-200 dark:border-neutral-700">
        <table class="min-w-full divide-y divide-neutral-100 dark:divide-neutral-700">
          <thead class="bg-neutral-100 dark:bg-neutral-800">
            <tr>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Model Name
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Provider
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Capabilities
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white dark:bg-neutral-900 divide-y divide-neutral-100 dark:divide-neutral-700">
            <tr v-for="model in models" :key="model.id" class="hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 font-medium text-neutral-700 dark:text-neutral-300">
                {{ model.name }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 text-neutral-500 dark:text-neutral-400">
                {{ model.provider_name }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 text-neutral-500 dark:text-neutral-400">
                <div class="flex space-x-2">
                  <span v-if="model.supports_json" class="inline-flex items-center rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-800 dark:bg-blue-900/20 dark:text-blue-400">
                    JSON
                  </span>
                  <span v-if="model.supports_tools" class="inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-800 dark:bg-green-900/20 dark:text-green-400">
                    Tools
                  </span>
                </div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <PrimaryButton 
                  @click="editModel(model)" 
                  buttonType="secondary"
                  size="xs"
                >
                  Edit
                </PrimaryButton>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Add/Edit Model Modal -->
    <div 
      v-if="showAddModelModal || showEditModelModal" 
      class="fixed inset-0 z-10 overflow-y-auto" 
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
                {{ showEditModelModal ? 'Edit Model' : 'Add New Model' }}
              </h3>
              <div class="mt-4">
                <form  class="space-y-4">
                  <div>
                    <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
                      Model Name
                      <span class="ml-1 text-red-500">*</span>
                    </label>
                    <input 
                      v-model="modelForm.name" 
                      required
                      placeholder="e.g., openai/gpt-4-turbo"
                      class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
                      type="text"
                    />
                    <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
                      Use the full model name from <a href="https://openrouter.ai/models" target="_blank" class="text-blue-500 underline">OpenRouter models</a>
                    </p>
                  </div>
                  
                  <div>
                    <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
                      Provider
                      <span class="ml-1 text-red-500">*</span>
                    </label>
                    <select
                      v-model="modelForm.provider_id"
                      required
                      class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
                    >
                      <option value="" disabled>Select a provider</option>
                      <option 
                        v-for="provider in providers" 
                        :key="provider.id" 
                        :value="provider.id"
                      >
                        {{ provider.name }}
                      </option>
                    </select>
                  </div>
                  
                  
                  <div class="flex items-center space-x-4 mt-4">
                    <div class="flex items-center space-x-2">
                      <input 
                        id="supports_json" 
                        v-model="modelForm.supports_json" 
                        type="checkbox"
                        class="h-4 w-4"
                      />
                      <label for="supports_json" class="text-sm text-neutral-700 dark:text-white">Supports JSON</label>
                    </div>
                    
                    <div class="flex items-center space-x-2">
                      <input 
                        id="supports_tools" 
                        v-model="modelForm.supports_tools" 
                        type="checkbox"
                        class="h-4 w-4"
                      />
                      <label for="supports_tools" class="text-sm text-neutral-700 dark:text-white">Supports Tools</label>
                    </div>
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
                      type="submit" 
                      @click="handleSubmit"
                      :loading="formLoading"
                      buttonType="primary"
                      size="sm"
                    >
                      {{ showEditModelModal ? 'Update' : 'Add' }}
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
import { ref, onMounted, reactive } from 'vue'
import { useModels, type CreateModelPayload } from '~/composables/useModels'
import { useProviders } from '~/composables/useProviders'
import type { Model } from '~/types/response/models'
import type { Provider } from '~/types/response/providers'
import PrimaryButton from '~/components/global/primary-button.vue'

definePageMeta({
  middleware: ['auth'],
  layout: 'logged-in'
})

const { models, loading, error: modelsError, fetchModels, createModel, updateModel } = useModels()
const { providers, loading: providersLoading, fetchProviders } = useProviders()

const showAddModelModal = ref(false)
const showEditModelModal = ref(false)
const formLoading = ref(false)
const currentModelId = ref<number | null>(null)
const formError = ref<string | null>(null)

const modelForm = reactive<CreateModelPayload>({
  name: '',
  provider_id: 4, // Default to OpenRouter
  supports_json: false,
  supports_tools: false
})

onMounted(async () => {
  await Promise.all([
    fetchModels(),
    fetchProviders()
  ])
})

function resetForm() {
  modelForm.name = ''
  modelForm.provider_id = 4
  modelForm.supports_json = false
  modelForm.supports_tools = false
  currentModelId.value = null
}

function editModel(model: Model) {
  modelForm.name = model.name
  modelForm.provider_id = model.provider_id
  modelForm.supports_json = model.supports_json
  modelForm.supports_tools = model.supports_tools
  currentModelId.value = model.id
  showEditModelModal.value = true
}

function closeModal() {
  showAddModelModal.value = false
  showEditModelModal.value = false
  resetForm()
  formError.value = null
}

async function handleSubmit() {
  formLoading.value = true
  formError.value = null
  try {
    if (showEditModelModal.value && currentModelId.value) {
      await updateModel(currentModelId.value, modelForm)
    } else {
      await createModel(modelForm)
    }
    await fetchModels() // Refresh the models list
    closeModal()
  } catch (err: any) {
    console.error('Error submitting form:', err)
    formError.value = err?.data?.message || err?.message || 'Failed to save model. Please try again.'
  } finally {
    formLoading.value = false
  }
}
</script>
