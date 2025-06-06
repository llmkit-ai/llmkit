<template>
  <div>
    <div>
      <!-- Static sidebar for desktop -->
      <div :class="isOutside ? 'w-14':'w-64'" class="fixed inset-y-0 z-30 flex flex-col border-r border-neutral-300 dark:border-neutral-700" style="transition: width 0.2s ease;">
        <div class="flex grow flex-col gap-y-5 overflow-hidden bg-white px-2 dark:bg-neutral-900">
          <div class="pt-4 shrink-0 flex items-center">
            <svg class="size-8 text-black dark:text-white" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
              <path d="M20 4v5l-9 7l-4 4l-3 -3l4 -4l7 -9z"></path>
              <path d="M6.5 11.5l6 6"></path>
            </svg>
          </div>

          <nav ref="sideNav" class="flex flex-1 flex-col">
            <ul role="list" class="flex flex-1 flex-col gap-y-7 overflow-hidden">
              <li>
                <ul role="list" class="px-2 space-y-4">
                  <li>
                    <NuxtLink 
                      to="/prompts" 
                      :class="[
                        'group flex gap-x-3 rounded-none -ml-2 p-2 text-sm/6 font-semibold',
                        route.name === 'prompts' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
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
                      <span :class="isOutside ? 'opacity-0':''">Prompts</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink 
                      to="/evals" 
                      :class="[
                        'group flex gap-x-3 rounded-none pt-2 pr-2 pb-2 text-sm/6 font-semibold',
                        route.name === 'evals' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
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
                      <span :class="isOutside ? 'opacity-0':''">Evals</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink 
                      to="/traces" 
                      :class="[
                        'group flex gap-x-3 rounded-none pt-2 pr-2 pb-2 text-sm/6 font-semibold',
                        route.name === 'traces' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
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
                      <span :class="isOutside ? 'opacity-0':''">Traces</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink 
                      to="/models" 
                      :class="[
                        'group flex gap-x-3 rounded-none pt-2 pr-2 pb-2 text-sm/6 font-semibold',
                        route.name === 'models' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
                    >
                      <svg 
                        class="size-6 shrink-0"
                        :class="route.name === 'models' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                        xmlns="http://www.w3.org/2000/svg" 
                        fill="none" 
                        viewBox="0 0 24 24" 
                        stroke-width="1.5" 
                        stroke="currentColor" 
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" d="M11.42 15.17 17.25 21A2.652 2.652 0 0 0 21 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 1 1-3.586-3.586l6.837-5.63m5.108-.233c.55-.164 1.163-.188 1.743-.14a4.5 4.5 0 0 0 4.486-6.336l-3.276 3.277a3.004 3.004 0 0 1-2.25-2.25l3.276-3.276a4.5 4.5 0 0 0-6.336 4.486c.091 1.076-.071 2.264-.904 2.95l-.102.085m-1.745 1.437L5.909 7.5H4.5L2.25 3.75l1.5-1.5L7.5 4.5v1.409l4.26 4.26m-1.745 1.437 1.745-1.437m6.615 8.206L15.75 15.75M4.867 19.125h.008v.008h-.008v-.008Z" />
                      </svg>
                      <span :class="isOutside ? 'opacity-0':''">Models</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink 
                      to="/providers" 
                      :class="[
                        'group flex gap-x-3 rounded-none pt-2 pr-2 pb-2 text-sm/6 font-semibold',
                        route.name === 'providers' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
                    >
                      <svg 
                        class="size-6 shrink-0"
                        :class="route.name === 'providers' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                        xmlns="http://www.w3.org/2000/svg" 
                        fill="none" 
                        viewBox="0 0 24 24" 
                        stroke-width="1.5" 
                        stroke="currentColor" 
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 0 1-3-3m3 3a3 3 0 1 0 0 6h13.5a3 3 0 1 0 0-6m-16.5-3a3 3 0 0 1 3-3h13.5a3 3 0 0 1 3 3m-19.5 0a4.5 4.5 0 0 1 .9-2.7L5.737 5.1a3.375 3.375 0 0 1 2.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 0 1 .9 2.7m0 0a3 3 0 0 1-3 3m0 3h.008v.008h-.008v-.008Zm0-6h.008v.008h-.008v-.008Zm-3 6h.008v.008h-.008v-.008Zm0-6h.008v.008h-.008v-.008Z" />
                      </svg>
                      <span :class="isOutside ? 'opacity-0':''">Providers</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink 
                      to="/tools" 
                      :class="[
                        'group flex gap-x-3 rounded-none pt-2 pr-2 pb-2 text-sm/6 font-semibold',
                        route.name === 'tools' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
                    >
                      <svg 
                        class="size-6 shrink-0"
                        :class="route.name === 'tools' ? 'text-black dark:text-white':'text-neutral-700 group-hover:text-black dark:text-neutral-300 dark:group-hover:text-white'"
                        xmlns="http://www.w3.org/2000/svg" 
                        fill="none" 
                        viewBox="0 0 24 24" 
                        stroke-width="1.5" 
                        stroke="currentColor" 
                      >
                        <path stroke-linecap="round" stroke-linejoin="round" d="M21.75 6.75a4.5 4.5 0 0 1-4.884 4.484c-1.076-.091-2.264.071-2.95.904l-7.152 8.684a2.548 2.548 0 1 1-3.586-3.586l8.684-7.152c.833-.686.995-1.874.904-2.95a4.5 4.5 0 0 1 6.336-4.486l-3.276 3.276a3.004 3.004 0 0 0 2.25 2.25l3.276-3.276c.256.565.398 1.192.398 1.852Z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4.867 19.125h.008v.008h-.008v-.008Z" />
                      </svg>
                      <span :class="isOutside ? 'opacity-0':''">Tools</span>
                    </NuxtLink>
                  </li>
                  <li>
                    <NuxtLink 
                      to="/settings" 
                      :class="[
                        'group flex gap-x-3 rounded-none pt-2 pr-2 pb-2 text-sm/6 font-semibold',
                        route.name === 'settings' ? 'text-black dark:text-white bg-neutral-200 dark:bg-neutral-800' : 'text-neutral-700 dark:text-neutral-300'
                      ]"
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
                      <span :class="isOutside ? 'opacity-0':''">Settings</span>
                    </NuxtLink>
                  </li>
                </ul>
              </li>
              <li class="mt-auto flex flex-col items-center pb-4">
                <div v-if="user" :class="isOutside ? 'flex flex-col items-center' : 'px-2 w-full'">
                  <div v-if="!isOutside" class="flex items-center overflow-hidden whitespace-nowrap mb-1">
                    <span class="font-medium text-black dark:text-white text-sm">{{ user.name }}</span>
                  </div>
                  <div v-if="isOutside" class="flex items-center overflow-hidden whitespace-nowrap mb-1">
                    <svg
                      class="size-5 text-neutral-700 dark:text-neutral-300"
                      xmlns="http://www.w3.org/2000/svg"
                      fill="currentColor"
                      viewBox="0 0 256 256"
                    >
                      <path
                        d="M216,40H40A16,16,0,0,0,24,56V200a16,16,0,0,0,16,16H216a16,16,0,0,0,16-16V56A16,16,0,0,0,216,40ZM40,56H80V200H40ZM216,200H96V56H216V200Z"
                      ></path>
                    </svg>
                  </div>
                  
                  <div v-if="!isOutside" class="flex items-center justify-between w-full mt-1">
                    <button 
                      @click="handleLogout"  
                      class="text-red-600 dark:text-red-400 text-xs hover:text-red-800 dark:hover:text-red-300"
                    >
                      Logout
                    </button>
                    <button 
                      @click="toggleDarkMode"
                      class="text-neutral-600 dark:text-neutral-400 text-xs hover:text-neutral-800 dark:hover:text-neutral-300"
                    >
                      {{ $colorMode.value === 'dark' ? 'Light' : 'Dark' }}
                    </button>
                  </div>
                </div>
              </li>
            </ul>
          </nav>
        </div>
      </div>

      <main class="py-10 bg-white dark:bg-neutral-900">
        <div class="px-4 sm:px-6 lg:px-8">
          <slot />
        </div>
      </main>

    </div>
  </div>
</template>

<script setup lang="ts">
import { useMouseInElement, onClickOutside } from '@vueuse/core';

const showOptionsMenu = ref(false);
const showMobileNav = ref(false);
const showMobileNavInner = ref(false);

const route = useRoute();
const sideNav = ref(null);

const { x, y, isOutside } = useMouseInElement(sideNav);

const { $colorMode } = useNuxtApp();
const { user, loading, error, isLoggedIn, logout } = useUser();

const flyout = ref(null);
onClickOutside(flyout, (_event) => { closeNav() });

function openNav() {
  showMobileNav.value = true;
  setTimeout(() => {
    showMobileNavInner.value = true;
  }, 1);
}

function closeNav() {
  showMobileNavInner.value = false;
  setTimeout(() => {
    showMobileNav.value = false;
  }, 500);
}

const toggleDarkMode = () => {
  showOptionsMenu.value = false;
  $colorMode.preference = $colorMode.value === 'dark' ? 'light' : 'dark';
};

const handleLogout = async () => {
  showOptionsMenu.value = false;
  await logout();
};
</script>
