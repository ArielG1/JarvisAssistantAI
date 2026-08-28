<script setup lang="ts">
import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useHudStore } from "@/stores/hud"
import ParticleBackground from "@/components/hud/ParticleBackground.vue"
import StatusBar from "@/components/hud/StatusBar.vue"
import SidePanel from "@/components/hud/SidePanel.vue"
import DockBar from "@/components/hud/DockBar.vue"
import ChatPanel from "@/components/chat/ChatPanel.vue"
import {
  STATES,
  STATE_ORDER,
  PANEL_CONFIG,
  cerebroStatusLabel,
  type LazyProcessStatus,
} from "@/types/hud"

const store = useHudStore()

const leftPanelOpen = ref(false)
const rightPanelOpen = ref(false)

const cerebroStatus = ref<LazyProcessStatus | null>(null)

async function fetchCerebroStatus() {
  try {
    cerebroStatus.value = await invoke<LazyProcessStatus>("lazy_get_status", {
      name: "cerebro",
    })
  } catch {
    cerebroStatus.value = null
  }
}

const panelMetrics = computed(() =>
  PANEL_CONFIG.map((panel) => {
    if (panel.id !== "cerebro" || !cerebroStatus.value) return panel.metrics
    const s = cerebroStatus.value
    return [
      { key: "status", label: "Estado", value: cerebroStatusLabel(s) },
      { key: "model", label: "Modelo", value: "llama3" },
      {
        key: "latency",
        label: "Inactivo",
        value: s.idle_secs > 0 ? `${s.idle_secs}s` : "—",
      },
    ]
  }),
)

fetchCerebroStatus()

function handleDockAction(id: string) {
  if (id === "chat") {
    leftPanelOpen.value = !leftPanelOpen.value
  } else if (id === "config" || id === "status") {
    rightPanelOpen.value = !rightPanelOpen.value
    if (rightPanelOpen.value) fetchCerebroStatus()
  }
}

function cycleState() {
  const idx = STATE_ORDER.indexOf(store.currentState)
  store.setState(STATE_ORDER[(idx + 1) % STATE_ORDER.length])
}
</script>

<template>
  <div class="relative w-screen h-screen overflow-hidden bg-jarvis-bg">
    <ParticleBackground />

    <StatusBar />

    <SidePanel side="left" :visible="leftPanelOpen">
      <ChatPanel />
    </SidePanel>

    <SidePanel side="right" :visible="rightPanelOpen">
      <div class="space-y-4">
        <h2 class="font-mono text-sm text-jarvis-violet tracking-wider uppercase">Panel Derecho</h2>
        <div v-for="(panel, idx) in PANEL_CONFIG" :key="panel.id" class="space-y-2">
          <div class="flex items-center gap-2">
            <span>{{ panel.icon }}</span>
            <span class="font-mono text-xs text-jarvis-muted">{{ panel.label }}</span>
          </div>
          <div v-if="panelMetrics[idx]" class="pl-6 space-y-1">
            <div v-for="metric in panelMetrics[idx]" :key="metric.key" class="flex justify-between">
              <span class="font-mono text-xs text-jarvis-muted">{{ metric.label }}</span>
              <span class="font-mono text-xs text-jarvis-cyan">{{ metric.value }}</span>
            </div>
          </div>
        </div>
      </div>
    </SidePanel>

    <main
      class="relative z-10 flex items-center justify-center h-full pt-12 pb-16 transition-all duration-300"
      :class="{
        'pl-64': leftPanelOpen,
        'pr-64': rightPanelOpen,
      }"
    >
      <div
        class="text-center cursor-pointer select-none"
        @click="cycleState"
      >
        <div
          class="w-32 h-32 mx-auto mb-6 rounded-full border-2 flex items-center justify-center
                 transition-all duration-500 hud-glow"
          :class="`status-${store.currentState}`"
        >
          <span class="text-4xl hud-pulse">
            {{ STATES[store.currentState].icon }}
          </span>
        </div>
        <p class="font-mono text-sm text-jarvis-muted">
          {{ STATES[store.currentState].description }}
        </p>
      </div>
    </main>

    <DockBar @action="handleDockAction" />
  </div>
</template>
