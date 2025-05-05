<template>
  <div>
    <button
      :type="htmlType"
      :disabled="disabled"
      :class="[
        'font-mono transition-colors border-2',
        sizeClasses[size],
        typeClasses[buttonType],
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
  buttonType: {
    type: String as () => 'default' | 'primary' | 'primary-inverse' | 'secondary' | 'success' | 'error' | 'danger',
    default: 'default'
  },
  htmlType: {
    type: String as () => 'button' | 'submit' | 'reset',
    default: 'button'
  },
  disabled: { type: Boolean, default: false }
})

const sizeClasses = {
  'xs': 'text-xs px-2.5 py-1',
  'sm': 'text-sm px-2.5 py-1',
  'md': 'text-base px-3 py-1.5',
  'lg': 'text-lg px-4 py-2'
};

const typeClasses = {
  'default': 'border-neutral-900 text-neutral-900 bg-transparent hover:bg-neutral-100 dark:border-neutral-300 dark:text-neutral-300 dark:hover:bg-neutral-800',
  'primary': 'border-black bg-black text-white hover:bg-neutral-800 dark:border-neutral-200 dark:bg-neutral-200 dark:text-black dark:hover:bg-neutral-300',
  'primary-inverse': 'border-black bg-transparent text-black hover:bg-neutral-100 dark:border-white dark:text-white dark:hover:bg-neutral-800',
  'secondary': 'border-neutral-300 bg-transparent text-neutral-700 hover:bg-neutral-50 dark:border-neutral-600 dark:text-neutral-300 dark:hover:bg-neutral-800',
  'success': 'border-green-600 bg-transparent text-green-600 hover:bg-green-50 dark:border-green-400 dark:text-green-400 dark:hover:bg-green-900/30',
  'error': 'border-red-600 bg-transparent text-red-600 hover:bg-red-50 dark:border-red-400 dark:text-red-400 dark:hover:bg-red-900/30',
  'danger': 'border-red-600 bg-red-600 text-white hover:bg-red-700 dark:border-red-500 dark:bg-red-500 dark:hover:bg-red-600'
};
</script>
