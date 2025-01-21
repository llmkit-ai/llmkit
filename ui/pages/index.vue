<template>
  <div class="flex h-screen bg-background">
    <div v-if="loading" class="absolute inset-0 bg-background/80 flex items-center justify-center">
      Loading prompts...
    </div>
    <div v-if="error" class="absolute inset-0 bg-red-100 text-red-600 p-4">
      Error: {{ error }}
    </div>

    <!-- Left sidebar -->
    <div class="w-64 border-r border-border">
      <div class="p-4 w-full">
        <button 
          @click="handleNewPrompt" 
          class="border-2 border-black p-2 flex items-center font-mono hover:bg-gray-50" 
          :disabled="loading"
        >
          <Plus class="w-4 h-4 mr-3 -mt-0.5" />
          NEW PROMPT
        </button>
      </div>
      <PromptList
        :prompts="prompts"
        :selected-prompt-id="selectedPrompt?.id"
        @select="handleSelectPrompt"
      />
    </div>

    <!-- Right side editor -->
    <div class="flex-1 flex flex-col">
      <div v-if="selectedPrompt" class="h-full flex flex-col">
        <div class="p-4 border-b border-border flex items-center justify-between">
          <div class="flex-1">
            <input
              v-model="editedPrompt.key"
              class="font-mono text-lg bg-transparent w-full focus:outline-none"
              placeholder="Prompt key..."
            />
          </div>
          <select 
            v-model="editedPrompt.model"
            class="font-mono text-sm bg-background border rounded px-2 py-1"
          >
            <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
            <option value="gpt-4">GPT-4</option>
          </select>
        </div>
        
        <div class="flex-1 p-4">
          <textarea
            v-model="editedPrompt.prompt"
            class="w-full h-full min-h-[300px] font-mono resize-none p-4 focus:outline-none"
            placeholder="Enter your prompt here..."
          />
        </div>

        <div class="p-4 border-t border-border flex justify-between">
          <div class="space-x-2 flex items-center">
            <button 
              @click="handleExecutePrompt" 
              class="flex items-center border-2 border-black p-2 hover:bg-gray-50"
            >
              <Play class="w-4 h-4 mr-2" />
              Execute
            </button>
            <button 
              @click="handleSaveChanges" 
              class="flex items-center border-2 border-black p-2 hover:bg-gray-50"
            >
              <Save class="w-4 h-4 mr-2" />
              Save Changes
            </button>
          </div>
          <button 
            @click="handleDeletePrompt(selectedPrompt.id)" 
            class="flex items-center border-2 border-black p-2 hover:bg-gray-50"
          >
            <Trash class="w-4 h-4 mr-2" />
            Delete Prompt
          </button>
        </div>
      </div>

      <div v-else class="flex-1 flex items-center justify-center text-muted-foreground font-mono">
        Select a prompt or create a new one
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Plus, Play, Save, Trash } from 'lucide-vue-next'
import type { Prompt } from '~/types/response/prompts'

const { 
  prompts, 
  loading, 
  error, 
  fetchPrompts, 
  createPrompt, 
  updatePrompt, 
  deletePrompt 
} = usePrompts()

// Initialize
onMounted(async () => {
  await fetchPrompts()
  if (prompts.value.length > 0) {
    handleSelectPrompt(prompts.value[0])
  }
})

const selectedPrompt = ref<Prompt | null>(null)
const editedPrompt = ref<Partial<Prompt>>({})

const handleSelectPrompt = (prompt: Prompt) => {
  selectedPrompt.value = prompt
  editedPrompt.value = { ...prompt }
}

const handleNewPrompt = async () => {
  const newPrompt = await createPrompt({
    key: "new-prompt",
    prompt: "New prompt content...",
    model: "sonnet-3.5"
  })
  if (newPrompt) handleSelectPrompt(newPrompt)
}

const handleDeletePrompt = async (id: number) => {
  await deletePrompt(id)
  if (selectedPrompt.value?.id === id) {
    selectedPrompt.value = null
    editedPrompt.value = {}
  }
}

const handleSaveChanges = async () => {
  if (!selectedPrompt.value) return
  await updatePrompt(selectedPrompt.value.id, editedPrompt.value)
}

const handleExecutePrompt = () => {
  console.log("Executing prompt:", selectedPrompt.value?.prompt)
}
</script>
