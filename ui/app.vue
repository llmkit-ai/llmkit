<template>
  <Html>
    <Head>
      <link rel="preconnect" href="https://fonts.googleapis.com">
      <link rel="preconnect" href="https://fonts.gstatic.com">
      <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700;800&display=swap" rel="stylesheet">
    </Head>
    <Body class="bg-white dark:bg-neutral-900 transition-colors duration-300 font-mono antialiased">
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
if (import.meta.client) {
  watchEffect(() => {
    document.documentElement.classList.toggle('dark', colorMode.value === 'dark')
  })
}
</script>
