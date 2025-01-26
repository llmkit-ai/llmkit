<template>
  <div class="font-mono">
    <!-- Form Section -->
    <div>
      <form v-if="mode === 'edit' || mode === 'new'" @submit.prevent="handleSubmit">
        <div class="space-y-12">
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900">
              {{ mode === 'edit' ? 'Edit Prompt' : 'New Prompt' }}
            </h2>
            <div class="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6">
              <!-- Prompt Key Input -->
              <div class="sm:col-span-4">
                <label for="prompt-key" class="block text-sm/6 font-medium text-gray-900">Prompt key</label>
                <div class="mt-2">
                  <div class="flex items-center border-2 border-black bg-white">
                    <input
                      v-model="promptKey"
                      type="text"
                      name="prompt-key"
                      id="prompt-key"
                      class="block min-w-0 grow p-2 text-base text-gray-900 focus:outline-none sm:text-sm/6"
                      placeholder="PROMPT-KEY-HERE"
                    >
                  </div>
                </div>
              </div>

              <!-- Model Selection -->
              <div class="sm:col-span-4">
                <label for="model" class="block text-sm/6 font-medium text-gray-900">Select model</label>
                <div class="relative mt-2">
                  <button
                    type="button"
                    class="grid w-full cursor-default grid-cols-1 border-2 border-black bg-white p-2 text-left text-gray-900"
                    aria-haspopup="listbox"
                    aria-expanded="true"
                    aria-labelledby="listbox-label"
                    @click="toggleDropdown"
                  >
                    <span class="col-start-1 row-start-1 flex w-full gap-2 pr-6">
                      <span class="truncate">{{ selectedModel ? selectedModel.model : 'Select a model' }}</span>
                      <span v-if="selectedModel" class="truncate text-gray-500">{{ selectedModel.provider }}</span>
                    </span>
                    <svg
                      class="col-start-1 row-start-1 size-5 self-center justify-self-end text-gray-500 sm:size-4"
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
                    class="absolute z-10 mt-1 max-h-60 w-full overflow-auto border-2 border-black bg-white py-1 text-base"
                    tabindex="-1"
                    role="listbox"
                    aria-labelledby="listbox-label"
                  >
                    <li
                      v-for="model in models"
                      :key="model.id"
                      class="relative cursor-default select-none py-2 pl-3 pr-9 text-gray-900"
                      :class="{ 'bg-black text-white': model.id === selectedModel?.id }"
                      role="option"
                      @click="selectModel(model)"
                    >
                      <div class="flex">
                        <span class="truncate">{{ model.model }}</span>
                        <span class="ml-2 truncate text-gray-500">{{ model.provider }}</span>
                      </div>
                      <span
                        v-if="model.id === selectedModel?.id"
                        class="absolute inset-y-0 right-0 flex items-center pr-4"
                        :class="{'text-white': model.id === selectedModel?.id, 'text-black': model.id !== selectedModel?.id}"
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

              <!-- LLM Parameters -->
              <div class="sm:col-span-2">
                <label for="max-tokens" class="block text-sm/6 font-medium text-gray-900">Max Tokens</label>
                <div class="mt-2">
                  <input
                    v-model.number="maxTokens"
                    type="number"
                    min="1"
                    id="max-tokens"
                    class="block w-full border-2 border-black p-2 text-base text-gray-900 focus:outline-none sm:text-sm/6"
                  >
                </div>
              </div>

              <div class="sm:col-span-2">
                <label for="temperature" class="block text-sm/6 font-medium text-gray-900">
                  Temperature ({{ temperatureValue.toFixed(2) }})
                </label>
                <div class="mt-2">
                  <input
                    v-model.number="temperatureValue"
                    type="range"
                    min="0"
                    max="2"
                    step="0.1"
                    id="temperature"
                    class="block w-full border-2 border-black p-2 text-base text-gray-900 focus:outline-none sm:text-sm/6"
                  >
                </div>
              </div>

              <div class="sm:col-span-2">
                <label class="inline-flex items-center gap-2">
                  <input
                    v-model="jsonMode"
                    type="checkbox"
                    class="border-2 border-black"
                  >
                  <span class="text-sm/6 font-medium text-gray-900">JSON Mode</span>
                </label>
              </div>

              <!-- Prompt Content -->
              <div class="col-span-full">
                <label for="prompt" class="block text-sm/6 font-medium text-gray-900">Prompt</label>
                <div class="mt-2">
                  <textarea
                    v-model="prompt"
                    name="prompt"
                    id="prompt"
                    rows="15"
                    class="block w-full border-2 border-black p-2 text-base text-gray-900 focus:outline-none sm:text-sm/6"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Form Actions -->
        <div class="mt-6 flex items-center justify-end gap-x-6">
          <button
            type="button"
            @click="handleCancel"
            class="text-sm/6 text-gray-900"
          >
            Cancel
          </button>
          <button
            :disabled="!formIsValid || isLoading"
            type="submit"
            class="text-sm/6 p-2 border-2 border-black disabled:opacity-50"
          >
            {{ isLoading ? 'Saving...' : mode === 'edit' ? 'Update' : 'Save' }}
          </button>
        </div>
      </form>

      <div v-if="mode === 'test' && props.prompt">
        <TestPrompt :prompt="props.prompt" />
      </div>

      <!-- View Mode -->
      <div v-if="mode === 'view'">
        <div class="px-4 sm:px-0">
          <h3 class="text-base/7 font-semibold text-gray-900">Prompt Details</h3>
          <p class="mt-1 max-w-2xl text-sm/6 text-gray-500">Configuration and content for this prompt.</p>
        </div>
        <div class="mt-6">
          <dl class="grid grid-cols-1 sm:grid-cols-2">
            <div class="border-t border-gray-100 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-gray-900">Prompt Key</dt>
              <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2">{{ promptKey }}</dd>
            </div>
            <div class="border-t border-gray-100 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-gray-900">Model</dt>
              <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2">{{ selectedModel?.model }} ({{ selectedModel?.provider }})</dd>
            </div>
            <div class="border-t border-gray-100 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-gray-900">Max Tokens</dt>
              <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2">{{ maxTokens }}</dd>
            </div>
            <div class="border-t border-gray-100 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-gray-900">Temperature</dt>
              <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2">{{ temperatureValue.toFixed(2) }}</dd>
            </div>
            <div class="border-t border-gray-100 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-gray-900">JSON Mode</dt>
              <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2">{{ jsonMode ? 'Enabled' : 'Disabled' }}</dd>
            </div>
            <div class="border-t border-gray-100 px-4 py-6 sm:col-span-2 sm:px-0">
              <dt class="text-sm/6 font-medium text-gray-900">Prompt Content</dt>
              <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2 whitespace-pre-wrap">{{ prompt }}</dd>
            </div>
          </dl>
        </div>
        <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-2">
          <button
            type="button"
            @click="handleEdit"
            class="text-sm/6 p-2 border-2 border-black"
          >
            Edit
          </button>
          <button
            type="button"
            @click="handleTest"
            class="text-sm/6 p-2 border-2 border-black bg-black text-white"
          >
            Test
          </button>
        </div>
      </div>
    </div>

    <!-- Execution Section -->
    <div v-if="mode === 'view'">
      <slot name="execution-section"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';
import type { Model } from '~/types/response/models';

const props = defineProps<{
  mode: 'view' | 'edit' | 'new' | 'test'
  prompt?: Prompt | null
}>();

// Inject event handlers
const handleCancel = inject('handleCancel', () => {});
const handleEdit = inject('handleEdit', () => {});
const handleSaved = inject<(prompt: Prompt) => void>('handleSaved', () => {
  console.error('handleSaved not provided');
});
const handleTest = inject('handleTest', () => {});

// Form state
const promptKey = ref(props.prompt?.key || '');
const prompt = ref(props.prompt?.prompt || '');
const selectedModelId = ref<number | null>(props.prompt?.model_id || null);
const maxTokens = ref(props.prompt?.max_tokens || 256);
const temperatureValue = ref(props.prompt?.temperature || 0.7);
const jsonMode = ref(props.prompt?.json_mode || false);
const isLoading = ref(false);
const isOpen = ref(false);

// Dependencies
const { createPrompt, updatePrompt } = usePrompts();
const { models, loading: modelsLoading, fetchModels } = useModels();

// Computed
const selectedModel = computed(() =>
  models.value?.find(m => m.id === selectedModelId.value) || null
);

const formIsValid = computed(() =>
  promptKey.value.trim() !== '' &&
  prompt.value.trim() !== '' &&
  selectedModelId.value !== null &&
  maxTokens.value > 0 &&
  temperatureValue.value >= 0 &&
  temperatureValue.value <= 2
);

// Watchers
watch(() => props.prompt, (newPrompt) => {
  if (newPrompt) {
    promptKey.value = newPrompt.key;
    prompt.value = newPrompt.prompt;
    selectedModelId.value = newPrompt.model_id;
    maxTokens.value = newPrompt.max_tokens;
    temperatureValue.value = newPrompt.temperature;
    jsonMode.value = newPrompt.json_mode;
  } else {
    resetForm();
  }
}, { immediate: true });

// Lifecycle
onMounted(fetchModels);

// Methods
function toggleDropdown() {
  isOpen.value = !isOpen.value;
}

function selectModel(model: Model) {
  selectedModelId.value = model.id;
  isOpen.value = false;
}

function resetForm() {
  promptKey.value = '';
  prompt.value = '';
  selectedModelId.value = null;
  maxTokens.value = 256;
  temperatureValue.value = 0.7;
  jsonMode.value = false;
}

async function handleSubmit() {
  if (!formIsValid.value || isLoading.value) return;
  
  isLoading.value = true;
  try {
    const payload = {
      key: promptKey.value,
      prompt: prompt.value,
      model_id: selectedModelId.value!,
      max_tokens: maxTokens.value,
      temperature: temperatureValue.value,
      json_mode: jsonMode.value
    };

    const result = props.mode === 'edit'
      ? await updatePrompt(props.prompt!.id, payload)
      : await createPrompt(payload);

    handleSaved(result)
  } finally {
    isLoading.value = false;
  }
}
</script>
