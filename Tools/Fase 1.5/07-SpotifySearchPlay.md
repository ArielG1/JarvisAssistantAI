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

## Funcionalidad de Cola (FIX 3)

La funcionalidad de cola SÍ se implementó en FIXED.md FIX 3. Se agregó
comando `add_to_spotify_queue` en spotify.rs y detección de intención
"agregar a la fila" en chat.ts.

### Comando Tauri

```rust
#[tauri::command]
pub async fn add_to_spotify_queue(track_id: String, user_access_token: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("https://api.spotify.com/v1/me/player/queue?uri=spotify:track:{}", track_id))
        .header("Authorization", format!("Bearer {}", user_access_token))
        .send().await
        .map_err(|e| e.to_string())?;

    if resp.status().is_success() {
        Ok("Canción agregada a la cola".into())
    } else {
        Err("Error al agregar a la cola".into())
    }
}
```

### Detección en chat

Patrones soportados:
- "agregá X a la fila"
- "sumá X a la cola"
- "poné X después de esta"
- "agrega X a la fila"

### Flujo

1. Detectar intención de agregar a la cola
2. Verificar que Spotify esté disponible
3. Buscar track por nombre
4. Invocar comando `add_to_spotify_queue`
5. Mostrar resultado en chat

### Requisito

Se requiere OAuth de usuario (`user_access_token`) que se obtiene vía
`authorize_spotify_user`. Ver FIXED.md FIX 3 para detalles completos.

## Nota para el futuro (fuera de esta fase)

Controlar volumen y otras funciones avanzadas de reproducción sin abrir
la app manualmente requiere OAuth completo de usuario + Spotify Premium
(Web Playback SDK o Connect API).
