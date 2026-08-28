import { defineStore } from "pinia"
import { ref } from "vue"

export interface AppError {
  id: string
  message: string
  type: "error" | "warning" | "info"
  context?: string
  timestamp: Date
  retryable: boolean
}

export const useErrorsStore = defineStore("errors", () => {
  const errors = ref<AppError[]>([])

  function addError(
    message: string,
    type: AppError["type"] = "error",
    context?: string,
    retryable = false,
  ): string {
    const id = `err_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
    errors.value.push({
      id,
      message,
      type,
      context,
      timestamp: new Date(),
      retryable,
    })
    return id
  }

  function removeError(id: string) {
    errors.value = errors.value.filter((e) => e.id !== id)
  }

  function clearErrors() {
    errors.value = []
  }

  return { errors, addError, removeError, clearErrors }
})
