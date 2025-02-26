<template>
  <div class="min-h-screen flex items-center justify-center bg-white dark:bg-neutral-900 p-6">
    <div class="w-full max-w-md">
      <!-- Logo and Title -->
      <div class="text-center mb-8">
        <div class="flex justify-center items-center space-x-2 mb-4">
          <svg class="size-12 text-black dark:text-white" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
            <path d="M20 4v5l-9 7l-4 4l-3 -3l4 -4l7 -9z"></path>
            <path d="M6.5 11.5l6 6"></path>
          </svg>
        </div>
        <h1 class="text-2xl font-mono font-bold text-black dark:text-white">llmkit</h1>
        <p class="mt-2 text-neutral-600 dark:text-neutral-400">Sign in to your account</p>
      </div>
      
      <!-- Login Form -->
      <div class="bg-neutral-100 dark:bg-neutral-800 p-8 border-2 border-black dark:border-white">
        <form @submit.prevent="login">
          <!-- Error Message -->
          <div v-if="error" class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 border border-red-400 dark:border-red-700 text-red-700 dark:text-red-400 text-sm">
            {{ error }}
          </div>
          
          <!-- Username Field -->
          <div class="mb-4">
            <label for="username" class="block text-sm font-medium text-neutral-900 dark:text-white mb-1">
              Username
            </label>
            <input
              id="username"
              v-model="username"
              type="text"
              class="w-full bg-white dark:bg-neutral-700 border-2 border-black dark:border-white p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
              required
              autocomplete="username"
            />
          </div>
          
          <!-- Password Field -->
          <div class="mb-6">
            <label for="password" class="block text-sm font-medium text-neutral-900 dark:text-white mb-1">
              Password
            </label>
            <input
              id="password"
              v-model="password"
              type="password"
              class="w-full bg-white dark:bg-neutral-700 border-2 border-black dark:border-white p-2 text-base focus:outline-none text-neutral-900 dark:text-white"
              required
              autocomplete="current-password"
            />
          </div>
          
          <!-- Submit Button -->
          <div>
            <button
              type="submit"
              class="w-full font-mono transition-colors border-2 border-black bg-black text-white dark:border-white dark:bg-white dark:text-black hover:bg-neutral-800 dark:hover:bg-neutral-200 p-2 text-base"
              :disabled="isLoading"
            >
              <span v-if="isLoading">Signing in...</span>
              <span v-else>Sign in</span>
            </button>
          </div>
        </form>
      </div>
      
      <!-- Color Mode Toggle -->
      <div class="mt-6 text-center">
        <button 
          @click="toggleDarkMode"
          class="text-sm text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white"
        >
          Toggle {{ $colorMode.value === 'dark' ? 'Light' : 'Dark' }} Mode
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

definePageMeta({
  layout: false,
  auth: {
    unauthenticatedOnly: true,
    navigateAuthenticatedTo: '/'
  }
})

const username = ref('')
const password = ref('')
const error = ref('')
const isLoading = ref(false)
const router = useRouter()
const { $colorMode } = useNuxtApp()

const toggleDarkMode = () => {
  $colorMode.preference = $colorMode.value === 'dark' ? 'light' : 'dark'
}

async function login() {
  error.value = ''
  isLoading.value = true
  
  try {
    const response = await $fetch('/api/auth/login', {
      method: 'POST',
      body: {
        username: username.value,
        password: password.value
      }
    })
    
    if (response.success) {
      // Redirect to home page after successful login
      router.push('/')
    }
  } catch (e: any) {
    error.value = e.data?.message || 'Failed to sign in. Please check your credentials.'
  } finally {
    isLoading.value = false
  }
}
</script>