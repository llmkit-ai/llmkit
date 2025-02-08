<template>
  <div>
    <div
      v-if="logsLoading"
      class="text-center text-neutral-500 dark:text-neutral-400"
    >
      loading logs...
    </div>
    <div v-else-if="logsError" class="text-red-500">
      error loading logs: {{ logsError }}
    </div>
    <ul v-else class="divide-y divide-neutral-200 dark:divide-neutral-700">
      <li v-for="log in logs" :key="log.id" class="py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <div
              class="text-sm font-medium text-neutral-900 dark:text-white"
            >
              {{ log.id }}
            </div>
            <div
              class="text-sm text-neutral-500 dark:text-neutral-400"
            >
              Status:
              <span :class="getStatusBadgeClass(log.status_code)">
                {{ log.status_code }}
              </span>
            </div>
            <div
              class="text-sm text-neutral-500 dark:text-neutral-400"
            >
              Created: {{ formatDate(log.created_at) }}
            </div>
            <div
              class="text-sm text-neutral-500 dark:text-neutral-400"
            >
              Model: {{ log.model_name }}
            </div>
          </div>
          <button
            @click="toggleLog(log.id)"
            class="text-sm font-medium text-neutral-700 hover:text-neutral-900 dark:text-neutral-300 dark:hover:text-neutral-100"
          >
            {{ expandedLogId === log.id ? 'Hide details' : 'View details' }}
          </button>
        </div>

        <div v-if="expandedLogId === log.id" class="mt-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Prompt id
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {{ log.prompt_id }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Model id
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {{ log.model_id }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Status code
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {{ log.status_code }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Input tokens
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {{ log.input_tokens }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Output tokens
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {{ log.output_tokens }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Reasoning tokens
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {{ log.reasoning_tokens }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Request body
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300 break-words overflow-x-auto"
              >
                {{ log.request_body }}
              </dd>
            </div>
            <div>
              <dt
                class="text-sm font-medium text-neutral-900 dark:text-white"
              >
                Response data
              </dt>
              <dd
                class="mt-1 text-sm text-neutral-700 dark:text-neutral-300 break-words overflow-x-auto"
              >
                {{ log.response_data }}
              </dd>
            </div>
          </div>
        </div>
      </li>
    </ul>

    <nav
      class="flex items-center justify-between border-t border-neutral-200 bg-white py-5 dark:bg-transparent dark:border-neutral-700"
      aria-label="Pagination"
      v-if="!logsLoading && !logsError"
    >
      <div class="hidden sm:block">
        <p class="text-sm text-neutral-700 dark:text-neutral-300">
          showing
          <span class="font-medium">{{
            (currentPage - 1) * pageSize + 1
          }}</span>
          to
          <span class="font-medium">{{
            Math.min(currentPage * pageSize, totalLogs)
          }}</span>
          of
          <span class="font-medium">{{ totalLogs }}</span>
          results
        </p>
      </div>
      <div class="flex flex-1 justify-between sm:justify-end">
        <button
          type="button"
          @click="goToPage(currentPage - 1)"
          :disabled="currentPage === 1"
          :class="[
            'text-sm/6 p-2 border-2',
            currentPage === 1
              ? 'border-neutral-300 text-neutral-500 cursor-not-allowed dark:border-neutral-600 dark:text-neutral-400'
              : 'border-black dark:border-white bg-black dark:bg-white text-white dark:text-black hover:bg-neutral-800 dark:hover:bg-neutral-200',
          ]"
        >
          Previous
        </button>
        <button
          type="button"
          @click="goToPage(currentPage + 1)"
          :disabled="logs.length < pageSize"
          :class="[
            'ml-3 text-sm/6 p-2 border-2',
            logs.length < pageSize
              ? 'border-neutral-300 text-neutral-500 cursor-not-allowed dark:border-neutral-600 dark:text-neutral-400'
              : 'border-black dark:border-white bg-black dark:bg-white text-white dark:text-black hover:bg-neutral-800 dark:hover:bg-neutral-200',
          ]"
        >
          Next
        </button>
      </div>
    </nav>
  </div>
</template>

<script setup lang="ts">
import { format } from 'date-fns'

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

const getStatusBadgeClass = (statusCode: number | null) => {
  if (statusCode === 200) {
    return 'px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100'
  } else {
    return 'px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-800 dark:text-red-100'
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
</script>
