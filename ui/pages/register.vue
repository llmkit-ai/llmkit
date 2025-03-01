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
      <h2 class="mt-10 text-center text-2xl font-bold text-black dark:text-white">Create admin account</h2>
      <p class="mt-3 text-center text-sm text-neutral-700 dark:text-neutral-300">This registration can only be done once for the system admin.</p>
    </div>

    <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
      <form class="space-y-6" @submit.prevent="handleRegister">
        <div v-if="errorMessage" class="p-3 bg-red-50 text-red-600 border border-red-300 dark:bg-red-900/30 dark:border-red-800 dark:text-red-400">
          {{ errorMessage }}
        </div>
        
        <div v-if="successMessage" class="p-3 bg-green-50 text-green-600 border border-green-300 dark:bg-green-900/30 dark:border-green-800 dark:text-green-400">
          {{ successMessage }}
        </div>
        
        <div>
          <label for="name" class="block text-sm font-medium text-black dark:text-white">Name</label>
          <div class="mt-2">
            <input 
              v-model="name" 
              type="text" 
              name="name" 
              id="name" 
              autocomplete="name" 
              required 
              class="block w-full rounded-none bg-white px-3 py-1.5 text-base text-black border-2 border-black dark:bg-neutral-800 dark:text-white dark:border-white focus:outline-none sm:text-sm"
            >
          </div>
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
          <label for="password" class="block text-sm font-medium text-black dark:text-white">Password</label>
          <div class="mt-2">
            <input 
              v-model="password" 
              type="password" 
              name="password" 
              id="password" 
              autocomplete="new-password" 
              required 
              class="block w-full rounded-none bg-white px-3 py-1.5 text-base text-black border-2 border-black dark:bg-neutral-800 dark:text-white dark:border-white focus:outline-none sm:text-sm"
              @focus="showPasswordRequirements = true"
              @blur="showPasswordRequirements = false"
            >
          </div>
          <div v-if="showPasswordRequirements || password" class="mt-2 text-xs text-neutral-700 dark:text-neutral-300 space-y-1">
            <p>Password must:</p>
            <ul class="list-disc pl-5 space-y-1">
              <li :class="{ 'text-green-600 dark:text-green-400': password.length >= 8 }">Be at least 8 characters long</li>
              <li :class="{ 'text-green-600 dark:text-green-400': /[A-Z]/.test(password) }">Contain at least one uppercase letter</li>
              <li :class="{ 'text-green-600 dark:text-green-400': /[a-z]/.test(password) }">Contain at least one lowercase letter</li>
              <li :class="{ 'text-green-600 dark:text-green-400': /[0-9]/.test(password) }">Contain at least one number</li>
              <li :class="{ 'text-green-600 dark:text-green-400': /[^A-Za-z0-9]/.test(password) }">Contain at least one special character</li>
            </ul>
          </div>
        </div>

        <div>
          <label for="confirm_password" class="block text-sm font-medium text-black dark:text-white">Confirm password</label>
          <div class="mt-2">
            <input 
              v-model="confirmPassword" 
              type="password" 
              name="confirm_password" 
              id="confirm_password" 
              autocomplete="new-password" 
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
            {{ isLoading ? 'Registering...' : 'Register' }}
          </PrimaryButton>
        </div>
      </form>

      <p class="mt-10 text-center text-sm text-neutral-700 dark:text-neutral-300">
        Already have an account?
        <NuxtLink to="/login" class="font-semibold text-black dark:text-white hover:underline">Sign in</NuxtLink>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const router = useRouter();
const name = ref('');
const email = ref('');
const password = ref('');
const confirmPassword = ref('');
const isLoading = ref(false);
const errorMessage = ref('');
const successMessage = ref('');
const showPasswordRequirements = ref(false);

// Password validation function
const validatePassword = (password: string) => {
  if (password.length < 8) {
    return { valid: false, message: 'Password must be at least 8 characters long' };
  }
  
  if (!/[A-Z]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one uppercase letter' };
  }
  
  if (!/[a-z]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one lowercase letter' };
  }
  
  if (!/[0-9]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one number' };
  }
  
  if (!/[^A-Za-z0-9]/.test(password)) {
    return { valid: false, message: 'Password must contain at least one special character' };
  }
  
  return { valid: true, message: '' };
};

const handleRegister = async () => {
  // Reset messages
  errorMessage.value = '';
  successMessage.value = '';
  
  // Validate form
  if (!name.value || !email.value || !password.value || !confirmPassword.value) {
    errorMessage.value = 'Please fill in all fields';
    return;
  }
  
  if (password.value !== confirmPassword.value) {
    errorMessage.value = 'Passwords do not match';
    return;
  }
  
  // Password validation
  const passwordValidation = validatePassword(password.value);
  if (!passwordValidation.valid) {
    errorMessage.value = passwordValidation.message;
    return;
  }

  isLoading.value = true;

  try {
    const response = await fetch('/v1/ui/auth/register', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: name.value,
        email: email.value,
        password: password.value
      }),
      credentials: 'include'
    });

    if (!response.ok) {
      const errorData = await response.text();
      throw new Error(errorData || 'Registration failed');
    }

    // Registration success
    successMessage.value = 'Registration successful! Redirecting to dashboard...';
    
    // Clear form
    name.value = '';
    email.value = '';
    password.value = '';
    confirmPassword.value = '';
    
    // After a delay, redirect to dashboard (since they're already logged in)
    setTimeout(() => {
      router.push('/');
    }, 2000);
  } catch (err) {
    console.error('Registration error:', err);
    errorMessage.value = err instanceof Error ? err.message : 'An unexpected error occurred';
  } finally {
    isLoading.value = false;
  }
};

definePageMeta({
  layout: 'default',
  middleware: ['auth'] // Auth middleware will check and skip for register page
});
</script>
