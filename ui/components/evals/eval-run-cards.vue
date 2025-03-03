<template>
  <div>
    <nav class="flex items-center justify-center" aria-label="Progress">
      <p class="text-sm font-medium dark:text-neutral-200 text-neutral-800">Scoring eval {{ evalIndex + 1 }} of {{ evalRuns.length }}</p>
      <ol role="list" class="ml-8 flex items-center space-x-5">
        <li v-for="(_, i) in evalRuns">
          <span v-if="i < evalIndex" class="block size-2.5 rounded-full bg-emerald-600 hover:bg-emerald-900">
            <span class="sr-only">Step 1</span>
          </span>
          <span v-if="i === evalIndex" class="relative flex items-center justify-center" aria-current="step">
            <span class="absolute flex size-5 p-px" aria-hidden="true">
              <span class="size-full rounded-full bg-emerald-200"></span>
            </span>
            <span class="relative block size-2.5 rounded-full bg-emerald-600" aria-hidden="true"></span>
            <span class="sr-only">Step 2</span>
          </span>
          <span v-if="i > evalIndex" class="block size-2.5 rounded-full bg-gray-200 hover:bg-gray-400">
            <span class="sr-only">Step 3</span>
          </span>
        </li>
      </ol>
    </nav>

    <div class="mt-4 border border-neutral-600 dark:border-neutral-500 p-4">
      <h3 class="text-neutral-600 dark:text-neutral-400 text-center">Select a score based on the output</h3>
      <div class="mt-4 flex justify-center">
        <div class="space-x-6">
          <button 
            v-for="i in [1,2,3,4,5]"
            type="button"
            class="dark:text-neutral-300 p-3 border dark:border-neutral-300 border-neutral-700 dark:hover:bg-neutral-700 dark:hover:border-neutral-100 hover:bg-neutral-300 hover:border-neutral-900"
            @click="executeUpdateEvalRunScore(i)"
          >
            {{ i }}
          </button>
        </div>
      </div>

      <div v-if="currentEvalRun" class="mt-10">
        <div class="mt-3">
          <dl class="grid grid-cols-1 sm:grid-cols-2">
            <div class="px-4 pb-4 sm:col-span-2 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Eval name</dt>
              <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ currentEvalRun.prompt_eval_name }}</dd>
            </div>
          </dl>
          <dl v-if="currentEval" class="grid grid-cols-1 sm:grid-cols-2">
            <div v-if="currentEval.system_prompt_input" class="px-4 pb-4 sm:col-span-2 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System input data</dt>
              <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ JSON.parse(currentEval.system_prompt_input) }}</dd>
            </div>
            <div class="px-4 pb-4 sm:col-span-2 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">User input data</dt>
              <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">
                {{ isJson(currentEval.user_prompt_input) ? JSON.parse(currentEval.user_prompt_input) : currentEval.user_prompt_input }}
              </dd>
            </div>
          </dl>
          <dl class="grid grid-cols-1 sm:grid-cols-2">
            <div class="dark:border-neutral-700 px-4 sm:col-span-2 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Output</dt>
              <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ currentEvalRun.output }}</dd>
            </div>
          </dl>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import type { PromptEvalRunResponse } from '~/types/response/prompt_eval_runs';
import type { PromptEvalResponse } from '~/types/response/prompt_eval'
import type { Prompt } from '~/types/response/prompts';

const props = defineProps<{
  evalRuns: PromptEvalRunResponse[]
  evals: PromptEvalResponse[],
  prompt: Prompt
}>()

const emits = defineEmits<{
  "score-eval-run-complete": [];
}>();

const { updateEvalRunScore } = usePromptEvalRuns();

const evalIndex = ref(0)

watch(evalIndex, (newIndex, _) => {
  if (newIndex > props.evalRuns.length - 1) {
    emits('score-eval-run-complete')
  }
})

const currentEvalRun = computed(() => {
  return props.evalRuns[evalIndex.value]
})

const currentEval = computed(() => {
  if (!currentEvalRun.value) {
    return null
  }

  const evalId = currentEvalRun.value.prompt_eval_id
  return props.evals.find(e => e.id === evalId)
})


// Helper function to check if a string is valid JSON
function isJson(str: string): boolean {
  try {
    JSON.parse(str);
    return true;
  } catch (e) {
    return false;
  }
}

async function executeUpdateEvalRunScore(score: number) {
  try {
    await updateEvalRunScore(currentEvalRun.value.id, score)
    evalIndex.value += 1
  } catch(e) {
    throw createError({ statusCode: 500, statusMessage: "Something went wrong when setting the eval run score" })
  }
}
</script>
