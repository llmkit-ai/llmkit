<template>
  <Html>
    <Body class="bg-white dark:bg-neutral-900 transition-colors duration-300">
      <NuxtLayout>
        <NuxtPage />
      </NuxtLayout>
    </Body>
  </Html>
</template>

<script setup lang="ts">
const colorMode = useColorMode()

// Server-side initial theme setup
useServerSeoMeta({
  // @ts-ignore
  script: [{
    innerHTML: import.meta.server ? `
      (function() {
        try {
          const stored = localStorage.getItem('nuxt-color-mode');
          const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          const colorMode = stored === 'dark' ? 'dark' : stored === 'light' ? 'light' : systemDark ? 'dark' : 'light';
          document.documentElement.classList.toggle('dark', colorMode === 'dark');
        } catch(e) {}
      })()
    ` : ''
  }]
})

// Client-side watcher
if (process.client) {
  watchEffect(() => {
    document.documentElement.classList.toggle('dark', colorMode.value === 'dark')
  })
}
</script>
