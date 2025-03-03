<template>
  <div v-if="!evalTestsLoading" class="font-mono">
    <div v-if="view === 'empty'">
      <div class="px-4 sm:px-0 flex items-center justify-between">
        <div>
          <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Prompt Evals</h3>
          <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Create test prompts and evaluate them over time across prompt versions.</p>
        </div>
      </div>

      <div v-if="evals.length === 0" class="mt-6">
        <button @click="view = 'create'" type="button" class="relative block w-full rounded-lg border-2 border-dashed border-neutral-300 dark:border-neutral-700 p-12 text-center hover:border-neutral-400 dark:hover:border-neutral-500 focus:outline-none focus:ring-2 focus:ring-neutral-500 focus:ring-offset-2">
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
          <span class="mt-2 block text-sm font-semibold text-neutral-900 dark:text-white">Create new test eval</span>
        </button>

        <div class="mt-6">
          <h3 class="text-lg font-bold text-neutral-700 dark:text-neutral-300">How it works</h3>
          <ul class="mt-2 pl-7 list-decimal space-y-2">
            <li>
              <h4 class="text-neutral-700 dark:text-neutral-300">Create test eval</h4>
              <p class="text-neutral-500 dark:text-neutral-400">Create several test evals that you want to run evaluations on. You will run these evals anytime you update your prompt.</p>
            </li>
            <li>
              <h4 class="text-neutral-700 dark:text-neutral-300">Run evals</h4>
              <p class="text-neutral-500 dark:text-neutral-400">This will execute each of your evals.</p>
            </li>
            <li>
              <h4 class="text-neutral-700 dark:text-neutral-300">Evaluate the output</h4>
              <p class="text-neutral-500 dark:text-neutral-400">Go through each of the eval outputs and evaluate the performance of your prompt.</p>
            </li>
            <li>
              <h4 class="text-neutral-700 dark:text-neutral-300">Track over time</h4>
              <p class="text-neutral-500 dark:text-neutral-400">Over time, you can view the performance of your prompts across versions based on your evals.</p>
            </li>
          </ul>
        </div>
      </div>
    </div>

    <div v-if="view === 'create'">
      <div class="px-4 sm:px-0 flex items-center justify-between">
        <div>
          <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Create new eval</h3>
          <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Create test prompts and evaluate them over time across prompt versions.</p>
        </div>
      </div>
      <EvalsCreatePromptEvalInput
        class="mt-6"
        :prompt="props.prompt"
        :key="props.prompt.id"
        @cancel="handlePromptEvalCancel()"
        @created="handleSampleCreated()"
      />
    </div>

    <div v-if="view === 'edit' && editingSample">
      <div class="px-4 sm:px-0 flex items-center justify-between">
        <div>
          <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Update eval</h3>
          <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Update your eval. We prepopulated the existing values for you.</p>
        </div>
      </div>
      <EvalsCreatePromptEvalInput
        class="mt-6"
        :prompt="props.prompt"
        :eval="editingSample"
        :key="`${props.prompt.id}-edit-${editingSample?.id}`"
        @cancel="view = 'view'"
        @updated="view = 'view'"
      />
    </div>

    <div v-if="view === 'view'">
      <EvalsViewPromptEvalInput
        :evals="evals"
        :prompt="prompt"
        @create-eval="view = 'create'"
        @edit-eval="handlePromptEvalEdit"
      />
    </div>

  </div>
</template>

<script setup lang="ts">
import { format } from 'date-fns';
import type { Prompt } from '~/types/response/prompts';

const props = defineProps<{
  prompt: Prompt
}>();

const view = ref<"empty" | "view" | "edit" |"create" | "eval">("view")
const editingSampleId = ref<number | null>(null)

const { fetchEvalByPrompt, evals, loading: evalTestsLoading } = usePromptEvals();

// Fetch evals on component mount
await fetchEvalByPrompt(props.prompt.id)

// Set initial view based on whether evals exist
if (evals.value.length === 0) {
  view.value = "empty"
}

const editingSample = computed(() => {
  if (!editingSampleId.value) { return null }

  return evals.value.find((s: any) => s.id === editingSampleId.value)
})

async function handleSampleCreated() {
  await fetchEvalByPrompt(props.prompt.id)
  view.value = 'view'
}

async function handlePromptEvalCancel() {
  if (evals.value.length === 0) {
    view.value = 'empty'
  }
  view.value = 'view'
}

async function handlePromptEvalEdit(id: number) {
  editingSampleId.value = id
  view.value = 'edit'
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
