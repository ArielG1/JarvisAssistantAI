import { defineStore } from "pinia"
import { ref } from "vue"
import type { JarvisStatus } from "@/types/status"

export const useHudStore = defineStore("hud", () => {
  const currentState = ref<JarvisStatus>("escuchando")
  const lastChangeAt = ref(0)
  const hiddenCards = ref<string[]>([])

  function setState(state: JarvisStatus) {
    if (state !== currentState.value) {
      currentState.value = state
      lastChangeAt.value = performance.now()
    }
  }

  function getState(): JarvisStatus {
    return currentState.value
  }

  function hideCard(id: string) {
    if (!hiddenCards.value.includes(id)) hiddenCards.value.push(id)
  }

  function showCard(id: string) {
    hiddenCards.value = hiddenCards.value.filter((c) => c !== id)
  }

  function isHidden(id: string) {
    return hiddenCards.value.includes(id)
  }

  return {
    currentState,
    lastChangeAt,
    hiddenCards,
    setState,
    getState,
    hideCard,
    showCard,
    isHidden,
  }
})
