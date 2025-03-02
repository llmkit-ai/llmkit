<template>
  <div v-if="!promptsLoading" class="font-mono">
    <!-- Sidebar -->
    <aside 
      class="bg-neutral-50 dark:bg-neutral-800 fixed inset-y-0 left-72 w-96 overflow-y-auto border-r border-neutral-200 dark:border-neutral-700 px-4 py-6 sm:px-6 lg:px-8 xl:block"
    >
      <div class="flex justify-between items-center">
        <h2 class="font-mono font-bold text-black dark:text-white">Prompts</h2>
      </div>
      <ul v-if="prompts && prompts.length > 0" role="list" class="mt-5 space-y-3 divide-y divide-neutral-100 dark:divide-neutral-700">
        <li 
          v-for="p in prompts" 
          :key="p.id"
          class="font-mono relative"
        >
          <div 
            v-if="selectedPrompt?.id === p.id" 
            class="absolute -left-3 inset-y-0 border-l-4 border-black dark:border-white"
          />
          <button @click="selectedPrompt = p, promptMode = 'view'" class="w-full text-left">
            <p class="text-sm/6 text-black dark:text-white">{{ p.key }}</p>
            <div class="flex items-center gap-x-2 text-xs/5 text-neutral-500 dark:text-neutral-400">
              <p>{{ p.model }}</p>
              <svg viewBox="0 0 2 2" class="size-0.5 fill-current">
                <circle cx="1" cy="1" r="1" />
              </svg>
              <p>{{ p.provider }}</p>
            </div>
          </button>
        </li>
      </ul>
    </aside>

    <!-- Main Content -->
    <div class="pl-96">
      <Evals 
        v-if="selectedPrompt"
        :prompt="selectedPrompt" 
        :mode="promptMode"
        :key="selectedPrompt.id"
      /> 
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';

definePageMeta({
  layout: "logged-in",
  middleware: ['auth']
})

const selectedPrompt = ref<Prompt | null>(null);
const selectedPromptCache = ref<Prompt | null>(null);
const promptMode = ref<'view' | 'edit' | 'new' | 'test'>('view');

// Provide emit handlers
provide('handleCancel', () => handleCancelClick());
provide('handleEdit', () => promptMode.value = 'edit');
provide('handleSaved', (prompt: Prompt) => handleSaved(prompt));
provide('handleTest', () => handleTest());

const { 
  prompts, 
  loading: promptsLoading,
  fetchPrompts 
} = usePrompts();

onMounted(async () => {
  await fetchPrompts();
  if (prompts.value?.length > 0) {
    selectedPrompt.value = prompts.value[0];
  } else {
    promptMode.value = 'new'
  }
});


function handleCancelClick() {
  if (selectedPromptCache.value) {
    selectedPrompt.value = selectedPromptCache.value
    selectedPromptCache.value = null
  }
  promptMode.value = 'view';
}

function handleNewClick() {
  selectedPromptCache.value = selectedPrompt.value
  selectedPrompt.value = null
  promptMode.value = 'new';
}

async function handleSaved(newPrompt: Prompt) {
  await fetchPrompts();
  selectedPrompt.value = newPrompt;
  promptMode.value = 'view';
}

async function handleTest() {
  promptMode.value = 'test';
}
</script>
