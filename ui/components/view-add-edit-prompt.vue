<template>
  <div class="font-mono">
    <!-- Form Section -->
    <div>
      <form v-if="mode === 'edit' || mode === 'new'" >
        <div class="space-y-12">
          <div>
            <h2 class="text-base/7 font-semibold text-neutral-900 dark:text-white">
              {{ mode === 'edit' ? 'Edit Prompt' : 'New Prompt' }}
            </h2>
            <div class="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6">
              <!-- Prompt Key Input -->
              <div class="sm:col-span-4">
                <label for="prompt-key" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt key</label>
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
              <div class="sm:col-span-4">
                <label for="model" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Select model</label>
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

              <!-- LLM Parameters -->
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

              <div class="sm:col-span-2">
                <label for="temperature" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">
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
                    class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                  >
                </div>
              </div>

              <div class="sm:col-span-2">
                <label class="inline-flex items-center gap-2">
                  <input
                    v-model="jsonMode"
                    type="checkbox"
                    class="border-2 border-black dark:border-white"
                  >
                  <span class="text-sm/6 font-medium text-neutral-900 dark:text-white">JSON Mode</span>
                </label>
              </div>

              <!-- Prompt Content -->
              <div class="col-span-full">
                <label for="system-prompt" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</label>
                <div class="mt-2">
                  <textarea
                    v-model="systemPrompt"
                    name="system-prompt"
                    id="system-prompt"
                    rows="5"
                    class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                  />
                </div>
              </div>
              <div class="col-span-full">
                <label for="user-prompt" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt</label>
                <div class="mt-2">
                  <textarea
                    v-model="userPrompt"
                    name="user-prompt"
                    id="user- prompt"
                    rows="1"
                    class="block w-full border-2 border-black dark:border-white bg-white dark:bg-neutral-800 p-2 text-base text-neutral-900 dark:text-white focus:outline-none sm:text-sm/6"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Form Actions -->
        <div class="mt-6 flex items-center justify-end gap-x-3">
          <PrimaryButton
            type="secondary"
            size="sm"
            @click="handleCancel()"
          >
            Cancel
          </PrimaryButton>
          <PrimaryButton
            type="primary"
            size="sm"
            @click="handleSubmit()"
          >
            Update
          </PrimaryButton>
        </div>
      </form>

      <div v-if="mode === 'test' && props.prompt">
        <TestPrompt :prompt="props.prompt" />
      </div>

      <!-- View Mode -->
      <div v-if="mode === 'view'">
        <div class="px-4 sm:px-0">
          <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Prompt Details</h3>
          <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Configuration and content for this prompt.</p>
        </div>
        <div v-if="props.prompt" class="mt-6">
          <dl class="grid grid-cols-1 sm:grid-cols-3">
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Key</dt>
              <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ promptKey }}</dd>
            </div>
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Prompt Version</dt>
              <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2"><b>{{ props.prompt.version_number }}</b> - <i>{{ formatDate(props.prompt.updated_at) }}</i></dd>
            </div>
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
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Model</dt>
              <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ selectedModel?.name }} ({{ selectedModel?.provider_name }})</dd>
            </div>
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Max Tokens</dt>
              <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ maxTokens }}</dd>
            </div>
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">Temperature</dt>
              <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ temperatureValue.toFixed(2) }}</dd>
            </div>
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-1 sm:px-0">
              <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">JSON Mode</dt>
              <dd class="mt-1 text-sm/6 text-neutral-700 dark:text-neutral-300 sm:mt-2">{{ jsonMode ? 'Enabled' : 'Disabled' }}</dd>
            </div>
            <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-6 sm:col-span-3 sm:px-0">
              <div class="px-4 sm:px-0">
                <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</dt>
                <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ systemPrompt }}</dd>
              </div>
              <div class="px-4 sm:px-0 mt-2">
                <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt</dt>
                <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ userPrompt }}</dd>
              </div>
            </div>
          </dl>
        </div>
        <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-3">
          <PrimaryButton
            type="secondary"
            size="sm"
            @click="handleEdit()"
          >
            Edit
          </PrimaryButton>
          <PrimaryButton
            type="primary"
            size="sm"
            @click="handleTest()"
          >
            Test
          </PrimaryButton>
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
import { format, parseISO } from 'date-fns';

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
const systemPrompt = ref(props.prompt?.system || '');
const userPrompt = ref(props.prompt?.user || '');
const selectedModelId = ref<number | null>(props.prompt?.model_id || null);
const maxTokens = ref(props.prompt?.max_tokens || 256);
const temperatureValue = ref(props.prompt?.temperature || 0.7);
const jsonMode = ref(props.prompt?.json_mode || false);
const isLoading = ref(false);
const isOpen = ref(false);
const showVersionDiff = ref(false)

// Dependencies
const { createPrompt, updatePrompt } = usePrompts();
const { models, loading: modelsLoading, fetchModels } = useModels();

// Computed
const selectedModel = computed(() =>
  models.value?.find(m => m.id === selectedModelId.value) || null
);

const formIsValid = computed(() =>
  promptKey.value.trim() !== '' &&
  systemPrompt.value.trim() !== '' &&
  selectedModelId.value !== null &&
  maxTokens.value > 0 &&
  temperatureValue.value >= 0 &&
  temperatureValue.value <= 2
);

// Watchers
watch(() => props.prompt, (newPrompt) => {
  if (newPrompt) {
    promptKey.value = newPrompt.key;
    systemPrompt.value = newPrompt.system;
    userPrompt.value = newPrompt.user;
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
  systemPrompt.value = '';
  userPrompt.value = '';
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
      system: systemPrompt.value,
      user: userPrompt.value,
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

function formatDate(dateString: string): string {
  const date = parseISO(dateString);
  return format(date, 'MM-dd-yyyy');
}

</script>
