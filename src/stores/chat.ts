import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import type { ChatMessage, MessageRole, MessageSource } from "@/types/message"
import { createMessage } from "@/types/message"
import { useHudStore } from "./hud"

type ResponseSource = "cerebro" | "web" | "llm"

interface CerebroFallbackResponse {
  response: string
  web_search_used: boolean
  search_results: Array<{ title: string; url: string; snippet: string }>
  source: ResponseSource
  source_url: string | null
  source_domain: string | null
}

export const useChatStore = defineStore("chat", () => {
  const messages = ref<ChatMessage[]>([])
  const isTyping = ref(false)
  const typingMessage = ref("")
  const cerebroStarted = ref(false)
  const webSearchActive = ref(false)

  function addMessage(content: string, role: MessageRole, source?: MessageSource) {
    messages.value.push(createMessage(content, role, source))
  }

  function clearMessages() {
    messages.value = []
  }

  function setTyping(typing: boolean, message?: string) {
    isTyping.value = typing
    typingMessage.value = message ?? ""
  }

  function setTypingSource(source: ResponseSource) {
    switch (source) {
      case "cerebro":
        typingMessage.value = ""
        break
      case "web":
        typingMessage.value = "Sin resultados en Cerebro — buscando en internet..."
        break
      case "llm":
        typingMessage.value = "Sin resultados de ningún lado — usando modelo local..."
        break
    }
  }

  async function ensureCerebroRunning() {
    if (cerebroStarted.value) return
    try {
      await invoke<void>("start_cerebro")
      cerebroStarted.value = true
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.warn("[chat] start_cerebro failed (may already be running):", msg)
      throw new Error(`Cerebro startup failed: ${msg}`)
    }
  }

  // NOTE: These patterns duplicate DIRECT_WEB_PATTERNS and trigger_words from
  // jarvis.config.toml. When modifying trigger words, sync both locations.
  function shouldShowWebNotification(query: string): boolean {
    const lower = query.toLowerCase()
    const bypassPatterns = ["dólar", "dolar", "clima", "tiempo", "hora", "marcador"]
    const searchTriggers = [
      "presidente", "quién es", "quien es", "noticias", "actualidad",
      "precio", "cotización", "cotizacion", "bitcoin", "euro",
      "último", "última", "ultimo", "ultima", "hoy", "ahora",
      "buscá", "googleá", "busca", "buscar", "internet",
      "deportes", "fútbol", "futbol", "partido",
    ]
    return (
      bypassPatterns.some((p) => lower.includes(p)) ||
      searchTriggers.some((p) => lower.includes(p))
    )
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

  function detectSpotifyQueueIntent(content: string): string | null {
    const lower = content.toLowerCase()
    const patterns = [
      /agreg(?:á|a)\s+(.+?)\s+(?:a\s+la\s+)?(?:fila|cola)/i,
      /sum(?:á|a)\s+(.+?)\s+(?:a\s+la\s+)?(?:fila|cola)/i,
      /(?:pon|poné)\s+(.+?)\s+(?:despu[eé]s|en\s+la\s+cola)/i,
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

  function sanitizeQuery(query: string): string {
    return query
      .replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, "")
      .trim()
      .slice(0, 200)
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

    const spotifyQueueQuery = detectSpotifyQueueIntent(content)
    if (spotifyQueueQuery) {
      const available = await invoke<boolean>("is_spotify_available")
      if (!available) {
        addMessage("⚠️ Spotify no está instalado en este equipo.", "system")
        return
      }
      hud.setState("pensando")
      setTyping(true, "🎵 Agregando a la cola de Spotify...")
      try {
        const result = await invoke<string>("add_to_spotify_queue", { query: spotifyQueueQuery })
        addMessage(`🎵 Agregado a la cola: ${result}`, "jarvis")
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
      const sanitizedQuery = sanitizeQuery(ytQuery)
      if (!sanitizedQuery) {
        addMessage("⚠️ Consulta de YouTube inválida.", "system")
        return
      }
      hud.setState("pensando")
      setTyping(true, "🎬 Buscando en YouTube...")
      try {
        const videoTitle = await invoke<string>("play_youtube", { query: sanitizedQuery })
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
      webSearchActive.value = true

      if (shouldShowWebNotification(content)) {
        hud.setState("trabajando")
        setTyping(true, "Sin resultados en Cerebro — buscando en internet...")
      }

      const history = messages.value.slice(-20, -1).map((m) => ({
        role: m.role === "jarvis" ? "assistant" : m.role === "system" ? "system" : "user",
        content: m.content,
      }))

      const result = await invoke<CerebroFallbackResponse>("send_to_cerebro_with_fallback", {
        message: content,
        history: history,
      })

      webSearchActive.value = false

      if (result.source === "web") {
        setTypingSource("web")
      } else if (result.source === "llm") {
        hud.setState("pensando")
        setTypingSource("llm")
      } else {
        setTypingSource("cerebro")
      }

      const fallback = "No tengo información sobre eso. ¿Puedes reformular tu pregunta?"
      const displayResponse = result.response?.trim() || fallback

      const source: MessageSource | undefined =
        result.source_url || result.source_domain
          ? { type: result.source, url: result.source_url ?? undefined, domain: result.source_domain ?? undefined }
          : undefined

      addMessage(displayResponse, "jarvis", source)

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
    setTypingSource,
    sendMessage,
  }
})
