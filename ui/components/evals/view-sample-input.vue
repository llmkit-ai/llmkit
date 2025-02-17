<template>
  <div>
    <div class="sm:flex sm:items-center">
      <div class="sm:flex-auto">
        <h1 class="text-base font-semibold text-gray-900">Sample Inputs and Evals</h1>
        <p class="mt-2 text-sm text-gray-700">View and edit sample inputs, evals, or kick off a new eval run.</p>
      </div>
      <div class="mt-4 sm:ml-16 sm:mt-0 sm:flex-none flex items-center space-x-2">
        <PrimaryButton
          @click="$emit('create-sample')"
          type="secondary"
          size="sm"
        >
          New sample
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
          <table class="min-w-full divide-y divide-gray-300">
            <thead>
              <tr>
                <th scope="col" class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-0">Id</th>
                <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Name</th>
                <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Updated at</th>
                <th scope="col" class="relative py-3.5 pl-3 pr-4 sm:pr-0">
                  <span class="sr-only">Edit</span>
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200">
              <tr v-for="s in samples">
                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-0">{{ s.id }}</td>
                <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-0">{{ s.name }}</td>
                <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500">{{ s.updated_at }}</td>
                <td class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0">
                  <button @click="$emit('edit-sample', s.id)" class="text-neutral-600 hover:text-neutral-900">Edit</button>
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
import type { PromptSampleResponse } from '~/types/response/prompt_samples'

const props = defineProps<{
  samples: PromptSampleResponse[]
}>()

const emits = defineEmits<{
  "create-sample": [];
  "edit-sample": [id: number];
  "start-eval": [];
}>();

</script>
