<template>
  <div>
    <!-- Loading state -->
    <div v-if="loading" class="mt-4 animate-pulse space-y-2">
      <div v-for="i in 3" :key="i" class="h-16 w-full rounded bg-neutral-200 dark:bg-neutral-800"></div>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="mt-4 rounded border-2 border-red-500 bg-red-100 p-4 text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {{ error }}
    </div>

    <!-- Empty state -->
    <div v-else-if="!tools.length" class="mt-4 flex flex-col items-center justify-center rounded border-2 border-dashed border-neutral-400 bg-neutral-100 p-8 text-center dark:border-neutral-700 dark:bg-neutral-800/50">
      <svg class="mb-2 size-8 text-neutral-500 dark:text-neutral-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21.75 6.75a4.5 4.5 0 0 1-4.884 4.484c-1.076-.091-2.264.071-2.95.904l-7.152 8.684a2.548 2.548 0 1 1-3.586-3.586l8.684-7.152c.833-.686.995-1.874.904-2.95a4.5 4.5 0 0 1 6.336-4.486l-3.276 3.276a3.004 3.004 0 0 0 2.25 2.25l3.276-3.276c.256.565.398 1.192.398 1.852Z" />
        <path d="M4.867 19.125h.008v.008h-.008v-.008Z" />
      </svg>
      <p class="text-neutral-600 dark:text-neutral-400">No tools added yet</p>
      <p class="mt-2 text-sm text-neutral-500 dark:text-neutral-500">Add a new tool by clicking the Add Tool button above.</p>
    </div>

    <!-- Tools list -->
    <div v-else class="mt-4">
      <div class="border border-neutral-200 dark:border-neutral-700">
        <table class="min-w-full divide-y divide-neutral-100 dark:divide-neutral-700">
          <thead class="bg-neutral-100 dark:bg-neutral-800">
            <tr>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Tool Name
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Function Name
              </th>
              <th scope="col" class="px-6 py-3 text-left text-xs/4 font-medium text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white dark:bg-neutral-900 divide-y divide-neutral-100 dark:divide-neutral-700">
            <tr v-for="tool in tools" :key="tool.id" class="hover:bg-neutral-50 dark:hover:bg-neutral-800">
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 font-medium text-neutral-700 dark:text-neutral-300">
                {{ tool.name }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm/6 text-neutral-500 dark:text-neutral-400">
                {{ tool.tool_name }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex space-x-2">
                  <PrimaryButton 
                    @click="$emit('view', tool)" 
                    buttonType="secondary"
                    size="xs"
                  >
                    View
                  </PrimaryButton>
                  <PrimaryButton 
                    @click="$emit('edit', tool)" 
                    buttonType="secondary"
                    size="xs"
                  >
                    Edit
                  </PrimaryButton>
                  <PrimaryButton 
                    @click="$emit('delete', tool)" 
                    buttonType="error"
                    size="xs"
                  >
                    Delete
                  </PrimaryButton>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div 
      v-if="showDeleteConfirm" 
      class="fixed inset-0 z-10 overflow-y-auto"
      aria-labelledby="modal-title" 
      role="dialog" 
      aria-modal="true"
    >
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-neutral-500 bg-opacity-75 transition-opacity" aria-hidden="true"></div>
        
        <span class="hidden sm:inline-block sm:align-middle sm:h-screen" aria-hidden="true">&#8203;</span>
        <div class="inline-block align-bottom bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 p-6 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <div>
            <div class="flex justify-between">
              <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white" id="modal-title">
                Delete Tool
              </h3>
              <button
                @click="cancelDelete"
                class="text-neutral-400 hover:text-neutral-500 dark:text-neutral-300 dark:hover:text-neutral-200"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
            <div class="mt-4">
              <p class="text-sm text-neutral-600 dark:text-neutral-400">
                Are you sure you want to delete the tool <span class="font-semibold">{{ toolToDelete?.name }}</span>? This action cannot be undone.
              </p>
              
              <div class="mt-6 flex justify-end space-x-3">
                <PrimaryButton 
                  type="button"
                  @click="cancelDelete"
                  buttonType="secondary"
                  size="sm"
                >
                  Cancel
                </PrimaryButton>
                <PrimaryButton 
                  type="button" 
                  @click="confirmDelete"
                  :loading="deleteLoading"
                  buttonType="error"
                  size="sm"
                >
                  Delete
                </PrimaryButton>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Tool } from '~/types/response/tools'
import PrimaryButton from '~/components/global/primary-button.vue'

const props = defineProps<{
  tools: Tool[]
  loading: boolean
  error: string | null
}>()

const emit = defineEmits<{
  view: [tool: Tool]
  edit: [tool: Tool]
  delete: [tool: Tool]
  deleteConfirmed: [toolId: number]
}>()

const showDeleteConfirm = ref(false)
const toolToDelete = ref<Tool | null>(null)
const deleteLoading = ref(false)

// We still use a modal for delete confirmation
watch(() => props.tools, () => {
  // Reset modal state if tools list changes
  showDeleteConfirm.value = false
  toolToDelete.value = null
  deleteLoading.value = false
}, { deep: true })

function cancelDelete() {
  showDeleteConfirm.value = false
  toolToDelete.value = null
}

function confirmDelete() {
  if (!toolToDelete.value) return
  
  deleteLoading.value = true
  emit('deleteConfirmed', toolToDelete.value.id)
  showDeleteConfirm.value = false
  toolToDelete.value = null
  deleteLoading.value = false
}

function onDelete(tool: Tool) {
  toolToDelete.value = tool
  showDeleteConfirm.value = true
}

// Expose the delete handler to parent
defineExpose({
  onDelete
})
</script>
