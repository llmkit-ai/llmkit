<template>
  <div>
    <form>
      <div class="space-y-12">
        <div class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3">
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">Eval name</h2>
            <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">Give your eval a name that is meaningful and recognizable.</p>
          </div>

          <div class="grid max-w-2xl grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 md:col-span-2">
            <div class="sm:col-span-full">
              <label for="sample-name" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">Eval name</label>
              <div class="mt-1">
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
        </div>

        <div class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3">
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">Prompt fields</h2>
            <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">These are the variables in your prompt that you need to provide input for.</p>
          </div>

          <div class="grid max-w-2xl grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 md:col-span-2">
            <div v-for="f in templateFields" class="col-span-3">
              <label :for="f" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">{{ f }}</label>
              <div class="mt-0.5">
                <input
                  v-model="jsonContext[f]"
                  type="text"
                  :name="f"
                  :id="f"
                  class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
                >
              </div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3">
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">Json input preview</h2>
            <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">The input that will be used for this eval based on prompt template variables.</p>
          </div>
          <div class="p-2 text-xs overflow-y-auto whitespace-pre h-32 grid max-w-2xl md:col-span-2 bg-neutral-100 dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100">
            {{ JSON.stringify(jsonContext, null, 2) }}
          </div>
        </div>
      </div>

      <div class="mt-6 flex items-center justify-end gap-x-6">
        <PrimaryButton
          @click="$emit('cancel')"
          type="secondary"
          size="sm"
        >
          Cancel
        </PrimaryButton>
        <PrimaryButton
          @click="handleEvalTestInput()"
          type="primary"
          size="sm"
          :disabled="sampleInputName === ''"
        >
          {{ sample ? 'Update' : 'Add' }} eval
        </PrimaryButton>
      </div>
    </form>
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-700 dark:text-white">Prompt contents</h3>
        <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">We will substitute the variables that you populate above into your prompt at evaluation time.</p>
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
import type { PromptEvalResponse } from '~/types/response/prompt_eval'
import { watch, ref, computed } from 'vue';

const props = defineProps<{
  prompt: Prompt,
  sample?: PromptEvalResponse | null
}>();

const emits = defineEmits(["cancel", "created", "updated"])

const { createEval, updateEval } = usePromptEvals();
const jsonContext = ref<Record<string, string>>({});
const sampleInputName = ref<string>("");

const templateFields = computed<string[]>(() => {
  if (!props.prompt?.system || !props.prompt?.user) return [];

  const template = `${props.prompt.system}\n${props.prompt.user}`;
  const uniqueFields = new Set<string>();

  // Regex patterns
  const handlebarsRegex = /\{\{\s*(\w+)\s*\}\}/g;
  const ifConditionRegex = /\{\%\s*if\s*(\w+)(?:\s+.*?)\s*\%\}/g;
  const elifConditionRegex = /\{\%\s*elif\s*(\w+)(?:\s+.*?)\s*\%\}/g;
  const forLoopRegex = /\{\%\s*for\s+\w+\s+in\s+(\w+)\s*\%\}/g;

  // Find all template variables
  const extractFields = (regex: RegExp) => {
    let match;
    while ((match = regex.exec(template)) !== null) {
      if (match[1]) uniqueFields.add(match[1]);
    }
  };

  extractFields(handlebarsRegex);
  extractFields(ifConditionRegex);
  extractFields(elifConditionRegex);
  extractFields(forLoopRegex);

  return Array.from(uniqueFields);
});


// Initialize JSON context with template field structure
const initializeJsonContext = () => {
  return Object.fromEntries(
    templateFields.value.map(field => [field, ''])
  );
};

// Watch for template fields changes to maintain JSON structure
watch(templateFields, (newFields) => {
  const currentContext = { ...jsonContext.value };
  // Preserve existing values while maintaining new field structure
  jsonContext.value = Object.fromEntries(
    newFields.map(field => [field, currentContext[field] || ''])
  );
});

// Watch for sample changes
watch(() => props.sample, (newEvalTest) => {
  if (newEvalTest) {
    sampleInputName.value = newEvalTest.name;
    try {
      // Handle potential double-encoded JSON
      let inputData = JSON.parse(newEvalTest.input_data);
      if (typeof inputData === 'string') {
        inputData = JSON.parse(inputData);
      }
      
      // Merge with initialized structure
      jsonContext.value = {
        ...initializeJsonContext(),
        ...inputData
      };
    } catch (e) {
      console.error('Error parsing input_data:', e);
      jsonContext.value = initializeJsonContext();
    }
  } else {
    sampleInputName.value = '';
    jsonContext.value = initializeJsonContext();
  }
}, { immediate: true, deep: true });

async function handleEvalTestInput() {
  if (!props.prompt?.id) return;

  // Clean the JSON context before saving
  const cleanContext = Object.fromEntries(
    Object.entries(jsonContext.value)
      .filter(([key]) => templateFields.value.includes(key))
  );

  const payload = {
    prompt_id: props.prompt.id,
    input_data: cleanContext,
    name: sampleInputName.value
  };

  try {
    if (props.sample) {
      await updateEval(props.sample.id, payload);
      emits("updated");
    } else {
      await createEval(payload);
      emits("created");
    }
  } catch (error) {
    console.error('Error saving sample:', error);
  }
}
</script>
