<template>
  <div>
    <div class="px-4 sm:px-0">
      <h3 class="text-base/7 font-semibold text-neutral-900 dark:text-white">Chat Test</h3>
      <p class="mt-1 max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">Test your chat prompt with an interactive chat interface.</p>
    </div>
    
    <!-- System Prompt Info -->
    <div class="mt-3">
      <dl class="grid grid-cols-1">
        <div class="border-t border-neutral-100 dark:border-neutral-700 px-4 py-4 sm:px-0">
          <dt class="text-sm/6 font-medium text-neutral-900 dark:text-white">System Prompt</dt>
          <dd class="text-sm/6 text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap bg-neutral-100 dark:bg-neutral-800 p-2">{{ prompt.system }}</dd>
        </div>
      </dl>
    </div>
    
    <!-- Dynamic Fields -->
    <div v-if="templateFields.length > 0" class="mt-4">
      <div class="px-4 sm:px-0">
        <h3 class="text-base/7 font-semibold text-neutral-700 dark:text-white">Template Variables</h3>
        <p class="max-w-2xl text-sm/6 text-neutral-500 dark:text-neutral-400">These variables will be used in the system prompt.</p>
      </div>
      <div class="mt-4 grid grid-cols-4 gap-x-2">
        <div v-for="f in templateFields" :key="f">
          <label :for="f" class="block text-sm/6 font-medium text-neutral-900 dark:text-white">{{ f }}</label>
          <div class="mt-0.5">
            <input 
              v-on:input="templateFieldInput" 
              type="text" 
              :name="f" 
              :id="f" 
              class="block w-full bg-white dark:bg-neutral-800 px-3 py-1.5 text-base text-neutral-900 dark:text-white outline outline-1 -outline-offset-1 outline-neutral-300 dark:outline-neutral-600 placeholder:text-neutral-400 dark:placeholder:text-neutral-500 focus:outline focus:outline-2 focus:-outline-offset-2 focus:outline-black dark:focus:outline-white sm:text-sm/6"
            >
          </div>
        </div>
      </div>
    </div>
    
    <!-- Chat Interface -->
    <div class="mt-6 border border-neutral-200 dark:border-neutral-700 rounded-lg overflow-hidden">
      <!-- Chat Messages -->
      <div class="h-96 overflow-y-auto p-4 bg-white dark:bg-neutral-900" ref="chatContainer">
        <div v-for="(message, index) in chatMessages" :key="index" class="mb-4">
          <div :class="[
            'p-3 rounded-lg max-w-3/4 whitespace-pre-wrap', 
            message.role === 'user' 
              ? 'bg-neutral-100 dark:bg-neutral-800 ml-auto' 
              : 'bg-neutral-200 dark:bg-neutral-700'
          ]">
            <div class="text-xs text-neutral-500 dark:text-neutral-400 mb-1">
              {{ message.role === 'user' ? 'You' : 'Assistant' }}
            </div>
            <div class="text-sm text-neutral-900 dark:text-neutral-200">
              {{ message.content }}
            </div>
          </div>
        </div>
        <div v-if="isStreaming" class="p-3 rounded-lg max-w-3/4 whitespace-pre-wrap bg-neutral-200 dark:bg-neutral-700">
          <div class="text-xs text-neutral-500 dark:text-neutral-400 mb-1">
            Assistant
          </div>
          <div class="text-sm text-neutral-900 dark:text-neutral-200">
            {{ streamingResponse }}
          </div>
        </div>
      </div>
      
      <!-- Input Area -->
      <div class="border-t border-neutral-200 dark:border-neutral-700 p-3 bg-white dark:bg-neutral-800 flex">
        <textarea 
          v-model="userInput"
          @keydown.enter.prevent="sendMessage"
          class="flex-grow resize-none outline-none bg-transparent p-2 text-neutral-900 dark:text-white placeholder:text-neutral-400"
          placeholder="Type your message..."
          rows="2"
        ></textarea>
        <PrimaryButton 
          @click="sendMessage" 
          type="primary"
          size="md"
          :disabled="isStreaming || userInput.trim() === ''"
        >
          Send
        </PrimaryButton>
      </div>
    </div>
    
    <!-- Debug Info -->
    <div v-if="Object.keys(jsonContext).length > 0" class="mt-5 bg-neutral-100 dark:bg-neutral-800 p-4">
      <div class="flex items-center justify-between">
        <p class="text-xs text-neutral-900 dark:text-neutral-300">Context</p>
        <button
          @click="showJsonContext = !showJsonContext"
          class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-300"
        >
          {{ showJsonContext ? 'Hide' : 'Show' }}
        </button>
      </div>
      <div v-if="showJsonContext" class="mt-3 dark:text-neutral-300 text-sm">
        <pre>{{ JSON.stringify(jsonContext, null, 2) }}</pre>
      </div>
    </div>
    
    <!-- Action Buttons -->
    <div class="mt-6 flex justify-end px-4 sm:px-0 space-x-2">
      <PrimaryButton
        type="secondary"
        size="sm"
        @click="resetChat"
      >
        Reset Chat
      </PrimaryButton>
      <PrimaryButton
        type="secondary"
        size="sm"
        @click="$emit('handle-cancel')"
      >
        Cancel
      </PrimaryButton>
      <PrimaryButton
        type="secondary"
        size="sm"
        @click="$emit('handle-edit')"
      >
        Edit
      </PrimaryButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted } from 'vue';
import type { Prompt } from '~/types/response/prompts';
import type { Message } from '~/types/response/prompts';
import { SSE } from 'sse.js';

const props = defineProps<{
  prompt: Prompt
}>();

const emit = defineEmits(["handle-edit", "handle-cancel"]);

const chatContainer = ref<HTMLElement | null>(null);
const userInput = ref('');
const chatMessages = ref<Message[]>([]);
const streamingResponse = ref('');
const isStreaming = ref(false);
const jsonContext = ref<Record<string, any>>({});
const showJsonContext = ref(false);

// Check if prompt is chat-enabled
onMounted(() => {
  if (!props.prompt.is_chat) {
    console.warn('This prompt is not configured for chat mode');
  }
});

// Extract template variables from system prompt
const templateFields = computed<string[]>(() => {
  if (!props.prompt || !props.prompt.system) return [];

  const template = props.prompt.system;
  const uniqueFields = new Set<string>();

  // Regex to find variables in {{ ... }} (Handlebars style)
  const handlebarsRegex = /\{\{\s*(\w+)\s*\}\}/g;
  let match;
  while ((match = handlebarsRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  // Regex to find variables in {% if variable ... %} conditions
  const ifConditionRegex = /\{\%\s*if\s*(\w+)(?:\s+.*?)\s*\%\}/g;
  while ((match = ifConditionRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  // Regex for {% elif variable ... %}
  const elifConditionRegex = /\{\%\s*elif\s*(\w+)(?:\s+.*?)\s*\%\}/g;
  while ((match = elifConditionRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  // Regex for {% for variable in ... %} (extracting the iterable variable)
  const forLoopRegex = /\{\%\s*for\s+\w+\s+in\s+(\w+)\s*\%\}/g;
  while ((match = forLoopRegex.exec(template)) !== null) {
    if (match[1]) {
      uniqueFields.add(match[1]);
    }
  }

  return Array.from(uniqueFields);
});

function templateFieldInput(event: any) {
  const key = event.target.id;
  const value = event.target.value;
  jsonContext.value[key] = value;
}

function scrollToBottom() {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
    }
  });
}

async function sendMessage() {
  if (userInput.value.trim() === '' || isStreaming.value) {
    return;
  }

  // Add user message to chat
  const userMessage: Message = {
    role: 'user',
    content: userInput.value
  };
  chatMessages.value.push(userMessage);
  
  // Clear input
  const inputText = userInput.value;
  userInput.value = '';
  scrollToBottom();
  
  // Start streaming response
  isStreaming.value = true;
  streamingResponse.value = '';
  
  try {
    // Create SSE connection for streaming
    const source = new SSE(`/api/v1/prompts/execute/${props.prompt.id}/chat/stream`, {
      headers: { 'Content-Type': 'application/json' },
      payload: JSON.stringify({
        context: jsonContext.value,
        messages: chatMessages.value
      })
    });
    
    source.addEventListener('message', function(e: any) {
      const data = e.data;
      
      // Check if this is the log ID message (last message)
      if (data.includes('log_id')) {
        isStreaming.value = false;
        
        // Add assistant message to chat history
        if (streamingResponse.value) {
          chatMessages.value.push({
            role: 'assistant',
            content: streamingResponse.value
          });
        }
        source.close();
        return;
      }
      
      // Append to streaming response
      streamingResponse.value += data;
      scrollToBottom();
    });
    
    source.addEventListener('error', function(e: any) {
      console.error('Error in SSE connection:', e);
      isStreaming.value = false;
      source.close();
    });
    
    source.stream();
  } catch (error) {
    console.error('Failed to send message:', error);
    isStreaming.value = false;
  }
}

function resetChat() {
  chatMessages.value = [];
  streamingResponse.value = '';
  isStreaming.value = false;
}
</script>

<style scoped>
/* Style for the chat interface */
</style>
