import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CerebroResponse } from '../types/cerebro'

export function useCerebroClient() {
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  async function queryCerebro(query: string): Promise<CerebroResponse> {
    isLoading.value = true
    error.value = null
    try {
      const result = await invoke<CerebroResponse>('query_cerebro', { query })
      return result
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error("[cerebro] queryCerebro error:", msg, e)
      error.value = msg
      throw new Error(msg)
    } finally {
      isLoading.value = false
    }
  }

  async function checkHealth(): Promise<boolean> {
    isLoading.value = true
    error.value = null
    try {
      const result = await invoke<boolean>('check_cerebro_health')
      return result
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error("[cerebro] checkHealth error:", msg, e)
      error.value = msg
      return false
    } finally {
      isLoading.value = false
    }
  }

  return { queryCerebro, checkHealth, isLoading, error }
}
