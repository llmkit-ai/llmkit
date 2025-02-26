<template>
  <div>
    <div class="fixed inset-y-0 z-50 flex w-72 flex-col">
      <div class="flex grow flex-col gap-y-5 overflow-y-auto border-r-2 border-black bg-white px-6 dark:border-neutral-700 dark:bg-neutral-900">
        <div class="flex h-16 shrink-0 items-center space-x-2">
          <svg class="size-8 text-black dark:text-white" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
            <path d="M20 4v5l-9 7l-4 4l-3 -3l4 -4l7 -9z"></path>
            <path d="M6.5 11.5l6 6"></path>
          </svg>
          <p class="font-mono font-bold text-black dark:text-white">llmkit</p>
        </div>
        <nav class="flex flex-1 flex-col">
          <ul role="list" class="flex flex-1 flex-col gap-y-7">
            <li>
              <ul role="list" class="-mx-2 space-y-4">
                <li>
                  <NuxtLink 
                    to="/prompts" 
                    class="group flex gap-x-3 rounded-none p-2 text-sm/6 font-semibold text-black border-2 border-black"
                    :class="route.name === 'prompts' ? 'dark:text-white dark:border-white bg-neutral-200 dark:bg-neutral-800':'dark:text-neutral-300 dark:border-neutral-300 text-neutral-700'"
                  >
                    <svg 
                      class="size-6 shrink-0" 
                      :class="route.name === 'prompts' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                      viewBox="0 0 24 24" 
                      fill="none" 
                      stroke="currentColor" 
                      stroke-linecap="round" 
                      stroke-linejoin="round" 
                      stroke-width="2"
                    >
                      <path d="M5 7l5 5l-5 5"></path>
                      <path d="M13 17l6 0"></path>
                    </svg>
                    Prompts
                  </NuxtLink>
                </li>
                <li>
                  <NuxtLink 
                    to="/traces" 
                    class="group flex gap-x-3 rounded-none p-2 text-sm/6 font-semibold text-black border-2 border-black"
                    :class="route.name === 'traces' ? 'dark:text-white dark:border-white bg-neutral-200 dark:bg-neutral-800':'dark:text-neutral-300 dark:border-neutral-300 text-neutral-700'"
                  >
                    <svg 
                      class="size-6 shrink-0"
                      :class="route.name === 'traces' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                      fill="none" 
                      viewBox="0 0 24 24" 
                      stroke-width="1.5" 
                      stroke="currentColor" 
                      aria-hidden="true" 
                      data-slot="icon"
                    >
                      <path d="M4 6c0 1.657 3.582 3 8 3s8 -1.343 8 -3s-3.582 -3 -8 -3s-8 1.343 -8 3"></path>
                      <path d="M4 6v6c0 1.657 3.582 3 8 3m8 -3.5v-5.5"></path>
                      <path d="M4 12v6c0 1.657 3.582 3 8 3"></path>
                      <path d="M18 18m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0"></path>
                      <path d="M20.2 20.2l1.8 1.8"></path>
                    </svg>
                    Traces
                  </NuxtLink>
                </li>
                <li>
                  <NuxtLink 
                    to="/evals" 
                    class="group flex gap-x-3 rounded-none p-2 text-sm/6 font-semibold text-black border-2 border-black"
                    :class="route.name === 'evals' ? 'dark:text-white dark:border-white bg-neutral-200 dark:bg-neutral-800':'dark:text-neutral-300 dark:border-neutral-300 text-neutral-700'"
                  >
                    <svg 
                      xmlns="http://www.w3.org/2000/svg" 
                      fill="none" 
                      viewBox="0 0 24 24" 
                      stroke-width="1.5" 
                      stroke="currentColor" 
                      class="size-6 shrink-0"
                      :class="route.name === 'evals' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                    >
                      <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 3.104v5.714a2.25 2.25 0 0 1-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 0 1 4.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.393A9.065 9.065 0 0 1 12 15a9.065 9.065 0 0 0-6.23-.693L5 14.5m14.8.8 1.402 1.402c1.232 1.232.65 3.318-1.067 3.611A48.309 48.309 0 0 1 12 21c-2.773 0-5.491-.235-8.135-.687-1.718-.293-2.3-2.379-1.067-3.61L5 14.5" />
                    </svg>
                    Evals
                  </NuxtLink>
                </li>
                <li>
                  <NuxtLink 
                    to="/settings" 
                    class="group flex gap-x-3 rounded-none p-2 text-sm/6 font-semibold text-black border-2 border-black"
                    :class="route.name === 'settings' ? 'dark:text-white dark:border-white bg-neutral-200 dark:bg-neutral-800':'dark:text-neutral-300 dark:border-neutral-300 text-neutral-700'"
                  >
                    <svg 
                      class="size-6 shrink-0" 
                      :class="route.name === 'settings' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                      viewBox="0 0 24 24" 
                      fill="none" 
                      stroke="currentColor" 
                      stroke-linecap="round" 
                      stroke-linejoin="round" 
                      stroke-width="2"
                    >
                      <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                      <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Settings
                  </NuxtLink>
                </li>
              </ul>
            </li>
          </ul>
        </nav>

        <!-- Settings section remains unchanged as it already has dark mode -->
        <div class="mb-4 mt-auto">
          <div class="relative">
            <button 
              @click="showSettingsMenu = !showSettingsMenu"
              class="flex w-full items-center gap-x-3 rounded-none p-2 text-sm/6 font-semibold text-neutral-700 hover:bg-neutral-100 hover:text-black border-2 border-black dark:border-white dark:text-neutral-300 dark:hover:text-neutral-200 dark:hover:bg-neutral-800"
            >
              <svg class="size-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              Settings
            </button>

            <!-- Settings Dropdown Menu -->
            <div 
              v-if="showSettingsMenu"
              class="absolute bottom-full left-0 w-full mb-2 origin-bottom-left bg-white border-2 border-black dark:border-white dark:bg-neutral-900"
            >
              <div class="p-2 space-y-2">
                <div class="p-2 text-sm dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer">
                  Account Settings
                </div>
                <div class="p-2 text-sm dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer">
                  Notifications
                </div>
                
                <button 
                  @click="toggleDarkMode"
                  class="w-full p-2 text-sm dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer flex justify-between items-center focus:outline-none"
                >
                  <span>Dark Mode</span>
                  <span class="text-xs opacity-75">
                    {{ $colorMode.preference === 'system' ? 'System' : $colorMode.value }}
                  </span>
                </button>

                <div 
                  @click.stop="$colorMode.preference = 'system'"
                  class="p-2 text-sm dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 cursor-pointer focus:outline-none"
                >
                  Use System Preference
                </div>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>

    <main class="py-10 pl-72 bg-white dark:bg-neutral-900">
      <div class="px-4 sm:px-6 lg:px-8">
        <slot />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
const showSettingsMenu = ref(false);

const route = useRoute()

const { $colorMode } = useNuxtApp();

const toggleDarkMode = () => {
  showSettingsMenu.value = false
  $colorMode.preference = $colorMode.value === 'dark' ? 'light' : 'dark';
};
</script>
