<template>
  <div>

  </div>
</template>

<script setup lang="ts">

const { 
  prompts, 
  loading: promptsLoading, 
  error: promptsError, 
  fetchPrompts, 
  createPrompt, 
  updatePrompt, 
  deletePrompt 
} = usePrompts()

const { 
  models, 
  loading: modelsLoading, 
  error: modelsError, 
  fetchModels, 
} = useModels()

// Initialize
onMounted(async () => {
  await fetchPrompts()
  await fetchModels()
})
</script>
