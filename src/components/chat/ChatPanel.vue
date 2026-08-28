<script setup lang="ts">
import { ref, watch, nextTick } from "vue"
import { useChatStore } from "@/stores/chat"
import ChatMessage from "./ChatMessage.vue"
import ChatInput from "./ChatInput.vue"

const store = useChatStore()
const scrollContainer = ref<HTMLDivElement>()

function scrollToBottom() {
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

watch(
  () => store.messages.length,
  () => scrollToBottom(),
)

function handleSend(content: string) {
  store.sendMessage(content)
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="px-4 py-3 border-b border-jarvis-border/50">
      <h2 class="font-mono text-sm text-jarvis-cyan tracking-wider uppercase">Chat</h2>
    </div>

    <div
      ref="scrollContainer"
      class="flex-1 overflow-y-auto px-4 py-3 space-y-3"
    >
      <template v-if="store.messages.length">
        <ChatMessage
          v-for="(msg, i) in store.messages"
          :key="msg.id"
          :message="msg"
          :is-last="i === store.messages.length - 1"
        />
      </template>
      <div v-else class="flex items-center justify-center h-full">
        <p class="font-mono text-sm text-jarvis-muted">Escribe un mensaje para comenzar...</p>
      </div>

      <div v-if="store.isTyping" class="flex gap-2 hud-fade-in">
        <div class="flex-shrink-0 w-7 h-7 rounded-full bg-jarvis-panel border border-jarvis-border/50 flex items-center justify-center text-xs">
          🤖
        </div>
        <div class="bg-jarvis-panel/80 border border-jarvis-border/50 rounded-lg px-3 py-2 font-mono text-sm">
          <span v-if="store.typingMessage" class="whitespace-pre-wrap">{{ store.typingMessage }}</span>
          <span v-else class="animate-pulse text-jarvis-muted">⋯</span>
        </div>
      </div>
    </div>

    <ChatInput :disabled="store.isTyping" :loading="store.isTyping" @send="handleSend" />
  </div>
</template>
