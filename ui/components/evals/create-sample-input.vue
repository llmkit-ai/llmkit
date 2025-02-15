<template>
  <div>
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-sm font-semibold text-neutral-700 dark:text-white">Sample input fields</h3>
        <p class="mt-0.5 max-w-2xl text-sm text-neutral-500 dark:text-neutral-400">The below fields are extracted based on handlebar syntax from your prompts. The values you populate them with will be your sample/test input for future evals.</p>
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
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-sm font-semibold text-neutral-700 dark:text-white">Sample input name</h3>
        <p class="mt-0.5 max-w-2xl text-sm text-neutral-500 dark:text-neutral-400">Give this sample input a name that aligns with the scenario that you're testing for this scenario.</p>
      </div>
      <div class="mt-4">
        <label for="sample-name" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Sample name</label>
        <div class="mt-0.5">
          <input 
            v-model="sampleInputName" 
            type="text" 
            name="sample-name" 
            id="sample-name-input" 
            class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
          >
        </div>
      </div>
    </div>
    <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-2">
      <PrimaryButton
        @click="$emit('cancel')"
        type="secondary"
        size="sm"
      >
        Cancel
      </PrimaryButton>
      <PrimaryButton
        @click="createSampleInput()"
        type="primary"
        size="sm"
        :disabled="sampleInputName === ''"
      >
        Add sample input
      </PrimaryButton>
    </div>
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-700 dark:text-white">Prompt contents</h3>
      </div>
      <dl class="mt-3 grid grid-cols-1 sm:grid-cols-2">
        <div class="px-4 pb-6 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ props.prompt?.system }}</dd>
        </div>
      </dl>
      <dl class="grid grid-cols-1 sm:grid-cols-2">
        <div class="dark:border-neutral-700 px-4 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ props.prompt?.user }}</dd>
        </div>
      </dl>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';

const props = defineProps<{
  prompt?: Prompt | null
}>();

const emits = defineEmits(["cancel", "created"])

const { createSample, deleteSample, samples } = usePromptSamples();
const jsonContext = ref({})
const sampleInputName = ref<string>("")

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

  props.prompt?.system.replace(`{{ name }}`, value)
}

async function createSampleInput() {
  await createSample({
    prompt_id: props.prompt!.id,
    input_data: JSON.stringify(jsonContext.value),
    name: sampleInputName.value
  })

  emits("created")
}
</script>
