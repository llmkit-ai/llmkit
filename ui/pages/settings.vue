<template>
  <div class="p-6 font-mono">
    <h1 class="text-xl font-semibold mb-6 text-neutral-900 dark:text-white">Settings</h1>
    
    <div class="mb-8">
      <h2 class="text-base/7 font-semibold mb-4 text-neutral-900 dark:text-white">API Keys</h2>
      <p class="text-sm/6 mb-4 text-neutral-500 dark:text-neutral-400">
        Generate API keys to use with the LLMKit API. Your API keys have full access to the API.
        Protect them like passwords.
      </p>
      
      <!-- API Keys List -->
      <div class="border border-neutral-200 dark:border-neutral-700 mb-4">
        <table class="min-w-full divide-y divide-neutral-100 dark:divide-neutral-700">
          <thead class="bg-neutral-100 dark:bg-neutral-800">
            <tr>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Name
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Created
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white dark:bg-neutral-900 divide-y divide-neutral-100 dark:divide-neutral-700">
            <tr v-if="isLoading">
              <td colspan="3" class="px-6 py-4">
                <div class="flex justify-center">
                  <span class="animate-pulse">Loading...</span>
                </div>
              </td>
            </tr>
            <tr v-else-if="!apiKeys.length">
              <td colspan="3" class="px-6 py-4 text-center text-sm/6 text-neutral-500 dark:text-neutral-400">
                No API keys found
              </td>
            </tr>
            <tr v-for="key in apiKeys" :key="key.id" class="hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 font-medium text-neutral-700 dark:text-neutral-300">
                {{ key.name }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 text-neutral-500 dark:text-neutral-400">
                {{ key.created_at }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <PrimaryButton 
                  @click="deleteApiKey(key.id)"
                  buttonType="error"
                  size="xs"
                >
                  Delete
                </PrimaryButton>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      
      <!-- New API Key Form -->
      <div v-if="!showNewKeyForm" class="mt-4">
        <PrimaryButton 
          @click="showNewKeyForm = true"
          buttonType="primary"
          size="sm"
        >
          Create new API key
        </PrimaryButton>
      </div>
      
      <div v-if="showNewKeyForm" class="mt-4 bg-neutral-100 dark:bg-neutral-800 p-4 border border-neutral-200 dark:border-neutral-700">
        <label class="block text-sm/6 font-medium text-neutral-900 dark:text-white mb-1">
          Key Name
        </label>
        <div class="flex">
          <input
            v-model="newKeyName"
            type="text"
            class="flex-grow bg-white dark:bg-neutral-800 border-2 border-black dark:border-white p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
            placeholder="My API Key"
          />
          <div class="flex ml-3 gap-x-3">
            <PrimaryButton
              @click="createApiKey"
              buttonType="primary"
              size="sm"
              :disabled="!newKeyName.trim()"
            >
              Create
            </PrimaryButton>
            <PrimaryButton
              @click="showNewKeyForm = false; newKeyName = ''"
              buttonType="secondary"
              size="sm"
            >
              Cancel
            </PrimaryButton>
          </div>
        </div>
      </div>
      
      <!-- New API Key Display Modal -->
      <div v-if="showApiKeyModal" class="fixed inset-0 z-10 overflow-y-auto" aria-labelledby="modal-title" role="dialog" aria-modal="true">
        <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
          <div class="fixed inset-0 bg-neutral-500 bg-opacity-75 transition-opacity" aria-hidden="true"></div>
          
          <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>
          <div class="inline-block align-bottom bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-6 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
            <div>
              <div class="mt-3 text-center sm:mt-5">
                <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white" id="modal-title">
                  Your API Key
                </h3>
                <div class="mt-2">
                  <p class="text-sm/6 text-neutral-500 dark:text-neutral-400">
                    Make sure to copy your API key now. You won't be able to see it again!
                  </p>
                  <div class="mt-4 bg-neutral-100 dark:bg-neutral-900 p-3 border border-neutral-200 dark:border-neutral-700">
                    <div class="flex items-center justify-between">
                      <code class="text-sm/6 break-all font-mono text-neutral-700 dark:text-neutral-300">{{ newApiKey }}</code>
                      <PrimaryButton 
                        @click="copyToClipboard"
                        buttonType="secondary"
                        size="xs"
                      >
                        Copy
                      </PrimaryButton>
                    </div>
                  </div>
                  <p v-if="copied" class="mt-2 text-xs text-green-600 dark:text-green-400">
                    Copied to clipboard!
                  </p>
                </div>
              </div>
            </div>
            <div class="mt-5 sm:mt-6 flex justify-end">
              <PrimaryButton 
                buttonType="primary"
                size="sm"
                @click="showApiKeyModal = false; newApiKey = ''; copied = false"
              >
                Done
              </PrimaryButton>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import PrimaryButton from '~/components/global/primary-button.vue'

definePageMeta({
  layout: "logged-in",
  middleware: ['auth']
})

const apiKeys = ref([])
const isLoading = ref(true)
const showNewKeyForm = ref(false)
const newKeyName = ref('')
const showApiKeyModal = ref(false)
const newApiKey = ref('')
const copied = ref(false)

onMounted(async () => {
  await fetchApiKeys()
})

async function fetchApiKeys() {
  isLoading.value = true
  try {
    const response = await fetch('/v1/ui/settings/api-keys')
    if (!response.ok) {
      throw new Error('Failed to fetch API keys')
    }
    const data = await response.json()
    apiKeys.value = data
  } catch (error) {
    console.error('Error fetching API keys:', error)
  } finally {
    isLoading.value = false
  }
}

async function createApiKey() {
  if (!newKeyName.value.trim()) return
  
  try {
    const response = await fetch('/v1/ui/settings/api-keys', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name: newKeyName.value.trim() }),
    })
    
    if (!response.ok) {
      throw new Error('Failed to create API key')
    }
    
    const data = await response.json()
    newApiKey.value = data.key
    showApiKeyModal.value = true
    showNewKeyForm.value = false
    newKeyName.value = ''
    await fetchApiKeys()
  } catch (error) {
    console.error('Error creating API key:', error)
  }
}

async function deleteApiKey(id) {
  if (!confirm('Are you sure you want to delete this API key? This action cannot be undone.')) {
    return
  }
  
  try {
    const response = await fetch(`/v1/ui/settings/api-keys/${id}`, {
      method: 'DELETE',
    })
    
    if (!response.ok) {
      throw new Error('Failed to delete API key')
    }
    
    await fetchApiKeys()
  } catch (error) {
    console.error('Error deleting API key:', error)
  }
}

function copyToClipboard() {
  navigator.clipboard.writeText(newApiKey.value)
    .then(() => {
      copied.value = true
      setTimeout(() => {
        copied.value = false
      }, 2000)
    })
    .catch((err) => {
      console.error('Could not copy text: ', err)
    })
}
</script>
