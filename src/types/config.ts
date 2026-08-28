export interface CerebroConfig {
  base_url: string
  timeout_secs: number
  binary_path: string
  idle_timeout_secs: number
}

export interface OllamaConfig {
  model: string
  base_url: string
}

export interface UiConfig {
  panels: string[]
}

export interface BootConfig {
  lazy_cerebro: boolean
}

export interface SearxngConfig {
  enabled: boolean
  port: number
  idle_timeout_secs: number
  docker_image: string
}

export interface WebSearchFallbackConfig {
  enabled: boolean
  keywords: string[]
  timeout_secs: number
  max_results: number
}

export interface SpotifyConfig {
  enabled: boolean
  client_id: string
  client_secret: string
}

export interface YoutubeConfig {
  enabled: boolean
}

export interface JarvisConfig {
  cerebro: CerebroConfig
  ollama: OllamaConfig
  ui: UiConfig
  boot?: BootConfig
  searxng?: SearxngConfig
  web_search_fallback?: WebSearchFallbackConfig
  spotify?: SpotifyConfig
  youtube?: YoutubeConfig
}
