<template>
  <div v-if="!promptsLoading" class="font-mono">
    <!-- Sidebar -->
    <aside 
      class="bg-gray-50 fixed inset-y-0 left-72 hidden w-96 overflow-y-auto border-r border-gray-200 px-4 py-6 sm:px-6 lg:px-8 xl:block"
    >
      <div class="flex justify-between items-center">
        <h2 class="font-mono font-bold">Prompts</h2>
        <button @click="handleNewClick" class="text-sm">+ New prompt</button>
      </div>

      <ul v-if="prompts && prompts.length > 0" role="list" class="mt-5 space-y-3 divide-y divide-gray-100">
        <li 
          v-for="p in prompts" 
          :key="p.id"
          class="font-mono relative"
        >
          <div 
            v-if="selectedPrompt?.id === p.id" 
            class="absolute -left-3 inset-y-0 border-l-4 border-black"
          />
          <button @click="selectedPrompt = p" class="w-full text-left">
            <p class="text-sm/6 text-black">{{ p.key }}</p>
            <div class="flex items-center gap-x-2 text-xs/5 text-gray-500">
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

    <!-- Empty State -->
    <!-- <div v-if="prompts && prompts.length === 0" class="h-[60vh] flex items-center justify-center"> -->
    <!--   <div> -->
    <!--     <p class="text-center text-gray-600">No prompts available</p> -->
    <!--     <div class="mt-10 flex justify-center"> -->
    <!--       <button  -->
    <!--         @click="handleNewClick"  -->
    <!--         class="p-2 border-2 border-black hover:bg-gray-50" -->
    <!--       > -->
    <!--         Create new prompt -->
    <!--       </button> -->
    <!--     </div> -->
    <!--   </div> -->
    <!-- </div> -->

    <!-- Main Content -->
    <div class="pl-96">
      <ViewAddEditPrompt 
        :prompt="selectedPrompt" 
        :mode="promptMode" 
        @edit="promptMode = 'edit'"
        @cancel="handleCancelClick"
        @saved="handleSaved"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';

const selectedPrompt = ref<Prompt | null>(null);
const selectedPromptCache = ref<Prompt | null>(null);
const promptMode = ref<'view' | 'edit' | 'new'>('view');

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
</script>
