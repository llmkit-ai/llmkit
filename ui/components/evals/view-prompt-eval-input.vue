<template>
  <div>
    <div v-if="mode === 'view'">
      <div class="sm:flex sm:items-center">
        <div class="sm:flex-auto">
          <h1 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">Evals</h1>
          <p class="mt-2 text-sm text-neutral-700 dark:text-neutral-300">View and edit sample inputs, evals, or kick off a new eval run.</p>
        </div>
        <div class="mt-4 sm:ml-16 sm:mt-0 sm:flex-none flex items-center space-x-2">
          <PrimaryButton
            @click="$emit('create-eval')"
            buttonType="secondary"
            size="sm"
          >
            New eval
          </PrimaryButton>
        </div>
      </div>
      <div class="mt-8 flow-root">
        <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
            <table class="min-w-full divide-y divide-neutral-300 dark:divide-neutral-700">
              <thead>
                <tr>
                  <th scope="col" class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100 sm:pl-0">Id</th>
                  <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Name</th>
                  <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Updated at</th>
                  <th scope="col" class="relative py-3.5 pl-3 pr-4 sm:pr-0">
                    <span class="sr-only">Edit</span>
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800">
                <tr v-for="s in evals">
                  <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">{{ s.id }}</td>
                  <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">{{ s.name }}</td>
                  <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-500 dark:text-neutral-400">{{ s.updated_at }}</td>
                  <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0">
                    <button @click="$emit('edit-eval', s.id)" class="text-neutral-600 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100">Edit</button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <div class="mt-16">
        <div class="sm:flex sm:items-center">
          <div class="sm:flex-auto">
            <h1 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">Current eval performance</h1>
            <p class="mt-2 text-sm text-neutral-700 dark:text-neutral-300">View eval performance for the current version of your prompt.</p>
          </div>
          <div class="mt-4 sm:ml-16 sm:mt-0 sm:flex-none flex items-center space-x-2">
            <label class="text-sm text-neutral-700 dark:text-neutral-300">Rounds:</label>
            <input type="number" min="1" v-model.number="rounds" class="w-16 border border-neutral-300 dark:border-neutral-600 px-2 py-1 rounded text-sm" />
            <PrimaryButton
              @click="executeEvalRun()"
              v-if="requiresEvalRun"
              :disabled="executeLoading"
              buttonType="primary"
              size="sm"
            >
              New eval run
            </PrimaryButton>
          </div>
        </div>
        <div v-if="evalRuns.length > 0" class="mt-8 flow-root">
          <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
            <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
              <table class="min-w-full divide-y divide-neutral-300 dark:divide-neutral-700">
                <thead>
                  <tr>
                    <th scope="col" class="pr-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Round</th>
                    <th scope="col" class="pr-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Score</th>
                    <th scope="col" class="py-3.5 px-3 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100 sm:pl-0">Eval Name</th>
                    <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Output</th>
                    <th scope="col" class="relative text-neutral-900 dark:text-neutral-100 py-3.5 pl-3 pr-4 sm:pr-0">Updated at</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800">
                  <tr v-for="r in evalRuns">
                    <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">
                      {{ r.round_number || 1 }}
                    </td>
                    <td 
                      class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-0"
                      :class="getScoreColor(r.score)"
                    >
                      {{ r.score ? r.score:'None' }}
                    </td>
                    <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">{{ r.prompt_eval_name }}</td>
                    <td class="whitespace-nowrap max-w-xs overflow-hidden truncate py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">{{ r.output }}</td>
                    <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm text-neutral-600 dark:text-neutral-400 font-medium sm:pr-0">{{ r.updated_at }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
        <div v-else class="mt-16">
          <div>
            <p v-if="!executeLoading" class="text-center text-neutral-600 dark:text-neutral-400">Evals not yet run for <b>Prompt Version: {{ prompt.version_number }}</b></p>
            <p v-else class="text-center text-neutral-600 dark:text-neutral-400 animate-pulse">Running evals now</p>
          </div>
        </div>

        <div v-if="evalRuns.length > 0 && promptPerformance" class="mt-8 flow-root">
          <div class="sm:flex-auto">
            <h1 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">Performance across versions</h1>
            <p class="mt-2 text-sm text-neutral-700 dark:text-neutral-300">Performance metrics of how your prompt has been performing over time.</p>
          </div>

          <EvalsPerformanceChart 
            :performance="promptPerformance"
            :prompt-name="props.prompt.key"
          />
        </div>
      </div>
    </div>

    <div v-if="mode === 'score-eval-run'">
      <div class="sm:flex-auto">
        <h1 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">Score eval runs</h1>
        <p class="mt-2 text-sm text-neutral-700 dark:text-neutral-300">Score each of the eval runs based on the quality of the output given your input for the specific eval.</p>
      </div>
      <EvalsEvalRunCards 
        :eval-runs="evalRuns"
        :evals="evals"
        :prompt="prompt"
        @score-eval-run-complete="handleScoringComplete()"
        class="mt-14"
      />
    </div>

  </div>
</template>

<script setup lang="ts">
import type { PromptEvalResponse } from '~/types/response/prompt_eval'
import type { Prompt } from '~/types/response/prompts';

const props = defineProps<{
  evals: PromptEvalResponse[],
  prompt: Prompt
}>()

const emits = defineEmits<{
  "create-eval": [];
  "edit-eval": [id: number];
  "start-eval": [];
}>();

const mode = ref<"view" | "score-eval-run">("view")
const rounds = ref(1)

const { createEvalRun, fetchEvalRunByPromptVersion, evalRuns, loading: evalRunsLoading } = usePromptEvalRuns();
await fetchEvalRunByPromptVersion(props.prompt.id, props.prompt.version_id)

const { getPromptPerformance, promptPerformance } = usePrompts();
await getPromptPerformance(props.prompt.id)

const requiresEvalRun = computed(() => {
  if (evalRuns.value.length === 0) {
    return true
  }

  return false
})

const requiresPartialEvalRun = computed(() => {
  const missingScoreCount = evalRuns.value.filter(er => er.score === null).length
  if (missingScoreCount > 0 && missingScoreCount !== evalRuns.value.length) {
    return true
  }

  return false
})

const requiresNoEvalRun = computed(() => {
  const missingScoreCount = evalRuns.value.filter(er => er.score === null).length
  if (missingScoreCount === 0) {
    return true
  }

  return false
})

const averageScore = computed(() => {
  if (!evalRuns.value || evalRuns.value.length === 0) {
    return null
  }

  const total = evalRuns.value.map(e => e.score).reduce((acc, val) => acc! + val!, 0) || 0

  return total / evalRuns.value.length
})

const executeLoading = ref(false)

async function executeEvalRun() {
  executeLoading.value = true
  await createEvalRun(props.prompt.id, props.prompt.version_id, rounds.value)
  mode.value = 'score-eval-run'
  executeLoading.value = false
}

async function handleScoringComplete() {
  await getPromptPerformance(props.prompt.id)
  await fetchEvalRunByPromptVersion(props.prompt.id, props.prompt.version_id)
  mode.value = 'view'
}

function getScoreColor(score: number | null) {
  if (!score) return "text-neutral-900 dark:text-neutral-100";
  
  const scoreMap = {
    1: "text-red-500",
    2: "text-orange-500",
    3: "text-yellow-500",
    4: "text-lime-500",
    5: "text-green-500"
  };
  
  const clampedScore = Math.max(1, Math.min(Math.round(score), 5));
  return scoreMap[clampedScore as keyof typeof scoreMap];
}

</script>
