use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::error;

use super::config::JarvisConfig;
use crate::cerebro::get_client;

#[derive(Debug, Serialize, Deserialize)]
pub struct CerebroResponse {
    pub response: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChatHistoryMessage {
    pub role: String,
    pub content: String,
}

#[tauri::command]
pub async fn query_cerebro(
    app: AppHandle,
    query: String,
    config: Option<JarvisConfig>,
) -> Result<CerebroResponse, String> {
    let response = crate::cerebro::ask_cerebro(&app, query, config, None).await?;
    Ok(CerebroResponse {
        response,
        status: "ok".to_string(),
    })
}

#[tauri::command]
pub async fn ask_llm(query: String) -> Result<CerebroResponse, String> {
    let response = crate::cerebro::ask_llm(&query, None).await?;
    Ok(CerebroResponse {
        response,
        status: "ok".to_string(),
    })
}

#[tauri::command]
pub async fn check_cerebro_health(config: Option<JarvisConfig>) -> Result<bool, String> {
    let cfg = config.unwrap_or_default();
    let url = format!("{}/health", cfg.cerebro.base_url);

    let client = get_client().map_err(|e| format!("HTTP client init failed: {e}"))?;
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(cfg.cerebro.timeout_secs))
        .send()
        .await
        .map_err(|e| format!("Error conectando a Cerebro: {}", e))?;

    Ok(response.status() == 200)
}

#[tauri::command]
pub async fn send_to_cerebro(app: AppHandle, message: String) -> Result<String, String> {
    match crate::cerebro::ask_cerebro(&app, message.clone(), None, None).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!(
                message_len = message.len(),
                error = %e,
                "send_to_cerebro falló"
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn send_to_cerebro_with_fallback(
    app: AppHandle,
    message: String,
    history: Option<Vec<ChatHistoryMessage>>,
) -> Result<crate::cerebro::CerebroFallbackResponse, String> {
    let hist = history.map(|h| h.into_iter().map(|m| (m.role, m.content)).collect());
    crate::cerebro::ask_cerebro_with_fallback(&app, message, None, hist).await
}
