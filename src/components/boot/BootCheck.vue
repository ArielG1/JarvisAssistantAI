<script setup lang="ts">
import type { BootStep, BootStepStatus } from "../../types/boot";

defineProps<{
  step: BootStep;
  retryable?: boolean;
}>();

const emit = defineEmits<{
  retry: [];
}>();

function statusIcon(status: BootStepStatus): string {
  switch (status) {
    case "ok":
      return "✓";
    case "error":
      return "✗";
    case "running":
      return "●●●";
    default:
      return "○";
  }
}

function statusColor(status: BootStepStatus): string {
  switch (status) {
    case "ok":
      return "text-jarvis-green";
    case "error":
      return "text-jarvis-red";
    case "running":
      return "text-jarvis-amber";
    default:
      return "text-jarvis-muted";
  }
}

function statusBg(status: BootStepStatus): string {
  switch (status) {
    case "error":
      return "bg-jarvis-red/10 border-jarvis-red/30";
    default:
      return "bg-jarvis-panel border-jarvis-border";
  }
}

function dots(label: string): string {
  const maxLen = 20;
  const remaining = Math.max(0, maxLen - label.length);
  return ".".repeat(remaining);
}
</script>

<template>
  <div
    class="flex items-center gap-3 py-2 px-4 rounded-lg border font-mono text-sm"
    :class="statusBg(step.status)"
  >
    <span
      :class="[
        statusColor(step.status),
        { 'animate-spin': step.status === 'running' },
      ]"
      class="w-8 text-center flex-shrink-0"
    >
      {{ statusIcon(step.status) }}
    </span>
    <span :class="statusColor(step.status)" class="tracking-wider">
      {{ step.label }}<span class="text-jarvis-muted">{{ dots(step.label) }}</span>
    </span>
    <span class="text-jarvis-muted text-xs truncate ml-2">
      {{ step.message }}
    </span>
    <button
      v-if="step.status === 'error' && retryable"
      class="ml-auto px-2 py-0.5 font-mono text-xs text-jarvis-cyan border border-jarvis-cyan/30
             rounded hover:bg-jarvis-cyan/10 transition-colors flex-shrink-0"
      @click="emit('retry')"
    >
      Reintentar
    </button>
  </div>
</template>
