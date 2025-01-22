<template>
  <div class="font-mono">
    <form v-if="!modelsLoading">
      <div class="space-y-12">
        <div>
          <h2 class="text-base/7 font-semibold text-gray-900">New prompt</h2>

          <div class="mt-10 grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6">
            <div class="sm:col-span-4">
              <label for="prompt-key" class="block text-sm/6 font-medium text-gray-900">Prompt key</label>
              <div class="mt-2">
                <div class="flex items-center rounded-md bg-white pl-3 outline outline-1 -outline-offset-1 outline-gray-300 focus-within:outline focus-within:outline-2 focus-within:-outline-offset-2 focus-within:outline-black-600">
                  <input v-model="promptKey" type="text" name="prompt-key" id="prompt-key" class="block min-w-0 grow py-1.5 pl-1 pr-3 text-base text-gray-900 placeholder:text-gray-400 focus:outline focus:outline-0 sm:text-sm/6" placeholder="PROMPT-KEY-HERE">
                </div>
              </div>
            </div>

            <div class="sm:col-span-4">
              <label for="model" class="block text-sm/6 font-medium text-gray-900">Select model</label>
              <div class="relative mt-2">
                <button
                  type="button"
                  class="grid w-full cursor-default grid-cols-1 rounded-md bg-white py-1.5 pl-3 pr-2 text-left text-gray-900 outline outline-1 -outline-offset-1 outline-gray-300 focus:outline focus:outline-2 focus:-outline-offset-2 sm:text-sm/6"
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
                  class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
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
                      class="absolute inset-y-0 right-0 flex items-center pr-4 text-black"
                    >
                      <svg class="size-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" data-slot="icon">
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

            <div class="col-span-full">
              <label for="about" class="block text-sm/6 font-medium text-gray-900">Prompt</label>
              <div class="mt-2">
                <textarea v-model="prompt" name="about" id="about" rows="15" class="block w-full rounded-md bg-white px-3 py-1.5 text-base text-gray-900 outline outline-1 -outline-offset-1 outline-gray-300 placeholder:text-gray-400 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black-600 sm:text-sm/6"></textarea>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="mt-6 flex items-center justify-end gap-x-6">
        <button type="button" class="text-sm/6 text-gray-900">Cancel</button>
        <button @click="handlePromptCreation()" v-if="formIsValid && !promptCreateLoading" type="submit" class="text-sm/6 p-2 border-2 border-black">Save</button>
      </div>
    </form>

  </div>
</template>

<script setup lang="ts">
import type { Model } from '~/types/response/models';


const promptKey = ref("")
const prompt = ref("")
const selectedModel = ref<Model | null>(null);
const isOpen = ref(false);

const { 
  createPrompt, 
} = usePrompts()

const { 
  models, 
  loading: modelsLoading, 
  fetchModels, 
} = useModels()

onMounted(async () => {
  await fetchModels()
})

const formIsValid = computed(() => {
  if (promptKey.value !== "" && prompt.value !== "" && selectedModel.value) {
    return true
  }

  return false
})

const toggleDropdown = () => {
  isOpen.value = !isOpen.value;
};

const selectModel = (model: Model) => {
  selectedModel.value = model;
  isOpen.value = false;
};


const promptCreateLoading = ref(false)
async function handlePromptCreation() {
  promptCreateLoading.value = true
  await createPrompt({ key: promptKey.value, prompt: prompt.value, model_id: selectedModel.value!.id })
  navigateTo("/prompts")
}
</script>
