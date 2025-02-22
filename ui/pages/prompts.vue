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
          <button @click="selectedPrompt = p" class="w-full text-left">
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
      <div class="font-mono">
        <div>
          <!-- Add or Edit Mode -->
          <div v-if="(mode === 'edit' || mode === 'new') && selectedPrompt">
            <PromptsAddEdit 
              v-if="mode === 'edit' || mode === 'new'" 
              :prompt="selectedPrompt"
              :models="models"
              :mode="mode"
              @handle-cancel="mode = 'view'"
              @handle-create="handleCreate"
              @handle-update="handleUpdate"
            />
          </div>

          <!-- Test Mode -->
          <div v-if="mode === 'test' && selectedPrompt">
            <PromptsTest 
              :prompt="selectedPrompt"
              @handle-cancel="mode = 'view'"
              @handle-edit="mode = 'edit'"
            />
          </div>

          <!-- View Mode -->
          <div v-if="mode === 'view' && selectedPrompt">
            <PromptsView 
              :prompt="selectedPrompt" 
              @handle-edit="mode = 'edit'"
              @handle-test="mode = 'test'"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';
import type { PromptCreateDTO, PromptUpdateDTO } from '~/types/components/prompt';

const mode = ref<'view' | 'edit' | 'new' | 'test'>('view');

const selectedPrompt = ref<Prompt | null>(null);
const selectedPromptCache = ref<Prompt | null>(null);

const { 
  prompts, 
  createPrompt,
  updatePrompt,
  loading: promptsLoading,
  fetchPrompts 
} = usePrompts();


const { models, fetchModels } = useModels();

onBeforeMount(async () => {
  await fetchModels()
  await fetchPrompts();
  if (prompts.value?.length > 0) {
    selectedPrompt.value = prompts.value[0];
  }
})

// function handleCancelClick() {
//   if (selectedPromptCache.value) {
//     selectedPrompt.value = selectedPromptCache.value
//     selectedPromptCache.value = null
//   }
//   promptMode.value = 'view';
// }
//
// function handleNewClick() {
//   selectedPromptCache.value = selectedPrompt.value
//   selectedPrompt.value = null
//   promptMode.value = 'new';
// }

// async function handleSaved(newPrompt: Prompt) {
//   await fetchPrompts();
//   selectedPrompt.value = newPrompt;
// }


async function handleCreate(payload: PromptCreateDTO) {
  try {
    await createPrompt(payload)
    mode.value = "view"
  } catch(e) {
    console.error(e)
  }
}

async function handleUpdate(payload: PromptUpdateDTO) {
  if (!selectedPrompt.value) {
    throw createError({ statusCode: 500, statusMessage: "Missing prompt" })
  }
  try {
    await updatePrompt(selectedPrompt.value.id, payload)
    mode.value = "view"
  } catch(e) {
    console.error(e)
  }
}
</script>
