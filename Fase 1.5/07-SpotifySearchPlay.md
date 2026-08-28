# Micro-fase J1.5-07 — Buscar y reproducir en Spotify

## Objetivo

Que JARVIS entienda pedidos tipo "poneme <canción> en Spotify" y la
reproduzca en la app de Spotify ya instalada en la computadora — sin
necesidad de que el usuario final haga login (Client Credentials Flow).

## Prerrequisitos

- Una app registrada en el dashboard de Spotify for Developers (Client
  ID + Client Secret) — gratis, uso personal
- App de Spotify instalada en la máquina (de escritorio)

## Tareas

### T1 — Registrar la app en Spotify for Developers

Paso manual único, fuera de JARVIS: crear la app en
developer.spotify.com, guardar Client ID y Secret en
`jarvis.config.toml` (o mejor, en una variable de entorno si se quiere
evitar tenerlas en texto plano en el archivo).

### T2 — Obtener el token (Client Credentials Flow)

```rust
async fn get_spotify_token(client_id: &str, client_secret: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .form(&[("grant_type", "client_credentials")])
        .basic_auth(client_id, Some(client_secret))
        .send().await
        .map_err(|e| e.to_string())?;
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    body["access_token"].as_str().map(String::from)
        .ok_or_else(|| "Spotify no devolvió token".into())
}
```

El token dura ~1h — cachearlo y renovarlo solo cuando expira, no pedirlo
en cada canción.

### T3 — Buscar el track

```
GET https://api.spotify.com/v1/search?q=<query>&type=track&limit=1
Authorization: Bearer <token>
```

Extraer el `id` del primer resultado.

### T4 — Reproducir (abrir en la app instalada)

```rust
open_url(&format!("spotify:track:{}", track_id))?;
```

Esto dispara el protocolo `spotify:` registrado por la app de escritorio
— no requiere ninguna API de reproducción propia.

### T5 — Detección de intención + respuesta en chat

Mismo patrón que YouTube (J1.5-06): palabras clave ("spotify", "poneme la
canción de..."), y confirmación breve en el chat con el nombre de la
canción/artista encontrado.

### T6 — Verificación

Pedir "poneme <canción> en Spotify" → confirmar que la app de Spotify se
abre (o pasa al frente si ya estaba abierta) y arranca la reproducción
del tema correcto.

## Entregable

Comando "reproducir X en Spotify" funcionando, delegando la reproducción
real a la app de escritorio ya instalada.

## Nota para el futuro (fuera de esta fase)

Controlar reproducción/cola/volumen sin abrir la app manualmente
requeriría OAuth completo de usuario + Spotify Premium (Web Playback SDK
o Connect API) — bastante más trabajo, se deja fuera del alcance de la
Fase 1.5.
