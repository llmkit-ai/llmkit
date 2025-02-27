<template>
  <div>
    <div class="fixed top-4 right-4 z-50">
      <button
        @click="toggleDarkMode"
        class="flex items-center justify-center p-2 rounded-full bg-white dark:bg-neutral-800 border-2 border-black dark:border-white"
      >
        <svg
          class="w-5 h-5 text-black dark:text-white"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            v-if="isDark"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
          ></path>
          <path
            v-else
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
          ></path>
        </svg>
      </button>
    </div>

    <slot />
  </div>
</template>

<script setup lang="ts">
const { $colorMode } = useNuxtApp();
const isDark = ref(false);

onMounted(() => {
  isDark.value = $colorMode.value === 'dark';
});

const toggleDarkMode = () => {
  isDark.value = !isDark.value;
  $colorMode.preference = isDark.value ? 'dark' : 'light';
};
</script>
