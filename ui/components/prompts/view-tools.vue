<template>
  <div>
    <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white mb-4">
      Associated Tools <span v-if="tools.length > 0" class="text-sm font-normal text-neutral-500">({{ tools.length }})</span>
      <span v-if="tools.length > 0 && modelSupportsTools === false" class="ml-2 text-xs text-amber-500">(Warning: Model doesn't support tools)</span>
    </h3>
    
    <div v-if="loading" class="animate-pulse p-4 space-y-2">
      <div v-for="i in 3" :key="i" class="h-12 bg-neutral-200 dark:bg-neutral-700 rounded"></div>
    </div>
    
    <div v-else-if="tools.length === 0" class="p-4 text-sm text-neutral-600 dark:text-neutral-400 italic bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-md">
      No tools associated with this prompt
    </div>
    
    <div v-else>
      <!-- Tool Accordion List -->
      <div class="border border-neutral-200 dark:border-neutral-700 rounded-md overflow-hidden">
        <div v-for="(tool, index) in tools" :key="tool.id">
          <!-- Tool Row with Expandable Details -->
          <div class="border-b border-neutral-200 dark:border-neutral-700 last:border-b-0">
            <!-- Tool Header Row -->
            <div 
              class="bg-white dark:bg-neutral-900 px-4 py-3 flex justify-between items-center cursor-pointer hover:bg-neutral-50 dark:hover:bg-neutral-800"
              @click="toggleTool(index)"
            >
              <div class="flex items-center">
                <span class="font-medium text-neutral-900 dark:text-white">{{ tool.tool_name }}</span>
              </div>
              <svg
                class="w-5 h-5 text-neutral-500 dark:text-neutral-400 transition-transform duration-200"
                :class="{ 'transform rotate-180': expandedTools[index] }"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
              </svg>
            </div>
            
            <!-- Tool Expanded Details -->
            <div 
              v-if="expandedTools[index]" 
              class="p-4 border-t border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800"
            >
              <div class="mb-2 text-sm text-neutral-600 dark:text-neutral-400">
                {{ tool.description }}
              </div>
              <div class="mt-3">
                <h5 class="text-xs font-medium text-neutral-900 dark:text-white mb-1">Parameters</h5>
                <pre class="p-2 bg-white dark:bg-neutral-900 text-xs overflow-auto text-neutral-800 dark:text-neutral-300 font-mono rounded">{{ formatJson(tool.parameters) }}</pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { Tool } from '~/types/response/tools';

const props = defineProps<{
  tools: Tool[];
  loading?: boolean;
  modelSupportsTools?: boolean;
}>();

// UI state
const expandedTools = ref<boolean[]>([]);

// Initialize expanded state for each tool
watch(() => props.tools, (newTools) => {
  expandedTools.value = newTools.map(() => false);
}, { immediate: true });

// Toggle individual tool expansion
function toggleTool(index: number) {
  if (index >= 0 && index < expandedTools.value.length) {
    expandedTools.value[index] = !expandedTools.value[index];
  }
}

// Format JSON for display
function formatJson(jsonString: string): string {
  try {
    const parsed = JSON.parse(jsonString);
    return JSON.stringify(parsed, null, 2);
  } catch (e) {
    return jsonString;
  }
}
</script>
