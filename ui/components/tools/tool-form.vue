<template>
  <div>
    <div class="mb-6 flex items-center">
      <button 
        @click="$emit('back')" 
        class="mr-3 p-1 rounded-full hover:bg-neutral-100 dark:hover:bg-neutral-800"
      >
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 text-neutral-600 dark:text-neutral-400">
          <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
        </svg>
      </button>
      <h2 class="text-xl font-semibold text-neutral-900 dark:text-white">
        {{ isEdit ? 'Edit Tool' : 'Add New Tool' }}
      </h2>
    </div>

    <form class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
          Tool Name
          <span class="ml-1 text-red-500">*</span>
        </label>
        <input 
          v-model="formData.name" 
          required
          placeholder="e.g., Weather API"
          class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
          type="text"
        />
      </div>
      
      <div>
        <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
          Function Name
          <span class="ml-1 text-red-500">*</span>
        </label>
        <input 
          v-model="formData.tool_name" 
          required
          placeholder="e.g., getWeather"
          class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
          type="text"
        />
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          This is the function name that will be used in the API
        </p>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
          Description
          <span class="ml-1 text-red-500">*</span>
        </label>
        <textarea 
          v-model="formData.description" 
          required
          placeholder="Describe what the tool does"
          rows="3"
          class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
        ></textarea>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          Detailed description helps the model understand when to use this tool
        </p>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-neutral-700 dark:text-white mb-1">
          Parameters Schema (JSON Schema)
          <span class="ml-1 text-red-500">*</span>
        </label>
        <textarea 
          v-model="formData.parameters" 
          required
          placeholder='{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "location": {
      "type": "string",
      "description": "Location to get weather for"
    }
  },
  "required": ["location"]
}'
          rows="12"
          class="block w-full bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-2 text-base focus:outline-none text-neutral-900 dark:text-white font-mono"
        ></textarea>
        <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
          Define the parameters for this tool using JSON Schema format
        </p>
      </div>
      
      <div class="flex items-center mt-4">
        <input 
          id="strict" 
          v-model="formData.strict" 
          type="checkbox"
          class="h-4 w-4"
        />
        <label for="strict" class="ml-2 text-sm text-neutral-700 dark:text-white">
          Strict Mode (enforce parameters schema strictly)
        </label>
      </div>
      
      <!-- Form error message -->
      <div v-if="error" class="mt-4 rounded border-2 border-red-500 bg-red-100 p-3 text-red-700 dark:bg-red-900/20 dark:text-red-400">
        {{ error }}
      </div>
      
      <div class="mt-6 flex justify-end space-x-3">
        <PrimaryButton 
          type="button"
          @click="$emit('back')"
          buttonType="secondary"
          size="sm"
        >
          Cancel
        </PrimaryButton>
        <PrimaryButton 
          type="submit" 
          @click="handleSubmit"
          :loading="loading"
          buttonType="primary"
          size="sm"
        >
          {{ isEdit ? 'Update' : 'Add' }}
        </PrimaryButton>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import type { Tool } from '~/types/response/tools'
import PrimaryButton from '~/components/global/primary-button.vue'

const props = defineProps<{
  tool?: Tool | null
  loading: boolean
  error: string | null
}>()

const emit = defineEmits<{
  submit: [data: any]
  back: []
}>()

const isEdit = computed(() => !!props.tool)

const formData = reactive({
  name: '',
  tool_name: '',
  description: '',
  parameters: '',
  strict: true
})

// Initialize form when tool changes
watch(() => props.tool, (newTool) => {
  if (newTool) {
    formData.name = newTool.name
    formData.tool_name = newTool.tool_name
    formData.description = newTool.description
    formData.parameters = newTool.parameters
    formData.strict = newTool.strict
  } else {
    // Reset form data for new tool
    formData.name = ''
    formData.tool_name = ''
    formData.description = ''
    formData.parameters = ''
    formData.strict = true
  }
}, { immediate: true })

function handleSubmit() {
  // Basic validation
  if (!formData.parameters.trim()) {
    return
  }

  emit('submit', { 
    ...formData,
    id: props.tool?.id 
  })
}
</script>
