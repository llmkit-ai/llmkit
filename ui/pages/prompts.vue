<template>
  <div v-if="!promptsLoading" class="font-mono">
    <!-- Sidebar -->
    <aside 
      v-if="isCurrentRoute"
      class="bg-neutral-50 dark:bg-neutral-800 fixed inset-y-0 left-72 w-96 overflow-y-auto border-r border-neutral-200 dark:border-neutral-700 px-4 py-6 sm:px-6 lg:px-8 xl:block z-10"
    >
      <div class="flex justify-between items-center">
        <h2 class="font-mono font-bold text-black dark:text-white">Prompts</h2>
        <PrimaryButton 
          @click="mode = 'new'"
          size="xs"
          buttonType="default"
        >
          New Prompt +
        </PrimaryButton>
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
          <button @click="selectedPrompt = p, mode = 'view'" class="w-full text-left">
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
              v-if="mode === 'edit'" 
              :prompt="selectedPrompt"
              :models="models"
              :mode="mode"
              @handle-cancel="mode = 'view'"
              @handle-create="handleCreate"
              @handle-update="handleUpdate"
            />
            <PromptsAddEdit 
              v-if="mode === 'new'" 
              :prompt="null"
              :models="models"
              :mode="mode"
              @handle-cancel="mode = 'view'"
              @handle-create="handleCreate"
              @handle-update="handleUpdate"
            />
          </div>

          <!-- Test Mode -->
          <div v-if="mode === 'test' && selectedPrompt">
            <!-- Use Chat Test for chat-enabled prompts -->
            <PromptsChatTest 
              v-if="selectedPrompt.is_chat"
              :prompt="selectedPrompt"
              @handle-cancel="mode = 'view'"
              @handle-edit="mode = 'edit'"
            />
            <!-- Use regular Test for standard prompts -->
            <PromptsTest 
              v-else
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
              @toggle-tools-modal="showToolsModal = !showToolsModal"
              @prompt-updated="handlePromptVersionChange"
            />
            
            <!-- Tools Management Modal -->
            <div v-if="showToolsModal">
              <ToolSelectorModal
                :prompt-version-id="selectedPrompt.version_id"
                :associated-tools="selectedPrompt.tools || []"
                @add-tool="handleToolAdd"
                @remove-tool="handleToolRemove"
                @close="showToolsModal = false"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts';
import type { PromptCreateDTO, PromptUpdateDTO } from '~/types/components/prompt';
import type { Tool } from '~/types/response/tools';
import PromptsChatTest from '~/components/prompts/chat-test.vue';
import ToolSelectorModal from '~/components/prompts/tool-selector-modal.vue';

definePageMeta({
  layout: "logged-in",
  middleware: ['auth']
})

const route = useRoute();
const isCurrentRoute = computed(() => route.name === 'prompts');

const mode = ref<'view' | 'edit' | 'new' | 'test'>('view');


const selectedPrompt = ref<Prompt | null>(null);
const showToolsModal = ref(false);

const { 
  prompts, 
  createPrompt,
  updatePrompt,
  loading: promptsLoading,
  fetchPrompts 
} = usePrompts();

const { associateToolWithPromptVersion, removeToolPromptAssociation } = useTools();
const { models, fetchModels } = useModels();

onBeforeMount(async () => {
  await fetchModels()
  await fetchPrompts();
  if (prompts.value?.length > 0) {
    selectedPrompt.value = prompts.value[0];
  }
})


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
    const updatedPrompt = await updatePrompt(selectedPrompt.value.id, payload)
    // Directly update the selectedPrompt with the returned data
    selectedPrompt.value = updatedPrompt
    mode.value = "view"
  } catch(e) {
    console.error(e)
  }
}

async function handleToolAdd(newTool: Tool) {
  if (!selectedPrompt.value) {
    throw createError({ statusCode: 500, statusMessage: "Missing prompt" })
  }
  
  if (selectedPrompt.value && selectedPrompt.value.version_id) {
    await associateToolWithPromptVersion({
      tool_id: newTool.id,
      prompt_version_id: selectedPrompt.value.version_id
    })

    selectedPrompt.value.tools.push(newTool)
  }
}

async function handleToolRemove(removedTool: Tool) {
  if (!selectedPrompt.value) {
    throw createError({ statusCode: 500, statusMessage: "Missing prompt" })
  }
  
  if (selectedPrompt.value && selectedPrompt.value.version_id) {
    // Process removals first
    await removeToolPromptAssociation({
      tool_id: removedTool.id, 
      prompt_version_id: selectedPrompt.value.version_id
    })

    const index = selectedPrompt.value.tools.findIndex(t => t.id = removedTool.id)
    selectedPrompt.value.tools.splice(index, 1)
  }
}

function handlePromptVersionChange(updatedPrompt: Prompt) {
  // Update the selected prompt with the new version
  selectedPrompt.value = updatedPrompt

  // Also update the prompt in the prompts list
  const index = prompts.value.findIndex(p => p.id === updatedPrompt.id)
  if (index !== -1) {
    prompts.value[index] = updatedPrompt
  }
}
</script>