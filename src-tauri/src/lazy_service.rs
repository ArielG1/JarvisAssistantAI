use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

pub struct LazyService {
    name: String,
    command: String,
    args: Vec<String>,
    working_dir: Option<String>,
    idle_timeout: Duration,
    process: Arc<Mutex<Option<Child>>>,
    last_activity: Arc<Mutex<Instant>>,
    running: Arc<Mutex<bool>>,
    watcher_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Clone for LazyService {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            working_dir: self.working_dir.clone(),
            idle_timeout: self.idle_timeout,
            process: Arc::clone(&self.process),
            last_activity: Arc::clone(&self.last_activity),
            running: Arc::clone(&self.running),
            watcher_handle: Arc::clone(&self.watcher_handle),
        }
    }
}

impl LazyService {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn working_dir(&self) -> Option<&str> {
        self.working_dir.as_deref()
    }

    pub fn idle_timeout(&self) -> Duration {
        self.idle_timeout
    }

    pub fn set_idle_timeout(&mut self, timeout: Duration) {
        self.idle_timeout = timeout;
    }

    pub fn new(
        name: String,
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
        idle_timeout: Duration,
    ) -> Self {
        Self {
            name,
            command,
            args,
            working_dir,
            idle_timeout,
            process: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            running: Arc::new(Mutex::new(false)),
            watcher_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn ensure_running(&self) -> Result<(), String> {
        let mut running = self.running.lock().await;
        if *running {
            let mut proc_guard = self.process.lock().await;
            if let Some(ref mut child) = *proc_guard {
                match child.try_wait() {
                    Ok(None) => {
                        drop(proc_guard);
                        *self.last_activity.lock().await = Instant::now();
                        return Ok(());
                    }
                    _ => {
                        info!(name = %self.name, "process exited, restarting");
                        *running = false;
                    }
                }
            }
        }

        self.cancel_watcher().await;

        info!(name = %self.name, cmd = %self.command, "starting process");

        let mut cmd = tokio::process::Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.kill_on_drop(true);

        let child = cmd.spawn().map_err(|e| {
            let msg = format!("failed to start {}: {}", self.name, e);
            error!("{}", msg);
            msg
        })?;

        *self.process.lock().await = Some(child);
        *running = true;
        *self.last_activity.lock().await = Instant::now();

        self.start_idle_watcher().await;

        info!(name = %self.name, "process started");
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let mut running = self.running.lock().await;
        if !*running {
            return false;
        }
        let mut proc_guard = self.process.lock().await;
        if let Some(ref mut child) = *proc_guard {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *running = false;
                    false
                }
            }
        } else {
            false
        }
    }

    pub async fn touch(&self) {
        *self.last_activity.lock().await = Instant::now();
    }

    pub async fn idle_duration(&self) -> Duration {
        self.last_activity.lock().await.elapsed()
    }

    pub async fn check_and_stop_if_idle(&self) -> bool {
        let idle = self.idle_duration().await;
        if idle >= self.idle_timeout {
            info!(
                name = %self.name,
                idle_secs = idle.as_secs(),
                "idle timeout reached, stopping"
            );
            self.stop().await;
            return true;
        }
        false
    }

    pub async fn stop(&self) {
        self.cancel_watcher().await;
        let mut running = self.running.lock().await;
        if let Some(mut child) = self.process.lock().await.take() {
            info!(name = %self.name, "stopping process");
            let _ = child.kill().await;
        }
        *running = false;
    }

    pub async fn healthcheck(&self, url: &str) -> Result<bool, String> {
        if !self.is_running().await {
            return Ok(false);
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            warn!(name = %self.name, url = %url, "healthcheck rejected: invalid URL scheme");
            return Err(format!("invalid URL scheme, only http/https allowed: {}", url));
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {}", e))?;

        match client.get(url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => {
                warn!(name = %self.name, ?e, "healthcheck failed");
                Ok(false)
            }
        }
    }

    async fn cancel_watcher(&self) {
        if let Some(handle) = self.watcher_handle.lock().await.take() {
            handle.abort();
        }
    }

    pub async fn start_idle_watcher(&self) {
        self.cancel_watcher().await;

        let name = self.name.clone();
        let process = Arc::clone(&self.process);
        let last_activity = Arc::clone(&self.last_activity);
        let running = Arc::clone(&self.running);
        let idle_timeout = self.idle_timeout;

        // Poll at idle_timeout / 4, floored to 1s to avoid hot-looping on
        // very short idle timeouts (e.g. 1s → poll every 1s, not 250ms).
        let handle = tokio::spawn(async move {
            let poll_interval = std::cmp::max(idle_timeout / 4, Duration::from_secs(1));

            loop {
                tokio::time::sleep(poll_interval).await;

                let is_alive = {
                    let mut proc_guard = process.lock().await;
                    if let Some(ref mut child) = *proc_guard {
                        matches!(child.try_wait(), Ok(None))
                    } else {
                        false
                    }
                };

                if !is_alive {
                    info!(name = %name, "process already dead, stopping watcher");
                    *running.lock().await = false;
                    break;
                }

                let idle = last_activity.lock().await.elapsed();
                if idle >= idle_timeout {
                    info!(
                        name = %name,
                        idle_secs = idle.as_secs(),
                        "idle timeout reached, killing process"
                    );
                    if let Some(mut child) = process.lock().await.take() {
                        let _ = child.kill().await;
                    }
                    *running.lock().await = false;
                    break;
                }
            }
        });

        *self.watcher_handle.lock().await = Some(handle);
    }
}

/// NOT thread-safe by itself. Callers must wrap in `Arc<Mutex<...>>`
/// (as done in `lib.rs`) before sharing across tasks/threads.
pub struct LazyServiceRegistry {
    services: HashMap<String, LazyService>,
}

impl LazyServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, service: LazyService) {
        self.services.insert(name, service);
    }

    pub fn get(&self, name: &str) -> Option<&LazyService> {
        self.services.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LazyService> {
        self.services.get_mut(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    pub fn has(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_ensure_running_and_stop() {
        let port = 18765;
        let svc = LazyService::new(
            "test-http".into(),
            "python".into(),
            vec![
                "-m".into(),
                "http.server".into(),
                port.to_string(),
            ],
            None,
            Duration::from_secs(5),
        );

        // Start
        svc.ensure_running().await.unwrap();
        assert!(svc.is_running().await, "process should be running");

        // Verify it responds over HTTP (retry a few times while server warms up)
        let url = format!("http://127.0.0.1:{}/", port);
        let mut responded = false;
        for _ in 0..10 {
            if reqwest::get(&url).await.is_ok() {
                responded = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(responded, "http.server should respond");

        // Stop
        svc.stop().await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!svc.is_running().await, "process should be dead after stop");
    }

    #[tokio::test]
    async fn test_no_duplicate_on_double_ensure() {
        let port = 18767;
        let svc = LazyService::new(
            "test-no-dup".into(),
            "python".into(),
            vec![
                "-m".into(),
                "http.server".into(),
                port.to_string(),
            ],
            None,
            Duration::from_secs(30),
        );

        // First ensure_running spawns the process
        svc.ensure_running().await.unwrap();
        assert!(svc.is_running().await, "should be running after first ensure");

        // Second ensure_running should be a no-op (idempotent)
        svc.ensure_running().await.unwrap();
        assert!(
            svc.is_running().await,
            "should still be running after second ensure"
        );

        // Verify the HTTP server is responsive (single process, not duplicate)
        let url = format!("http://127.0.0.1:{}/", port);
        let mut responded = false;
        for _ in 0..10 {
            if reqwest::get(&url).await.is_ok() {
                responded = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(
            responded,
            "http.server should respond — single process alive"
        );

        svc.stop().await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!svc.is_running().await, "process should be dead after stop");
    }

    #[tokio::test]
    async fn test_ensure_running_idempotent() {
        let port = 18768;
        let svc = LazyService::new(
            "test-idempotent".into(),
            "python".into(),
            vec![
                "-m".into(),
                "http.server".into(),
                port.to_string(),
            ],
            None,
            Duration::from_secs(30),
        );

        // Stop first to ensure clean state
        svc.stop().await;
        assert!(!svc.is_running().await);

        // Call ensure_running three times — all should succeed
        svc.ensure_running().await.unwrap();
        svc.ensure_running().await.unwrap();
        svc.ensure_running().await.unwrap();
        assert!(
            svc.is_running().await,
            "should be running after multiple ensure calls"
        );

        // Verify HTTP server responds (single process)
        let url = format!("http://127.0.0.1:{}/", port);
        let mut responded = false;
        for _ in 0..10 {
            if reqwest::get(&url).await.is_ok() {
                responded = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(responded, "single process should respond to HTTP");

        svc.stop().await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!svc.is_running().await);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = LazyServiceRegistry::new();
        assert!(!registry.has("cerebro"));

        let svc = LazyService::new(
            "cerebro".into(),
            "echo".into(),
            vec!["hello".into()],
            None,
            Duration::from_secs(60),
        );

        registry.register("cerebro".into(), svc);
        assert!(registry.has("cerebro"));
        assert!(registry.get("cerebro").is_some());

        let retrieved = registry.get("cerebro").unwrap();
        assert_eq!(retrieved.name(), "cerebro");
        assert_eq!(retrieved.command(), "echo");
        assert_eq!(retrieved.args(), &["hello"]);
        assert_eq!(retrieved.idle_timeout(), Duration::from_secs(60));

        let names = registry.list();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"cerebro".to_string()));

        // get_mut works too
        let svc_mut = registry.get_mut("cerebro").unwrap();
        svc_mut.set_idle_timeout(Duration::from_secs(30));
        assert_eq!(
            registry.get("cerebro").unwrap().idle_timeout(),
            Duration::from_secs(30)
        );
    }

    #[tokio::test]
    async fn test_idle_watcher_kills_process() {
        let port = 18766;
        let svc = LazyService::new(
            "test-idle".into(),
            "python".into(),
            vec![
                "-m".into(),
                "http.server".into(),
                port.to_string(),
            ],
            None,
            Duration::from_secs(1), // very short idle timeout
        );

        svc.ensure_running().await.unwrap();
        assert!(svc.is_running().await);

        // Let the idle watcher kill it (timeout = 1s, poll = max(1s/4,1s) = 1s)
        // Wait up to 5s for the watcher to act
        for _ in 0..10 {
            if !svc.is_running().await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        assert!(
            !svc.is_running().await,
            "idle watcher should have killed the process"
        );
    }
}
