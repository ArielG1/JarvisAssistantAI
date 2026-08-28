<script setup lang="ts">
defineEmits<{
  (e: "action", id: string): void
}>()

const items = [
  { id: "chat", icon: "💬", label: "Chat" },
  { id: "config", icon: "⚙", label: "Config" },
  { id: "status", icon: "📊", label: "Status" },
]
</script>

<template>
  <nav
    class="fixed bottom-0 left-0 right-0 z-30 h-14 flex items-center justify-center gap-2
           bg-jarvis-panel/70 backdrop-blur-md border-t border-jarvis-border/50"
  >
    <button
      v-for="item in items"
      :key="item.id"
      :title="item.label"
      class="group relative flex items-center justify-center w-10 h-10 rounded-lg
             transition-all duration-200 hover:bg-jarvis-cyan/10
             hover:shadow-[0_0_12px_rgba(0,212,255,0.3)]"
      @click="$emit('action', item.id)"
    >
      <span class="text-xl">{{ item.icon }}</span>
      <span
        class="absolute -top-8 left-1/2 -translate-x-1/2 px-2 py-1 text-xs font-mono
               text-jarvis-cyan bg-jarvis-panel border border-jarvis-border rounded
               opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap"
      >
        {{ item.label }}
      </span>
    </button>
  </nav>
</template>
