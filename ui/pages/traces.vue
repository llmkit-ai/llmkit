<template>
  <div class="font-mono">
    <div
      v-if="logsLoading"
      class="text-center text-neutral-500 dark:text-neutral-400 animate-pulse"
    >
      $ loading logs...
    </div>
    <div v-else-if="logsError" class="text-red-500 dark:text-red-400">
      error: {{ logsError }}
    </div>
    <div v-else class="terminal-container">
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        <li 
          v-for="log in logs" 
          :key="log.id" 
          class="py-3 px-4 hover:bg-neutral-50 dark:hover:bg-neutral-900/50 transition-colors"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-4">
              <span class="text-emerald-600 dark:text-emerald-400">$</span>
              <div class="text-neutral-900 dark:text-neutral-300">
                <span class="text-neutral-500 dark:text-neutral-400">[{{ formatDate(log.created_at) }}]</span>
                <span :class="getStatusBadgeClass(log.status_code)" class="ml-2">{{ log.status_code }}</span>
              </div>
              <div class="text-neutral-500 dark:text-neutral-400">
                {{ log.model_name }}
              </div>
            </div>
            <button
              @click="toggleLog(log.id)"
              class="text-neutral-700 hover:text-neutral-900 dark:text-neutral-300 dark:hover:text-neutral-100"
            >
              {{ expandedLogId === log.id ? '◄ collapse' : '► expand' }}
            </button>
          </div>

          <div v-if="expandedLogId === log.id" class="mt-3 ml-6">
            <div class="grid gap-2 text-sm">
              <div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2">
                <span class="text-neutral-500 dark:text-neutral-400">prompt:</span>
                <span class="text-neutral-900 dark:text-neutral-300">{{ log.prompt_id || 'n/a' }}</span>
                
                <span class="text-neutral-500 dark:text-neutral-400">provider id:</span>
                <span class="text-neutral-900 dark:text-neutral-300">{{ log.provider_response_id }}</span>
                
                <span class="text-neutral-500 dark:text-neutral-400">tokens:</span>
                <span class="text-neutral-900 dark:text-neutral-300">
                  [in:{{ log.input_tokens }} out:{{ log.output_tokens }} rt:{{ log.reasoning_tokens }}]
                </span>
                
                <span v-if="log.request_body" class="text-neutral-500 dark:text-neutral-400">request:</span>
                <pre v-if="log.request_body" class="p-2 bg-neutral-200 dark:bg-neutral-800 rounded text-neutral-900 dark:text-neutral-300 overflow-x-auto">{{ parseJson(log.request_body) }}</pre>
                
                <span v-if="log.response_data" class="text-neutral-500 dark:text-neutral-400">response:</span>
                <pre v-if="log.response_data" class="p-2 bg-neutral-200 dark:bg-neutral-800 rounded text-neutral-900 dark:text-neutral-300 overflow-x-auto">{{ parseJson(log.response_data) }}</pre>
              </div>
            </div>
          </div>
        </li>
      </ul>
      <div class="terminal-header bg-neutral-100 border-t border-neutral-400 dark:bg-neutral-900 p-2 text-sm">
        <span class="text-neutral-500 dark:text-neutral-400">// logs {{ (currentPage - 1) * pageSize + 1 }}-{{ Math.min(currentPage * pageSize, totalLogs) }} of {{ totalLogs }}</span>
      </div>
    </div>

    <nav
      class="mt-4 flex items-center justify-between py-5 dark:border-neutral-800"
      v-if="!logsLoading && !logsError"
    >
      <div class="flex-1 flex justify-between sm:justify-end space-x-2">
        <PrimaryButton
          buttonType="primary"
          size="sm"
          :disabled="currentPage === 1"
          @click="goToPage(currentPage - 1)"
        >
          ← prev
        </PrimaryButton>

        <PrimaryButton
          buttonType="primary"
          size="sm"
          :disabled="logs.length < pageSize"
          @click="goToPage(currentPage + 1)"
          class="ml-2"
        >
          next →
        </PrimaryButton>
      </div>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { format } from 'date-fns'

definePageMeta({
  layout: "logged-in",
  middleware: ['auth']
})

const pageSize = 10 // items per page
const currentPage = ref(1)

const {
  logs,
  loading: logsLoading,
  error: logsError,
  fetchLogs,
  fetchLogsCount,
  totalLogs,
} = useLogs()

const expandedLogId = ref<number | null>(null)

const toggleLog = (id: number) => {
  expandedLogId.value = expandedLogId.value === id ? null : id
}

const formatDate = (dateString: string | undefined) => {
  if (!dateString) return 'n/a'
  try {
    return format(new Date(dateString), 'yyyy-mm-dd hh:mm:ss')
  } catch (error) {
    console.error('error formatting date', error)
    return 'invalid date'
  }
}

// const getStatusBadgeClass = (statusCode: number | null) => {
//   if (statusCode === 200) {
//     return 'px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100'
//   } else {
//     return 'px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-800 dark:text-red-100'
//   }
// }

const getStatusBadgeClass = (statusCode: number | null) => {
  if (statusCode === 200) {
    return 'text-green-600 dark:text-green-400'
  } else {
    return 'text-red-600 dark:text-red-400'
  }
}

const goToPage = (page: number) => {
  currentPage.value = page
  fetchLogs(page, pageSize)
}

onMounted(async () => {
  await fetchLogs(currentPage.value, pageSize)
  await fetchLogsCount()
})

function parseJson(json: string | null) {
  if (!json) { return null; } 

  try {
    return JSON.parse(json)
  } catch {
    return json
  }
}
</script>

<style>
.terminal-container {
  @apply border border-neutral-200 dark:border-neutral-800 rounded;
  box-shadow: 0 1px 3px rgba(0,0,0,0.02);
}

.terminal-header {
  @apply border-b border-neutral-200 dark:border-neutral-800;
}

pre {
  @apply font-mono text-xs p-3;
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
