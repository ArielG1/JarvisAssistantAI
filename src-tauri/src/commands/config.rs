use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_FILENAME: &str = "jarvis.config.toml";
const ENV_CONFIG_PATH: &str = "JARVIS_CONFIG_PATH";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CerebroConfig {
    pub base_url: String,
    pub timeout_secs: u64,
    #[serde(default = "default_cerebro_binary_path")]
    pub binary_path: String,
    #[serde(default = "default_cerebro_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

impl Default for CerebroConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout_secs: 15,
            binary_path: String::new(),
            idle_timeout_secs: 600,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub binary_path: String,
    pub model_path: String,
    pub port: u16,
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: u32,
    #[serde(default = "default_context_size")]
    pub context_size: u32,
}

fn default_cerebro_binary_path() -> String {
    String::new()
}
fn default_cerebro_idle_timeout_secs() -> u64 {
    600
}

fn default_gpu_layers() -> u32 {
    0
}
fn default_context_size() -> u32 {
    4096
}

impl LlmConfig {
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            binary_path: "llama-server".to_string(),
            model_path: String::new(),
            port: 8081,
            gpu_layers: 0,
            context_size: 4096,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub panels: Vec<String>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            panels: vec![
                "chat".to_string(),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BootConfig {
    pub lazy_cerebro: bool,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            lazy_cerebro: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearxngConfig {
    pub enabled: bool,
    pub port: u16,
    pub idle_timeout_secs: u64,
    pub docker_image: String,
}

impl Default for SearxngConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8888,
            idle_timeout_secs: 300,
            docker_image: "searxng/searxng".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSearchFallbackConfig {
    pub enabled: bool,
    pub keywords: Vec<String>,
    pub timeout_secs: u64,
    pub max_results: usize,
}

impl Default for WebSearchFallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            keywords: vec![
                "noticias".into(),
                "últimas".into(),
                "actual".into(),
                "precio".into(),
                "clima".into(),
                "latest".into(),
                "current".into(),
                "today".into(),
                "news".into(),
                "qué hora es".into(),
                "cuánto vale".into(),
                "cuánto cuesta".into(),
                "quién es".into(),
                "qué es".into(),
                "dónde está".into(),
                "cuándo".into(),
            ],
            timeout_secs: 10,
            max_results: 5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpotifyConfig {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub user_access_token: String,
    #[serde(default)]
    pub user_refresh_token: String,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            client_id: String::new(),
            client_secret: String::new(),
            user_access_token: String::new(),
            user_refresh_token: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YoutubeConfig {
    pub enabled: bool,
}

impl Default for YoutubeConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JarvisConfig {
    pub cerebro: CerebroConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    pub ui: UiConfig,
    #[serde(default)]
    pub boot: BootConfig,
    #[serde(default)]
    pub searxng: SearxngConfig,
    #[serde(default)]
    pub web_search_fallback: WebSearchFallbackConfig,
    #[serde(default)]
    pub spotify: SpotifyConfig,
    #[serde(default)]
    pub youtube: YoutubeConfig,
}

impl Default for JarvisConfig {
    fn default() -> Self {
        Self {
            cerebro: CerebroConfig::default(),
            llm: LlmConfig::default(),
            ui: UiConfig::default(),
            boot: BootConfig::default(),
            searxng: SearxngConfig::default(),
            web_search_fallback: WebSearchFallbackConfig::default(),
            spotify: SpotifyConfig::default(),
            youtube: YoutubeConfig::default(),
        }
    }
}

fn find_config_path() -> PathBuf {
    // 1. Environment variable override
    if let Ok(env_path) = std::env::var(ENV_CONFIG_PATH) {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return p;
        }
    }

    // 2. Next to the executable (e.g. target/debug/jarvis-desktop.exe)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let p = exe_dir.join(CONFIG_FILENAME);
            if p.exists() {
                return p;
            }
        }
    }

    // 3. Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(CONFIG_FILENAME);
        if p.exists() {
            return p;
        }
    }

    // 4. Project root heuristic: walk up from exe dir looking for Cargo.toml / package.json
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(mut dir) = exe_path.parent().map(|p| p.to_path_buf()) {
            for _ in 0..10 {
                if dir.join("Cargo.toml").exists() || dir.join("package.json").exists() {
                    let p = dir.join(CONFIG_FILENAME);
                    if p.exists() {
                        return p;
                    }
                }
                if !dir.pop() {
                    break;
                }
            }
        }
    }

    // 5. Fallback: return exe dir path (even if missing) so caller can write default
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(CONFIG_FILENAME);
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(CONFIG_FILENAME)
}

pub fn load_config_sync() -> Result<JarvisConfig, String> {
    let path = find_config_path();
    eprintln!("[config] load_config_sync path={}", path.display());
    if !path.exists() {
        let default = JarvisConfig::default();
        write_default_config(&path)?;
        return Ok(default);
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Error reading config file: {}", e))?;
    let config: JarvisConfig =
        toml::from_str(&content).map_err(|e| format!("Error parsing config file: {}", e))?;
    eprintln!("[config] llm.model_path={}", config.llm.model_path);
    eprintln!("[config] llm.binary_path={}", config.llm.binary_path);
    Ok(config)
}

fn write_default_config(path: &PathBuf) -> Result<(), String> {
    let default = JarvisConfig::default();
    let content =
        toml::to_string_pretty(&default).map_err(|e| format!("Error serializing default config: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Error writing default config: {}", e))?;
    Ok(())
}

pub fn save_config_sync(config: &JarvisConfig) -> Result<(), String> {
    let path = find_config_path();
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Error serializing config: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Error writing config file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn load_config() -> Result<JarvisConfig, String> {
    load_config_sync()
}

#[tauri::command]
pub async fn save_config(config: JarvisConfig) -> Result<(), String> {
    let path = find_config_path();
    let content =
        toml::to_string_pretty(&config).map_err(|e| format!("Error serializing config: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Error writing config file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_config_value(key: String) -> Result<String, String> {
    let config = load_config().await?;
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["cerebro", "base_url"] => Ok(config.cerebro.base_url),
        ["cerebro", "timeout_secs"] => Ok(config.cerebro.timeout_secs.to_string()),
        ["cerebro", "binary_path"] => Ok(config.cerebro.binary_path),
        ["cerebro", "idle_timeout_secs"] => Ok(config.cerebro.idle_timeout_secs.to_string()),
        ["llm", "binary_path"] => Ok(config.llm.binary_path),
        ["llm", "model_path"] => Ok(config.llm.model_path),
        ["llm", "port"] => Ok(config.llm.port.to_string()),
        ["llm", "gpu_layers"] => Ok(config.llm.gpu_layers.to_string()),
        ["llm", "context_size"] => Ok(config.llm.context_size.to_string()),
        ["ui", "panels"] => {
            serde_json::to_string(&config.ui.panels)
                .map_err(|e| format!("Error serializing panels: {}", e))
        }
        ["boot", "lazy_cerebro"] => Ok(config.boot.lazy_cerebro.to_string()),
        ["searxng", "enabled"] => Ok(config.searxng.enabled.to_string()),
        ["searxng", "port"] => Ok(config.searxng.port.to_string()),
        ["searxng", "idle_timeout_secs"] => Ok(config.searxng.idle_timeout_secs.to_string()),
        ["searxng", "docker_image"] => Ok(config.searxng.docker_image),
        ["web_search_fallback", "enabled"] => Ok(config.web_search_fallback.enabled.to_string()),
        ["web_search_fallback", "timeout_secs"] => Ok(config.web_search_fallback.timeout_secs.to_string()),
        ["web_search_fallback", "max_results"] => Ok(config.web_search_fallback.max_results.to_string()),
        ["web_search_fallback", "keywords"] => {
            serde_json::to_string(&config.web_search_fallback.keywords)
                .map_err(|e| format!("Error serializing keywords: {}", e))
        }
        ["spotify", "enabled"] => Ok(config.spotify.enabled.to_string()),
        ["spotify", "client_id"] => Ok(config.spotify.client_id),
        ["spotify", "client_secret"] => Ok(config.spotify.client_secret),
        ["spotify", "user_access_token"] => Ok(config.spotify.user_access_token),
        ["spotify", "user_refresh_token"] => Ok(config.spotify.user_refresh_token),
        ["youtube", "enabled"] => Ok(config.youtube.enabled.to_string()),
        _ => Err(format!("Unknown config key: {}", key)),
    }
}
