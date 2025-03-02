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
      
      <!-- Show rendered prompts with variables filled in -->
      <dl v-if="Object.keys(jsonContext).length > 0" class="grid grid-cols-1 sm:grid-cols-2 mt-4">
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Rendered System Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ renderedSystemPrompt }}</dd>
        </div>
        
        <div v-if="props.prompt.prompt_type === 'dynamic_both'" class="dark:border-neutral-700 px-4 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Rendered User Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ renderedUserPrompt }}</dd>
        </div>
      </dl>
    </div>
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-700 dark:text-white">Dynamic fields</h3>
        <p class="max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">The below fields are extracted based on handlebar syntax from your prompts. Populating them will dynamically swap the values into your prompt at runtime.</p>
      </div>
      <form @submit.prevent="execute">
        <div class="mt-4 grid grid-cols-4 gap-x-2">
          <!-- Add debugging info to see what fields are found -->
          <p v-if="templateFields.length === 0" class="col-span-4 text-red-500">No dynamic fields found in templates</p>
          
          <div v-for="f in templateFields" :key="f" class="mb-4">
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
      </form>
    </div>
    <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-2">
      <PrimaryButton
        buttonType="secondary"
        size="sm"
        @click="$emit('handle-cancel')"
      >
        Cancel
      </PrimaryButton>
      <PrimaryButton
        buttonType="secondary"
        size="sm"
        @click="$emit('handle-edit')"
      >
        Edit
      </PrimaryButton>
      <PrimaryButton
        v-if="!props.prompt.json_mode"
        buttonType="primary"
        size="sm"
        :disabled="executeLoading"
        @click="executeStream()"
      >
        Stream
      </PrimaryButton>
      <PrimaryButton
        buttonType="primary"
        size="sm"
        :disabled="executeLoading"
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
  log,
  fetchLogById,
  fetchLogByProviderId
} = useLogs();


const systemPrompt = ref(props.prompt.system)
const userPrompt = ref(props.prompt.user)
const jsonContext = ref({})
const testResponse = ref<string | null>(null)
const logResponse = ref<ApiLogReponse | null>(null)
const showLog = ref(false)
const showJsonContext = ref(false)
const showResponse = ref(true)
const executeLoading = ref(false)


// Computed property for rendered system prompt with variables replaced
const renderedSystemPrompt = computed(() => {
  if (!systemPrompt.value) return '';
  
  let rendered = systemPrompt.value;
  // Replace all handlebars variables with their values from jsonContext
  Object.entries(jsonContext.value).forEach(([key, value]) => {
    const regex = new RegExp(`\\{\\{\\s*${key}\\s*\\}\\}`, 'g');
    rendered = rendered.replace(regex, value as string);
  });
  
  return rendered;
});

// Computed property for rendered user prompt with variables replaced
const renderedUserPrompt = computed(() => {
  if (!userPrompt.value) return '';
  
  let rendered = userPrompt.value;
  // Replace all handlebars variables with their values from jsonContext
  Object.entries(jsonContext.value).forEach(([key, value]) => {
    const regex = new RegExp(`\\{\\{\\s*${key}\\s*\\}\\}`, 'g');
    rendered = rendered.replace(regex, value as string);
  });
  
  return rendered;
});

const templateFields = computed<string[]>(() => {
  // Only require system prompt to be present
  if (!props.prompt || !props.prompt.system) return [];

  // Use user prompt only if it exists
  let template = props.prompt.system;
  if (props.prompt.user) {
    template += '\n' + props.prompt.user;
  }
  
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

  // Update the context object with the new value
  jsonContext.value[key] = value
}

async function execute() {
  try {
    executeLoading.value = true
    // Prepare messages based on prompt type
    let messages = [];
    
    // For dynamic_both prompts, we need to handle system and user contexts separately
    if (props.prompt.prompt_type === 'dynamic_both') {
      messages = [
        {
          role: 'system',
          content: JSON.stringify(jsonContext.value)
        },
        {
          role: 'user',
          content: JSON.stringify(jsonContext.value)
        }
      ];
    } else {
      // For other prompt types, just include the system message
      messages = [
        {
          role: 'system',
          content: JSON.stringify(jsonContext.value)
        }
      ];
    }
    
    // Get API client from composable
    const { executeApiCompletion } = usePrompts();
    
    // Execute the prompt using the composable
    const response = await executeApiCompletion(
      props.prompt.key,
      // @ts-ignore
      messages,
      props.prompt.json_mode
    );
    
    // Extract content from response
    if (response.choices && response.choices.length > 0) {
      testResponse.value = response.choices[0].message.content;
      
      // Get log by provider response ID
      if (response.id) {
        await getLogByProviderId(response.id);
      }
    }
  } catch (err) {
    console.error('Error executing prompt:', err);
  } finally {
    executeLoading.value = false
  }
}

// Error handling
const error = ref<Error | null>(null)

const executeStream = async () => {
  executeLoading.value = true
  testResponse.value = '';
  error.value = null;

  // Prepare messages based on prompt type
  let messages = [];
  
  // For dynamic_both prompts, we need to handle system and user contexts separately
  if (props.prompt.prompt_type === 'dynamic_both') {
    messages = [
      {
        role: 'system',
        content: JSON.stringify(jsonContext.value)
      },
      {
        role: 'user',
        content: JSON.stringify(jsonContext.value)
      }
    ];
  } else {
    // For other prompt types, just include the system message
    messages = [
      {
        role: 'system',
        content: JSON.stringify(jsonContext.value)
      }
    ];
  }

  // Get API client from composable
  const { executeApiCompletionStream } = usePrompts();
  
  // Execute streaming using the composable
  await executeApiCompletionStream(
    props.prompt.key,
    // @ts-ignore
    messages,
    props.prompt.json_mode,
    async (chunk) => {
      // Handle each chunk of the stream
      
      try {
        // Parse chunk as JSON
        const parsed = JSON.parse(chunk);
        
        // Check if this is the [DONE] sentinel
        if (parsed.choices && 
            parsed.choices.length > 0 && 
            parsed.choices[0].delta && 
            parsed.choices[0].delta.content === "[DONE]") {
          
          // Get log using the provider response ID
          if (parsed.id) {
            await getLogByProviderId(parsed.id);
          }
          executeLoading.value = false;
          return;
        }
        
        // Process regular content delta
        if (parsed.choices && parsed.choices.length > 0) {
          const choice = parsed.choices[0];
          
          // Add delta content if available
          if (choice.delta && choice.delta.content) {
            testResponse.value += choice.delta.content;
          }
        }
      } catch (err) {
        console.error("Error parsing streaming response:", err);
      }
    },
    (err) => {
      error.value = err;
    }
  );
}

async function getLogRecord(log_id: number) {
  await fetchLogById(log_id)
  logResponse.value = log.value
}

async function getLogByProviderId(provider_id: string) {
  await fetchLogByProviderId(provider_id)
  logResponse.value = log.value
}
</script>

<style scoped>
.response-content {
  white-space: pre-line;
}
</style>
