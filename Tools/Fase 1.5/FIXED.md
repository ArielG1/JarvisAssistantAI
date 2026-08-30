# FIXED.md — Exact implementation instructions for JarvisAssistantAI

## READ THIS FIRST (rules for the coding assistant applying these fixes)

You are applying a fixed, pre-reviewed set of changes to an existing
Rust (Tauri) + Vue 3 + Pinia project. Follow these rules exactly:

1. Apply the fixes **in order, one at a time** (FIX 1, then FIX 2, etc.).
   Do not skip ahead and do not combine fixes.
2. Every fix below gives you an exact **"FIND THIS EXACT TEXT"** block
   and an exact **"REPLACE WITH THIS EXACT TEXT"** block, OR says
   **"CREATE NEW FILE"** with the full file content.
3. Before editing, open the target file and locate the FIND block
   **character-for-character**. If you cannot find it exactly (even a
   whitespace difference), **STOP and report the mismatch** — do not
   guess, do not "fix it anyway" with different code, do not invent
   your own version of the change.
4. Do not modify, reformat, rename, or "improve" any code outside the
   exact blocks shown. Do not add comments, semicolons, or imports that
   are not explicitly listed.
5. After each fix, run the build command listed in that fix's
   "VERIFY" step before moving to the next fix. If the build fails,
   stop and report the exact compiler/linter error — do not attempt to
   silently patch around it with unrelated changes.
6. If a fix references a helper function that must be added, add it in
   the exact location specified (right after/before the anchor text
   given), not at a location you choose yourself.

---

## FIX 1 — SearXNG: real restart bug + start it once at boot (always on)

### Why

SearXNG is only ~512MB, so it should just stay running, started once
when the app boots, instead of being started/stopped on demand.

There is also a real bug: once the SearXNG Docker container has been
started successfully one time, the internal `LazyProcessManager` never
notices if that container later dies (crash, manual `docker stop`,
Docker Desktop restart). This happens because `docker run -d` detaches
immediately, so the tracked child process exits with a "success" status
within a fraction of a second of starting the container — and the code
treats "child process already exited successfully" as "nothing to do,
already fine", so it never issues a new `docker run` even when the real
container is confirmed gone by `docker inspect`. This is very likely the
cause of "SearXNG search fails intermittently" reported earlier.

### File: `src-tauri/src/commands/lazy_process.rs`

**Step 1.1 — Add a `reset` method to `LazyProcessManager`.**

FIND THIS EXACT TEXT:
```rust
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
```

REPLACE WITH THIS EXACT TEXT:
```rust
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

    /// Forces the manager to forget its tracked child process without
    /// touching the real underlying service (e.g. a detached Docker
    /// container). Use this before re-attempting `start()` when an
    /// external check (like `docker inspect`) has confirmed the real
    /// service is not running, but the manager's own state does not
    /// reflect that yet.
    pub fn reset(&mut self) {
        self.process = None;
        self.started = false;
    }
}
```

**Step 1.2 — Add a `reset` method to `LazyProcessHandle`.**

FIND THIS EXACT TEXT:
```rust
    pub async fn touch(&self) {
        let mut mgr = self.inner.lock().await;
        mgr.touch();
    }
```

REPLACE WITH THIS EXACT TEXT:
```rust
    pub async fn touch(&self) {
        let mut mgr = self.inner.lock().await;
        mgr.touch();
    }

    /// See `LazyProcessManager::reset`.
    pub async fn reset(&self) {
        let mut mgr = self.inner.lock().await;
        mgr.reset();
    }
```

### File: `src-tauri/src/commands/searxng.rs`

**Step 1.3 — Make `ensure_running` public so it can be called from `lib.rs` at boot.**

FIND THIS EXACT TEXT:
```rust
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
```

REPLACE WITH THIS EXACT TEXT:
```rust
pub async fn ensure_running(app: &AppHandle) -> Result<(), String> {
    let mgr_arc = get_manager(app)?;
    let mgr = mgr_arc.lock().await;
    let port = mgr.config.port;
    drop(mgr);

    ensure_docker_available().await?;

    let container_confirmed_running = is_container_running().await.unwrap_or(false);

    if container_confirmed_running {
        match wait_for_health(port, 5).await {
            Ok(()) => {
                info!("SearXNG container already running and healthy");
                return Ok(());
            }
            Err(_) => {
                warn!("SearXNG container running but unhealthy, restarting...");
            }
        }
    }

    // The real container is confirmed NOT running (or unhealthy) at this
    // point. Always clean up any stale container with the same name AND
    // reset the manager's internal tracked-process state before trying to
    // start again — otherwise `handle.start()` may silently no-op if it
    // still thinks the previous (already-exited) `docker run -d` child
    // process means "everything is fine".
    cleanup_existing_container().await?;
    let mgr = mgr_arc.lock().await;
    mgr.handle.reset().await;
    info!("starting SearXNG Docker container");
    mgr.handle.start().await?;
    drop(mgr);

    wait_for_health(port, 30).await
}
```

### File: `src-tauri/src/lib.rs`

**Step 1.4 — Start SearXNG eagerly at boot instead of waiting for the first search.**

FIND THIS EXACT TEXT:
```rust
            // Init SearXNG manager
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = searxng::init_searxng(app_handle).await {
                    warn!(?e, "SearXNG init skipped");
                }
            });
```

REPLACE WITH THIS EXACT TEXT:
```rust
            // Init SearXNG manager and start the container immediately at
            // boot (it stays running for the whole app session — see
            // FIX 1 in FIXED.md for why it is no longer lazy).
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = searxng::init_searxng(app_handle.clone()).await {
                    warn!(?e, "SearXNG init skipped");
                    return;
                }
                if let Err(e) = searxng::ensure_running(&app_handle).await {
                    warn!(?e, "SearXNG failed to start at boot, will retry on first search");
                }
            });
```

### VERIFY (run before moving to FIX 2)

```
cd src-tauri
cargo build
```

Must compile with no errors. Then manually: start the app, wait ~10
seconds, run `docker ps` and confirm a container named `jarvis-searxng`
is running without having sent any chat message yet.

---

## FIX 2 — Route weather/time/exchange-rate queries directly to web search

### Why

Right now every query always tries Cerebro first
(`ask_cerebro_with_fallback` in `src-tauri/src/cerebro.rs`). For queries
like "clima en Córdoba" (weather), Cerebro will essentially never have
this, so trying it first just wastes time before falling back.

### File: `src-tauri/src/commands/searxng.rs`

**Step 2.1 — Add a bypass-detection function right after `should_trigger_web_search`.**

FIND THIS EXACT TEXT:
```rust
pub fn should_trigger_web_search(query: &str, config: &WebSearchFallbackConfig) -> bool {
    if !config.enabled {
        return false;
    }
    let query_lower = query.to_lowercase();
    config.keywords.iter().any(|kw| query_lower.contains(&kw.to_lowercase()))
}
```

REPLACE WITH THIS EXACT TEXT:
```rust
pub fn should_trigger_web_search(query: &str, config: &WebSearchFallbackConfig) -> bool {
    if !config.enabled {
        return false;
    }
    let query_lower = query.to_lowercase();
    config.keywords.iter().any(|kw| query_lower.contains(&kw.to_lowercase()))
}

/// Queries matching these topics almost never have an answer in Cerebro
/// (weather, current time, exchange rates, live scores), so it is not
/// worth trying Cerebro first — go straight to the web.
const DIRECT_WEB_PATTERNS: &[&str] = &[
    "clima", "temperatura", "pronóstico", "pronostico",
    "qué hora es", "que hora es",
    "cotización", "cotizacion", "dólar", "dolar", "precio del dólar", "precio del dolar",
    "resultado de", "marcador de",
];

pub fn should_bypass_cerebro(query: &str) -> bool {
    let query_lower = query.to_lowercase();
    DIRECT_WEB_PATTERNS.iter().any(|p| query_lower.contains(p))
}
```

### File: `src-tauri/src/cerebro.rs`

**Step 2.2 — Check the bypass before trying Cerebro.**

FIND THIS EXACT TEXT:
```rust
    let fallback_cfg = &config.web_search_fallback;

    // Nivel 1: Cerebro directo
    match ask_cerebro(app, query.clone(), Some(config.clone()), history.clone()).await {
```

REPLACE WITH THIS EXACT TEXT:
```rust
    let fallback_cfg = &config.web_search_fallback;

    // Direct bypass: skip Cerebro entirely for topics it will basically
    // never have an answer for (weather, time, exchange rates, scores).
    if searxng::should_bypass_cerebro(&query) {
        info!("direct web bypass for query: {}", &query[..query.len().min(80)]);
        if let Ok((context, results)) = searxng::search_web_for_context(app, &query, fallback_cfg).await {
            if !context.is_empty() {
                let enhanced_query = format!(
                    "{}\n\nUsa la siguiente información de la web para responder:\n{}",
                    query, context
                );
                match ask_llm(&enhanced_query, history.clone()).await {
                    Ok(response) => {
                        return Ok(CerebroFallbackResponse {
                            response,
                            web_search_used: true,
                            search_results: results,
                        });
                    }
                    Err(e) => {
                        warn!("LLM failed after direct web bypass ({}), continuing to normal flow", e);
                    }
                }
            }
        }
        // If the bypass path failed for any reason, fall through to the
        // normal flow below instead of giving up.
    }

    // Nivel 1: Cerebro directo
    match ask_cerebro(app, query.clone(), Some(config.clone()), history.clone()).await {
```

### VERIFY

```
cd src-tauri
cargo build
```

Then manually: with Cerebro NOT running, ask "¿qué clima hace en
Córdoba?" and confirm the response comes back via web search without
first waiting for a failed Cerebro attempt (check the logs for the line
`direct web bypass for query`).

---

## FIX 3 — "Add to queue" for Spotify (e.g. "agregá esto a la fila")

### Why

Today `spotify.rs` only has `play_spotify` (replaces what's playing).
There is no "add to queue" command, and `chat.ts` has no pattern that
recognizes phrases like "agregá X a la fila".

### File: `src-tauri/src/commands/spotify.rs`

**Step 3.1 — Add a new `add_to_spotify_queue` command at the end of the file.**

FIND THIS EXACT TEXT (the very last lines of the file):
```rust
    open::that(&track_uri).map_err(|e| format!("Failed to open Spotify URI: {}", e))?;
    Ok(format!(
        "Reproduciendo: {} - {} en Spotify",
        top.name, top.artist
    ))
}
```

REPLACE WITH THIS EXACT TEXT:
```rust
    open::that(&track_uri).map_err(|e| format!("Failed to open Spotify URI: {}", e))?;
    Ok(format!(
        "Reproduciendo: {} - {} en Spotify",
        top.name, top.artist
    ))
}

#[tauri::command]
pub async fn add_to_spotify_queue(query: String) -> Result<String, String> {
    let results = search_spotify(query).await?;
    let top = &results[0];
    let track_id = top.uri.strip_prefix("spotify:track:").unwrap_or(&top.uri);
    let track_uri = format!("spotify:track:{}", track_id);

    let config = load_config_sync().map_err(|e| e.to_string())?;
    if config.spotify.user_access_token.is_empty() {
        return Err(
            "Spotify no está autorizado para controlar la reproducción. Ejecutá primero la autorización de usuario."
                .into(),
        );
    }

    let client = Client::new();
    let resp = client
        .post("https://api.spotify.com/v1/me/player/queue")
        .header("Authorization", format!("Bearer {}", config.spotify.user_access_token))
        .query(&[("uri", track_uri.as_str())])
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() || r.status().as_u16() == 204 => {
            return Ok(format!("{} - {}", top.name, top.artist));
        }
        Ok(r) if r.status().as_u16() == 401 => {
            if !config.spotify.user_refresh_token.is_empty() {
                if let Ok((new_access, _)) = refresh_user_token(&config.spotify.user_refresh_token).await {
                    let client = Client::new();
                    let retry = client
                        .post("https://api.spotify.com/v1/me/player/queue")
                        .header("Authorization", format!("Bearer {}", new_access))
                        .query(&[("uri", track_uri.as_str())])
                        .send()
                        .await;
                    if let Ok(r) = retry {
                        if r.status().is_success() || r.status().as_u16() == 204 {
                            return Ok(format!("{} - {}", top.name, top.artist));
                        }
                    }
                }
            }
            Err("No se pudo autenticar con Spotify (token expirado). Reintentá la autorización.".into())
        }
        Ok(r) => Err(format!(
            "Spotify respondió con error {} al agregar a la cola (¿hay algo reproduciéndose ya?)",
            r.status()
        )),
        Err(e) => Err(format!("No se pudo agregar a la cola: {}", e)),
    }
}
```

### File: `src-tauri/src/lib.rs`

**Step 3.2 — Register the new command.**

FIND THIS EXACT TEXT:
```rust
            spotify::is_spotify_available,
            spotify::search_spotify,
            spotify::play_spotify,
            spotify::authorize_spotify_user,
        ])
```

REPLACE WITH THIS EXACT TEXT:
```rust
            spotify::is_spotify_available,
            spotify::search_spotify,
            spotify::play_spotify,
            spotify::add_to_spotify_queue,
            spotify::authorize_spotify_user,
        ])
```

### File: `src/stores/chat.ts`

**Step 3.3 — Add the intent-detection function, right after `detectSpotifyIntent`.**

FIND THIS EXACT TEXT:
```typescript
  function detectSpotifyIntent(content: string): string | null {
    const lower = content.toLowerCase()
    const patterns = [
      /reproduce(?:r)?\s+(?:en\s+)?spotify\s+(.+)/i,
      /(?:pon|coloca|busca)\s+(?:en\s+)?spotify\s+(.+)/i,
      /(?:escuchar|oír|oir)\s+(.+)\s+en\s+spotify/i,
    ]
    for (const pattern of patterns) {
      const match = lower.match(pattern)
      if (match) return match[1].trim()
    }
    return null
  }
```

REPLACE WITH THIS EXACT TEXT:
```typescript
  function detectSpotifyIntent(content: string): string | null {
    const lower = content.toLowerCase()
    const patterns = [
      /reproduce(?:r)?\s+(?:en\s+)?spotify\s+(.+)/i,
      /(?:pon|coloca|busca)\s+(?:en\s+)?spotify\s+(.+)/i,
      /(?:escuchar|oír|oir)\s+(.+)\s+en\s+spotify/i,
    ]
    for (const pattern of patterns) {
      const match = lower.match(pattern)
      if (match) return match[1].trim()
    }
    return null
  }

  function detectSpotifyQueueIntent(content: string): string | null {
    const lower = content.toLowerCase()
    const patterns = [
      /agreg(?:á|a)\s+(.+?)\s+(?:a\s+la\s+)?(?:fila|cola)/i,
      /sum(?:á|a)\s+(.+?)\s+(?:a\s+la\s+)?(?:fila|cola)/i,
      /(?:pon|poné)\s+(.+?)\s+(?:despu[eé]s|en\s+la\s+cola)/i,
    ]
    for (const pattern of patterns) {
      const match = lower.match(pattern)
      if (match) return match[1].trim()
    }
    return null
  }
```

**Step 3.4 — Wire it into `sendMessage`, BEFORE the `detectSpotifyIntent` check (order matters, so "agregá X a la fila" is not mistakenly matched as "play X").**

FIND THIS EXACT TEXT:
```typescript
    const spotifyQuery = detectSpotifyIntent(content)
    if (spotifyQuery) {
```

REPLACE WITH THIS EXACT TEXT:
```typescript
    const spotifyQueueQuery = detectSpotifyQueueIntent(content)
    if (spotifyQueueQuery) {
      const available = await invoke<boolean>("is_spotify_available")
      if (!available) {
        addMessage("⚠️ Spotify no está instalado en este equipo.", "system")
        return
      }
      hud.setState("pensando")
      setTyping(true, "🎵 Agregando a la cola de Spotify...")
      try {
        const result = await invoke<string>("add_to_spotify_queue", { query: spotifyQueueQuery })
        addMessage(`🎵 Agregado a la cola: ${result}`, "jarvis")
        setTyping(false)
        setTimeout(() => hud.setState("escuchando"), 2000)
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        addMessage(`⚠️ Error con Spotify: ${msg}`, "system")
        setTyping(false)
        hud.setState("escuchando")
      }
      return
    }

    const spotifyQuery = detectSpotifyIntent(content)
    if (spotifyQuery) {
```

### VERIFY

```
cd src-tauri
cargo build
cd ..
npm run build
```

Both must succeed with no errors. Then manually: with something already
playing on Spotify, type "agregá Bohemian Rhapsody a la fila" in the
chat and confirm it gets added to the Spotify queue (not played
immediately).

---

## FIX 4 — Remove the unused/dead `lazy_service.rs` module

### Why

`src-tauri/src/lazy_service.rs` defines `LazyServiceRegistry`. It is
instantiated in `lib.rs` but its own comment says
`// TODO: Wire LazyServiceRegistry into tauri command handlers` — it is
never actually used anywhere. The real, active implementation used by
both Cerebro and SearXNG is `src-tauri/src/commands/lazy_process.rs`
(`LazyProcessRegistry`). Having two different, similarly-named
implementations in the same project is confusing and error-prone.
Deleting the unused one does not change any behavior.

### File: `src-tauri/src/lib.rs`

**Step 4.1 — Remove the module declaration and import.**

FIND THIS EXACT TEXT:
```rust
pub(crate) mod commands;
mod cerebro;
pub mod lazy_service;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use tauri::Manager;
use crate::commands::lazy_process;
use crate::commands::searxng;
use crate::commands::spotify;
use lazy_service::LazyServiceRegistry;
```

REPLACE WITH THIS EXACT TEXT:
```rust
pub(crate) mod commands;
mod cerebro;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use tauri::Manager;
use crate::commands::lazy_process;
use crate::commands::searxng;
use crate::commands::spotify;
```

**Step 4.2 — Remove the block that instantiates it.**

FIND THIS EXACT TEXT:
```rust
            let registry = Arc::new(Mutex::new(registry));
            app.manage(registry);

            // Manage LazyServiceRegistry via tauri::State
            // TODO: Wire LazyServiceRegistry into tauri command handlers
            // (e.g. list_services, start_service) so it's actually used.
            let service_registry = Arc::new(Mutex::new(LazyServiceRegistry::new()));
            app.manage(service_registry);

            // Init SearXNG manager and start the container immediately at
```

REPLACE WITH THIS EXACT TEXT:
```rust
            let registry = Arc::new(Mutex::new(registry));
            app.manage(registry);

            // Init SearXNG manager and start the container immediately at
```

> Note: this FIND block assumes FIX 1 (Step 1.4) was already applied —
> the comment line `// Init SearXNG manager and start the container
> immediately at` only exists after that fix. If you are applying FIX 4
> before FIX 1 for some reason, match against
> `// Init SearXNG manager` (without "and start...") instead.

### File: `src-tauri/src/lazy_service.rs`

**Step 4.3 — Delete this file entirely.**

```
rm src-tauri/src/lazy_service.rs
```

### VERIFY

```
cd src-tauri
cargo build
```

Must compile with no errors and no "unused import" warnings related to
`lazy_service`.

---

## FIX 5 — HUD visual rewrite (particle brain + floating cards + always-visible chat)

### Why

The current `ParticleBackground.vue` renders random floating dots across
the whole screen (no brain shape at all). Cards are hidden inside
sliding side panels that only open when a dock button is clicked, and
the chat lives inside the left side panel — not visible by default. This
does not match the agreed final design (3D particle brain, floating
cards always visible in a column, chat always visible at the bottom).

This fix is larger than the others: it creates 2 new files and replaces
2 existing files in full (not partial edits), because the visual
structure changed too much for small patches to be safe.

### Step 5.1 — Install the 3D library

```
npm install three
npm install -D @types/three
```

### Step 5.2 — CREATE NEW FILE: `src/components/hud/ParticleBrain.vue`

```vue
<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from "vue"
import * as THREE from "three"
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js"
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js"
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js"
import { useHudStore } from "@/stores/hud"
import { STATUS_COLORS } from "@/types/status"

const hud = useHudStore()
const container = ref<HTMLDivElement | null>(null)
let raf = 0
let renderer: THREE.WebGLRenderer
let composer: EffectComposer | null = null
let bloomPass: UnrealBloomPass | null = null

function hexNumber(hex: string): number {
  return parseInt(hex.replace("#", ""), 16)
}

function angDiff(a: number, b: number) {
  let d = a - b
  while (d > Math.PI) d -= 2 * Math.PI
  while (d < -Math.PI) d += 2 * Math.PI
  return d
}
function bump(theta: number, center: number, width: number, amount: number) {
  const d = angDiff(theta, center)
  return amount * Math.exp(-(d * d) / (2 * width * width))
}
function gauss(x: number, center: number, width: number, amount: number) {
  const d = x - center
  return amount * Math.exp(-(d * d) / (2 * width * width))
}
function randomDir() {
  const u = Math.random()
  const v = Math.random()
  const theta = 2 * Math.PI * u
  const phi = Math.acos(2 * v - 1)
  return {
    dx: Math.sin(phi) * Math.cos(theta),
    dy: Math.cos(phi),
    dz: Math.sin(phi) * Math.sin(theta),
  }
}
function brainRadiusDir(dx: number, dy: number, dz: number) {
  const theta = Math.atan2(dz, dx)
  let r =
    1 +
    0.1 * Math.sin(4 * theta + 0.3) +
    0.06 * Math.sin(9 * theta + 1.2) +
    0.04 * Math.sin(15 * theta + 2.0) +
    0.05 * Math.sin(6 * dy * 3.0 + theta)
  r += bump(theta, 0.9, 0.4, 0.16) * Math.max(0, -dy + 0.3)
  r += bump(theta, 2.35, 0.35, 0.1) * Math.max(0, -dy + 0.3)
  r -= gauss(dy, -0.92, 0.1, 0.6)
  const nearMidplane = Math.exp(-(dz * dz) / (2 * 0.05 * 0.05))
  r -= nearMidplane * Math.max(0, dy) * 0.1
  return Math.max(r, 0.2)
}
const RX = 1.55, RY = 1.05, RZ = 1.0
function brainPoint(shell: number) {
  const { dx, dy, dz } = randomDir()
  const r = brainRadiusDir(dx, dy, dz) * shell
  return new THREE.Vector3(dx * r * RX, dy * r * RY, dz * r * RZ)
}
function makeGlowTexture() {
  const c = document.createElement("canvas")
  c.width = c.height = 128
  const g = c.getContext("2d")!
  const grad = g.createRadialGradient(64, 64, 0, 64, 64, 64)
  grad.addColorStop(0, "rgba(255,255,255,1)")
  grad.addColorStop(0.35, "rgba(255,255,255,0.6)")
  grad.addColorStop(1, "rgba(255,255,255,0)")
  g.fillStyle = grad
  g.fillRect(0, 0, 128, 128)
  return new THREE.CanvasTexture(c)
}

onMounted(() => {
  const el = container.value!
  const scene = new THREE.Scene()
  const camera = new THREE.PerspectiveCamera(45, el.clientWidth / el.clientHeight, 0.1, 100)
  camera.position.set(0, 0, 6.4)

  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.setSize(el.clientWidth, el.clientHeight)
  renderer.setClearColor(0x000000, 0)
  el.appendChild(renderer.domElement)

  try {
    composer = new EffectComposer(renderer)
    composer.addPass(new RenderPass(scene, camera))
    bloomPass = new UnrealBloomPass(new THREE.Vector2(el.clientWidth, el.clientHeight), 1.15, 0.75, 0.12)
    composer.addPass(bloomPass)
  } catch {
    composer = null
    bloomPass = null
  }

  const brainGroup = new THREE.Group()
  scene.add(brainGroup)

  const glowTex = makeGlowTexture()

  const CORE_N = 500
  const corePositions: THREE.Vector3[] = []
  const corePhase = new Float32Array(CORE_N * 3)
  for (let i = 0; i < CORE_N; i++) {
    corePositions.push(brainPoint(0.92 + Math.random() * 0.08))
    corePhase[i * 3] = Math.random() * Math.PI * 2
    corePhase[i * 3 + 1] = Math.random() * Math.PI * 2
    corePhase[i * 3 + 2] = Math.random() * Math.PI * 2
  }
  const coreGeo = new THREE.BufferGeometry().setFromPoints(corePositions)
  const coreMat = new THREE.PointsMaterial({
    map: glowTex,
    color: hexNumber(STATUS_COLORS[hud.currentState]),
    size: 0.09,
    transparent: true,
    opacity: 0.95,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  })
  brainGroup.add(new THREE.Points(coreGeo, coreMat))

  const THRESH = 0.24
  const linePairs: number[] = []
  for (let i = 0; i < CORE_N; i++) {
    for (let j = i + 1; j < CORE_N; j++) {
      if (corePositions[i].distanceTo(corePositions[j]) < THRESH) linePairs.push(i, j)
    }
  }
  const lineGeo = new THREE.BufferGeometry()
  const linePosArray = new Float32Array(linePairs.length * 3)
  for (let k = 0; k < linePairs.length; k += 2) {
    const i = linePairs[k], j = linePairs[k + 1]
    const base = k * 3
    linePosArray[base] = corePositions[i].x
    linePosArray[base + 1] = corePositions[i].y
    linePosArray[base + 2] = corePositions[i].z
    linePosArray[base + 3] = corePositions[j].x
    linePosArray[base + 4] = corePositions[j].y
    linePosArray[base + 5] = corePositions[j].z
  }
  lineGeo.setAttribute("position", new THREE.BufferAttribute(linePosArray, 3))
  const lineMat = new THREE.LineBasicMaterial({
    color: hexNumber(STATUS_COLORS[hud.currentState]),
    transparent: true,
    opacity: 0.18,
    blending: THREE.AdditiveBlending,
  })
  brainGroup.add(new THREE.LineSegments(lineGeo, lineMat))

  const DUST_N = 1800
  const dustPositions: THREE.Vector3[] = []
  const dustPhase = new Float32Array(DUST_N * 3)
  for (let i = 0; i < DUST_N; i++) {
    dustPositions.push(brainPoint(0.5 + Math.random() * 0.9))
    dustPhase[i * 3] = Math.random() * Math.PI * 2
    dustPhase[i * 3 + 1] = Math.random() * Math.PI * 2
    dustPhase[i * 3 + 2] = Math.random() * Math.PI * 2
  }
  const dustGeo = new THREE.BufferGeometry().setFromPoints(dustPositions)
  const dustMat = new THREE.PointsMaterial({
    map: glowTex,
    color: hexNumber(STATUS_COLORS[hud.currentState]),
    size: 0.03,
    transparent: true,
    opacity: 0.5,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  })
  brainGroup.add(new THREE.Points(dustGeo, dustMat))

  scene.add(new THREE.AmbientLight(0xffffff, 0.2))

  let autoRotate = true
  let isDragging = false, lastX = 0, lastY = 0, rotVelY = 0.0025
  el.addEventListener("pointerdown", (e) => {
    isDragging = true
    autoRotate = false
    lastX = e.clientX
    lastY = e.clientY
  })
  window.addEventListener("pointerup", () => { isDragging = false })
  window.addEventListener("pointermove", (e) => {
    if (!isDragging) return
    const dx = e.clientX - lastX, dy = e.clientY - lastY
    brainGroup.rotation.y += dx * 0.005
    brainGroup.rotation.x += dy * 0.005
    rotVelY = dx * 0.0006
    lastX = e.clientX
    lastY = e.clientY
  })

  function onResize() {
    camera.aspect = el.clientWidth / el.clientHeight
    camera.updateProjectionMatrix()
    renderer.setSize(el.clientWidth, el.clientHeight)
    composer?.setSize(el.clientWidth, el.clientHeight)
  }
  window.addEventListener("resize", onResize)

  let tBrain = 0
  let lastStateSeen = hud.currentState
  let turbulence = 0
  function updateStateColors() {
    const c = hexNumber(STATUS_COLORS[hud.currentState])
    coreMat.color.setHex(c)
    lineMat.color.setHex(c)
    dustMat.color.setHex(c)
  }

  function animate() {
    raf = requestAnimationFrame(animate)
    tBrain += 0.016
    if (lastStateSeen !== hud.currentState) {
      updateStateColors()
      lastStateSeen = hud.currentState
    }

    if (autoRotate) brainGroup.rotation.y += 0.0022
    else if (!isDragging) {
      brainGroup.rotation.y += rotVelY
      rotVelY *= 0.96
      if (Math.abs(rotVelY) < 0.0004) autoRotate = true
    }

    const sinceBurst = performance.now() - hud.lastChangeAt
    const burst = sinceBurst < 700 ? 1 - sinceBurst / 700 : 0
    const burstEase = burst * burst

    const breathe = 0.85 + 0.15 * Math.sin(tBrain * 0.6) + burstEase * 0.5
    coreMat.opacity = Math.min(1, 0.75 * breathe + 0.15)
    coreMat.size = 0.09 + burstEase * 0.05
    lineMat.opacity = Math.min(0.9, 0.12 * breathe + 0.05 + burstEase * 0.35)
    dustMat.opacity = Math.min(1, 0.5 + burstEase * 0.4)
    brainGroup.scale.setScalar(1 + burstEase * 0.12)
    if (bloomPass) bloomPass.strength = 1.15 + burstEase * 1.4

    const wantTurbulence = hud.currentState === "pensando" || hud.currentState === "trabajando" ? 1 : 0
    turbulence += (wantTurbulence - turbulence) * 0.05
    const turbAmp = hud.currentState === "trabajando" ? 0.16 : 0.11

    if (turbulence > 0.004) {
      const corePos = coreGeo.attributes.position.array as Float32Array
      for (let i = 0; i < CORE_N; i++) {
        const base = corePositions[i]
        const amp = turbAmp * turbulence
        corePos[i * 3] = base.x + amp * Math.sin(tBrain * 2.3 + corePhase[i * 3])
        corePos[i * 3 + 1] = base.y + amp * Math.sin(tBrain * 2.7 + corePhase[i * 3 + 1])
        corePos[i * 3 + 2] = base.z + amp * Math.sin(tBrain * 3.1 + corePhase[i * 3 + 2])
      }
      coreGeo.attributes.position.needsUpdate = true

      const dustPos = dustGeo.attributes.position.array as Float32Array
      for (let i = 0; i < DUST_N; i++) {
        const base = dustPositions[i]
        const amp = turbAmp * 0.6 * turbulence
        dustPos[i * 3] = base.x + amp * Math.sin(tBrain * 2.1 + dustPhase[i * 3])
        dustPos[i * 3 + 1] = base.y + amp * Math.sin(tBrain * 2.5 + dustPhase[i * 3 + 1])
        dustPos[i * 3 + 2] = base.z + amp * Math.sin(tBrain * 2.9 + dustPhase[i * 3 + 2])
      }
      dustGeo.attributes.position.needsUpdate = true

      const linePos = lineGeo.attributes.position.array as Float32Array
      for (let k = 0; k < linePairs.length; k += 2) {
        const i = linePairs[k], j = linePairs[k + 1]
        const base = k * 3
        linePos[base] = corePos[i * 3]
        linePos[base + 1] = corePos[i * 3 + 1]
        linePos[base + 2] = corePos[i * 3 + 2]
        linePos[base + 3] = corePos[j * 3]
        linePos[base + 4] = corePos[j * 3 + 1]
        linePos[base + 5] = corePos[j * 3 + 2]
      }
      lineGeo.attributes.position.needsUpdate = true
    }

    if (composer) composer.render()
    else renderer.render(scene, camera)
  }
  animate()

  onBeforeUnmount(() => {
    cancelAnimationFrame(raf)
    window.removeEventListener("resize", onResize)
    coreGeo.dispose()
    lineGeo.dispose()
    dustGeo.dispose()
    coreMat.dispose()
    lineMat.dispose()
    dustMat.dispose()
    renderer.dispose()
    el.removeChild(renderer.domElement)
  })
})
</script>

<template>
  <div ref="container" class="particle-brain" />
</template>

<style scoped>
.particle-brain {
  width: 100%;
  height: 100%;
  cursor: grab;
}
.particle-brain:active {
  cursor: grabbing;
}
</style>
```

### Step 5.3 — CREATE NEW FILE: `src/components/hud/FloatingCard.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ id: string; title: string; hidden: boolean }>()
defineEmits<{ (e: "hide", id: string): void }>()
</script>

<template>
  <div
    class="card w-56 rounded-2xl border border-jarvis-cyan/25 bg-jarvis-panel/40
           backdrop-blur-md shadow-[0_8px_32px_rgba(0,0,0,0.4)] px-4 py-3 transition-all duration-300"
    :class="props.hidden ? 'opacity-0 max-h-0 py-0 pointer-events-none overflow-hidden' : 'max-h-56'"
  >
    <div class="flex items-center justify-between mb-2">
      <h3 class="font-mono text-[10px] tracking-[0.2em] text-jarvis-text/85">{{ props.title }}</h3>
      <button class="text-jarvis-muted hover:text-jarvis-cyan text-xs" @click="$emit('hide', props.id)">
        ✕
      </button>
    </div>
    <slot />
  </div>
</template>

<style scoped>
.card {
  animation: floaty 8s ease-in-out infinite;
}
@keyframes floaty {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}
</style>
```

### Step 5.4 — `src/stores/hud.ts`: add fields needed by the burst effect and hidden cards.

FIND THIS EXACT TEXT (the entire current file content):
```typescript
import { defineStore } from "pinia"
import { ref } from "vue"
import type { JarvisStatus } from "@/types/status"

export const useHudStore = defineStore("hud", () => {
  const currentState = ref<JarvisStatus>("escuchando")

  function setState(state: JarvisStatus) {
    currentState.value = state
  }

  function getState(): JarvisStatus {
    return currentState.value
  }

  return { currentState, setState, getState }
})
```

REPLACE WITH THIS EXACT TEXT:
```typescript
import { defineStore } from "pinia"
import { ref } from "vue"
import type { JarvisStatus } from "@/types/status"

export const useHudStore = defineStore("hud", () => {
  const currentState = ref<JarvisStatus>("escuchando")
  const lastChangeAt = ref(0)
  const hiddenCards = ref<string[]>([])

  function setState(state: JarvisStatus) {
    if (state !== currentState.value) {
      currentState.value = state
      lastChangeAt.value = performance.now()
    }
  }

  function getState(): JarvisStatus {
    return currentState.value
  }

  function hideCard(id: string) {
    if (!hiddenCards.value.includes(id)) hiddenCards.value.push(id)
  }

  function showCard(id: string) {
    hiddenCards.value = hiddenCards.value.filter((c) => c !== id)
  }

  function isHidden(id: string) {
    return hiddenCards.value.includes(id)
  }

  return {
    currentState,
    lastChangeAt,
    hiddenCards,
    setState,
    getState,
    hideCard,
    showCard,
    isHidden,
  }
})
```

### Step 5.5 — `src/views/HUD.vue`: full file replacement.

DELETE the entire current content of `src/views/HUD.vue` and REPLACE
it with exactly this:

```vue
<script setup lang="ts">
import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useHudStore } from "@/stores/hud"
import ParticleBrain from "@/components/hud/ParticleBrain.vue"
import StatusBar from "@/components/hud/StatusBar.vue"
import FloatingCard from "@/components/hud/FloatingCard.vue"
import ChatPanel from "@/components/chat/ChatPanel.vue"
import {
  PANEL_CONFIG,
  cerebroStatusLabel,
  type LazyProcessStatus,
} from "@/types/hud"

const hud = useHudStore()
const cerebroStatus = ref<LazyProcessStatus | null>(null)

async function fetchCerebroStatus() {
  try {
    cerebroStatus.value = await invoke<LazyProcessStatus>("lazy_get_status", {
      name: "cerebro",
    })
  } catch {
    cerebroStatus.value = null
  }
}

const panelMetrics = computed(() =>
  PANEL_CONFIG.map((panel) => {
    if (panel.id !== "cerebro" || !cerebroStatus.value) return panel.metrics
    const s = cerebroStatus.value
    return [
      { key: "status", label: "Estado", value: cerebroStatusLabel(s) },
      { key: "model", label: "Modelo", value: "llama3" },
      {
        key: "latency",
        label: "Inactivo",
        value: s.idle_secs > 0 ? `${s.idle_secs}s` : "—",
      },
    ]
  }),
)

fetchCerebroStatus()
setInterval(fetchCerebroStatus, 5000)
</script>

<template>
  <div class="relative w-screen h-screen overflow-hidden bg-jarvis-bg">
    <ParticleBrain />

    <StatusBar />

    <div class="fixed top-16 right-8 z-20 flex flex-col gap-4 w-56">
      <FloatingCard
        v-for="(panel, idx) in PANEL_CONFIG"
        :key="panel.id"
        :id="panel.id"
        :title="panel.label"
        :hidden="hud.isHidden(panel.id)"
        @hide="hud.hideCard"
      >
        <div v-if="panelMetrics[idx]" class="space-y-1">
          <div
            v-for="metric in panelMetrics[idx]"
            :key="metric.key"
            class="flex justify-between"
          >
            <span class="font-mono text-xs text-jarvis-muted">{{ metric.label }}</span>
            <span class="font-mono text-xs text-jarvis-cyan">{{ metric.value }}</span>
          </div>
        </div>
      </FloatingCard>
    </div>

    <div class="fixed bottom-24 right-8 z-20 flex flex-col gap-2">
      <div
        v-for="panel in PANEL_CONFIG.filter((p) => hud.isHidden(p.id))"
        :key="panel.id"
        class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-jarvis-panel/60
               border border-jarvis-border text-[10px] font-mono tracking-wide
               text-jarvis-cyan cursor-pointer hover:border-jarvis-cyan"
        @click="hud.showCard(panel.id)"
      >
        {{ panel.label }}
      </div>
    </div>

    <div class="fixed bottom-10 left-1/2 -translate-x-1/2 w-[min(90vw,640px)] z-20 h-[45vh] max-h-[420px]">
      <ChatPanel />
    </div>
  </div>
</template>
```

> This removes `SidePanel`, `DockBar`, and the click-to-cycle-state
> circle entirely. If any other file imports `SidePanel.vue` or
> `DockBar.vue`, leave those component files in place (do not delete
> them) — just stop importing/using them here. Do not delete
> `SidePanel.vue` or `DockBar.vue` as part of this fix.

### Step 5.6 — `src/components/chat/ChatPanel.vue`: hide the scrollbar.

FIND THIS EXACT TEXT:
```vue
    <div
      ref="scrollContainer"
      class="flex-1 overflow-y-auto px-4 py-3 space-y-3"
    >
```

REPLACE WITH THIS EXACT TEXT:
```vue
    <div
      ref="scrollContainer"
      class="flex-1 overflow-y-auto px-4 py-3 space-y-3 [scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden"
    >
```

### VERIFY

```
npm run build
cd src-tauri
cargo build
```

Both must succeed. Then run the app and confirm:
- A 3D brain (not random floating dots) is visible in the center as soon
  as the HUD loads.
- The chat is visible immediately at the bottom, with no need to click
  any dock button to open it.
- The three cards (SISTEMA, CEREBRO · MCP, MÓDULOS) are stacked in a
  column on the right, each with a working "✕" button that hides it and
  makes a small chip appear at the bottom-right to bring it back.
- The chat message list scrolls with the mouse wheel but shows no
  visible scrollbar.
- Clicking a dock button is no longer required or present.

---

## Suggested order

FIX 1 → FIX 4 → FIX 2 → FIX 3 → FIX 5

(Fixes 2 and 3 are independent of each other and can be swapped. FIX 5
is last because it is the largest and does not depend on the others.)
