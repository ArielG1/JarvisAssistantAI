<script setup lang="ts">
import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useHudStore } from "@/stores/hud"
import ParticleBrain from "@/components/hud/ParticleBrain.vue"
import StatusBar from "@/components/hud/StatusBar.vue"
import FloatingCard from "@/components/hud/FloatingCard.vue"
import ChatPanel from "@/components/chat/ChatPanel.vue"
import {
  PANEL_CONFIG,
  cerebroStatusLabel,
  type LazyProcessStatus,
} from "@/types/hud"

const hud = useHudStore()
const cerebroStatus = ref<LazyProcessStatus | null>(null)
const llmModel = ref("...")

async function fetchCerebroStatus() {
  try {
    cerebroStatus.value = await invoke<LazyProcessStatus>("lazy_get_status", {
      name: "cerebro",
    })
  } catch {
    cerebroStatus.value = null
  }
}

async function fetchLlmModel() {
  try {
    llmModel.value = await invoke<string>("get_llm_model")
  } catch {
    llmModel.value = "..."
  }
}

const panelMetrics = computed(() =>
  PANEL_CONFIG.map((panel) => {
    if (panel.id !== "cerebro" || !cerebroStatus.value) return panel.metrics
    const s = cerebroStatus.value
    return [
      { key: "status", label: "Estado", value: cerebroStatusLabel(s) },
      { key: "model", label: "Modelo", value: llmModel.value },
      {
        key: "latency",
        label: "Inactivo",
        value: s.idle_secs > 0 ? `${s.idle_secs}s` : "—",
      },
    ]
  }),
)

fetchCerebroStatus()
setInterval(fetchCerebroStatus, 5000)
fetchLlmModel()
setInterval(fetchLlmModel, 60000)
</script>

<template>
  <div class="relative w-screen h-screen overflow-hidden bg-jarvis-bg">
    <ParticleBrain />

    <StatusBar />

    <div class="fixed top-16 right-8 z-20 flex flex-col gap-4 w-56">
      <FloatingCard
        v-for="(panel, idx) in PANEL_CONFIG"
        :key="panel.id"
        :id="panel.id"
        :title="panel.label"
        :hidden="hud.isHidden(panel.id)"
        @hide="hud.hideCard"
      >
        <div v-if="panelMetrics[idx]" class="space-y-1">
          <div
            v-for="metric in panelMetrics[idx]"
            :key="metric.key"
            class="flex justify-between"
          >
            <span class="font-mono text-xs text-jarvis-muted">{{ metric.label }}</span>
            <span class="font-mono text-xs text-jarvis-cyan truncate max-w-[120px]" :title="metric.value">{{ metric.value }}</span>
          </div>
        </div>
      </FloatingCard>
    </div>

    <div class="fixed bottom-24 right-8 z-20 flex flex-col gap-2">
      <div
        v-for="panel in PANEL_CONFIG.filter((p) => hud.isHidden(p.id))"
        :key="panel.id"
        class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-jarvis-panel/60
               border border-jarvis-border text-[10px] font-mono tracking-wide
               text-jarvis-cyan cursor-pointer hover:border-jarvis-cyan"
        @click="hud.showCard(panel.id)"
      >
        {{ panel.label }}
      </div>
    </div>

    <div class="fixed bottom-10 left-1/2 -translate-x-1/2 w-[min(90vw,640px)] z-20 h-[45vh] max-h-[420px]">
      <ChatPanel />
    </div>
  </div>
</template>
