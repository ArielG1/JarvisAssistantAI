<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from "vue"

const props = defineProps<{
  message: string
  type: "error" | "warning" | "info"
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  retry: []
}>()

const timer = ref<ReturnType<typeof setTimeout> | null>(null)

function startAutoDismiss() {
  clearTimer()
  timer.value = setTimeout(() => {
    emit("close")
  }, 5000)
}

function clearTimer() {
  if (timer.value) {
    clearTimeout(timer.value)
    timer.value = null
  }
}

watch(
  () => props.visible,
  (v) => {
    if (v) startAutoDismiss()
    else clearTimer()
  },
)

onMounted(() => {
  if (props.visible) startAutoDismiss()
})

onBeforeUnmount(() => clearTimer())

const borderColor: Record<string, string> = {
  error: "border-jarvis-red",
  warning: "border-jarvis-amber",
  info: "border-cyan-400",
}

const bgColor: Record<string, string> = {
  error: "bg-jarvis-red/10",
  warning: "bg-jarvis-amber/10",
  info: "bg-cyan-400/10",
}

const textColor: Record<string, string> = {
  error: "text-jarvis-red",
  warning: "text-jarvis-amber",
  info: "text-cyan-400",
}

const iconColor: Record<string, string> = {
  error: "text-jarvis-red",
  warning: "text-jarvis-amber",
  info: "text-cyan-400",
}
</script>

<template>
  <Transition name="error-banner">
    <div
      v-if="visible"
      class="error-banner flex items-center gap-3 px-4 py-3 rounded-lg border-l-4 shadow-lg"
      :class="[borderColor[type], bgColor[type]]"
    >
      <svg
        v-if="type === 'error'"
        class="w-5 h-5 shrink-0"
        :class="iconColor[type]"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
          clip-rule="evenodd"
        />
      </svg>
      <svg
        v-else-if="type === 'warning'"
        class="w-5 h-5 shrink-0"
        :class="iconColor[type]"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
          clip-rule="evenodd"
        />
      </svg>
      <svg
        v-else
        class="w-5 h-5 shrink-0"
        :class="iconColor[type]"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
          clip-rule="evenodd"
        />
      </svg>

      <span class="flex-1 text-sm" :class="textColor[type]">{{ message }}</span>

      <div class="flex items-center gap-2">
        <button
          v-if="type !== 'info'"
          class="error-retry-btn px-2 py-1 text-xs rounded border border-current/30 hover:bg-current/10 transition-colors"
          :class="textColor[type]"
          @click="emit('retry')"
        >
          Reintentar
        </button>
        <button
          class="p-1 rounded hover:bg-white/10 transition-colors"
          :class="textColor[type]"
          @click="emit('close')"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path
              fill-rule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      </div>
    </div>
  </Transition>
</template>
