<template>
  <div>
    <div class="px-4 sm:px-0">
      <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Test Prompt</h3>
      <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Execute, test, and evaluate prompt.</p>
    </div>
    <div class="mt-3">
      <dl class="grid grid-cols-1 sm:grid-cols-2">
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ systemPrompt }}</dd>
        </div>
      </dl>
      <!-- Only show User Prompt for dynamic_both type -->
      <dl v-if="props.prompt.prompt_type === 'dynamic_both'" class="grid grid-cols-1 sm:grid-cols-2">
        <div class="dark:border-neutral-700 px-4 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ userPrompt }}</dd>
        </div>
      </dl>
    </div>
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-700 dark:text-white">Dynamic fields</h3>
        <p class="max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">The below fields are extracted based on handlebar syntax from your prompts. Populating them will dynamically swap the values into your prompt at runtime.</p>
      </div>
      <div class="mt-4 grid grid-cols-4 gap-x-2">
        <div v-for="f in templateFields">
          <label :for="f" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">{{ f }}</label>
          <div class="mt-0.5">
            <input 
              v-on:input="templateFieldInput" 
              type="text" 
              :name="f" 
              :id="f" 
              class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
            >
          </div>
        </div>
      </div>
    </div>
    <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-2">
      <PrimaryButton
        type="secondary"
        size="sm"
        @click="$emit('handle-cancel')"
      >
        Cancel
      </PrimaryButton>
      <PrimaryButton
        type="secondary"
        size="sm"
        @click="$emit('handle-edit')"
      >
        Edit
      </PrimaryButton>
      <PrimaryButton
        type="primary"
        size="sm"
        @click="executeStream()"
      >
        Stream
      </PrimaryButton>
      <PrimaryButton
        type="primary"
        size="sm"
        @click="execute()"
      >
        Execute
      </PrimaryButton>
    </div>
    <div v-if="Object.keys(jsonContext).length > 0" class="mt-5 bg-neutral-100 dark:bg-neutral-800 p-4">
      <div class="flex items-center justify-between">
        <p class="text-xs text-neutral-900 dark:text-neutral-300">Json context</p>
        <button
          @click="showJsonContext = !showJsonContext"
          class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-300"
        >
          {{ showJsonContext ? 'Hide' : 'Show' }}
        </button>
      </div>
      <div v-if="showJsonContext" class="mt-3 dark:text-neutral-300 text-sm">
        {{ jsonContext }}
      </div>
    </div>
    <div v-if="testResponse" class="mt-5 bg-neutral-100 dark:bg-neutral-800 p-4">
      <div class="flex items-center justify-between">
        <p class="text-xs text-neutral-900 dark:text-neutral-300">Response</p>
        <button
          @click="showResponse = !showResponse"
          class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-300"
        >
          {{ showResponse ? 'Hide' : 'Show' }}
        </button>
      </div>
      <div v-if="showResponse" class="response-content mt-3 dark:text-neutral-300 text-sm">
        {{ testResponse }}
      </div>
    </div>
    <div v-if="logResponse" class="mt-5 bg-neutral-100 dark:bg-neutral-800 p-4">
      <div class="flex items-center justify-between">
        <p class="text-xs text-neutral-900 dark:text-neutral-300">Log</p>
        <button
          @click="showLog = !showLog"
          class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-300"
        >
          {{ showLog ? 'Hide' : 'Show' }}
        </button>
      </div>
      <div v-if="showLog" class="mt-3 ml-6">
        <div class="grid gap-2 text-sm">
          <div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2">
            <span class="text-neutral-500 dark:text-neutral-400">prompt:</span>
            <span class="text-neutral-900 dark:text-neutral-300">{{
              logResponse.prompt_id || 'n/a'
            }}</span>

            <span class="text-neutral-500 dark:text-neutral-400">tokens:</span>
            <span class="text-neutral-900 dark:text-neutral-300">
              [in:{{ logResponse.input_tokens }} out:{{
                logResponse.output_tokens
              }} rt:{{ logResponse.reasoning_tokens }}]
            </span>

            <span class="text-neutral-500 dark:text-neutral-400">request:</span>
            <pre
              v-if="logResponse.request_body"
              class="p-2 bg-neutral-200 dark:bg-neutral-800 rounded text-neutral-900 dark:text-neutral-300 overflow-x-auto"
            >{{ JSON.parse(logResponse.request_body!) }}</pre>

            <span class="text-neutral-500 dark:text-neutral-400">response:</span>
            <pre
              v-if="logResponse.response_data"
              class="p-2 bg-neutral-200 dark:bg-neutral-800 rounded text-neutral-900 dark:text-neutral-300 overflow-x-auto"
            >{{ JSON.parse(logResponse.response_data!) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ApiLogReponse } from '~/types/response/logs';
import type { Prompt } from '~/types/response/prompts';

const props = defineProps<{
  prompt: Prompt
}>();

const emit = defineEmits(["handle-edit", "handle-cancel"])

const { 
  executePrompt,
} = usePrompts();

const { 
  log,
  fetchLogById
} = useLogs();


const systemPrompt = ref(props.prompt.system)
const userPrompt = ref(props.prompt.user)
const jsonContext = ref({})
const testResponse = ref<string | null>(null)
const logResponse = ref<ApiLogReponse | null>(null)
const showLog = ref(false)
const showJsonContext = ref(false)
const showResponse = ref(true)


const templateFields = computed<string[]>(() => {
  if (!props.prompt || !props.prompt.system || !props.prompt.user) return [];

  const template = `${props.prompt.system}\n${props.prompt.user}`;
  const uniqueFields = new Set<string>();

  // Regex to find variables in {{ ... }} (Handlebars style)
  const handlebarsRegex = /\{\{\s*(\w+)\s*\}\}/g;
  let match;
  while ((match = handlebarsRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  // Regex to find variables in {% if variable ... %} conditions
  const ifConditionRegex = /\{\%\s*if\s*(\w+)(?:\s+.*?)\s*\%\}/g; // Added non-capturing group for stuff after variable
  while ((match = ifConditionRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  // Regex for {% elif variable ... %}
  const elifConditionRegex = /\{\%\s*elif\s*(\w+)(?:\s+.*?)\s*\%\}/g;
  while ((match = elifConditionRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  // Regex for {% for variable in ... %} (extracting the iterable variable)
  const forLoopRegex = /\{\%\s*for\s+\w+\s+in\s+(\w+)\s*\%\}/g; // Extracts the iterable (e.g., 'items' in 'for item in items')
  while ((match = forLoopRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }


  return Array.from(uniqueFields);
});

function templateFieldInput(event: any) {
  const key = event.target.id
  const value = event.target.value

  // @ts-ignore
  jsonContext.value[key] = value

  systemPrompt.value.replace(`{{ name }}`, value)
}

async function execute() {
  const res = await executePrompt(props.prompt.id, jsonContext.value)
  testResponse.value = res.content
  logResponse.value = res.log
}

const { startStream } = useSSE()
const error = ref<Error | null>(null)

const executeStream = async () => {
  testResponse.value = ''
  error.value = null

  await startStream(
    jsonContext.value,
    `/api/v1/prompts/execute/${props.prompt.id}/stream`,
    {
      onMessage: async (chunk) => {
        if (chunk.includes("log_id")) {
          const logChunk = JSON.parse(chunk)
          const logId = logChunk["log_id"]
          await getLogRecord(logId)
          return
        }
        testResponse.value += chunk
      },
      onError: (err) => {
        error.value = err
      },
    }
  )
}

async function getLogRecord(log_id: number) {
  await fetchLogById(log_id)
  logResponse.value = log.value
}
</script>

<style scoped>
.response-content {
  white-space: pre-line;
}
</style>
