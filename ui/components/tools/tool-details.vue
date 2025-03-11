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
        Tool Details
      </h2>
    </div>

    <div class="space-y-6">
      <div class="border border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900 p-6 rounded-none">
        <div class="space-y-4">
          <div class="flex justify-between">
            <h3 class="text-lg font-medium text-neutral-900 dark:text-white">{{ tool.name }}</h3>
          </div>
          
          <div>
            <h4 class="text-sm font-medium text-neutral-700 dark:text-neutral-300">Function Name</h4>
            <p class="mt-1 text-sm text-neutral-900 dark:text-white font-mono">{{ tool.tool_name }}</p>
          </div>
          
          <div>
            <h4 class="text-sm font-medium text-neutral-700 dark:text-neutral-300">Description</h4>
            <p class="mt-1 text-sm text-neutral-900 dark:text-white">{{ tool.description }}</p>
          </div>
          
          <div>
            <h4 class="text-sm font-medium text-neutral-700 dark:text-neutral-300">Parameters Schema</h4>
            <pre class="mt-1 bg-neutral-100 dark:bg-neutral-700 p-3 rounded text-xs font-mono whitespace-pre-wrap text-neutral-900 dark:text-white overflow-auto max-h-80">{{ formatJson(tool.parameters) }}</pre>
          </div>
          
          <div>
            <h4 class="text-sm font-medium text-neutral-700 dark:text-neutral-300">Strict Mode</h4>
            <p class="mt-1 text-sm text-neutral-900 dark:text-white">{{ tool.strict ? 'Enabled' : 'Disabled' }}</p>
          </div>
          
          <div>
            <h4 class="text-sm font-medium text-neutral-700 dark:text-neutral-300">Created</h4>
            <p class="mt-1 text-sm text-neutral-900 dark:text-white">{{ formatDate(tool.created_at) }}</p>
          </div>

          <div>
            <h4 class="text-sm font-medium text-neutral-700 dark:text-neutral-300">Last Updated</h4>
            <p class="mt-1 text-sm text-neutral-900 dark:text-white">{{ formatDate(tool.updated_at) }}</p>
          </div>
        </div>
      </div>

      <!-- Action buttons -->
      <div class="flex space-x-3">
        <PrimaryButton 
          @click="$emit('back')" 
          buttonType="secondary"
          size="sm"
        >
          Back to List
        </PrimaryButton>
        <PrimaryButton 
          @click="$emit('edit', tool)" 
          buttonType="secondary"
          size="sm"
        >
          Edit Tool
        </PrimaryButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Tool } from '~/types/response/tools'
import PrimaryButton from '~/components/global/primary-button.vue'

const props = defineProps<{
  tool: Tool
}>()

const emit = defineEmits<{
  back: []
  edit: [tool: Tool]
}>()

function formatDate(dateString?: string): string {
  if (!dateString) return ''
  
  const date = new Date(dateString)
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

function formatJson(jsonString: string): string {
  try {
    // Try to parse and prettify if it's valid JSON
    const parsed = JSON.parse(jsonString)
    return JSON.stringify(parsed, null, 2)
  } catch {
    // Return as-is if not valid JSON
    return jsonString
  }
}
</script>
