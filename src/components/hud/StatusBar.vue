<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import { useHudStore } from "@/stores/hud"
import { STATUS_COLORS, STATUS_LABELS } from "@/types/status"

const store = useHudStore()
const time = ref("")
let timer = 0

function updateTime() {
  const now = new Date()
  time.value = now.toLocaleTimeString("es-ES", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  })
}

onMounted(() => {
  updateTime()
  timer = window.setInterval(updateTime, 1000)
})

onUnmounted(() => {
  clearInterval(timer)
})
</script>

<template>
  <header
    class="fixed top-0 left-0 right-0 z-30 h-12 flex items-center justify-between px-6
           bg-jarvis-panel/70 backdrop-blur-md border-b border-jarvis-border/50"
  >
    <span class="font-mono text-sm tracking-[0.3em] text-jarvis-cyan font-semibold">
      J.A.R.V.I.S.
    </span>

    <span
      class="font-mono text-sm font-medium transition-colors duration-300"
      :style="{ color: STATUS_COLORS[store.currentState] }"
    >
      {{ STATUS_LABELS[store.currentState] }}
    </span>

    <span class="font-mono text-sm text-jarvis-muted tabular-nums">
      {{ time }}
    </span>
  </header>
</template>
