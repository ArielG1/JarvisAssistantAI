use std::path::PathBuf;
use std::sync::Arc;

use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::commands::config::{load_config_sync, SearxngConfig, WebSearchFallbackConfig};
use crate::commands::lazy_process::{LazyProcessConfig, LazyProcessHandle};

const CONTAINER_NAME: &str = "jarvis-searxng";
const SETTINGS_FILENAME: &str = "searxng-settings.yml";

fn resolve_settings_path() -> PathBuf {
    // 1. Environment variable override
    if let Ok(env_path) = std::env::var("SEARXNG_SETTINGS") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return p;
        }
    }

    // 2. Next to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let p = exe_dir.join(SETTINGS_FILENAME);
            if p.exists() {
                return p;
            }
        }
    }

    // 3. Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(SETTINGS_FILENAME);
        if p.exists() {
            return p;
        }
    }

    // 4. Project root heuristic: walk up from exe dir
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(mut dir) = exe_path.parent().map(|p| p.to_path_buf()) {
            for _ in 0..10 {
                if dir.join("Cargo.toml").exists() || dir.join("package.json").exists() {
                    let p = dir.join(SETTINGS_FILENAME);
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

    // 5. Fallback: exe dir (even if missing)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(SETTINGS_FILENAME);
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(SETTINGS_FILENAME)
}

pub struct SearxngManager {
    pub handle: LazyProcessHandle,
    pub config: SearxngConfig,
}

impl SearxngManager {
    pub fn new(config: SearxngConfig) -> Self {
        let hc_url = format!("http://localhost:{}/", config.port);
        let settings_path = resolve_settings_path();

        let mut docker_args: Vec<String> = vec![
            "run".to_string(),
            "-d".to_string(),
            "--name".to_string(),
            CONTAINER_NAME.to_string(),
            "--rm".to_string(),
            "-p".to_string(),
            format!("{}:8080", config.port),
        ];

        // Mount custom settings.yml if it exists
        if settings_path.exists() {
            let settings_str = settings_path.to_string_lossy().to_string();
            docker_args.push("-v".to_string());
            docker_args.push(format!("{}:/etc/searxng/settings.yml", settings_str));
            info!(path = %settings_str, "mounting SearXNG settings");
        } else {
            warn!("searxng-settings.yml not found, using default SearXNG config (JSON API may be disabled)");
        }

        let img = &config.docker_image;
        if !img.chars().all(|c| c.is_alphanumeric() || "-_./:".contains(c)) {
            warn!(image = %img, "invalid docker image name, falling back to default");
            docker_args.push("searxng/searxng:latest".to_string());
        } else {
            docker_args.push(img.clone());
        }

        let lazy_config = LazyProcessConfig {
            name: CONTAINER_NAME.to_string(),
            command: Some("docker".to_string()),
            args: docker_args,
            idle_timeout_secs: config.idle_timeout_secs,
            healthcheck_url: Some(hc_url),
            healthcheck_interval_secs: Some(3),
        };
        Self {
            handle: LazyProcessHandle::new(lazy_config),
            config,
        }
    }

    fn base_url(&self) -> String {
        format!("http://localhost:{}", self.config.port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearxngApiResponse {
    results: Vec<SearxngApiResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearxngApiResult {
    title: String,
    url: String,
    #[serde(default)]
    content: String,
}

fn get_manager(app: &AppHandle) -> Result<Arc<Mutex<SearxngManager>>, String> {
    let state = app.try_state::<Arc<Mutex<SearxngManager>>>();
    state
        .map(|s| s.inner().clone())
        .ok_or_else(|| "SearXNG manager not initialized".to_string())
}

static DOCKER_AVAILABLE: Lazy<Result<(), String>> = Lazy::new(|| {
    let output = std::process::Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .map_err(|e| format!("Docker not found or not accessible: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Docker error: {}", stderr.trim()));
    }
    Ok(())
});

async fn ensure_docker_available() -> Result<(), String> {
    DOCKER_AVAILABLE.clone()
}

async fn cleanup_existing_container() -> Result<(), String> {
    let output = tokio::process::Command::new("docker")
        .args(["rm", "-f", CONTAINER_NAME])
        .output()
        .await
        .map_err(|e| format!("Failed to cleanup container: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("No such container") {
            warn!("container cleanup warning: {}", stderr.trim());
        }
    }
    Ok(())
}

async fn wait_for_health(
    port: u16,
    timeout_secs: u64,
) -> Result<(), String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let url = format!("http://localhost:{}/", port);
    let start = tokio::time::Instant::now();
    let deadline = std::time::Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() >= deadline {
            return Err(format!(
                "SearXNG healthcheck timed out after {}s",
                timeout_secs
            ));
        }
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("SearXNG is healthy on port {}", port);
                return Ok(());
            }
            Ok(resp) => {
                warn!("SearXNG returned status {}", resp.status());
            }
            Err(e) => {
                warn!("SearXNG healthcheck attempt failed: {}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn is_container_running() -> Result<bool, String> {
    let output = tokio::process::Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", CONTAINER_NAME])
        .output()
        .await
        .map_err(|e| format!("Failed to check container state: {}", e))?;
    if !output.status.success() {
        return Ok(false);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "true")
}

async fn ensure_running(app: &AppHandle) -> Result<(), String> {
    let mgr_arc = get_manager(app)?;
    let mgr = mgr_arc.lock().await;
    let port = mgr.config.port;
    drop(mgr);

    ensure_docker_available().await?;

    if let Ok(running) = is_container_running().await {
        if running {
            match wait_for_health(port, 5).await {
                Ok(()) => {
                    info!("SearXNG container already running and healthy");
                    return Ok(());
                }
                Err(_) => {
                    warn!("SearXNG container running but unhealthy, restarting...");
                    cleanup_existing_container().await?;
                }
            }
        }
    }

    info!("starting SearXNG Docker container");
    let mgr = mgr_arc.lock().await;
    mgr.handle.start().await?;
    drop(mgr);

    wait_for_health(port, 30).await
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn start_searxng(app: AppHandle) -> Result<String, String> {
    let mgr_arc = get_manager(&app)?;
    let mgr = mgr_arc.lock().await;
    let port = mgr.config.port;
    drop(mgr);

    ensure_docker_available().await?;

    if let Ok(running) = is_container_running().await {
        if running {
            match wait_for_health(port, 5).await {
                Ok(()) => {
                    info!("SearXNG container already running and healthy");
                    return Ok(format!("SearXNG already running on port {}", port));
                }
                Err(_) => {
                    warn!("SearXNG container running but unhealthy, restarting...");
                    cleanup_existing_container().await?;
                }
            }
        }
    }

    let mgr = mgr_arc.lock().await;
    mgr.handle.start().await?;
    drop(mgr);

    wait_for_health(port, 30).await?;

    Ok(format!("SearXNG running on port {}", port))
}

#[tauri::command]
pub async fn stop_searxng(app: AppHandle) -> Result<String, String> {
    let mgr_arc = get_manager(&app)?;

    {
        let mgr = mgr_arc.lock().await;
        mgr.handle.stop().await?;
    }

    let _ = tokio::process::Command::new("docker")
        .args(["rm", "-f", CONTAINER_NAME])
        .output()
        .await;

    Ok("SearXNG stopped".to_string())
}

#[derive(Serialize)]
pub struct SearxngStatus {
    pub running: bool,
    pub healthy: bool,
    pub port: u16,
}

#[tauri::command]
pub async fn searxng_status(app: AppHandle) -> Result<SearxngStatus, String> {
    let mgr_arc = get_manager(&app)?;
    let mgr = mgr_arc.lock().await;
    let port = mgr.config.port;
    let running = mgr.handle.is_running().await;
    let healthy = if running {
        mgr.handle.healthcheck().await.unwrap_or(false)
    } else {
        false
    };
    drop(mgr);

    Ok(SearxngStatus {
        running,
        healthy,
        port,
    })
}

#[tauri::command]
pub async fn search_web(app: AppHandle, query: String) -> Result<Vec<SearchResult>, String> {
    ensure_running(&app).await?;

    let mgr_arc = get_manager(&app)?;
    let mgr = mgr_arc.lock().await;
    let base_url = mgr.base_url();
    drop(mgr);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let url = format!("{}/search?q={}&format=json", base_url, urlencoding::encode(&query));

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Search request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("SearXNG returned {}: {}", status, body));
    }

    let api_resp: SearxngApiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse search response: {}", e))?;

    let results: Vec<SearchResult> = api_resp
        .results
        .into_iter()
        .take(10)
        .map(|r| SearchResult {
            title: r.title,
            url: r.url,
            snippet: r.content,
        })
        .collect();

    Ok(results)
}

pub fn should_trigger_web_search(query: &str, config: &WebSearchFallbackConfig) -> bool {
    if !config.enabled {
        return false;
    }
    let query_lower = query.to_lowercase();
    config.keywords.iter().any(|kw| query_lower.contains(&kw.to_lowercase()))
}

// --- YouTube Search ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoResult {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearxngVideoResponse {
    results: Vec<SearxngVideoResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearxngVideoResult {
    title: String,
    url: String,
    #[serde(default)]
    thumbnail: String,
}

#[tauri::command]
pub async fn search_youtube(app: AppHandle, query: String) -> Result<Vec<VideoResult>, String> {
    ensure_running(&app).await?;

    let mgr_arc = get_manager(&app)?;
    let mgr = mgr_arc.lock().await;
    let base_url = mgr.base_url();
    drop(mgr);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let url = format!(
        "{}/search?q={}&format=json&categories=videos",
        base_url,
        urlencoding::encode(&query)
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("YouTube search request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("SearXNG returned {}: {}", status, body));
    }

    let api_resp: SearxngVideoResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse video search response: {}", e))?;

    let results: Vec<VideoResult> = api_resp
        .results
        .into_iter()
        .take(3)
        .map(|r| VideoResult {
            title: r.title,
            url: r.url,
            thumbnail: r.thumbnail,
        })
        .collect();

    Ok(results)
}

#[tauri::command]
pub async fn play_youtube(app: AppHandle, query: String) -> Result<String, String> {
    let results = search_youtube(app, query.clone()).await?;

    let video = results
        .into_iter()
        .next()
        .ok_or_else(|| format!("No se encontraron videos para: {}", query))?;

    open::that(&video.url)
        .map_err(|e| format!("No se pudo abrir el navegador: {}", e))?;

    Ok(video.title)
}

pub fn format_results_as_context(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return String::new();
    }
    let mut ctx = String::from("Información encontrada en la web:\n\n");
    for (i, r) in results.iter().enumerate() {
        ctx.push_str(&format!("{}. {} — {}\n   Fuente: {}\n\n", i + 1, r.title, r.snippet, r.url));
    }
    ctx
}

pub async fn search_web_for_context(
    app: &AppHandle,
    query: &str,
    config: &WebSearchFallbackConfig,
) -> Result<(String, Vec<SearchResult>), String> {
    ensure_running(app).await?;

    let max = config.max_results;
    let timeout = config.timeout_secs;

    let mgr_arc = get_manager(app)?;
    let mgr = mgr_arc.lock().await;
    let base_url = mgr.base_url();
    drop(mgr);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let url = format!("{}/search?q={}&format=json", base_url, urlencoding::encode(query));

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Search request failed: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("SearXNG returned {}: {}", status, body));
    }

    let api_resp: SearxngApiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse search response: {}", e))?;

    let results: Vec<SearchResult> = api_resp
        .results
        .into_iter()
        .take(max)
        .map(|r| SearchResult {
            title: r.title,
            url: r.url,
            snippet: r.content,
        })
        .collect();

    let context = format_results_as_context(&results);
    Ok((context, results))
}

#[tauri::command]
pub async fn init_searxng(app: AppHandle) -> Result<String, String> {
    let config = load_config_sync()
        .map_err(|e| format!("Failed to load config: {}", e))?;

    if !config.searxng.enabled {
        return Ok("SearXNG disabled in config".to_string());
    }

    let mgr = SearxngManager::new(config.searxng);
    let mgr_arc = Arc::new(Mutex::new(mgr));
    app.manage(mgr_arc);

    Ok("SearXNG manager initialized".to_string())
}
