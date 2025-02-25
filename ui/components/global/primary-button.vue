<template>
  <div>
    <button
      type="button"
      :disabled="disabled"
      :class="[
        'font-mono transition-colors border-2', // 👈 Added 'border' here
        sizeClasses[size],
        typeClasses[type],
        outline ? 'bg-transparent' : 'border-transparent',
        disabled ? 'opacity-50 cursor-not-allowed' : '',
        $attrs.class
      ]"
    >
      <span v-if="$slots.iconLeft" class="inline-flex mr-2">
        <slot name="iconLeft" />
      </span>
      <slot />
      <span v-if="$slots.iconRight" class="inline-flex ml-2">
        <slot name="iconRight" />
      </span>
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps({
  size: {
    type: String as () => 'xs' | 'sm' | 'md' | 'lg',
    default: 'md'
  },
  type: {
    type: String as () => 'default' | 'primary' | 'secondary' | 'success' | 'error',
    default: 'default'
  },
  outline: { type: Boolean, default: true },
  disabled: { type: Boolean, default: false }
})

const sizeClasses = {
  'xs': 'text-xs px-2.5 py-1',
  'sm': 'text-sm px-2.5 py-1',
  'md': 'text-base px-3 py-1.5',
  'lg': 'text-lg px-4 py-2'
};

const typeClasses = {
  'default': 'border-neutral-900 text-neutral-900 hover:bg-neutral-100 dark:border-neutral-300 dark:text-neutral-300 dark:hover:bg-neutral-800',
  'primary': 'border-black bg-black text-neutral-900 dark:text-neutral-900 hover:text-neutral-100 hover:bg-neutral-800 dark:border-neutral-200 dark:bg-neutral-200 dark:text-black dark:hover:bg-neutral-300',
  'secondary': 'border-neutral-300 text-neutral-700 hover:bg-neutral-50 dark:border-neutral-600 dark:text-neutral-300 dark:hover:bg-neutral-800',
  'success': 'border-green-600 text-green-600 hover:bg-green-50 dark:border-green-400 dark:text-green-400 dark:hover:bg-green-900/30',
  'error': 'border-red-600 text-red-600 hover:bg-red-50 dark:border-red-400 dark:text-red-400 dark:hover:bg-red-900/30'
};
</script>
