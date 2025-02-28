<template>
  <div class="h-screen flex min-h-full flex-col justify-center px-6 py-12 lg:px-8 font-mono bg-white dark:bg-neutral-900">
    <div class="sm:mx-auto sm:w-full sm:max-w-sm">
      <div class="flex justify-center items-center space-x-2">
        <svg class="size-8 text-black dark:text-white" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
          <path d="M20 4v5l-9 7l-4 4l-3 -3l4 -4l7 -9z"></path>
          <path d="M6.5 11.5l6 6"></path>
        </svg>
        <p class="font-mono font-bold text-black dark:text-white">llmkit</p>
      </div>
      <h2 class="mt-10 text-center text-2xl font-bold text-black dark:text-white">Sign in to your account</h2>
    </div>

    <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
      <form class="space-y-6" @submit.prevent="handleLogin">
        <div v-if="errorMessage" class="p-3 bg-red-50 text-red-600 border border-red-300 dark:bg-red-900/30 dark:border-red-800 dark:text-red-400">
          {{ errorMessage }}
        </div>
        
        <div>
          <label for="email" class="block text-sm font-medium text-black dark:text-white">Email address</label>
          <div class="mt-2">
            <input 
              v-model="email" 
              type="email" 
              name="email" 
              id="email" 
              autocomplete="email" 
              required 
              class="block w-full rounded-none bg-white px-3 py-1.5 text-base text-black border-2 border-black dark:bg-neutral-800 dark:text-white dark:border-white focus:outline-none sm:text-sm"
            >
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between">
            <label for="password" class="block text-sm font-medium text-black dark:text-white">Password</label>
            <div class="text-sm">
              <a href="#" tabindex="-1" class="font-semibold text-black dark:text-white hover:underline">Forgot password?</a>
            </div>
          </div>
          <div class="mt-2">
            <input 
              v-model="password" 
              type="password" 
              name="password" 
              id="password" 
              autocomplete="current-password" 
              required 
              class="block w-full rounded-none bg-white px-3 py-1.5 text-base text-black border-2 border-black dark:bg-neutral-800 dark:text-white dark:border-white focus:outline-none sm:text-sm"
            >
          </div>
        </div>

        <div>
          <PrimaryButton 
            buttonType="primary" 
            htmlType="submit"
            size="md"
            class="w-full justify-center"
            :outline="false"
            :disabled="isLoading"
          >
            {{ isLoading ? 'Signing in...' : 'Sign in' }}
          </PrimaryButton>
        </div>
      </form>

      <p class="mt-10 text-center text-sm text-neutral-700 dark:text-neutral-300">
        Don't have an account?
        <NuxtLink to="/register" class="font-semibold text-black dark:text-white hover:underline">Register now</NuxtLink>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const router = useRouter();
const email = ref('');
const password = ref('');
const isLoading = ref(false);
const errorMessage = ref('');

// We'll just send the plaintext password for now
// In a production environment, this would be sent over HTTPS

const handleLogin = async () => {
  if (!email.value || !password.value) {
    errorMessage.value = 'Please enter both email and password';
    return;
  }

  isLoading.value = true;
  errorMessage.value = '';

  try {
    const response = await fetch('/v1/ui/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: email.value,
        password: password.value
      }),
      credentials: 'include' // Important for cookies
    });

    if (!response.ok) {
      const errorData = await response.text();
      throw new Error(errorData || 'Login failed. Please check your credentials.');
    }

    // Successfully logged in - redirect to dashboard
    router.push('/');
  } catch (err) {
    console.error('Login error:', err);
    errorMessage.value = err instanceof Error ? err.message : 'An unexpected error occurred';
  } finally {
    isLoading.value = false;
  }
};

definePageMeta({
  layout: 'default',
  middleware: ['auth'] // Auth middleware will check and skip for login page
});
</script>
