import { defineStore } from "pinia"
import { ref } from "vue"
import type { JarvisStatus } from "@/types/status"

export const useHudStore = defineStore("hud", () => {
  const currentState = ref<JarvisStatus>("escuchando")

  function setState(state: JarvisStatus) {
    currentState.value = state
  }

  function getState(): JarvisStatus {
    return currentState.value
  }

  return { currentState, setState, getState }
})
