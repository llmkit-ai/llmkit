<template>
  <div class="font-mono pl-12">
    <!-- List View -->
    <div v-if="currentView === 'list'">
      <div class="flex w-full items-center justify-between mb-6">
        <h1 class="text-xl font-semibold text-neutral-900 dark:text-white">Tools</h1>
        <PrimaryButton @click="currentView = 'add'" buttonType="primary" size="sm">
          Add Tool
        </PrimaryButton>
      </div>
      
      <ToolsListView 
        :tools="tools" 
        :loading="loading" 
        :error="error"
        @view="onViewTool"
        @edit="onEditTool"
        @delete="onDeleteTool"
        @deleteConfirmed="handleDeleteTool"
        ref="listViewRef"
      />
    </div>

    <!-- Add View -->
    <div v-else-if="currentView === 'add'">
      <ToolForm
        :loading="formLoading"
        :error="error"
        @submit="handleCreateTool"
        @back="currentView = 'list'"
      />
    </div>

    <!-- Edit View -->
    <div v-else-if="currentView === 'edit'">
      <ToolForm
        :tool="selectedTool"
        :loading="formLoading"
        :error="error"
        @submit="handleUpdateTool"
        @back="currentView = 'list'"
      />
    </div>

    <!-- Details View -->
    <div v-else-if="currentView === 'details'">
      <ToolDetails
        :tool="selectedTool!"
        @back="currentView = 'list'"
        @edit="onEditTool"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useTools } from '~/composables/useTools'
import type { Tool } from '~/types/response/tools'
import PrimaryButton from '~/components/global/primary-button.vue'
import ToolsListView from '~/components/tools/list-view.vue'
import ToolForm from '~/components/tools/tool-form.vue'
import ToolDetails from '~/components/tools/tool-details.vue'

definePageMeta({
  middleware: ['auth'],
  layout: 'logged-in'
})

const { tools, loading, error, fetchTools, createTool, updateTool, deleteTool } = useTools()
const listViewRef = ref<InstanceType<typeof ToolsListView> | null>(null)

// View state
const currentView = ref<'list' | 'add' | 'edit' | 'details'>('list')
const formLoading = ref(false)
const selectedTool = ref<Tool | null>(null)

onMounted(async () => {
  await fetchTools()
})

// Event handlers for list view
function onViewTool(tool: Tool) {
  selectedTool.value = tool
  currentView.value = 'details'
}

function onEditTool(tool: Tool) {
  selectedTool.value = tool
  currentView.value = 'edit'
}

function onDeleteTool(tool: Tool) {
  if (listViewRef.value) {
    listViewRef.value.onDelete(tool)
  }
}

// Form submit handlers
async function handleCreateTool(formData: any) {
  try {
    formLoading.value = true
    await createTool({
      name: formData.name,
      tool_name: formData.tool_name,
      description: formData.description,
      parameters: formData.parameters,
      strict: formData.strict
    })
    await fetchTools() // Refresh the list
    currentView.value = 'list'
  } catch (err) {
    console.error('Failed to create tool:', err)
  } finally {
    formLoading.value = false
  }
}

async function handleUpdateTool(formData: any) {
  if (!formData.id) return
  
  try {
    formLoading.value = true
    await updateTool(formData.id, {
      name: formData.name,
      tool_name: formData.tool_name,
      description: formData.description,
      parameters: formData.parameters,
      strict: formData.strict
    })
    await fetchTools() // Refresh the list
    currentView.value = 'list'
  } catch (err) {
    console.error('Failed to update tool:', err)
  } finally {
    formLoading.value = false
  }
}

async function handleDeleteTool(toolId: number) {
  try {
    await deleteTool(toolId)
    // No need to refresh the list as deleteTool updates the state
  } catch (err) {
    console.error('Failed to delete tool:', err)
  }
}
</script>
