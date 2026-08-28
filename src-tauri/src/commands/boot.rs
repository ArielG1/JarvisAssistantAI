use serde::Serialize;
use std::process::{Child, Command};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use super::config::load_config_sync;

/// Kill any existing llama-server processes on the given port.
/// Uses `taskkill` on Windows, `pkill` on Unix.
fn kill_existing_llamacpp(port: u16) {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "llama-server.exe"])
        .spawn();
    // Also try killing by port via netstat+taskkill (Windows)
    if cfg!(target_os = "windows") {
        if let Ok(output) = Command::new("cmd")
            .args([
                "/C",
                &format!("for /f \"tokens=5\" %a in ('netstat -ano ^| findstr :{} ^| findstr LISTENING') do taskkill /F /PID %a", port),
            ])
            .output()
        {
            if output.status.success() {
                println!("[boot] Killed existing process on port {}", port);
            }
        }
    } else {
        let _ = Command::new("pkill").args(["-f", "llama-server"]).spawn();
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BootStatus {
    pub step: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BootStep {
    pub id: String,
    pub label: String,
    pub status: String,
    pub message: String,
}

impl BootStep {
    fn running(id: &str, label: &str, msg: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: "running".to_string(),
            message: msg.to_string(),
        }
    }

    fn ok(id: &str, label: &str, msg: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: "ok".to_string(),
            message: msg.to_string(),
        }
    }

    fn error(id: &str, label: &str, msg: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: "error".to_string(),
            message: msg.to_string(),
        }
    }
}

fn create_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

fn spawn_llamacpp(config: &super::config::LlmConfig) -> Result<Child, String> {
    println!("[boot] Spawning llama-server on port {}", config.port);
    let mut cmd = Command::new(&config.binary_path);
    cmd.arg("--port").arg(config.port.to_string());
    if !config.model_path.is_empty() {
        cmd.arg("--model").arg(&config.model_path);
        let model_name = std::path::Path::new(&config.model_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model");
        cmd.arg("--alias").arg(model_name);
    }
    if config.gpu_layers > 0 {
        cmd.arg("--n-gpu-layers").arg(config.gpu_layers.to_string());
    }
    if config.context_size > 0 {
        cmd.arg("--ctx-size").arg(config.context_size.to_string());
    }
    cmd.spawn()
        .map_err(|e| format!("Could not launch llama-server: {e}"))
}

async fn check_llm_internal(app: &AppHandle, client: &reqwest::Client, llm_base_url: &str, config: &super::config::LlmConfig) -> Result<(), String> {
    let health_url = format!("{}/health", llm_base_url);
    println!("[boot] Checking LLM at {}", llm_base_url);

    // ── Phase 1: Check if llama-server is already running via /health ──
    let already_running = client.get(&health_url).send().await.map_or(false, |r| r.status().is_success());

    let mut spawned_child: Option<Child> = None;

    if !already_running {
        // Kill any stale llama-server before spawning a new one (race condition guard)
        println!("[boot] Killing any existing llama-server before spawn...");
        kill_existing_llamacpp(config.port);
        // Brief pause for OS to release port
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        println!("[boot] llama-server not running, attempting to spawn...");
        let _ = app.emit(
            "boot-step",
            &BootStep::running("llamacpp", "LLAMACPP", "Iniciando llama-server..."),
        );

        match spawn_llamacpp(config) {
            Ok(child) => {
                spawned_child = Some(child);
                println!("[boot] llama-server process launched, waiting for API...");
            }
            Err(e) => {
                let _ = app.emit("boot-step", &BootStep::error("llamacpp", "LLAMACPP", &e));
                return Err(e);
            }
        }
    }

    // ── Phase 2: Poll until API responds or timeout ──
    // 90 attempts × 3s = 270s max wait (models can take 60s+ to load on slow disks)
    let max_attempts = 90u32;
    for attempt in 1..=max_attempts {
        if let Some(ref mut child) = spawned_child {
            match child.try_wait() {
                Ok(Some(exit)) => {
                    let msg = format!(
                        "llama-server process exited immediately (exit code: {:?})",
                        exit.code()
                    );
                    let _ = child.kill();
                    let _ = app.emit("boot-step", &BootStep::error("llamacpp", "LLAMACPP", &msg));
                    return Err(msg);
                }
                Ok(None) => {}
                Err(e) => {
                    let msg = format!("Failed to check llama-server process: {e}");
                    let _ = child.kill();
                    let _ = app.emit("boot-step", &BootStep::error("llamacpp", "LLAMACPP", &msg));
                    return Err(msg);
                }
            }
        }

        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let _ = app.emit("boot-step", &BootStep::ok("llamacpp", "LLAMACPP", "Modelo cargado y listo"));
                return Ok(());
            }
            _ => {
                if attempt < max_attempts {
                    let msg = if spawned_child.is_some() {
                        format!("Esperando llama-server... ({}/{})", attempt, max_attempts)
                    } else {
                        format!("Esperando LLM... ({}/{})", attempt, max_attempts)
                    };
                    let _ = app.emit("boot-step", &BootStep::running("llamacpp", "LLAMACPP", &msg));
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        }
    }

    if let Some(mut child) = spawned_child {
        let _ = child.kill();
        let msg = format!(
            "llama-server launched but API not responding in {}",
            llm_base_url
        );
        let _ = app.emit("boot-step", &BootStep::error("llamacpp", "LLAMACPP", &msg));
        Err(msg)
    } else {
        let msg = format!("LLM no responde en {}", llm_base_url);
        let _ = app.emit("boot-step", &BootStep::error("llamacpp", "LLAMACPP", &msg));
        Err(msg)
    }
}

async fn check_modelo_internal(client: &reqwest::Client, llm_base_url: &str, model_path: &str) -> Result<(), String> {
    if model_path.is_empty() {
        return Err("model_path está vacío. Configura [llm].model_path en jarvis.config.toml".to_string());
    }
    let model_name = std::path::Path::new(model_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(model_path);
    let url = format!("{}/v1/chat/completions", llm_base_url);
    println!("[boot] Preloading model '{}' at {}", model_name, url);

    let body = serde_json::json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "test"}],
        "max_tokens": 1
    });

    let mut attempts = 0u32;
    let max_attempts = 3;
    loop {
        match client.post(&url).json(&body).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("[boot] Model '{}' preloaded successfully", model_name);
                return Ok(());
            }
            Ok(resp) if attempts + 1 < max_attempts => {
                attempts += 1;
                let msg = format!("Modelo respondió con código {}, reintentando... ({}/{})", resp.status(), attempts, max_attempts);
                println!("[boot] {}", msg);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Ok(resp) => {
                return Err(format!("Modelo respondió con código {}", resp.status()));
            }
            Err(e) if attempts + 1 < max_attempts => {
                attempts += 1;
                let msg = format!("Esperando modelo... ({}/{}): {}", attempts, max_attempts, e);
                println!("[boot] {}", msg);
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
            Err(e) => {
                return Err(format!("Modelo no disponible: {e}"));
            }
        }
    }
}

async fn check_cerebro_internal(client: &reqwest::Client, cerebro_url: &str) -> Result<String, String> {
    let health_url = format!("{}/health", cerebro_url);
    println!("[boot] Checking Cerebro at {}", health_url);
    let max_retries = 3u32;

    for attempt in 1..=max_retries {
        match client.get(&health_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("Error parseando JSON: {e}"))?;
                let message = body["message"]
                    .as_str()
                    .unwrap_or("Cerebro operativo")
                    .to_string();
                return Ok(message);
            }
            Ok(resp) if resp.status().is_server_error() => {
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                return Err(format!("Cerebro respondió con código {}", resp.status()));
            }
            Ok(resp) => {
                return Err(format!("Cerebro respondió con código {}", resp.status()));
            }
            Err(e) => {
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                return Err(format!("Cerebro no disponible: {e}"));
            }
        }
    }
    return Err("Model loading retries exhausted".to_string());
}

#[tauri::command]
pub async fn run_boot_sequence(app: AppHandle) -> Result<(), String> {
    println!("[boot] Starting boot sequence");
    let config = load_config_sync().unwrap_or_else(|e| {
        eprintln!("[boot] Warning: failed to load config, using defaults: {e}");
        Default::default()
    });
    let client = create_client(30)?;

    let llm_base_url = config.llm.base_url();

    // ── Step 1: llama-server ──
    println!("[boot] Step 1: Checking llama-server");
    let _ = app.emit(
        "boot-step",
        &BootStep::running("llamacpp", "LLAMACPP", "Verificando..."),
    );
    check_llm_internal(&app, &client, &llm_base_url, &config.llm).await?;
    println!("[boot] Step 1: llama-server OK");

    // ── Step 2: Preload model ──
    if config.llm.model_path.is_empty() {
        let msg = "model_path está vacío. Configura [llm].model_path en jarvis.config.toml".to_string();
        println!("[boot] Step 2: {}", msg);
        let _ = app.emit("boot-step", &BootStep::error("modelo", "MODELO", &msg));
        return Err(msg);
    }
    println!("[boot] Step 2: Preloading model");
    let _ = app.emit(
        "boot-step",
        &BootStep::running("modelo", "MODELO", "Pre-cargando modelo..."),
    );
    match check_modelo_internal(&client, &llm_base_url, &config.llm.model_path).await {
        Ok(()) => {
            let _ = app.emit(
                "boot-step",
                &BootStep::ok("modelo", "MODELO", "Modelo pre-cargado"),
            );
            println!("[boot] Step 2: Model preloaded OK");
        }
        Err(msg) => {
            let _ = app.emit(
                "boot-step",
                &BootStep::error("modelo", "MODELO", &msg),
            );
            println!("[boot] Step 2: Model preload failed: {}", msg);
            return Err(msg);
        }
    }

    println!("[boot] Boot sequence complete");

    Ok(())
}

// ── Lazy Cerebro start (on-demand) ──

#[tauri::command]
pub async fn start_cerebro(app: AppHandle) -> Result<(), String> {
    println!("[boot] Starting Cerebro on-demand via lazy process manager");

    let _ = app.emit(
        "boot-step",
        &BootStep::running("cerebro", "CEREBRO", "Iniciando Cerebro..."),
    );

    // Start via lazy process manager
    let state = app.state::<Arc<Mutex<super::lazy_process::LazyProcessRegistry>>>();
    {
        let registry = state.lock().await;
        let handle = registry.handles.get("cerebro").ok_or_else(|| {
            "Cerebro not registered in lazy process registry".to_string()
        })?;
        handle.start().await?;
        handle.touch().await;
    }

    // Verify health
    let config = load_config_sync().unwrap_or_else(|e| {
        eprintln!("[boot] Warning: failed to load config, using defaults: {e}");
        Default::default()
    });
    let client = create_client(config.cerebro.timeout_secs)?;
    let cerebro_url = config.cerebro.base_url;

    match check_cerebro_internal(&client, &cerebro_url).await {
        Ok(message) => {
            let _ = app.emit(
                "boot-step",
                &BootStep::ok("cerebro", "CEREBRO", &message),
            );
            println!("[boot] Cerebro started OK: {}", message);
            Ok(())
        }
        Err(msg) => {
            let _ = app.emit("boot-step", &BootStep::error("cerebro", "CEREBRO", &msg));
            println!("[boot] Cerebro start failed: {}", msg);
            Err(msg)
        }
    }
}

// ── Per-step retry ──

#[tauri::command]
pub async fn run_boot_step(app: AppHandle, step_id: String) -> Result<(), String> {
    println!("[boot] Retrying step: {}", step_id);
    let config = load_config_sync().unwrap_or_else(|e| {
        eprintln!("[boot] Warning: failed to load config, using defaults: {e}");
        Default::default()
    });
    let client = create_client(30)?;

    let llm_base_url = config.llm.base_url();

    match step_id.as_str() {
        "llamacpp" => {
            let _ = app.emit(
                "boot-step",
                &BootStep::running("llamacpp", "LLAMACPP", "Verificando..."),
            );
            check_llm_internal(&app, &client, &llm_base_url, &config.llm).await
        }
        "modelo" => {
            let _ = app.emit(
                "boot-step",
                &BootStep::running("modelo", "MODELO", "Pre-cargando modelo..."),
            );
            match check_modelo_internal(&client, &llm_base_url, &config.llm.model_path).await {
                Ok(()) => {
                    let _ = app.emit(
                        "boot-step",
                        &BootStep::ok("modelo", "MODELO", "Modelo pre-cargado"),
                    );
                    Ok(())
                }
                Err(msg) => {
                    let _ = app.emit(
                        "boot-step",
                        &BootStep::error("modelo", "MODELO", &msg),
                    );
                    Err(msg)
                }
            }
        }
        "cerebro" => start_cerebro(app).await,
        _ => Err(format!("Unknown step: {}", step_id)),
    }
}

#[tauri::command]
pub async fn check_cerebro() -> Result<BootStatus, String> {
    let client = create_client(5)?;
    let config = load_config_sync().unwrap_or_else(|e| {
        eprintln!("[boot] Warning: failed to load config, using defaults: {e}");
        Default::default()
    });
    let cerebro_url = config.cerebro.base_url;

    match client
        .get(format!("{}/health", cerebro_url))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("Error parseando JSON: {e}"))?;

                let message = body["message"]
                    .as_str()
                    .unwrap_or("Cerebro operativo");

                Ok(BootStatus {
                    step: "cerebro".to_string(),
                    status: "ok".to_string(),
                    message: message.to_string(),
                })
            } else {
                Ok(BootStatus {
                    step: "cerebro".to_string(),
                    status: "error".to_string(),
                    message: format!("Cerebro respondió con código {}", resp.status()),
                })
            }
        }
        Err(e) => Ok(BootStatus {
            step: "cerebro".to_string(),
            status: "error".to_string(),
            message: format!("Cerebro no disponible: {e}"),
        }),
    }
}
