<template>
  <div class="font-mono">
    <div class="px-4 sm:px-0 flex items-center justify-between">
      <div>
        <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Prompt Evals</h3>
        <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Create test prompts and evaluate them over time across prompt versions.</p>
      </div>
      <button @click="createSampleInput = true" class="text-sm text-black dark:text-neutral-200 hover:text-neutral-700 dark:hover:text-neutral-300">+ New Sample</button>
    </div>

    <div v-if="samples.length === 0" class="mt-6">
      <button @click="createSampleInput = true" v-if="!createSampleInput" type="button" class="relative block w-full rounded-lg border-2 border-dashed border-neutral-300 p-12 text-center hover:border-neutral-400 focus:outline-none focus:ring-2 focus:ring-neutral-500 focus:ring-offset-2">
        <svg 
          xmlns="http://www.w3.org/2000/svg" 
          fill="none" 
          viewBox="0 0 24 24" 
          stroke-width="1.5" 
          stroke="currentColor" 
          class="mx-auto size-12 text-neutral-700 dark:text-neutral-300"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 3.104v5.714a2.25 2.25 0 0 1-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 0 1 4.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0 1 12 15a9.065 9.065 0 0 0-6.23-.693L5 14.5m14.8.8 1.402 1.402c1.232 1.232.65 3.318-1.067 3.611A48.309 48.309 0 0 1 12 21c-2.773 0-5.491-.235-8.135-.687-1.718-.293-2.3-2.379-1.067-3.61L5 14.5" />
        </svg>
        <span class="mt-2 block text-sm font-semibold text-neutral-900">Create new test input</span>
      </button>

      <EvalsCreateSampleInput 
        v-else 
        :prompt="props.prompt" 
        @cancel="createSampleInput = false"
        @created="handleSampleCreated()"
      />
    </div>

    <div v-else class="mt-6">
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        <li 
          v-for="(s, i) in samples" 
          :key="s.id" 
          class="p-2 hover:bg-neutral-50 dark:hover:bg-neutral-900/50 transition-colors"
        >
          <div class="flex items-center justify-between">
            <div>
              <div class="dark:text-neutral-200 text-neutral-800">
                Sample {{ i + 1 }}: {{ s.name }}
              </div>
              <div>
                <span class="dark:text-neutral-500 text-neutral-500 text-sm">Last updated: {{ formatDate(s.updated_at) }}</span>
              </div>
            </div>
            <button
              @click="toggleSample(s.id)"
              class="text-neutral-700 hover:text-neutral-900 dark:text-neutral-300 dark:hover:text-neutral-100"
            >
              {{ expandedSample === s.id ? 'Collapse' : 'View' }}
            </button>
          </div>

          <div v-if="expandedSample === s.id" class="mt-3 dark:bg-neutral-700 bg-neutral-100 p-2">
            <div class="grid gap-2 text-sm">
              <div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2">
                <span class="text-neutral-500 dark:text-neutral-400">Sample name:</span>
                <span class="text-neutral-900 dark:text-neutral-300">{{ s.name }}</span>

                <span class="text-neutral-500 dark:text-neutral-400">Sample updated at:</span>
                <span class="text-neutral-900 dark:text-neutral-300">{{ s.updated_at }}</span>
                
                <span class="text-neutral-500 dark:text-neutral-400">Sample input:</span>
                <pre class="p-2 bg-neutral-200 dark:bg-neutral-800 rounded text-neutral-900 dark:text-neutral-300 overflow-x-auto">{{ JSON.parse(s.input_data) }}</pre>
              </div>
            </div>
          </div>
        </li>
      </ul>
    </div>

  </div>
</template>

<script setup lang="ts">
import { format, parseISO } from 'date-fns';
import type { Prompt } from '~/types/response/prompts';
import type { Model } from '~/types/response/models';

const props = defineProps<{
  prompt: Prompt
}>();

const createSampleInput = ref(false)
const expandedSample = ref<number | null>(null)

const toggleSample = (id: number) => {
  expandedSample.value = expandedSample.value === id ? null : id
}

const { fetchSamplesByPrompt, updateSample, createSample, deleteSample, samples } = usePromptSamples();

await fetchSamplesByPrompt(props.prompt.id)

async function handleSampleCreated() {
  await fetchSamplesByPrompt(props.prompt.id)
  createSampleInput.value = false
}

const formatDate = (dateString: string | undefined) => {
  if (!dateString) return 'n/a'
  try {
    return format(new Date(dateString), 'yyyy-mm-dd')
  } catch (error) {
    console.error('error formatting date', error)
    return 'invalid date'
  }
}

</script>
