<template>
  <aside class="h-[calc(100vh-65px)]">
    <div class="p-4 space-y-2">
      <div
        v-for="prompt in prompts"
        :key="prompt.id"
        @click="emit('select', prompt)"
        class="p-3 cursor-pointer font-mono text-sm hover:bg-muted transition-colors"
        :class="{ 'bg-muted': selectedPromptId === prompt.id }"
      >
        <div class="truncate">{{ prompt.key }}</div>
        <div class="text-xs text-muted-foreground mt-1">{{ prompt.model }}</div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { Prompt } from '~/types/response/prompts'

defineProps<{
  prompts: Prompt[]
  selectedPromptId?: number
}>()

const emit = defineEmits<{
  (e: 'select', prompt: Prompt): void
}>()
</script>
