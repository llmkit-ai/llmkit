<template>
  <div>
    <!-- Tool Selector Button -->
    <div class="mb-4">
      <div class="flex justify-between items-center">
        <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">
          Associated Tools ({{ selectedTools.length }})
        </h3>
        <PrimaryButton
          buttonType="primary"
          size="sm"
          @click="openModal"
        >
          Manage Tools
        </PrimaryButton>
      </div>

      <!-- Selected Tools Preview -->
      <div v-if="selectedTools.length === 0" class="mt-2 text-sm text-neutral-500 dark:text-neutral-400 italic">
        No tools selected
      </div>
      <div v-else class="mt-2 flex flex-wrap gap-2">
        <div
          v-for="tool in selectedTools"
          :key="tool.id"
          class="inline-flex items-center px-2.5 py-1 rounded-md text-sm bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700"
        >
          <span class="mr-1 font-medium">{{ tool.tool_name }}</span>
          <span class="text-xs text-neutral-500">(v{{ tool.version_number }})</span>
        </div>
      </div>
    </div>

    <!-- Tool Selector Modal -->
    <div
      v-if="isModalOpen"
      class="fixed inset-0 z-50 overflow-y-auto"
      aria-labelledby="modal-title"
      role="dialog"
      aria-modal="true"
    >
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <!-- Background overlay -->
        <div
          class="fixed inset-0 bg-neutral-500 bg-opacity-75 transition-opacity"
          aria-hidden="true"
          @click="closeModal"
        ></div>

        <!-- Modal panel -->
        <div class="inline-block align-bottom bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-700 rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-4xl sm:w-full">
          <div class="p-6">
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-lg font-medium text-neutral-900 dark:text-white">
                Manage Tools
              </h3>
              <button
                @click="closeModal"
                class="text-neutral-400 hover:text-neutral-500 dark:text-neutral-300 dark:hover:text-neutral-200"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Search and Pagination Controls -->
            <div class="mb-4 flex flex-col sm:flex-row sm:justify-between sm:items-center gap-2">
              <div class="relative w-full sm:w-64">
                <input
                  v-model="searchTerm"
                  type="text"
                  placeholder="Search tools..."
                  class="w-full p-2 border-2 border-black dark:border-white bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white"
                  @input="debouncedSearch"
                />
              </div>
              
              <div class="flex items-center space-x-2">
                <span class="text-sm text-neutral-600 dark:text-neutral-400">
                  {{ paginationText }}
                </span>
                <PrimaryButton
                  buttonType="secondary"
                  size="xs"
                  :disabled="currentPage === 1"
                  @click="prevPage"
                >
                  Previous
                </PrimaryButton>
                <PrimaryButton
                  buttonType="secondary"
                  size="xs"
                  :disabled="currentPage === totalPages || totalPages === 0"
                  @click="nextPage"
                >
                  Next
                </PrimaryButton>
              </div>
            </div>

            <!-- Tools Table -->
            <div class="border border-neutral-200 dark:border-neutral-700 rounded-md overflow-hidden mb-4">
              <div v-if="loading" class="animate-pulse p-4 space-y-3">
                <div v-for="i in 5" :key="i" class="h-10 bg-neutral-200 dark:bg-neutral-700 rounded"></div>
              </div>
              
              <div v-else-if="filteredTools.length === 0" class="p-6 text-center text-neutral-500 dark:text-neutral-400">
                <p v-if="searchTerm">No tools matching "{{ searchTerm }}"</p>
                <p v-else>No tools available</p>
              </div>
              
              <table v-else class="min-w-full divide-y divide-neutral-200 dark:divide-neutral-700">
                <thead class="bg-neutral-100 dark:bg-neutral-800">
                  <tr>
                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                      Select
                    </th>
                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                      Display Name
                    </th>
                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                      Function Name
                    </th>
                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                      Version
                    </th>
                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                      Description
                    </th>
                  </tr>
                </thead>
                <tbody class="bg-white dark:bg-neutral-900 divide-y divide-neutral-200 dark:divide-neutral-700">
                  <tr v-for="tool in paginatedTools" :key="tool.id" class="hover:bg-neutral-50 dark:hover:bg-neutral-800">
                    <td class="px-6 py-4 whitespace-nowrap">
                      <input
                        type="checkbox"
                        :checked="isToolSelected(tool)"
                        @change="toggleTool(tool)"
                        class="h-4 w-4 text-black focus:ring-black border-neutral-300 dark:border-neutral-700 dark:bg-neutral-800 rounded"
                      />
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-neutral-900 dark:text-white">
                      {{ tool.name || tool.tool_name }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-neutral-500 dark:text-neutral-400">
                      {{ tool.tool_name }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-neutral-500 dark:text-neutral-400">
                      v{{ tool.version_number }}
                    </td>
                    <td class="px-6 py-4 text-sm text-neutral-500 dark:text-neutral-400 max-w-xs truncate">
                      {{ tool.description }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>

            <!-- Button controls -->
            <div class="flex justify-end">
              <PrimaryButton
                buttonType="primary"
                size="sm"
                @click="closeModal"
              >
                Done
              </PrimaryButton>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';
import { useTools } from '~/composables/useTools';
import type { Tool, ToolVersion } from '~/types/response/tools';

const props = defineProps<{
  promptId?: number;
  promptVersionId?: number;
  initialTools: ToolVersion[];
}>();

const emit = defineEmits<{
  'update:tools': [tools: ToolVersion[]];
}>();

// UI State
const isModalOpen = ref(false);
const searchTerm = ref('');
const currentPage = ref(1);
const perPage = 10;

// Selected tools are managed by this component
const selectedTools = ref<ToolVersion[]>([...props.initialTools]); // Clone to avoid reference issues

// Get tools state and methods from the composable
const { tools, loading, error, fetchTools, associateToolVersionWithPromptVersion, removeToolPromptAssociation } = useTools();

// Computed properties
const filteredTools = computed(() => {
  if (!searchTerm.value) {
    return tools.value;
  }
  
  const term = searchTerm.value.toLowerCase();
  return tools.value.filter(tool => 
    tool.name.toLowerCase().includes(term) || 
    tool.tool_name.toLowerCase().includes(term) ||
    tool.description.toLowerCase().includes(term)
  );
});

const totalPages = computed(() => {
  return Math.ceil(filteredTools.value.length / perPage);
});

const paginatedTools = computed(() => {
  const startIndex = (currentPage.value - 1) * perPage;
  return filteredTools.value.slice(startIndex, startIndex + perPage);
});

const paginationText = computed(() => {
  if (filteredTools.value.length === 0) {
    return 'No results';
  }
  
  const start = (currentPage.value - 1) * perPage + 1;
  const end = Math.min(start + perPage - 1, filteredTools.value.length);
  return `${start}-${end} of ${filteredTools.value.length}`;
});

// Initialize
onMounted(async () => {
  try {
    await fetchTools();
  } catch (error) {
    console.error('Error loading tools on mount:', error);
  }
});

// Watch for changes in initial tools to keep local state in sync
watch(() => props.initialTools, (newTools) => {
  selectedTools.value = [...newTools];
}, { deep: true });

// Watch for changes in selected tools to emit events
watch(selectedTools, (newTools) => {
  emit('update:tools', newTools);
}, { deep: true });

const debouncedSearch = useDebounceFn(() => {
  currentPage.value = 1; // Reset to first page on new search
}, 300);

function isToolSelected(tool: Tool): boolean {
  // Check if a tool version from this tool is already selected
  // First try to match by tool_id
  if (selectedTools.value.some(t => t.tool_id === tool.id)) {
    return true;
  }
  
  // As a fallback, try to match by current_tool_version_id if it exists
  if (tool.current_tool_version_id) {
    return selectedTools.value.some(t => t.id === tool.current_tool_version_id);
  }
  
  return false;
}

async function toggleTool(tool: Tool) {
  // Convert Tool to ToolVersion for consistent handling
  const toolVersion: ToolVersion = {
    id: tool.current_tool_version_id || tool.id,
    tool_id: tool.id,
    version_number: tool.version_number,
    tool_name: tool.tool_name,
    description: tool.description,
    parameters: tool.parameters,
    strict: tool.strict,
    created_at: tool.created_at
  };

  const isSelected = isToolSelected(tool);
  
  if (isSelected) {
    // Remove the tool
    const index = selectedTools.value.findIndex(t => t.tool_id === tool.id);
    if (index !== -1) {
      const removedTool = selectedTools.value[index];
      selectedTools.value.splice(index, 1);
      
      // If editing an existing prompt, also remove the association in the backend
      if (props.promptVersionId) {
        try {
          await removeToolPromptAssociation({
            tool_version_id: removedTool.id,
            prompt_version_id: props.promptVersionId
          });
        } catch (error) {
          console.error('Error removing tool association:', error);
          // On error, revert the removal
          selectedTools.value.push(removedTool);
        }
      }
    }
  } else {
    // Add the tool
    selectedTools.value.push(toolVersion);
    
    // If editing an existing prompt, also update the association in the backend
    if (props.promptVersionId) {
      try {
        await associateToolVersionWithPromptVersion({
          tool_version_id: toolVersion.id,
          prompt_version_id: props.promptVersionId
        });
      } catch (error) {
        console.error('Error associating tool with prompt:', error);
        // On error, revert the addition
        selectedTools.value = selectedTools.value.filter(t => t.id !== toolVersion.id);
      }
    }
  }
}

async function openModal() {
  // Make sure tools are loaded before opening modal
  if (tools.value.length === 0) {
    try {
      await fetchTools();
    } catch (error) {
      console.error('Failed to load tools before opening modal:', error);
    }
  }
  isModalOpen.value = true;
}

function closeModal() {
  isModalOpen.value = false;
}

function nextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
  }
}

function prevPage() {
  if (currentPage.value > 1) {
    currentPage.value--;
  }
}
</script>
