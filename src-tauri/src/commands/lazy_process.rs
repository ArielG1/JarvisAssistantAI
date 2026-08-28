use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyProcessConfig {
    pub name: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    pub idle_timeout_secs: u64,
    #[serde(default)]
    pub healthcheck_url: Option<String>,
    #[serde(default)]
    pub healthcheck_interval_secs: Option<u64>,
}

struct ManagedProcess {
    child: tokio::process::Child,
    last_used: Instant,
}

pub struct LazyProcessManager {
    name: String,
    command: Option<String>,
    args: Vec<String>,
    healthcheck_url: Option<String>,
    #[allow(dead_code)] // kept for healthcheck loop implementation when needed
    healthcheck_interval: Duration,
    idle_timeout: Duration,
    process: Option<ManagedProcess>,
    started: bool,
}

impl LazyProcessManager {
    pub fn new(config: LazyProcessConfig) -> Self {
        let hc_interval = config
            .healthcheck_interval_secs
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(5));

        Self {
            name: config.name,
            command: config.command,
            args: config.args,
            healthcheck_url: config.healthcheck_url,
            healthcheck_interval: hc_interval,
            idle_timeout: Duration::from_secs(config.idle_timeout_secs),
            process: None,
            started: false,
        }
    }

    pub fn is_external(&self) -> bool {
        self.command.is_none()
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if let Some(ref mut mp) = self.process {
            if let Some(status) = mp.child.try_wait().ok().flatten() {
                if !status.success() {
                    warn!(name = %self.name, ?status, "process exited, restarting");
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }

        // External service: no command to spawn, just mark as started
        let Some(ref cmd) = self.command else {
            info!(name = %self.name, "registering external service");
            self.process = None;
            self.started = true;
            return Ok(());
        };

        info!(name = %self.name, cmd = %cmd, "starting process");

        if !std::path::Path::new(cmd).exists() && which::which(cmd).is_err() {
            return Err(format!("command not found: {}", cmd));
        }

        let mut tokio_cmd = tokio::process::Command::new(cmd);
        tokio_cmd.args(&self.args);
        tokio_cmd.kill_on_drop(true);

        let child = tokio_cmd.spawn().map_err(|e| {
            let msg = format!("failed to start {}: {}", self.name, e);
            error!("{}", msg);
            msg
        })?;

        self.process = Some(ManagedProcess {
            child,
            last_used: Instant::now(),
        });
        self.started = true;

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if self.is_external() {
            info!(name = %self.name, "unregistering external service");
            self.process = None;
            self.started = false;
            return Ok(());
        }
        if let Some(mut mp) = self.process.take() {
            info!(name = %self.name, "stopping process");
            let _ = mp.child.kill().await;
            self.started = false;
        }
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if self.is_external() {
            return self.started;
        }
        if let Some(ref mut mp) = self.process {
            match mp.child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub async fn healthcheck(&mut self) -> Result<bool, String> {
        if !self.is_running() {
            return Ok(false);
        }

        if let Some(ref url) = self.healthcheck_url {
            match reqwest::get(url).await {
                Ok(resp) => Ok(resp.status().is_success()),
                Err(e) => {
                    warn!(name = %self.name, ?e, "healthcheck failed");
                    Ok(false)
                }
            }
        } else {
            Ok(true)
        }
    }

    pub fn touch(&mut self) {
        if let Some(ref mut mp) = self.process {
            mp.last_used = Instant::now();
        }
    }

    pub fn idle_duration(&self) -> Option<Duration> {
        self.process.as_ref().map(|mp| mp.last_used.elapsed())
    }

    pub async fn full_status(&mut self) -> (bool, bool, Option<Duration>) {
        let running = self.is_running();
        let healthy = if running {
            self.healthcheck().await.unwrap_or(false)
        } else {
            false
        };
        let idle = self.idle_duration();
        (running, healthy, idle)
    }

    #[allow(dead_code)] // used in tests; kept for API completeness
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn check_and_stop_if_idle(&mut self) -> bool {
        if let Some(idle) = self.idle_duration() {
            if idle >= self.idle_timeout {
                info!(
                    name = %self.name,
                    idle_secs = idle.as_secs(),
                    "idle timeout reached, stopping"
                );
                let _ = self.stop().await;
                return true;
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct LazyProcessHandle {
    inner: Arc<Mutex<LazyProcessManager>>,
}

impl LazyProcessHandle {
    pub fn new(config: LazyProcessConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LazyProcessManager::new(config))),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut mgr = self.inner.lock().await;
        mgr.start().await
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut mgr = self.inner.lock().await;
        mgr.stop().await
    }

    pub async fn is_running(&self) -> bool {
        let mut mgr = self.inner.lock().await;
        mgr.is_running()
    }

    pub async fn healthcheck(&self) -> Result<bool, String> {
        let mut mgr = self.inner.lock().await;
        mgr.healthcheck().await
    }

    pub async fn touch(&self) {
        let mut mgr = self.inner.lock().await;
        mgr.touch();
    }

    pub async fn get_status(&self) -> (bool, bool, Option<Duration>) {
        let mut mgr = self.inner.lock().await;
        mgr.full_status().await
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LazyProcessStatus {
    pub name: String,
    pub running: bool,
    pub healthy: bool,
    pub idle_secs: u64,
}

pub struct IdleWatcher {
    handles: Vec<LazyProcessHandle>,
    shutdown_tx: broadcast::Sender<()>,
}

impl IdleWatcher {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self { handles: Vec::new(), shutdown_tx }
    }

    pub fn register(&mut self, handle: LazyProcessHandle) {
        self.handles.push(handle);
    }

    pub async fn start_watching(&self, _app: AppHandle) {
        let handles = self.handles.clone();
        let mut rx = self.shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        for handle in &handles {
                            let mut mgr = handle.inner.lock().await;
                            let _ = mgr.check_and_stop_if_idle().await;
                        }
                    }
                    _ = rx.recv() => {
                        break;
                    }
                }
            }
        });
    }

    #[allow(dead_code)] // kept for public API; IdleWatcher shutdown channel wired in start_watching
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

// --- Tauri Commands ---

pub struct LazyProcessRegistry {
    pub handles: std::collections::HashMap<String, LazyProcessHandle>,
}

pub fn init_registry() -> LazyProcessRegistry {
    LazyProcessRegistry {
        handles: std::collections::HashMap::new(),
    }
}

pub fn register_process(registry: &mut LazyProcessRegistry, config: LazyProcessConfig) {
    let name = config.name.clone();
    let handle = LazyProcessHandle::new(config);
    registry.handles.insert(name, handle);
}

#[tauri::command]
pub async fn lazy_start(
    app: AppHandle,
    name: String,
) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let handle = registry
        .handles
        .get(&name)
        .ok_or_else(|| format!("process '{}' not registered", name))?;
    handle.start().await?;
    handle.touch().await;
    Ok(())
}

#[tauri::command]
pub async fn lazy_stop(
    app: AppHandle,
    name: String,
) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let handle = registry
        .handles
        .get(&name)
        .ok_or_else(|| format!("process '{}' not registered", name))?;
    handle.stop().await
}

#[tauri::command]
pub async fn lazy_is_running(
    app: AppHandle,
    name: String,
) -> Result<bool, String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let handle = registry
        .handles
        .get(&name)
        .ok_or_else(|| format!("process '{}' not registered", name))?;
    Ok(handle.is_running().await)
}

#[tauri::command]
pub async fn lazy_healthcheck(
    app: AppHandle,
    name: String,
) -> Result<bool, String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let handle = registry
        .handles
        .get(&name)
        .ok_or_else(|| format!("process '{}' not registered", name))?;
    handle.healthcheck().await
}

#[tauri::command]
pub async fn lazy_touch(
    app: AppHandle,
    name: String,
) -> Result<(), String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let handle = registry
        .handles
        .get(&name)
        .ok_or_else(|| format!("process '{}' not registered", name))?;
    handle.touch().await;
    Ok(())
}

#[tauri::command]
pub async fn lazy_get_status(
    app: AppHandle,
    name: String,
) -> Result<LazyProcessStatus, String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let handle = registry
        .handles
        .get(&name)
        .ok_or_else(|| format!("process '{}' not registered", name))?;
    let (running, healthy, idle) = handle.get_status().await;
    Ok(LazyProcessStatus {
        name: name.clone(),
        running,
        healthy,
        idle_secs: idle.map(|d| d.as_secs()).unwrap_or(0),
    })
}

#[tauri::command]
pub async fn lazy_list(
    app: AppHandle,
) -> Result<Vec<LazyProcessStatus>, String> {
    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    let mut statuses = Vec::new();
    for (name, handle) in &registry.handles {
        let (running, healthy, idle) = handle.get_status().await;
        statuses.push(LazyProcessStatus {
            name: name.clone(),
            running,
            healthy,
            idle_secs: idle.map(|d| d.as_secs()).unwrap_or(0),
        });
    }
    Ok(statuses)
}
