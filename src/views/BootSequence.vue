<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useBootSequence } from "../composables/useBootSequence";
import BootCheck from "../components/boot/BootCheck.vue";

const emit = defineEmits<{ complete: [success: boolean] }>();

const { runChecks, retryStep, bootStore } = useBootSequence();

const progress = computed(() => {
  const total = bootStore.steps.length;
  const done = bootStore.steps.filter(
    (s) => s.status === "ok" || s.status === "error"
  ).length;
  return Math.round((done / total) * 100);
});

const failed = computed(() => bootStore.steps.some((s) => s.status === "error"));
const allDone = computed(() => bootStore.steps.every((s) => s.status === "ok" || s.status === "error"));

const errorMessages = computed(() =>
  bootStore.steps
    .filter((s) => s.status === "error" && s.message)
    .map((s) => `${s.label}: ${s.message}`)
);

onMounted(async () => {
  const success = await runChecks();
  if (success) {
    setTimeout(() => emit("complete", true), 1200);
  }
});
</script>

<template>
  <div class="min-h-screen bg-jarvis-bg flex items-center justify-center">
    <div class="w-full max-w-lg mx-auto px-6">
      <div class="text-center mb-10">
        <h1
          class="text-5xl font-mono font-bold tracking-widest text-jarvis-cyan animate-pulse-glow"
        >
          J.A.R.V.I.S.
        </h1>
        <p class="mt-3 text-sm font-mono text-jarvis-muted">
          Inicializando sistemas...
        </p>
      </div>

      <div class="space-y-3 mb-6">
        <BootCheck
          v-for="(step, index) in bootStore.steps"
          :key="step.id"
          :step="step"
          :retryable="step.status === 'error'"
          @retry="retryStep(index)"
        />
      </div>

      <div class="w-full h-1.5 rounded-full bg-jarvis-border overflow-hidden mb-2">
        <div
          class="h-full rounded-full transition-all duration-500 ease-out"
          :class="failed ? 'bg-jarvis-red' : 'bg-jarvis-cyan'"
          :style="{ width: `${progress}%` }"
        />
      </div>
      <p class="text-xs font-mono text-jarvis-muted text-right mb-4">
        {{ progress }}%
      </p>

      <div v-if="failed && allDone" class="rounded-lg border border-jarvis-red/30 bg-jarvis-red/5 p-4">
        <p class="text-sm font-mono text-jarvis-red font-semibold mb-2">
          ✗ Error en el arranque
        </p>
        <ul class="space-y-1 mb-4">
          <li
            v-for="(msg, i) in errorMessages"
            :key="i"
            class="text-xs font-mono text-jarvis-red/80 pl-2"
          >
            • {{ msg }}
          </li>
        </ul>
        <p class="text-xs font-mono text-jarvis-muted mb-3">
          Verifica que llama-server esté corriendo y tenga un modelo descargado.
        </p>
        <button
          class="w-full px-4 py-2 font-mono text-sm text-jarvis-cyan border border-jarvis-cyan/30
                 rounded hover:bg-jarvis-cyan/10 transition-colors"
          @click="runChecks"
        >
          Reintentar
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes pulse-glow {
  0%,
  100% {
    text-shadow: 0 0 8px rgba(0, 212, 255, 0.4), 0 0 20px rgba(0, 212, 255, 0.1);
  }
  50% {
    text-shadow: 0 0 16px rgba(0, 212, 255, 0.8), 0 0 40px rgba(0, 212, 255, 0.3);
  }
}

.animate-pulse-glow {
  animation: pulse-glow 2s ease-in-out infinite;
}
</style>
