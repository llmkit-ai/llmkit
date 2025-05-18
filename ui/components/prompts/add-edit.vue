<template>
  <div>
    <div v-if="mode === 'new' && currentCreatePromptStep === 1">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-neutral-200">
          Create new prompt
        </h3>
        <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">
          Select which type of prompt you want to create. Depending on the specific
          application there are different needs.
        </p>
      </div>
      <div class="mt-6 grid grid-cols-1 gap-4">
        <button
          v-for="c in createPromptOptions"
          :key="c.type"
          class="text-left inline-flex p-4 border border-neutral-600 dark:border-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 dark:hover:border-neutral-200 hover:border-neutral-900"
          @click="selectPromptType(c.type)"
        >
          <div>
            <h3 class="text-lg font-semibold text-neutral-900 dark:text-neutral-200">
              {{ c.title }}
            </h3>
            <p class="mt-1 text-neutral-600 dark:text-neutral-400">
              {{ c.description }}
            </p>

            <h4 class="mt-6 font-semibold text-neutral-900 dark:text-neutral-300">
              Use cases
            </h4>
            <ul class="mt-1 pl-6 list-disc text-neutral-600 dark:text-neutral-400">
              <li v-for="u in c.usedFor" :key="u" class="">{{ u }}</li>
            </ul>
          </div>
        </button>
      </div>
    </div>

    <form v-if="mode === 'edit' || (mode === 'new' && currentCreatePromptStep === 2)">
      <div class="space-y-12">
        <div>
          <h2 class="text-base/7 font-semibold text-neutral-900 dark:text-white">
            {{ mode === 'edit' ? 'Edit Prompt' : 'New Prompt' }}
          </h2>
          <div class="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6">
            <!-- Prompt Key Input -->
            <div class="sm:col-span-3">
              <label for="prompt-key" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Key</label>
              <div class="mt-2">
                <div class="flex items-center border-2 border-black dark:border-white bg-white dark:bg-neutral-800">
                  <input
                    v-model="promptKey"
                    type="text"
                    name="prompt-key"
                    id="prompt-key"
                    class="block min-w-0 grow p-2 text-base text-neutral-900 dark:text-white bg-white dark:bg-neutral-800 focus:outline-none sm:text-sm/6"
                    placeholder="PROMPT-KEY-HERE"
                  >
                </div>
              </div>
            </div>
            
            <!-- Model Selection -->
            <div class="sm:col-span-3">
              <label for="model" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Model</label>
              <div class="relative mt-2">
                <button
                  type="button"
                  class="grid w-full cursor-default grid-cols-1 border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-left text-neutral-900 dark:text-white"
                  aria-haspopup="listbox"
                  aria-expanded="true"
                  aria-labelledby="listbox-label"
                  @click="toggleDropdown"
                >
                  <span class="col-start-1 row-start-1 flex w-full gap-2 pr-6">
                    <span class="truncate">{{ selectedModel ? selectedModel.name : 'Select a model' }}</span>
                    <span v-if="selectedModel" class="truncate text-neutral-500 dark:text-neutral-400">{{ selectedModel.provider_name }}</span>
                  </span>
                  <svg
                    class="col-start-1 row-start-1 size-5 self-center justify-self-end text-neutral-500 dark:text-neutral-400 sm:size-4"
                    viewBox="0 0 16 16"
                    fill="currentColor"
                    aria-hidden="true"
                    data-slot="icon"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M5.22 10.22a.75.75 0 0 1 1.06 0L8 11.94l1.72-1.72a.75.75 0 1 1 1.06 1.06l-2.25 2.25a.75.75 0 0 1-1.06 0l-2.25-2.25a.75.75 0 0 1 0-1.06ZM10.78 5.78a.75.75 0 0 1-1.06 0L8 4.06 6.28 5.78a.75.75 0 0 1-1.06-1.06l2.25-2.25a.75.75 0 0 1 1.06 0l2.25 2.25a.75.75 0 0 1 0 1.06Z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </button>
                <ul
                  v-if="isOpen"
                  class="absolute z-10 mt-1 max-h-60 w-full overflow-auto border-2 border-black dark:border-white bg-white dark:bg-neutral-800 py-1 text-base"
                  tabindex="-1"
                  role="listbox"
                  aria-labelledby="listbox-label"
                >
                  <li
                    v-for="model in models"
                    :key="model.id"
                    class="relative cursor-default select-none py-2 pl-3 pr-9 text-neutral-900 dark:text-white"
                    :class="{ 'bg-black text-white dark:text-black': model.id === selectedModel?.id }"
                    role="option"
                    @click="selectModel(model)"
                  >
                    <div class="flex">
                      <span class="truncate">{{ model.name }}</span>
                      <span class="ml-2 truncate text-neutral-500 dark:text-neutral-400">{{ model.provider_name }}</span>
                    </div>
                    <span
                      v-if="model.id === selectedModel?.id"
                      class="absolute inset-y-0 right-0 flex items-center pr-4"
                      :class="{'text-white dark:text-black': model.id === selectedModel?.id, 'text-black dark:text-white': model.id !== selectedModel?.id}"
                    >
                      <svg class="size-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                        <path
                          fill-rule="evenodd"
                          d="M16.704 4.153a.75.75 0 0 1 .143 1.052l-8 10.5a.75.75 0 0 1-1.127.075l-4.5-4.5a.75.75 0 0 1 1.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 0 1 1.05-.143Z"
                          clip-rule="evenodd"
                        />
                      </svg>
                    </span>
                  </li>
                </ul>
              </div>
            </div>

            <!-- Prompt Type Dropdown -->
            <div class="sm:col-span-2">
              <label class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Type</label>
              <div class="mt-2">
                <select 
                  v-model="promptType"
                  @change="handlePromptTypeChange"
                  class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                >
                  <option 
                    v-for="option in createPromptOptions" 
                    :key="option.type" 
                    :value="option.type"
                  >
                    {{ option.title }}
                  </option>
                </select>
              </div>
            </div>
            
            <!-- Mode Options -->
            <div class="sm:col-span-4 grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm/6 font-medium text-neutral-900 dark:text-white">JSON Mode</label>
                <div class="mt-2">
                  <select 
                    v-model="jsonMode"
                    class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                  >
                    <option :value="false">No</option>
                    <option :value="true">Yes</option>
                  </select>
                </div>
                <p v-if="selectedModel && jsonMode && !selectedModel.supports_json" class="mt-1 text-xs text-amber-500">
                  Warning: Selected model doesn't support JSON mode
                </p>
              </div>
              
              <div>
                <label class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Chat Mode</label>
                <div class="mt-2">
                  <select 
                    v-model="isChat"
                    :disabled="!canEnableChat || jsonMode"
                    class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                    :class="{ 'opacity-50': !canEnableChat || jsonMode }"
                  >
                    <option :value="false">No</option>
                    <option :value="true" :disabled="!canEnableChat || jsonMode">Yes</option>
                  </select>
                </div>
                <div class="text-xs text-neutral-500">
                  <span v-if="!canEnableChat">
                    Chat mode is only available for Static and Dynamic System prompts
                  </span>
                  <span v-else-if="jsonMode">
                    Chat mode cannot be used with JSON mode
                  </span>
                </div>
              </div>
            </div>
            
            <!-- Max Tokens -->
            <div class="sm:col-span-2">
              <label for="max-tokens" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Max Tokens</label>
              <div class="mt-2">
                <input
                  v-model.number="maxTokens"
                  type="number"
                  min="1"
                  id="max-tokens"
                  class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                >
              </div>
            </div>

            <!-- Temperature -->
            <div class="sm:col-span-2">
              <label for="temperature" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">
                Temperature
              </label>
              <div class="mt-2">
                <input
                  v-model.number="temperatureValue"
                  type="number"
                  min="0"
                  max="2"
                  step="0.1"
                  id="temperature"
                  class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                >
              </div>
              <p class="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
                Value between 0 and 2 (0.7 is recommended for balanced responses)
              </p>
            </div>
              
            <!-- JSON Schema (only visible when JSON Mode is enabled) -->
            <div v-if="jsonMode" class="col-span-full">
              <label for="json-schema" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">
                JSON Schema <span class="font-normal text-neutral-500 dark:text-neutral-400">(optional)</span>
              </label>
              <div class="mt-2">
                <textarea
                  v-model="jsonSchema"
                  name="json-schema"
                  id="json-schema"
                  rows="3"
                  placeholder="Enter a valid JSON Schema to validate responses"
                  @input="debouncedValidateSchema"
                  class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6 font-mono"
                  :class="{'border-amber-500 dark:border-amber-400': isValidatingSchema, 'border-red-500 dark:border-red-400': schemaValidationErrors.length > 0 && !isValidatingSchema}"
                />
              </div>
              <p class="mt-1 text-xs" :class="{'text-neutral-500 dark:text-neutral-400': schemaValidationErrors.length === 0, 'text-red-500 dark:text-red-400': schemaValidationErrors.length > 0}">
                <span v-if="schemaValidationErrors.length > 0">
                  {{ schemaValidationErrors[0] }}
                </span>
                <span v-else-if="isValidatingSchema">
                  Validating schema...
                </span>
                <span v-else>
                  Adding a JSON Schema helps enforce structure in model responses when JSON mode is enabled.
                </span>
              </p>
              <p v-if="selectedModel && jsonSchema && !selectedModel.supports_json_schema" class="mt-1 text-xs text-amber-500">
                Warning: Selected model doesn't support JSON Schema
              </p>
            </div>

            <!-- System Prompt -->
            <div class="col-span-full">
              <label for="system-prompt" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</label>
              <div class="mt-2">
                <textarea
                  v-model="systemPrompt"
                  name="system-prompt"
                  id="system-prompt"
                  ref="systemPromptTextarea"
                  :rows="getTextareaRows(systemPrompt, 5, 1000)"
                  class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                />
              </div>
            </div>
            
            <!-- Show User Prompt only for Dynamic System & User Prompts type -->
            <div v-if="promptType === 'dynamic_both'" class="col-span-full">
              <label for="user-prompt" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt</label>
              <div class="mt-2">
                <textarea
                  v-model="userPrompt"
                  name="user-prompt"
                  id="user-prompt"
                  ref="userPromptTextarea"
                  :rows="getTextareaRows(userPrompt, 1, 1000)"
                  class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                />
              </div>
            </div>
            
            <!-- We've moved Tool Management to the View page -->
          </div>
        </div>
      </div>

      <!-- Form Actions -->
      <div class="mt-6 flex items-center justify-end gap-x-3">
        <PrimaryButton
          buttonType="secondary"
          size="sm"
          @click="$emit('handle-cancel')"
        >
          Cancel
        </PrimaryButton>
        <PrimaryButton
          v-if="mode === 'edit'"
          buttonType="danger"
          size="sm"
          @click="handleDelete()"
        >
          Delete
        </PrimaryButton>
        <PrimaryButton
          buttonType="primary"
          size="sm"
          :disabled="!isFormValid"
          @click="handleSubmit()"
        >
          {{ mode === 'edit' ? 'Update':'Create'  }}
        </PrimaryButton>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';
import type { Model } from '~/types/response/models';
import type { PromptCreateDTO, PromptUpdateDTO } from '~/types/components/prompt';
import type { SchemaValidationResponse } from '~/types/response/schema';
import { useDebounceFn } from '@vueuse/core';

const props = defineProps<{
  prompt: Prompt | null
  models: Model[]
  mode: "edit" | "new"
}>();

const emit = defineEmits<{
  "handle-cancel": [];
  "handle-create": [prompt: PromptCreateDTO];
  "handle-update": [prompt: PromptUpdateDTO];
  "handle-delete": [id: number];
}>();

// Initialize form values from props
const promptKey = ref(props.prompt?.key || '');
const systemPrompt = ref(props.prompt?.system || '');
const userPrompt = ref(props.prompt?.user || '');
const selectedModelId = ref<number | null>(props.prompt?.model_id || null);
const maxTokens = ref(props.prompt?.max_tokens || 256);
const temperatureValue = ref(props.prompt?.temperature || 0.7);
const jsonMode = ref(props.prompt?.json_mode || false);
const jsonSchema = ref(props.prompt?.json_schema || '');
const isValidatingSchema = ref(false);
const schemaValidationErrors = ref<string[]>([]);
const promptType = ref(props.prompt?.prompt_type || 'static');
// Private backing field for chat mode
const _isChat = ref(props.prompt?.json_mode ? false : props.prompt?.is_chat || false);
const isOpen = ref(false);
// Tools are now managed outside the edit view

const currentCreatePromptStep = ref(1);

const createPromptOptions = ref([
  { 
    title: "Static System Prompt", 
    type: "static",
    description: "Best used for back and forth chat scenarios or very simple prompts without any dynamic input.", 
    usedFor: ["Chat style prompts", "Basic prompts"],
    canBeChat: true
  },
  { 
    title: "Dynamic System Prompt", 
    type: "dynamic_system",
    description: "Best used for when you want a dynamic system prompt and your user input is dynamic text/json.", 
    usedFor: ["Dynamic application conditions", "Non structured user input", "One shot prompts (not chat)"],
    canBeChat: true
  },
  { 
    title: "Dynamic System & User Prompts", 
    type: "dynamic_both",
    description: "Best used for when you have structured system and user prompts and will provide a consistent structured payload.", 
    usedFor: ["Dynamic application conditions", "Dynamic and structured user input", "One shot prompts (not chat)"],
    canBeChat: false
  },
])

function selectPromptType(type: string) {
  promptType.value = type;
  // If the selected type doesn't support chat, disable chat mode
  const option = createPromptOptions.value.find(opt => opt.type === type);
  if (option && !option.canBeChat) {
    _isChat.value = false;
  }
  
  // Clear user prompt for non-dynamic_both types
  if (type !== 'dynamic_both') {
    userPrompt.value = '';
  }
  
  currentCreatePromptStep.value = 2;
}

// Schema validation
const { validateJsonSchema } = usePrompts();

const validateSchema = async () => {
  if (!jsonSchema.value.trim() || !jsonMode.value) {
    schemaValidationErrors.value = [];
    return;
  }

  try {
    isValidatingSchema.value = true;
    const response = await validateJsonSchema(jsonSchema.value);
    
    if (response.valid) {
      schemaValidationErrors.value = [];
    } else if (response.errors && response.errors.length > 0) {
      schemaValidationErrors.value = response.errors;
    }
  } catch (err) {
    // JSON parse errors are already handled in the validation function
    if (!(err instanceof SyntaxError)) {
      schemaValidationErrors.value = ['Failed to validate schema'];
    }
  } finally {
    isValidatingSchema.value = false;
  }
};

// Debounced validation
const debouncedValidateSchema = useDebounceFn(validateSchema, 500);

function handlePromptTypeChange() {
  // If the selected type doesn't support chat, disable chat mode
  const option = createPromptOptions.value.find(opt => opt.type === promptType.value);
  if (option && !option.canBeChat) {
    _isChat.value = false;
  }
  
  // Clear user prompt for non-dynamic_both types
  if (promptType.value !== 'dynamic_both') {
    userPrompt.value = '';
  }
  
  // Reset validation error display when type changes
  showValidationErrors.value = false;
}

// Computed
const selectedModel = computed(() =>
  props.models.find(m => m.id === selectedModelId.value) || null
);

const canEnableChat = computed(() => {
  const option = createPromptOptions.value.find(opt => opt.type === promptType.value);
  return option ? option.canBeChat : false;
});

// Computed property with getter/setter for chat mode
const isChat = computed({
  get: () => {
    // If JSON mode is enabled or prompt type doesn't support chat, always return false
    if (jsonMode.value || !canEnableChat.value) {
      return false;
    }
    return _isChat.value;
  },
  set: (value) => {
    // Only allow setting to true if JSON mode is disabled and prompt type supports chat
    if (!jsonMode.value && canEnableChat.value) {
      _isChat.value = value;
    } else {
      _isChat.value = false;
    }
  }
});

function toggleDropdown() {
  isOpen.value = !isOpen.value;
}

function selectModel(model: Model) {
  selectedModelId.value = model.id;
  isOpen.value = false;

  // Show warnings for potentially incompatible features
  if (jsonMode.value && !model.supports_json) {
    // Add a warning notification about JSON mode incompatibility
    console.warn("Selected model doesn't support JSON mode but JSON mode is enabled");
  }
  
  if (jsonSchema.value && !model.supports_json_schema) {
    // Add a warning notification about JSON Schema incompatibility
    console.warn("Selected model doesn't support JSON Schema but a schema is defined");
  }
  
  if (props.prompt?.tools?.length > 0 && !model.supports_tools) {
    // Add a warning notification about tools incompatibility
    console.warn("Selected model doesn't support tools but tools are associated with this prompt");
  }
}

// Auto-expand textarea based on content, respecting min and max rows
function getTextareaRows(text: string, minRows: number, maxRows: number): number {
  if (!text) return minRows;
  
  const lineCount = (text.match(/\n/g) || []).length + 1;
  return Math.min(Math.max(lineCount, minRows), maxRows);
}


// Computed property for validation errors
const validationErrors = computed(() => {
  const errors: string[] = [];
  
  // Check prompt key
  if (!promptKey.value.trim()) {
    errors.push('Prompt key is required');
  }
  
  // Check system prompt
  if (!systemPrompt.value.trim()) {
    errors.push('System prompt is required');
  }
  
  // Check user prompt only for dynamic_both
  if (promptType.value === 'dynamic_both' && !userPrompt.value.trim()) {
    errors.push('User prompt is required for Dynamic System & User Prompts type');
  }
  
  // Check model selection
  if (selectedModelId.value === null) {
    errors.push('Model selection is required');
  }
  
  // Validate JSON Schema if provided and JSON mode is enabled
  if (jsonMode.value && jsonSchema.value.trim()) {
    try {
      JSON.parse(jsonSchema.value);
    } catch (e) {
      errors.push('JSON Schema must be valid JSON');
    }

    // Add schema validation errors if available
    if (schemaValidationErrors.value.length > 0) {
      errors.push(...schemaValidationErrors.value);
    }
  }
  
  return errors;
});

// Computed property to check if form is valid
const isFormValid = computed(() => {
  // Basic validation errors
  if (validationErrors.value.length > 0) {
    return false;
  }
  
  // JSON schema validation - prevent submission if schema is invalid
  if (jsonMode.value && jsonSchema.value.trim() && schemaValidationErrors.value.length > 0) {
    return false;
  }
  
  return true;
});

// Watch for changes
watch(jsonMode, (newVal) => {
  // If JSON mode is enabled, validate schema
  if (newVal) {
    _isChat.value = false;
  } else {
    // Clear validation errors when JSON mode is disabled
    schemaValidationErrors.value = [];
  }
});

const handleDelete = () => {
  if (props.prompt?.id) {
    if (confirm(`Are you sure you want to delete the prompt "${props.prompt.key}"? This action cannot be undone.`)) {
      emit("handle-delete", props.prompt.id);
    }
  }
};

const handleSubmit = async () => {
  // If JSON mode is enabled with a schema, validate it immediately before proceeding
  if (jsonMode.value && jsonSchema.value.trim()) {
    await validateSchema();
  }
  
  // Check if form is valid (this includes schema validation errors check)
  if (!isFormValid.value) {
    return;
  }
  
  // Ensure user prompt is empty for non-dynamic_both types
  const finalUserPrompt = promptType.value === 'dynamic_both' ? userPrompt.value : '';
  
  // Only include jsonSchema if jsonMode is enabled
  const finalJsonSchema = jsonMode.value ? jsonSchema.value || null : null;
  
  // Get the actual chat mode value - should be false if json_mode is true
  const finalChatMode = !jsonMode.value && canEnableChat.value && _isChat.value;
  
  // We're no longer handling tools in this component
  
  if (props.mode === 'new') {
    emit("handle-create", {
      key: promptKey.value,
      system: systemPrompt.value,
      user: finalUserPrompt,
      model_id: selectedModelId.value,
      max_tokens: maxTokens.value,
      temperature: temperatureValue.value,
      json_mode: jsonMode.value,
      json_schema: finalJsonSchema,
      prompt_type: promptType.value,
      is_chat: finalChatMode
    });
  } else {
    emit("handle-update", {
      id: props.prompt!.id,
      key: promptKey.value,
      system: systemPrompt.value,
      user: finalUserPrompt,
      model_id: selectedModelId.value,
      max_tokens: maxTokens.value,
      temperature: temperatureValue.value,
      json_mode: jsonMode.value,
      json_schema: finalJsonSchema,
      prompt_type: promptType.value,
      is_chat: finalChatMode
    });
  }
};
</script>
