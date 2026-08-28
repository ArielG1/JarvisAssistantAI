import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import type { ChatMessage, MessageRole } from "@/types/message"
import { createMessage } from "@/types/message"
import { useHudStore } from "./hud"

interface CerebroFallbackResponse {
  response: string
  web_search_used: boolean
  search_results: Array<{ title: string; url: string; snippet: string }>
}

export const useChatStore = defineStore("chat", () => {
  const messages = ref<ChatMessage[]>([])
  const isTyping = ref(false)
  const typingMessage = ref("")
  const cerebroStarted = ref(false)
  const webSearchActive = ref(false)

  function addMessage(content: string, role: MessageRole) {
    messages.value.push(createMessage(content, role))
  }

  function clearMessages() {
    messages.value = []
  }

  function setTyping(typing: boolean, message?: string) {
    isTyping.value = typing
    typingMessage.value = message ?? ""
  }

  async function ensureCerebroRunning() {
    if (cerebroStarted.value) return
    try {
      await invoke<void>("start_cerebro")
      cerebroStarted.value = true
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.warn("[chat] start_cerebro failed (may already be running):", msg)
      cerebroStarted.value = true
    }
  }

  function detectSpotifyAuthIntent(content: string): boolean {
    const patterns = [
      /autorizar\s+spotify/i,
      /authorize\s+spotify/i,
      /spotify\s+autorizar/i,
      /spotify\s+authorize/i,
      /conectar\s+spotify/i,
      /connect\s+spotify/i,
    ]
    return patterns.some(p => p.test(content))
  }

  function detectSpotifyIntent(content: string): string | null {
    const lower = content.toLowerCase()
    const patterns = [
      /reproduce(?:r)?\s+(?:en\s+)?spotify\s+(.+)/i,
      /(?:pon|coloca|busca)\s+(?:en\s+)?spotify\s+(.+)/i,
      /(?:escuchar|oír|oir)\s+(.+)\s+en\s+spotify/i,
    ]
    for (const pattern of patterns) {
      const match = lower.match(pattern)
      if (match) return match[1].trim()
    }
    return null
  }

  function detectYouTubeIntent(content: string): string | null {
    const lower = content.toLowerCase()
    const patterns = [
      /(?:reproduce|pon|busca(?:r)?\s+video(?:s)?)\s+(.+?)\s+en\s+youtube/i,
      /(?:reproduce|pon|busca(?:r)?)\s+video(?:s)?\s+(.+?)(?:\s+en\s+youtube)?$/i,
      /(?:youtube|en\s+youtube)\s+(.+)/i,
      /(.+?)\s+en\s+youtube/i,
    ]
    for (const pat of patterns) {
      const m = lower.match(pat) ?? content.match(pat)
      if (m && m[1]) return m[1].trim()
    }
    if (lower.includes("youtube") || lower.includes("video")) {
      const cleaned = lower
        .replace(/reproduce|pon|busca(?:r)?|video(?:s)?|en|youtube/gi, "")
        .trim()
      if (cleaned.length > 1) return cleaned
    }
    return null
  }

  async function sendMessage(content: string) {
    const hud = useHudStore()

    addMessage(content, "user")

    if (detectSpotifyAuthIntent(content)) {
      hud.setState("pensando")
      setTyping(true, "🔗 Iniciando autorización de Spotify...")
      try {
        const result = await invoke<string>("authorize_spotify_user")
        addMessage(`✅ ${result}`, "jarvis")
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        addMessage(`⚠️ Error: ${msg}`, "system")
      }
      setTyping(false)
      hud.setState("escuchando")
      return
    }

    const spotifyQuery = detectSpotifyIntent(content)
    if (spotifyQuery) {
      const available = await invoke<boolean>("is_spotify_available")
      if (!available) {
        addMessage("⚠️ Spotify no está instalado en este equipo.", "system")
        return
      }
      hud.setState("pensando")
      setTyping(true, "🎵 Buscando en Spotify...")
      try {
        const result = await invoke<string>("play_spotify", { query: spotifyQuery })
        addMessage(`🎵 ${result}`, "jarvis")
        setTyping(false)
        setTimeout(() => hud.setState("escuchando"), 2000)
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        addMessage(`⚠️ Error con Spotify: ${msg}`, "system")
        setTyping(false)
        hud.setState("escuchando")
      }
      return
    }

    const ytQuery = detectYouTubeIntent(content)
    if (ytQuery) {
      hud.setState("pensando")
      setTyping(true, "🎬 Buscando en YouTube...")
      try {
        const videoTitle = await invoke<string>("play_youtube", { query: ytQuery })
        addMessage(`🎬 Abriendo: ${videoTitle} en YouTube`, "jarvis")
        setTyping(false)
        setTimeout(() => hud.setState("escuchando"), 2000)
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        addMessage(`⚠️ No se pudo reproducir el video: ${msg}`, "system")
        setTyping(false)
        hud.setState("escuchando")
      }
      return
    }

    hud.setState("pensando")
    setTyping(true, "Procesando...")

    try {
      await ensureCerebroRunning()

      setTyping(true, "🔍 Buscando en web...")
      webSearchActive.value = true

      const history = messages.value.slice(0, -1).map((m) => ({
        role: m.role === "jarvis" ? "assistant" : m.role === "system" ? "system" : "user",
        content: m.content,
      }))

      const result = await invoke<CerebroFallbackResponse>("send_to_cerebro_with_fallback", {
        message: content,
        history: history,
      })

      webSearchActive.value = false

      if (result.web_search_used) {
        setTyping(true, "📋 Resultados de búsqueda integrados...")
        await new Promise((r) => setTimeout(r, 500))
      }

      const fallback = "No tengo información sobre eso. ¿Puedes reformular tu pregunta?"
      const displayResponse = result.response?.trim() || fallback
      addMessage(displayResponse, "jarvis")

      hud.setState("respondiendo")
      setTyping(false)

      setTimeout(() => {
        hud.setState("escuchando")
      }, 3000)
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error("[chat] sendMessage error:", msg, e)
      webSearchActive.value = false
      addMessage(
        `⚠️ No se pudo conectar con Cerebro. Verifica que esté ejecutándose. (${msg})`,
        "system"
      )

      setTyping(false)
      hud.setState("escuchando")
    }
  }

  return {
    messages,
    isTyping,
    typingMessage,
    webSearchActive,
    addMessage,
    clearMessages,
    setTyping,
    sendMessage,
  }
})
