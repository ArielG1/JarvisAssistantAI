use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::commands::config::{load_config_sync, save_config_sync};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyToken {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackResult {
    pub name: String,
    pub artist: String,
    pub uri: String,
    pub preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifySearchResponse {
    tracks: Option<SpotifyTracks>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTracks {
    items: Vec<SpotifyTrack>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrack {
    name: String,
    uri: String,
    artists: Vec<SpotifyArtist>,
    preview_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtist {
    name: String,
}

struct TokenCache {
    token: Option<SpotifyToken>,
    obtained_at: std::time::Instant,
}

impl TokenCache {
    fn new() -> Self {
        Self {
            token: None,
            obtained_at: std::time::Instant::now(),
        }
    }

    fn is_valid(&self) -> bool {
        if let Some(ref t) = self.token {
            self.obtained_at.elapsed().as_secs() < t.expires_in.saturating_sub(60)
        } else {
            false
        }
    }
}

static TOKEN_CACHE: once_cell::sync::Lazy<Arc<Mutex<TokenCache>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(TokenCache::new())));

#[derive(Debug, Serialize, Deserialize)]
struct UserTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

async fn refresh_user_token(refresh_token: &str) -> Result<(String, String), String> {
    let config = load_config_sync().map_err(|e| format!("Config load error: {}", e))?;
    let spotify = config.spotify;

    let client = Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", &spotify.client_id),
    ];

    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed ({}): {}", status, body));
    }

    let token_data: UserTokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token refresh response: {}", e))?;

    let new_access = token_data.access_token;
    let new_refresh = token_data.refresh_token.unwrap_or_else(|| refresh_token.to_string());

    let mut config = load_config_sync().map_err(|e| format!("Config load error: {}", e))?;
    config.spotify.user_access_token = new_access.clone();
    config.spotify.user_refresh_token = new_refresh.clone();
    save_config_sync(&config).map_err(|e| format!("Failed to save refreshed tokens: {}", e))?;

    Ok((new_access, new_refresh))
}

async fn get_user_access_token() -> Result<String, String> {
    let config = load_config_sync().map_err(|e| e.to_string())?;
    if config.spotify.user_access_token.is_empty() {
        return Err("Spotify user not authorized. Run `authorize_spotify_user` first.".into());
    }
    Ok(config.spotify.user_access_token)
}

async fn get_access_token() -> Result<String, String> {
    let mut cache = TOKEN_CACHE.lock().await;
    if cache.is_valid() {
        return Ok(cache.token.as_ref().expect("token validity checked by is_valid()").access_token.clone());
    }

    let config = load_config_sync().map_err(|e| format!("Config load error: {}", e))?;
    let spotify = config.spotify;

    if !spotify.enabled {
        return Err("Spotify está deshabilitado en la configuración".into());
    }
    if spotify.client_id.is_empty() || spotify.client_secret.is_empty() {
        return Err("Spotify client_id/client_secret no configurados en jarvis.config.toml".into());
    }

    let client = Client::new();
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &spotify.client_id),
        ("client_secret", &spotify.client_secret),
    ];

    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Spotify token request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Spotify auth failed ({}): {}", status, body));
    }

    let token: SpotifyToken = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Spotify token: {}", e))?;

    let result = token.access_token.clone();
    cache.token = Some(token);
    cache.obtained_at = std::time::Instant::now();

    Ok(result)
}

#[tauri::command]
pub async fn is_spotify_available() -> Result<bool, String> {
    let home = dirs::home_dir().ok_or("Cannot find home dir")?;
    let paths = [
        home.join("AppData/Roaming/Spotify/Spotify.exe"),
        home.join("AppData/Local/Packages/SpotifyAB.SpotifyMusic_zpdnekdrzrea0/LocalCache/Local/Spotify.exe"),
    ];
    for path in &paths {
        if path.exists() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[tauri::command]
pub async fn search_spotify(query: String) -> Result<Vec<TrackResult>, String> {
    let token = get_access_token().await?;
    let client = Client::new();

    let encoded_query = urlencoding::encode(&query);
    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=3",
        encoded_query
    );

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Spotify search request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Spotify search failed ({}): {}", status, body));
    }

    let search: SpotifySearchResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Spotify search response: {}", e))?;

    let tracks = search
        .tracks
        .map(|t| t.items)
        .unwrap_or_default();

    let results: Vec<TrackResult> = tracks
        .into_iter()
        .map(|t| TrackResult {
            name: t.name,
            artist: t
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            uri: t.uri,
            preview_url: t.preview_url,
        })
        .collect();

    if results.is_empty() {
        return Err(format!("No se encontraron canciones para: {}", query));
    }

    Ok(results)
}

const SPOTIFY_SCOPES: &str = "user-modify-playback-state user-read-playback-state";

#[tauri::command]
pub async fn authorize_spotify_user() -> Result<String, String> {
    let config = load_config_sync().map_err(|e| e.to_string())?;
    let spotify = config.spotify;

    if spotify.client_id.is_empty() {
        return Err("Spotify client_id not configured".into());
    }

    let redirect_uri = "http://127.0.0.1:8889/callback";
    let auth_url = format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&show_dialog=true",
        spotify.client_id,
        urlencoding::encode(redirect_uri),
        urlencoding::encode(SPOTIFY_SCOPES),
    );

    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    let listener = TcpListener::bind("127.0.0.1:8889")
        .map_err(|e| format!("Failed to start callback server on port 8889: {}", e))?;
    listener.set_nonblocking(true).ok();

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    let handle = std::thread::spawn(move || {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed().as_secs() > 120 {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let reader = BufReader::new(stream.try_clone().unwrap());
                    let mut path = String::new();
                    for line in reader.lines() {
                        let line = line.unwrap_or_default();
                        if line.starts_with("GET ") {
                            path = line.split_whitespace().nth(1).unwrap_or("").to_string();
                            break;
                        }
                    }

                    let code = if path.starts_with("/callback") {
                        let query = path.split('?').nth(1).unwrap_or("");
                        let params: Vec<&str> = query.split('&').collect();
                        params.iter()
                            .find(|p| p.starts_with("code="))
                            .map(|p| p[5..].to_string())
                    } else {
                        None
                    };

                    let body = match code {
                        Some(_) => "<html><body><h2>Autorizado. Puedes cerrar esta pestana.</h2></body></html>",
                        None => {
                            if path.contains("error=") {
                                "<html><body><h2>Error en la autorizacion.</h2></body></html>"
                            } else {
                                "<html><body><h2>Esperando autorizacion...</h2></body></html>"
                            }
                        }
                    };

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());

                    if let Some(c) = code {
                        let _ = tx.send(c);
                        break;
                    }
                }
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    });

    let code = rx.recv().map_err(|_| "Authorization timed out (120s)".to_string())?;
    let _ = handle.join();

    let client = Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", redirect_uri),
        ("client_id", &spotify.client_id),
        ("client_secret", &spotify.client_secret),
    ];

    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed ({}): {}", status, body));
    }

    let token_data: UserTokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let mut config = load_config_sync().map_err(|e| e.to_string())?;
    config.spotify.user_access_token = token_data.access_token;
    if let Some(refresh) = token_data.refresh_token {
        config.spotify.user_refresh_token = refresh;
    }
    save_config_sync(&config).map_err(|e| format!("Failed to save tokens: {}", e))?;

    Ok("Spotify user authorized successfully".into())
}

#[tauri::command]
pub async fn play_spotify(query: String) -> Result<String, String> {
    let results = search_spotify(query).await?;
    let top = &results[0];
    let track_id = top.uri.strip_prefix("spotify:track:").unwrap_or(&top.uri);
    let track_uri = format!("spotify:track:{}", track_id);

    let config = load_config_sync().map_err(|e| e.to_string())?;
    if !config.spotify.user_access_token.is_empty() {
        let client = Client::new();
        let resp = client
            .put("https://api.spotify.com/v1/me/player/play")
            .header("Authorization", format!("Bearer {}", config.spotify.user_access_token))
            .json(&serde_json::json!({ "uris": [&track_uri] }))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 204 => {
                return Ok(format!("Reproduciendo: {} - {}", top.name, top.artist));
            }
            Ok(r) if r.status().as_u16() == 401 => {
                if !config.spotify.user_refresh_token.is_empty() {
                    if let Ok((new_access, _)) = refresh_user_token(&config.spotify.user_refresh_token).await {
                        let client = Client::new();
                        let retry = client
                            .put("https://api.spotify.com/v1/me/player/play")
                            .header("Authorization", format!("Bearer {}", new_access))
                            .json(&serde_json::json!({ "uris": [&track_uri] }))
                            .send()
                            .await;
                        if let Ok(r) = retry {
                            if r.status().is_success() || r.status().as_u16() == 204 {
                                return Ok(format!("Reproduciendo: {} - {}", top.name, top.artist));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    open::that(&track_uri).map_err(|e| format!("Failed to open Spotify URI: {}", e))?;
    Ok(format!(
        "Reproduciendo: {} - {} en Spotify",
        top.name, top.artist
    ))
}
