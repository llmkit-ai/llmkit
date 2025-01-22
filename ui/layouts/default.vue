<template>
  <div>
    <div class="fixed inset-y-0 z-50 flex w-72 flex-col">
      <div class="flex grow flex-col gap-y-5 overflow-y-auto border-r-2 border-black bg-white px-6">
        <div class="flex h-16 shrink-0 items-center space-x-2">
          <svg class="size-8" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
            <path d="M20 4v5l-9 7l-4 4l-3 -3l4 -4l7 -9z"></path>
            <path d="M6.5 11.5l6 6"></path>
          </svg>
          <p class="font-mono font-bold">llmkit</p>
        </div>
        <nav class="flex flex-1 flex-col">
          <ul role="list" class="flex flex-1 flex-col gap-y-7">
            <li>
              <ul role="list" class="-mx-2 space-y-4">
                <li>
                  <NuxtLink href="/prompts" class="group flex gap-x-3 rounded-none bg-gray-100 p-2 text-sm/6 font-semibold text-black border-2 border-black">
                    <svg class="size-6 shrink-0 text-black" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
                      <path d="M5 7l5 5l-5 5"></path>
                      <path d="M13 17l6 0"></path>
                    </svg>
                    Prompts
                  </NuxtLink>
                </li>
                <li>
                  <NuxtLink href="/traces" class="group flex gap-x-3 rounded-none p-2 text-sm/6 font-semibold text-gray-700 hover:bg-gray-100 hover:text-black border-2 border-black">
                    <svg class="size-6 shrink-0 text-gray-700 group-hover:text-black" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
                      <path d="M4 6c0 1.657 3.582 3 8 3s8 -1.343 8 -3s-3.582 -3 -8 -3s-8 1.343 -8 3"></path>
                      <path d="M4 6v6c0 1.657 3.582 3 8 3m8 -3.5v-5.5"></path>
                      <path d="M4 12v6c0 1.657 3.582 3 8 3"></path>
                      <path d="M18 18m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0"></path>
                      <path d="M20.2 20.2l1.8 1.8"></path>
                    </svg>
                    Traces
                  </NuxtLink>
                </li>
              </ul>
            </li>
          </ul>
        </nav>
      </div>
    </div>

    <main class="py-10 pl-72">
      <div class="px-4 sm:px-6 lg:px-8">
        <slot />
      </div>
    </main>
  </div>
</template>
