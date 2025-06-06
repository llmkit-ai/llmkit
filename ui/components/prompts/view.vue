<template>
  <div>
    <div class="px-4 sm:px-0">
      <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Prompt Details</h3>
      <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Configuration and content for this prompt.</p>
    </div>
    <div v-if="props.prompt" class="mt-6">
      <dl class="grid grid-cols-1 sm:grid-cols-3">
        <!-- Prompt Key -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Key</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ props.prompt.key }}</dd>
        </div>

        <!-- Version Info with Selector -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Version</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2 flex items-center">
            <select 
              v-if="versions.length > 0" 
              v-model="selectedVersionId" 
              @change="changeVersion" 
              class="mr-2 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-600 rounded text-sm/6"
            >
              <option v-for="version in versions" :key="version.version_id" :value="version.version_id">
                Version {{ version.version_number }}
              </option>
            </select>
            <span v-else><b>{{ props.prompt.version_number }}</b></span>
            <span class="ml-1">- <i>{{ formatDate(props.prompt.updated_at) }}</i></span>
          </dd>
        </div>


        <!-- Model -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-3 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Model</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">
            <div class="flex flex-col space-y-2">
              <div>{{ props.prompt.model }} ({{ props.prompt.provider }})</div>
              <div class="flex flex-wrap gap-2">
                <span v-if="props.prompt.supports_json" class="inline-flex items-center rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-800 dark:bg-blue-900/20 dark:text-blue-400">
                  JSON
                </span>
                <span v-if="props.prompt.supports_json_schema" class="inline-flex items-center rounded-full bg-purple-100 px-2 py-0.5 text-xs text-purple-800 dark:bg-purple-900/20 dark:text-purple-400">
                  JSON Schema
                </span>
                <span v-if="props.prompt.supports_tools" class="inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-800 dark:bg-green-900/20 dark:text-green-400">
                  Tools
                </span>
                <span v-if="props.prompt.is_reasoning" class="inline-flex items-center rounded-full bg-amber-100 px-2 py-0.5 text-xs text-amber-800 dark:bg-amber-900/20 dark:text-amber-400">
                  Reasoning
                </span>
                <span v-if="!props.prompt.supports_json && !props.prompt.supports_json_schema && !props.prompt.supports_tools && !props.prompt.is_reasoning" class="text-neutral-500 dark:text-neutral-400 text-xs">
                  No capabilities
                </span>
              </div>
            </div>
          </dd>
        </div>

        <!-- Prompt Type -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Type</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ formatPromptType(props.prompt.prompt_type) }}</dd>
        </div>

        <!-- JSON Mode -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">JSON Mode</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ props.prompt.json_mode ? 'Enabled' : 'Disabled' }}</dd>
        </div>

        <!-- Chat Mode -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Chat Mode</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ props.prompt.is_chat ? 'Enabled' : 'Disabled' }}</dd>
        </div>
        
        <!-- Reasoning Effort (only for reasoning models) -->
        <div v-if="props.prompt.is_reasoning" class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Reasoning Effort</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ props.prompt.reasoning_effort || 'Default' }}</dd>
        </div>
        
        <!-- Max Tokens -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Max Tokens</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ props.prompt.max_tokens }}</dd>
        </div>

        <!-- Temperature -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Temperature</dt>
          <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ props.prompt.temperature.toFixed(2) }}</dd>
        </div>

        <!-- JSON Schema -->
        <div v-if="props.prompt.json_mode && props.prompt.json_schema" class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-3 sm:px-0">
          <div class="flex items-center justify-between">
            <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">JSON Schema</dt>
            <button
              @click="showJsonSchema = !showJsonSchema"
              class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-300"
            >
              {{ showJsonSchema ? 'Hide' : 'Show' }}
            </button>
          </div>
          <!-- Schema preview with gradient -->
          <dd v-if="!showJsonSchema" class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 font-mono whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2 relative max-h-20 overflow-hidden">
            {{ formatJsonSchema(props.prompt.json_schema) }}
            <div class="absolute inset-x-0 bottom-0 h-16 bg-gradient-to-t from-neutral-100 dark:from-neutral-800 to-transparent"></div>
          </dd>
          <!-- Full schema -->
          <dd v-else class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 font-mono whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ formatJsonSchema(props.prompt.json_schema) }}</dd>
        </div>


        <!-- Version Diff -->
        <div v-if="props.prompt.system_version_diff || props.prompt.user_version_diff" class="col-span-3 bg-neutral-100 dark:bg-neutral-800 p-4">
          <div class="flex items-center justify-between">
            <p class="text-xs text-neutral-900 dark:text-neutral-300">Prompt version diff</p>
            <button
              @click="showVersionDiff = !showVersionDiff"
              class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-300"
            >
              {{ showVersionDiff ? 'Hide' : 'Show' }}
            </button>
          </div>
          <div v-if="showVersionDiff" class="whitespace-pre-line mt-3 dark:text-neutral-300 text-sm">
            <div v-if="props.prompt.system_version_diff" class="dark:bg-neutral-700 bg-neutral-200 p-1">
              <p class="font-bold text-xs text-neutral-900 dark:text-neutral-300">System prompt diff</p>
              <p class="mt-1 text-xs text-neutral-900 dark:text-neutral-300">{{ props.prompt.system_version_diff }}</p>
            </div>
            <div v-if="props.prompt.user_version_diff" class="mt-5 dark:bg-neutral-700 bg-neutral-200 p-1">
              <p class="font-bold text-xs text-neutral-900 dark:text-neutral-300">User prompt diff</p>
              <p class="mt-1 text-xs text-neutral-900 dark:text-neutral-300">{{ props.prompt.user_version_diff }}</p>
            </div>
          </div>
        </div>
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-3 sm:px-0">
          <div class="px-4 sm:px-0">
            <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</dt>
            <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ props.prompt.system }}</dd>
          </div>
          <!-- Only show User Prompt for dynamic_both type if it exists -->
          <div v-if="props.prompt.prompt_type === 'dynamic_both' && props.prompt.user" class="px-4 sm:px-0 mt-2">
            <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt</dt>
            <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ props.prompt.user }}</dd>
          </div>
        </div>
        
        <!-- Associated Tools Section -->
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-3 sm:px-0">
          <ViewTools
            :tools="props.prompt.tools || []"
            :chat-mode="props.prompt.is_chat"
            :model-supports-tools="props.prompt.supports_tools" 
          />
        </div>
      </dl>
    </div>
    <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-3">
      <PrimaryButton
        buttonType="secondary"
        size="sm"
        @click="handleManageTools()"
      >
        Manage Tools
      </PrimaryButton>
      <PrimaryButton
        buttonType="secondary"
        size="sm"
        @click="handleEdit()"
      >
        Edit
      </PrimaryButton>
      <PrimaryButton
        buttonType="primary"
        size="sm"
        @click="handleTest()"
      >
        Test
      </PrimaryButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { format, parseISO } from 'date-fns';
import type { Prompt } from '~/types/response/prompts';
import ViewTools from './view-tools.vue';

const props = defineProps<{
  prompt?: Prompt | null
}>();

const emit = defineEmits([
  "handle-edit", 
  "handle-test", 
  "toggle-tools-modal",
  "prompt-updated"
])

const { fetchPromptVersions, setActiveVersion } = usePrompts()
const showVersionDiff = ref(false)
const showJsonSchema = ref(false)
const versions = ref<Prompt[]>([])
const selectedVersionId = ref<number | null>(null)
const isLoading = ref(false)

watchEffect(async () => {
  if (props.prompt?.id) {
    selectedVersionId.value = props.prompt.version_id
    try {
      isLoading.value = true
      versions.value = await fetchPromptVersions(props.prompt.id)
    } catch (error) {
      console.error('Error fetching versions:', error)
    } finally {
      isLoading.value = false
    }
  }
})

async function changeVersion() {
  if (!props.prompt?.id || !selectedVersionId.value) return
  
  try {
    isLoading.value = true
    const updatedPrompt = await setActiveVersion(props.prompt.id, selectedVersionId.value)
    emit("prompt-updated", updatedPrompt)
  } catch (error) {
    console.error('Error changing version:', error)
  } finally {
    isLoading.value = false
  }
}

function handleEdit() {
  emit("handle-edit")
}

function handleTest() {
  emit("handle-test")
}

function handleManageTools() {
  emit("toggle-tools-modal")
}

function formatDate(dateString: string): string {
  const date = parseISO(dateString);
  return format(date, 'MM-dd-yyyy');
}

function formatPromptType(type: string): string {
  switch (type) {
    case 'static':
      return 'Static System Prompt';
    case 'dynamic_system':
      return 'Dynamic System Prompt';
    case 'dynamic_both':
      return 'Dynamic System & User Prompts';
    default:
      return type;
  }
}

function formatJsonSchema(schema: string | null): string {
  if (!schema) return '';
  
  try {
    // Parse and prettify the JSON schema
    const parsed = JSON.parse(schema);
    return JSON.stringify(parsed, null, 2);
  } catch (e) {
    // Return the original string if it's not valid JSON
    return schema;
  }
}
</script>
