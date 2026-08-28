use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt as tracing_fmt, EnvFilter};

pub fn init_logging() -> WorkerGuard {
    let log_path = get_log_path();

    let file_appender = tracing_appender::rolling::never(&log_path.parent().unwrap_or(&log_path), "jarvis.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Logging inicializado en {:?}", log_path);
    guard
}

fn get_log_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("logs");
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("logs")
}

#[allow(dead_code)]
pub fn log_error(context: &str, error: &str) {
    error!("[{}] {}", context, error);
}

#[allow(dead_code)]
pub fn log_warning(context: &str, message: &str) {
    warn!("[{}] {}", context, message);
}

#[allow(dead_code)]
pub fn log_info(context: &str, message: &str) {
    info!("[{}] {}", context, message);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub delay_ms: u64,
}

#[allow(dead_code)]
impl RetryPolicy {
    pub fn new(max_retries: u32, delay_ms: u64) -> Self {
        Self {
            max_retries,
            delay_ms,
        }
    }

    pub fn exponential_delay(&self, attempt: u32) -> Duration {
        let factor = 2u64.pow(attempt);
        Duration::from_millis(self.delay_ms * factor)
    }
}

#[allow(dead_code)]
pub async fn with_retry<F, Fut, T>(
    policy: &RetryPolicy,
    mut f: F,
) -> Result<T, JarvisError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, JarvisError>>,
{
    let mut last_err = None;

    for attempt in 0..=policy.max_retries {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt < policy.max_retries {
                    let delay = policy.exponential_delay(attempt);
                    log_warning(
                        "retry",
                        &format!(
                            "Intento {}/{} falló: {}. Reintentando en {:?}",
                            attempt + 1,
                            policy.max_retries,
                            e,
                            delay
                        ),
                    );
                    sleep(delay).await;
                }
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        JarvisError::Unknown("Max retries alcanzado".to_string())
    }))
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JarvisError {
    ConnectionTimeout(String),
    ServiceUnavailable(String),
    ParseError(String),
    Unknown(String),
}

impl fmt::Display for JarvisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JarvisError::ConnectionTimeout(service) => {
                write!(f, "Timeout conectando a {}", service)
            }
            JarvisError::ServiceUnavailable(service) => {
                write!(f, "{} no está disponible", service)
            }
            JarvisError::ParseError(detail) => {
                write!(f, "Error de parseo: {}", detail)
            }
            JarvisError::Unknown(msg) => {
                write!(f, "Error desconocido: {}", msg)
            }
        }
    }
}

impl std::error::Error for JarvisError {}

impl From<String> for JarvisError {
    fn from(s: String) -> Self {
        if s.contains("timeout") || s.contains("Timeout") {
            JarvisError::ConnectionTimeout(s)
        } else if s.contains("no disponible") || s.contains("unavailable") {
            JarvisError::ServiceUnavailable(s)
        } else if s.contains("parse") || s.contains("Parse") {
            JarvisError::ParseError(s)
        } else {
            JarvisError::Unknown(s)
        }
    }
}

impl From<reqwest::Error> for JarvisError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            JarvisError::ConnectionTimeout(e.to_string())
        } else if e.is_connect() {
            JarvisError::ServiceUnavailable(e.to_string())
        } else {
            JarvisError::Unknown(e.to_string())
        }
    }
}
