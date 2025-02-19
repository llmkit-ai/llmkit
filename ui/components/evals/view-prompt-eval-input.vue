<template>
  <div>
    <div class="sm:flex sm:items-center">
      <div class="sm:flex-auto">
        <h1 class="text-base font-semibold text-neutral-900 dark:text-neutral-100">Sample Inputs and Evals</h1>
        <p class="mt-2 text-sm text-neutral-700 dark:text-neutral-300">View and edit sample inputs, evals, or kick off a new eval run.</p>
      </div>
      <div class="mt-4 sm:ml-16 sm:mt-0 sm:flex-none flex items-center space-x-2">
        <PrimaryButton
          @click="$emit('create-eval')"
          type="secondary"
          size="sm"
        >
          New eval
        </PrimaryButton>
        <PrimaryButton
          type="primary"
          size="sm"
        >
          New eval run
        </PrimaryButton>
      </div>
    </div>
    <div class="mt-8 flow-root">
      <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
          <table class="min-w-full divide-y divide-neutral-300 dark:divide-neutral-700">
            <thead>
              <tr>
                <th scope="col" class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100 sm:pl-0">Id</th>
                <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Name</th>
                <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 dark:text-neutral-100">Updated at</th>
                <th scope="col" class="relative py-3.5 pl-3 pr-4 sm:pr-0">
                  <span class="sr-only">Edit</span>
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800">
              <tr v-for="s in evals">
                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">{{ s.id }}</td>
                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 dark:text-neutral-100 sm:pl-0">{{ s.name }}</td>
                <td class="whitespace-nowrap px-3 py-4 text-sm text-neutral-500 dark:text-neutral-400">{{ s.updated_at }}</td>
                <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0">
                  <button @click="$emit('edit-eval', s.id)" class="text-neutral-600 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100">Edit</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PromptEvalResponse } from '~/types/response/prompt_eval'

const props = defineProps<{
  evals: PromptEvalResponse[]
}>()

const emits = defineEmits<{
  "create-eval": [];
  "edit-eval": [id: number];
  "start-eval": [];
}>();

</script>
