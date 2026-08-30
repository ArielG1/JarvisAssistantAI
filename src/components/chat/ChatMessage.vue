<script setup lang="ts">
import type { ChatMessage } from "@/types/message"

defineProps<{
  message: ChatMessage
  isLast?: boolean
}>()
</script>

<template>
  <div
    class="flex gap-2 hud-fade-in"
    :class="message.role === 'user' ? 'justify-end' : 'justify-start'"
  >
    <div v-if="message.role === 'jarvis'" class="flex-shrink-0 w-7 h-7 rounded-full bg-jarvis-panel border border-jarvis-border/50 flex items-center justify-center text-xs">
      🤖
    </div>
    <div v-else-if="message.role === 'system'" class="flex-shrink-0 w-7 h-7 rounded-full bg-amber-500/20 border border-amber-500/30 flex items-center justify-center text-xs">
      ⚠
    </div>

    <div
      class="max-w-[75%] rounded-lg px-3 py-2 font-mono text-sm"
      :class="[
        message.role === 'user'
          ? 'bg-jarvis-cyan/10 border border-jarvis-cyan/30 text-jarvis-text'
          : message.role === 'system'
            ? 'bg-amber-500/10 border border-amber-500/30 text-amber-400'
            : 'bg-jarvis-panel/80 border border-jarvis-border/50 text-jarvis-text',
      ]"
    >
      <p class="whitespace-pre-wrap break-words">{{ message.content }}</p>
      <div
        v-if="message.role === 'jarvis' && message.source?.type === 'web' && message.source.domain"
        class="mt-1.5 pt-1.5 border-t border-jarvis-border/30"
      >
        <span class="text-[10px] text-jarvis-muted">
          Fuente:
          <a
            v-if="message.source.url"
            :href="message.source.url"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-jarvis-cyan transition-colors"
          >{{ message.source.domain }}</a>
          <span v-else>{{ message.source.domain }}</span>
        </span>
      </div>
      <span class="block mt-1 text-[10px]" :class="message.role === 'system' ? 'text-amber-500/60' : 'text-jarvis-muted'">
        {{ message.timestamp.toLocaleTimeString('es-ES', { hour: '2-digit', minute: '2-digit' }) }}
      </span>
    </div>

    <div v-if="message.role === 'user'" class="flex-shrink-0 w-7 h-7 rounded-full bg-jarvis-cyan/20 border border-jarvis-cyan/30 flex items-center justify-center text-xs">
      👤
    </div>
  </div>
</template>
