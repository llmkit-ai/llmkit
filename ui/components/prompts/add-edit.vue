<template>
  <form>
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
        @click="$emit('handle-cancel')"
      >
        Cancel
      </PrimaryButton>
      <PrimaryButton
        type="primary"
        size="sm"
        @click="handleSubmit()"
      >
        {{ mode === 'edit' ? 'Update':'Create'  }}
      </PrimaryButton>
    </div>
  </form>

</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';
import type { Model } from '~/types/response/models';
import type { PromptCreateDTO, PromptUpdateDTO } from '~/types/components/prompt';

const props = defineProps<{
  prompt: Prompt | null
  models: Model[]
  mode: "edit" | "new"
}>();

const emit = defineEmits<{
  "handle-cancel": [];
  "handle-create": [prompt: PromptCreateDTO];
  "handle-update": [prompt: PromptUpdateDTO];
}>();

const promptKey = ref(props.prompt?.key || '');
const systemPrompt = ref(props.prompt?.system || '');
const userPrompt = ref(props.prompt?.user || '');
const selectedModelId = ref<number | null>(props.prompt?.model_id || null);
const maxTokens = ref(props.prompt?.max_tokens || 256);
const temperatureValue = ref(props.prompt?.temperature || 0.7);
const jsonMode = ref(props.prompt?.json_mode || false);
const isOpen = ref(false);

// Computed
const selectedModel = computed(() =>
  props.models.find(m => m.id === selectedModelId.value) || null
);

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

const handleSubmit = () => {
  if (props.mode === 'new') {
    emit("handle-create", {
      key: promptKey.value,
      system: systemPrompt.value,
      user: userPrompt.value,
      model_id: selectedModelId.value,
      max_tokens: maxTokens.value,
      temperature: temperatureValue.value,
      json_mode: jsonMode.value,
    });
  } else {
    emit("handle-update", {
      id: props.prompt!.id,
      key: promptKey.value,
      system: systemPrompt.value,
      user: userPrompt.value,
      model_id: selectedModelId.value,
      max_tokens: maxTokens.value,
      temperature: temperatureValue.value,
      json_mode: jsonMode.value,
    });
  }
};
</script>
