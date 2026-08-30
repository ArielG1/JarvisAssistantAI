<script setup lang="ts">
const props = defineProps<{ id: string; title: string; hidden: boolean }>()
defineEmits<{ (e: "hide", id: string): void }>()
</script>

<template>
  <div
    class="card w-56 rounded-2xl border border-jarvis-cyan/25 bg-jarvis-panel/40
           backdrop-blur-md shadow-[0_8px_32px_rgba(0,0,0,0.4)] px-4 py-3 transition-all duration-300"
    :class="props.hidden ? 'opacity-0 max-h-0 py-0 pointer-events-none overflow-hidden' : 'max-h-56'"
  >
    <div class="flex items-center justify-between mb-2">
      <h3 class="font-mono text-[10px] tracking-[0.2em] text-jarvis-text/85">{{ props.title }}</h3>
      <button class="text-jarvis-muted hover:text-jarvis-cyan text-xs" @click="$emit('hide', props.id)">
        ✕
      </button>
    </div>
    <slot />
  </div>
</template>

<style scoped>
.card {
  animation: floaty 8s ease-in-out infinite;
}
@keyframes floaty {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}
</style>
