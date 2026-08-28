<script setup lang="ts">
import { ref, watch } from "vue"

const props = defineProps<{
  disabled?: boolean
  loading?: boolean
}>()

const emit = defineEmits<{
  (e: "send", content: string): void
}>()

const input = ref("")
const textarea = ref<HTMLTextAreaElement>()

function handleSend() {
  const trimmed = input.value.trim()
  if (!trimmed || props.disabled || props.loading) return
  emit("send", trimmed)
  input.value = ""
  resize()
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

function resize() {
  const el = textarea.value
  if (!el) return
  el.style.height = "auto"
  el.style.height = Math.min(el.scrollHeight, 120) + "px"
}

watch(input, resize)
</script>

<template>
  <div class="border-t border-jarvis-border/50 bg-jarvis-panel/60 backdrop-blur-md p-3">
    <div class="flex items-end gap-2">
      <textarea
        ref="textarea"
        v-model="input"
        rows="1"
        placeholder="Escribe tu mensaje..."
        :disabled="disabled || loading"
        class="flex-1 resize-none bg-transparent border border-jarvis-border/50 rounded-lg px-3 py-2
               font-mono text-sm text-jarvis-text placeholder-jarvis-muted
               focus:outline-none focus:border-jarvis-cyan/60 focus:shadow-[0_0_8px_rgba(0,212,255,0.2)]
               disabled:opacity-50 disabled:cursor-not-allowed transition-all"
        @keydown="handleKeydown"
      />
      <button
        :disabled="!input.trim() || disabled || loading"
        class="flex-shrink-0 w-9 h-9 rounded-lg flex items-center justify-center
               bg-jarvis-cyan/20 border border-jarvis-cyan/30 text-jarvis-cyan
               hover:bg-jarvis-cyan/30 transition-colors
               disabled:opacity-40 disabled:cursor-not-allowed"
        @click="handleSend"
      >
        <span v-if="loading" class="animate-pulse">⋯</span>
        <span v-else>→</span>
      </button>
    </div>
  </div>
</template>
