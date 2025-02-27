<template>
  <div></div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'loggedIn',
  middleware: ['auth']
});

navigateTo("/prompts")
</script>
