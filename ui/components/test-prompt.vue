<template>
  <div>
    <div class="px-4 sm:px-0">
      <h3 class="text-base/7 font-semibold text-gray-900">Test Prompt</h3>
      <p class="mt-1 max-w-2xl text-sm/6 text-gray-500">Execute, test, and evaluate prompt.</p>
    </div>

    <div class="mt-6">
      <dl class="grid grid-cols-1 sm:grid-cols-2">
        <div class="border-t border-gray-100 px-4 py-6 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-gray-900">Prompt preview</dt>
          <dd class="mt-1 text-sm/6 text-gray-700 sm:mt-2 whitespace-pre-wrap">{{ promptPreview }}</dd>
        </div>
      </dl>
    </div>

    <div class="grid grid-cols-4 gap-x-2">
      <div v-for="f in templateFields">
        <label :for="f" class="block text-sm/6 font-medium text-gray-900">{{ f }}</label>
        <div class="mt-0.5">
          <input v-on:input="templateFieldInput" type="text" :name="f" :id="f" class="block w-full bg-white px-3 py-1.5 text-base text-gray-900 outline outline-1 -outline-offset-1 outline-gray-300 placeholder:text-gray-400 focus:outline focus:outline-2 focus:-outline-offset-2 sm:text-sm/6">
        </div>
      </div>
    </div>

    <div class="mt-5 bg-gray-100 p-4">
      <p class="text-xs">Json context</p>
      <div class="mt-4">
        {{ jsonContext }}
      </div>
    </div>

    <div v-if="testResponse" class="mt-5 bg-gray-100 p-4">
      <p class="text-xs">Response</p>
      <div class="mt-4">
        {{ testResponse }}
      </div>
    </div>

    <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-2">
      <button
        type="button"
        @click="handleCancel"
        class="text-sm/6 p-2"
      >
        Cancel
      </button>
      <button
        type="button"
        @click="handleEdit"
        class="text-sm/6 p-2 border-2 border-black"
      >
        Edit
      </button>
      <button
        type="button"
        @click="execute()"
        class="text-sm/6 p-2 border-2 border-black bg-black text-white"
      >
        Execute
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';

const props = defineProps<{
  prompt: Prompt
}>();

const handleCancel = inject('handleCancel', () => {});
const handleEdit = inject('handleEdit', () => {});

const { 
  executePrompt
} = usePrompts();

const promptPreview = ref(props.prompt.prompt)
const jsonContext = ref({})
const testResponse = ref<string | null>(null)

const templateFields = computed<string[]>(() => {
  if (!props.prompt) return [];
  
  const HANDLEBARS_REGEX = /\{\{\s*(\w+)\s*\}\}/g;
  const matches = Array.from(props.prompt.prompt.matchAll(HANDLEBARS_REGEX));
  const uniqueFields = new Set<string>();
  
  for (const match of matches) {
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

  promptPreview.value.replace(`{{ name }}`, value)
}

async function execute() {
  const res = await executePrompt(props.prompt.id, jsonContext.value)
  testResponse.value = res
}
</script>
