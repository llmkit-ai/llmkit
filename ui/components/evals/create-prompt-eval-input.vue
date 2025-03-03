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
                  v-model="evalName"
                  type="text"
                  name="sample-name"
                  id="sample-name-input"
                  class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
                >
              </div>
            </div>
          </div>
        </div>

        <!-- Dynamic fields section for templates with variables -->
        <div v-if="systemTemplateFields.length > 0" class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3">
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">System Variables</h2>
            <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">These are the variables detected in your system prompt template.</p>
          </div>

          <div class="grid max-w-2xl grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 md:col-span-2">
            <div v-for="f in systemTemplateFields" class="col-span-3">
              <label :for="f" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">{{ f }}</label>
              <div class="mt-0.5">
                <input
                  v-model="systemJsonContext[f]"
                  type="text"
                  :name="f"
                  :id="f"
                  class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
                >
              </div>
            </div>
          </div>
        </div>

        <div v-if="userTemplateFields.length > 0" class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3">
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">User Variables</h2>
            <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">These are the variables detected in your user prompt template.</p>
          </div>

          <div class="grid max-w-2xl grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 md:col-span-2">
            <div v-for="f in userTemplateFields" class="col-span-3">
              <label :for="f" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">{{ f }}</label>
              <div class="mt-0.5">
                <input
                  v-model="userJsonContext[f]"
                  type="text"
                  :name="f"
                  :id="f"
                  class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
                >
              </div>
            </div>
          </div>
        </div>

        <!-- Direct user prompt input for static prompts -->
        <div 
          v-if="props.prompt?.prompt_type === 'static' || props.prompt?.prompt_type === 'dynamic_system'" 
          class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3"
        >
          <div>
            <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">User Input</h2>
            <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">For static prompts, you can provide direct user input.</p>
          </div>

          <div class="grid max-w-2xl grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 md:col-span-2">
            <div class="col-span-6">
              <label for="direct-user-input" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">User Message</label>
              <div class="mt-0.5">
                <textarea
                  v-model="directUserInput"
                  id="direct-user-input"
                  rows="4"
                  class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
                ></textarea>
              </div>
            </div>
          </div>
        </div>

        <!-- Viewing input context -->
        <!-- <div v-if="Object.keys(systemJsonContext).length > 0" class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3"> -->
        <!--   <div> -->
        <!--     <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">System input Json</h2> -->
        <!--     <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">The input that will be used for this eval based on prompt template variables.</p> -->
        <!--   </div> -->
        <!--   <div class="p-2 text-xs overflow-y-auto whitespace-pre h-32 grid max-w-2xl md:col-span-2 bg-neutral-100 dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100"> -->
        <!--     {{ JSON.stringify(systemJsonContext, null, 2) }} -->
        <!--   </div> -->
        <!-- </div> -->
        <!-- <div v-if="Object.keys(userJsonContext).length > 0" class="grid grid-cols-1 gap-x-8 gap-y-10 border-b border-gray-900/10 dark:border-gray-100/10 pb-12 md:grid-cols-3"> -->
        <!--   <div> -->
        <!--     <h2 class="text-base/7 font-semibold text-gray-900 dark:text-gray-100">User input Json</h2> -->
        <!--     <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">The input that will be used for this eval based on prompt template variables.</p> -->
        <!--   </div> -->
        <!--   <div class="p-2 text-xs overflow-y-auto whitespace-pre h-32 grid max-w-2xl md:col-span-2 bg-neutral-100 dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100"> -->
        <!--     {{ JSON.stringify(userJsonContext, null, 2) }} -->
        <!--   </div> -->
        <!-- </div> -->
      </div>

      <div class="mt-6 flex items-center justify-end gap-x-6">
        <PrimaryButton
          @click="$emit('cancel')"
          buttonType="secondary"
          size="sm"
        >
          Cancel
        </PrimaryButton>
        <PrimaryButton
          @click="handleEvalTestInput()"
          buttonType="primary"
          size="sm"
          :disabled="evalName === ''"
        >
          {{ eval ? 'Update' : 'Add' }} eval
        </PrimaryButton>
      </div>
    </form>
    <div class="mt-6">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-700 dark:text-white">Prompt Templates</h3>
        <p class="mt-1 text-sm/6 text-gray-600 dark:text-gray-400">These are the templates that your evaluation data will be substituted into.</p>
      </div>
      <dl class="mt-3 grid grid-cols-1 sm:grid-cols-2">
        <div class="px-4 pb-6 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt Template</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ props.prompt?.system }}</dd>
        </div>
      </dl>
      <dl v-if="props.prompt?.user" class="grid grid-cols-1 sm:grid-cols-2">
        <div class="dark:border-neutral-700 px-4 sm:col-span-2 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">User Prompt Template</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ props.prompt?.user }}</dd>
        </div>
      </dl>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';
import type { PromptEvalResponse } from '~/types/response/prompt_eval'
import type { CreatePromptEvalRequest, UpdatePromptEvalRequest } from '~/types/request/prompt_eval';

const props = defineProps<{
  prompt: Prompt,
  eval?: PromptEvalResponse | null
}>();

const emits = defineEmits(["cancel", "created", "updated"])

const { createEval, updateEval } = usePromptEvals();
const systemJsonContext = ref<Record<string, string>>(props.eval?.system_prompt_input ? JSON.parse(props.eval.system_prompt_input) : {});
const userJsonContext = ref<Record<string, string>>({});
const evalName = ref<string>(props.eval?.name || "");
const directUserInput = ref<string>("");

// Populate fields based on the prompt type and existing data
onMounted(() => {
  if (props.eval) {
    // Handle system prompt input (always JSON for dynamic prompts)
    if (props.eval.system_prompt_input && ['dynamic_both', 'dynamic_system'].includes(props.prompt?.prompt_type || '')) {
      try {
        systemJsonContext.value = JSON.parse(props.eval.system_prompt_input);
      } catch (e) {
        console.error('Error parsing system_prompt_input JSON:', e);
      }
    }
    
    // Handle user prompt input based on prompt type
    if (props.eval.user_prompt_input) {
      if (props.prompt?.prompt_type === 'static' || props.prompt?.prompt_type === 'dynamic_system') {
        // For static or dynamic_system, user input could be a direct string
        console.log(props.eval.user_prompt_input)
        directUserInput.value = props.eval.user_prompt_input;
      } else if (props.prompt?.prompt_type === 'dynamic_both') {
        // For dynamic_both, user input should be JSON
        try {
          userJsonContext.value = JSON.parse(props.eval.user_prompt_input);
        } catch (e) {
          console.error('Error parsing user_prompt_input JSON:', e);
        }
      }
    }
  }
});

const systemTemplateFields = computed<string[]>(() => {
  if (!props.prompt?.system) return [];
  let template = props.prompt.system;
  return extractTemplateFields(template)
});

const userTemplateFields = computed<string[]>(() => {
  if (!props.prompt?.user) return [];
  let template = props.prompt.user;
  return extractTemplateFields(template)
});

function extractTemplateFields(template: string) {
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

}

async function handleEvalTestInput() {
  if (!props.prompt?.id) return;

  // Clean the JSON context before saving
  const cleanSystemContext = Object.fromEntries(
    Object.entries(systemJsonContext.value)
      .filter(([key]) => systemTemplateFields.value.includes(key))
  );

  const cleanUserContext = Object.fromEntries(
    Object.entries(userJsonContext.value)
      .filter(([key]) => userTemplateFields.value.includes(key))
  );

  // Create payload with separate system and user prompt inputs based on prompt type
  let system_prompt_input: object | undefined = undefined;
  let user_prompt_input: object | string;
  
  // Handle different prompt types
  if (props.prompt.prompt_type === 'static') {
    // For static prompts, there's no system_prompt_input needed
    // And user_prompt_input can be direct text rather than JSON
    user_prompt_input = directUserInput.value || "";
  } 
  else if (props.prompt.prompt_type === 'dynamic_both') {
    // For dynamic_both, we need JSON context for both system and user
    system_prompt_input = cleanSystemContext;
    user_prompt_input = JSON.stringify(cleanUserContext);
  } 
  else if (props.prompt.prompt_type === 'dynamic_system') {
    // For dynamic_system, we need JSON for system but can have direct input for user
    system_prompt_input = cleanSystemContext;
    user_prompt_input = directUserInput.value;
  } else {
    // Default case (shouldn't happen)
    user_prompt_input = JSON.stringify(cleanUserContext);
  }

  if (props.eval) {
    const updatePayload: UpdatePromptEvalRequest = {
      system_prompt_input,
      user_prompt_input,
      name: evalName.value
    };

    try {
      await updateEval(props.eval.id, updatePayload);
      emits("updated");
    } catch (error) {
      console.error('Error updating sample:', error);
    }
  } else {
    const createPayload: CreatePromptEvalRequest = {
      prompt_id: props.prompt.id,
      system_prompt_input,
      user_prompt_input,
      name: evalName.value
    };

    try {
      await createEval(createPayload);
      emits("created");
    } catch (error) {
      console.error('Error creating sample:', error);
    }
  }
}
</script>
