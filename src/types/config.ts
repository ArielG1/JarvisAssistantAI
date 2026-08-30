export interface CerebroConfig {
  base_url: string
  timeout_secs: number
  binary_path: string
  idle_timeout_secs: number
}

export interface LlmConfig {
  binary_path: string
  model_path: string
  port: number
  gpu_layers: number
  context_size: number
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
  base_url: string
  idle_timeout_secs?: number
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
  user_access_token: string
  user_refresh_token: string
}

export interface YoutubeConfig {
  enabled: boolean
}

export interface WebSearchTriggerConfig {
  trigger_words: string[]
}

export interface JarvisConfig {
  cerebro: CerebroConfig
  llm: LlmConfig
  ui: UiConfig
  boot?: BootConfig
  searxng?: SearxngConfig
  web_search_trigger?: WebSearchTriggerConfig
  web_search_fallback?: WebSearchFallbackConfig
  spotify?: SpotifyConfig
  youtube?: YoutubeConfig
}
