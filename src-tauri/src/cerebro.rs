use std::sync::{Arc, LazyLock};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Manager;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::commands::config::{load_config, JarvisConfig};
use crate::commands::lazy_process::LazyProcessRegistry;
use crate::commands::searxng::{self, SearchResult};

pub(crate) static CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP compartido: {e}"))
});

pub(crate) fn get_client() -> Result<&'static Client, String> {
    CLIENT.as_ref().map_err(|e| e.clone())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CerebroQueryRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CerebroQueryResponse {
    pub response: String,
    pub status: String,
}

pub async fn ask_cerebro(
    app: &AppHandle,
    query: String,
    config: Option<JarvisConfig>,
    history: Option<Vec<(String, String)>>,
) -> Result<String, String> {
    let config = match config {
        Some(c) => c,
        None => load_config().await.unwrap_or_default(),
    };

    let state = app.state::<Arc<Mutex<LazyProcessRegistry>>>();
    let registry = state.lock().await;
    if let Some(handle) = registry.handles.get("cerebro") {
        handle.start().await?;
        handle.touch().await;
    }

    let url = format!("{}/api/query", config.cerebro.base_url);

    let messages = history.map(|hist| {
        hist.into_iter()
            .map(|(role, content)| serde_json::json!({ "role": role, "content": content }))
            .collect::<Vec<_>>()
    });
    let body = CerebroQueryRequest { query, messages };

    let client = match get_client() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to initialize HTTP client");
            return Err(e);
        }
    };

    let response = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = if e.is_timeout() {
                error!(
                    endpoint = %url,
                    error = %e,
                    "Timeout conectando con Cerebro"
                );
                "Tiempo de espera agotado al conectar con Cerebro"
            } else if e.is_connect() {
                error!(
                    endpoint = %url,
                    error = %e,
                    "No se pudo conectar con Cerebro"
                );
                "No se pudo conectar con Cerebro"
            } else {
                error!(
                    endpoint = %url,
                    error = %e,
                    "Error de red al conectar con Cerebro"
                );
                "Error de red al conectar con Cerebro"
            };
            return Err(err_msg.into());
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        error!(
            endpoint = %url,
            status = %status,
            "Cerebro respondió con error HTTP"
        );
        return Err(format!("Error del servidor de Cerebro: {}", status));
    }

    let data: CerebroQueryResponse = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            error!(
                endpoint = %url,
                error = %e,
                "Respuesta JSON inválida de Cerebro"
            );
            return Err("Respuesta inválida de Cerebro".to_string());
        }
    };

    Ok(data.response)
}

pub async fn ask_llm(query: &str, history: Option<Vec<(String, String)>>) -> Result<String, String> {
    let cfg = load_config().await.unwrap_or_default();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let mut messages: Vec<serde_json::Value> = Vec::new();
    if let Some(hist) = &history {
        for (role, content) in hist {
            messages.push(serde_json::json!({
                "role": role,
                "content": content,
            }));
        }
    }
    messages.push(serde_json::json!({
        "role": "user",
        "content": query,
    }));

    let resp = client
        .post(format!("{}/v1/chat/completions", cfg.llm.base_url()))
        .json(&serde_json::json!({
            "messages": messages,
            "stream": false
        }))
        .send()
        .await
        .map_err(|e| format!("llama-server no responde: {e}"))?;
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    body["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "Respuesta inválida de llama-server".into())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CerebroFallbackResponse {
    pub response: String,
    pub web_search_used: bool,
    pub search_results: Vec<SearchResult>,
}

pub async fn ask_cerebro_with_fallback(
    app: &tauri::AppHandle,
    query: String,
    config: Option<JarvisConfig>,
    history: Option<Vec<(String, String)>>,
) -> Result<CerebroFallbackResponse, String> {
    let config = match config {
        Some(c) => c,
        None => load_config().await.unwrap_or_default(),
    };

    let fallback_cfg = &config.web_search_fallback;

    // Nivel 1: Cerebro directo
    match ask_cerebro(app, query.clone(), Some(config.clone()), history.clone()).await {
        Ok(response) => {
            return Ok(CerebroFallbackResponse {
                response,
                web_search_used: false,
                search_results: vec![],
            });
        }
        Err(e) => {
            warn!("Cerebro no disponible ({}), intentando fallback", e);
        }
    }

    // Nivel 2: Web search + Cerebro
    let needs_search = searxng::should_trigger_web_search(&query, fallback_cfg);
    if needs_search {
        info!("web search fallback triggered for query: {}", &query[..query.len().min(80)]);
        let search_result = searxng::search_web_for_context(app, &query, fallback_cfg).await;

        match search_result {
            Ok((context, results)) if !context.is_empty() => {
                let enhanced_query = format!(
                    "{}\n\nUsa la siguiente información de la web para responder:\n{}",
                    query, context
                );
                match ask_cerebro(app, enhanced_query, Some(config.clone()), history.clone()).await {
                    Ok(response) => {
                        return Ok(CerebroFallbackResponse {
                            response,
                            web_search_used: true,
                            search_results: results,
                        });
                    }
                    Err(e) => {
                        warn!("Cerebro falló tras web search ({}), probando LLM local", e);
                    }
                }
            }
            Ok(_) => {
                info!("web search returned no results");
            }
            Err(e) => {
                error!("web search failed: {}", e);
            }
        }
    }

    // Nivel 3: LLM local vía llama-server
    info!("Fallback a LLM local para query: {}", &query[..query.len().min(80)]);
    match ask_llm(&query, history).await {
        Ok(response) => Ok(CerebroFallbackResponse {
            response,
            web_search_used: false,
            search_results: vec![],
        }),
        Err(e) => {
            error!("Todos los fallbacks fallaron: LLM local error: {}", e);
            Err(format!("Todos los fallbacks fallaron: {}", e))
        }
    }
}
