# Changelog — JARVIS

---

## Fase 1: Interfaz + Chat con Cerebro — 2026-08-21

**Plan ejecutado:** Fase 1/Fase-00-Overview.md

### Tareas completadas

| ID | Tarea | Estado |
|----|-------|--------|
| T1 | Bootstrap del proyecto Tauri | ✅ |
| T2 | Secuencia de arranque (boot sequence) | ✅ |
| T3 | Shell visual (HUD) | ✅ |
| T4 | Panel de chat | ✅ |
| T5 | Cliente HTTP de Cerebro | ✅ |
| T6 | Conexión chat ↔ Cerebro | ✅ |
| T7 | Configuración persistente | ✅ |
| T8 | Manejo de errores y logging | ✅ |

### Archivos modificados/creados

**Rust (src-tauri/):**
- Cargo.toml, tauri.conf.json, build.rs
- src/lib.rs, src/main.rs
- src/commands/{mod, boot, cerebro, config, errors}.rs

**Vue/TypeScript (src/):**
- main.ts, App.vue
- views/{HUD, BootSequence}.vue
- components/hud/{ParticleBackground, SidePanel, DockBar, StatusBar}.vue
- components/chat/{ChatPanel, ChatMessage, ChatInput}.vue
- components/boot/BootCheck.vue
- components/ui/ErrorBanner.vue
- stores/{boot, hud, chat, config, errors}.ts
- composables/{useBootSequence, useCerebroClient, useConfig, useErrorHandler}.ts
- types/{message, status, cerebro, config, boot, jarvis-error}.ts
- assets/styles/{main, hud, errors}.css

**Config:**
- jarvis.config.toml

### Auditoría

- **Veredicto:** changes_needed (corregido)
- **2 findings críticos corregidos:**
  1. CSP deshabilitado → configurado con política restrictiva
  2. useCerebroClient singleton → refs por instancia
- **7 warnings** (mejoras futuras: URLs hardcodeadas en boot.rs, caché de config, etc.)

---

## [2026-08-22] Fase 1 — Bootstrap del proyecto (J1.01)

**Plan:** `Fase 1/Fase-01-Bootstrap.md`

### Tareas completadas
- T1: Crear proyecto `jarvis-shell` con Tauri v2 (cargo tauri init)
- T2: Configurar tauri.conf.json (ventana sin decoración, 1024x640, fondo #030a12)
- T3: Crear index.html mínimo placeholder oscuro
- T4: Verificación con `cargo tauri dev` — build y ventana exitosos

### Archivos creados/modificados
- `jarvis-shell/src-tauri/tauri.conf.json` — config ventana + CSP + identifier
- `jarvis-shell/src/index.html` — placeholder oscuro
- `jarvis-shell/vite.config.ts` — exclude target/ del watcher (fix Windows EBUSY)
- `jarvis-shell/package.json` — type: module
- `jarvis-shell/src-tauri/Cargo.toml` — crate-type: rlib

### Auditoría
- Veredicto: `changes_needed` → correcciones aplicadas
- Hallazgos corregidos: CSP null, identifier placeholder, frontendDist, crate-type innecesario
- Build re-verificado post-fix: ✅

### Notas
- Fix necesario para Windows: vite.config.ts exclude src-tauri/ del file watcher (EBUSY conflict)
- El HUD real se porta en J1.03

---

## 2026-08-22 — Micro-fase J1.02: Secuencia de arranque (boot sequence)

### Archivos modificados
- `src-tauri/src/commands/boot.rs` — Tipos BootStep/BootStepStatus, función run_boot_sequence con verificación de Ollama (retry/polling), modelo (carga con timeout extendido), y Cerebro (retries). Eventos Tauri boot-step en tiempo real.
- `src-tauri/src/lib.rs` — Registro del comando run_boot_sequence en invoke_handler.
- `src/types/boot.ts` — Interfaz BootStep con status tipado (pending|running|ok|error).
- `src/stores/boot.ts` — Pinia store con setFromEvent para recibir eventos del backend.
- `src/composables/useBootSequence.ts` — Orquestador que invoca run_boot_sequence y escucha eventos boot-step.
- `src/components/boot/BootCheck.vue` — Componente presentacional con iconos de estado, animación de spinner, estética JARVIS.
- `src/views/BootSequence.vue` — Pantalla completa de arranque con progreso paso a paso, barra de progreso, animación de título.
- `src/App.vue` — Wiring: BootSequence se muestra antes del HUD, transición al completar.

### Cambios clave
- Backend emite eventos `boot-step` en tiempo real para cada paso del boot
- Ollama: polling con retry (hasta 15s) en vez de sleep fijo
- Modelo: timeout extendido (30s) para descargas lentas
- Cerebro: 3 retries con delay para errores transitorios
- HTTP client reutilizable via función helper
- JSON parse con error handling explícito

### Auditoría
- 14 findings de static analysis (0 critical, 3 high, 5 medium, 4 low, 2 info)
- Correcciones aplicadas: shared HTTP client, Ollama retry/polling, model timeout, JSON parse handling, Cerebro retries

---

## 2026-08-22 — Micro-fase J1.03: Shell visual (HUD)

**Plan:** `Fase 1/Fase-03-HudShell.md`

### Tareas completadas
- T1: Integración HUD en frontend (HUD.vue, ParticleBackground, etc.)
- T2: Configuración `PANEL_CONFIG` (SISTEMA, CEREBRO·MCP, MÓDULOS)
- T3: Sistema de estados `STATES` (escuchando, pensando, trabajando, respondiendo)
- T4: Transición boot → HUD (fade effect)
- T5: Verificación y corrección de errores de compilación/runtime

### Archivos modificados/creados
- `src/types/hud.ts` (Nuevo) — Configuración de paneles y estados.
- `src/App.vue` — Fade transition entre Boot y HUD.
- `src/views/HUD.vue` — Integración dinámica con `STATES` y `PANEL_CONFIG`.
- `src/views/BootSequence.vue` — UI de error mejorada (retry button, mensajes visibles).
- `src-tauri/src/commands/boot.rs` — Fixes: funciones duplicadas, tipos, logs, modelo `qwen2.5:3b`, puerto Cerebro `8765`.
- `src-tauri/tauri.conf.json` — Permisos Tauri v2 (`core:event:allow-listen`, etc.).
- `vite.config.ts` — Desactivado watcher de Vite (fix EBUSY).
- `src-tauri/src/commands/{cerebro, errors}.rs` — Fix warnings (dead code).

### Notas
- Se corrigieron conflictos de compilación en `boot.rs` (funciones duplicadas).
- Se solucionó el error `EBUSY` en Windows desactivando el watcher de Vite.
- Se añadieron permisos necesarios en `tauri.conf.json` para Tauri v2.
- Se mejoró la UI de error en `BootSequence.vue` para cumplir con el plan (quedarse en boot si falla).

---

## 2026-08-23 — Sincronización de planes Phase-J1 (Revisión 2)

**Motivo:** Los archivos `Phase-J1-0X` en `Fase 1/` contenían versiones actualizadas de los planes Fase-00 a Fase-03. Se compararon y sincronizaron los cambios a los archivos oficiales.

### Archivos modificados
- `Fase 1/Fase-02-StartupSequence.md` — T4 actualizado (Revisión 2)
- `Fase 1/Fase-03-HudShell.md` — Reescritura completa (Revisión 2)

### Cambios en Fase-02 (StartupSequence)

**T4 — Pantalla de arranque (Revisión 2):**
- Pantalla de boot ahora muestra el componente `ParticleBrain` del HUD (consistencia visual desde el primer segundo)
- Manejo de error mejorado:
  - Botón **individual** por step para iniciar ese proceso puntual
  - Botón **"Reintentar todo"** que re-chequea los 3 ítems (detecta cambios manuales del usuario sin reiniciar la app)
- Referencia a `jarvis-boot-error.html` como ejemplo de implementación
- Clarificación: no se avanza al HUD hasta que los 3 ítems queden en `Ok`

### Cambios en Fase-03 (HudShell) — Reescritura completa

**Stack:** HTML/CSS/JS plano → **Vue 3 + Pinia**

**Cerebro 3D de partículas:**
- Canvas 2D → **Three.js** con nube de puntos 3D, silueta matemática (sin modelo importado)
- 3 capas visuales: núcleo + líneas (red neuronal), polvo ambiental, chispas con resplandor
- Bloom real vía `UnrealBloomPass` con fallback silencioso
- Rotación automática + arrastre manual con inercia

**Movimiento real por estado (no solo color):**
- `pensando`/`trabajando`: turbulencia real (oscilación con fase aleatoria)
- Transiciones suaves (ease) al entrar/salir de estados con turbulencia
- **Ráfaga de cambio de estado**: animación de ~0.7s (expansión + bloom intenso → decaimiento)
- `escuchando`/`respondiendo`: quietos (solo respiración/pulso sutil de opacidad)

**Cards flotantes:**
- Flex columna (no coordenadas fijas), alineación siempre uniforme
- Estilo vidrio: `backdrop-filter: blur(...)`, borde/glow según estado
- Animación de levitación desfasada
- Botón ocultar por card → colapsa espacio + chip "restaurar" en bandeja

**Panel de chat:**
- Scroll oculto (`scrollbar-width: none` + `::-webkit-scrollbar { display: none }`)
- 3 estilos de burbuja: usuario, JARVIS, aviso no bloqueante (borde ámbar)

**Tareas reescritas:**
- T1: Componente `ParticleBrain` (Three.js en Vue, con `dispose()` limpio)
- T2: Componente `FloatingCard` genérico (store en Pinia, no estado local)
- T3: Store `useHudStore` centralizado (estado, turbulencia, timestamp de ráfaga)

### Archivos de referencia mencionados
- `jarvis-hud-main.html`, `jarvis-boot-ok.html`, `jarvis-boot-error.html` (maquetas iterativas)
- `OneFixed.md` Fix 2 (estilo de burbuja de aviso)

---

## 2026-08-24 — Micro-fase J1.04: Panel de chat (eco local)

**Plan:** `Fase 1/Fase-04-ChatPanel.md`

### Tareas completadas
- T1: Contenedor de chat en el HUD (ya existía en SidePanel izquierdo)
- T2: Estructura de mensaje `{role, content, timestamp}` — role simplificado a `"user" | "jarvis"`
- T3: Input y envío con eco mock — `[mock] recibí: <texto>` tras delay 800ms
- T4: Disparo de estados — `pensando` (violeta) → `respondiendo` (verde) → `escuchando` (cian)
- T5: Verificación — build completo exitoso

### Archivos modificados
- `src/types/message.ts` — `MessageRole` simplificado a `"user" | "jarvis"` (eliminado `"assistant" | "system"`)
- `src/stores/chat.ts` — Eco local mock en lugar de llamada a Cerebro (imports de `useCerebroClient` eliminados)
- `src/stores/boot.ts` — Fix: import no usado `BootStepStatus` eliminado

### Notas
- El chat ya tenía componentes pre-existentes (ChatPanel, ChatMessage, ChatInput) de fases anteriores
- El cambio principal fue desconectar la llamada real a Cerebro y reemplazarla por eco simulado
- La conexión real a Cerebro se reconectará en J1.06

---

## 2026-08-24 — Fix: vue-tsc incompatibilidad con TypeScript 5.9

### Problema
- `vue-tsc@1.8.27` fallaba con `Search string not found: "/supportedTSExtensions = .*(?=;)/"`
- Causa: TypeScript 5.7+ movió `tsc.js` → `_tsc.js`, breaking change en `@volar/typescript`

### Solución
- `vue-tsc` actualizado de `^1.8.0` a `^2.2.0` (versión con fix)
- Build ahora ejecuta `vue-tsc --noEmit && vite build` exitosamente

### Archivos modificados
- `package.json` — Dependencia `vue-tsc` actualizada

---

## [2026-08-24] Fase 05: Cliente HTTP de Cerebro

**Plan:** `Fase 1/Fase-05-CerebroClient.md`

### Tareas completadas
- T1: Cliente HTTP (Rust) — reqwest + `ask_cerebro`
- T2: Timeout y manejo de error — 15s, mensajes legibles
- T3: Comando Tauri — `send_to_cerebro` registrado
- T4: Verificación aislada — instrucciones de prueba

### Archivos modificados
- `src-tauri/Cargo.toml`
- `src-tauri/src/cerebro.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/cerebro.rs`
- `src-tauri/src/commands/config.rs`

### Auditoría
- Corregido: `reqwest::Client` compartido (`LazyLock`)
- Corregido: Eliminación de duplicación lógica
- Corregido: URL y timeout dinámicos en `check_cerebro_health`

---

## [2026-08-24] — Micro-fase J1.06: Conexión chat ↔ Cerebro

### Tareas completadas
- **T1**: Reemplazo del eco simulado por llamada real a `invoke('send_to_cerebro', { message })` en el store de chat
- **T2**: Estados reales de HUD implementados — `pensando` durante promesa pendiente, `respondiendo` tras respuesta (~3s), errores como mensaje de sistema ámbar

### Archivos modificados
- `src/stores/chat.ts` — Integración con `send_to_cerebro`, estados de HUD, manejo de errores
- `src/types/message.ts` — Tipo `MessageStatus` agregado para mensajes de error
- `src/components/chat/ChatMessage.vue` — Estilo para mensajes de sistema/error (texto ámbar)

### Auditoría
- Veredicto: **approved**
- Hallazgos: 3 warnings menores (debounce no implementado, timeout no tracked, status hardcoded en createMessage), 2 info (dead code en isLast prop, composable no reutilizado). Ninguno crítico.

### Fase 1 — Estado
JARVIS arranca, muestra dependencias, HUD activo, y permite chatear por texto con Cerebro usando datos reales.

---

## [2026-08-24] Fase 07 — Configuración persistente

**Plan:** `Fase 1/Fase-07-Config.md`

### Tareas completadas
- T1: Actualizar `jarvis.config.toml` con campos del plan (`base_url`, `timeout_secs`, `panels`)
- T2: Actualizar `JarvisConfig` struct con `JARVIS_CONFIG_PATH` env var override
- T3: Reemplazar hardcodeos en `boot.rs` — ahora carga config al inicio
- T4: Actualizar `cerebro.rs` con campos renombrados
- T5: Actualizar `commands/cerebro.rs` con campos renombrados
- T6: Actualizar frontend config types y store

### Archivos modificados
- `jarvis.config.toml`
- `src-tauri/src/commands/config.rs`
- `src-tauri/src/cerebro.rs`
- `src-tauri/src/commands/cerebro.rs`
- `src-tauri/src/commands/boot.rs`
- `src/types/config.ts`
- `src/stores/config.ts`
- `src/composables/useConfig.ts` (audit correction)

### Auditoría
- Veredicto: **approved** (3 minor corrections applied)
- Hallazgos corregidos:
  1. Old field names in composable (`useConfig.ts`)
  2. Phantom `theme` field removed
  3. Inconsistent defaults between backend and frontend

---

## [2026-08-24] — Fase 08: Manejo de errores y logging

### Tareas completadas
- **T1**: Migración de logging a `tracing` + `tracing-appender`. Logs a `<data_dir>/logs/jarvis.log` con nivel configurable.
- **T2**: Botón "Reintentar" por step individual en BootSequence. Nuevo comando `run_boot_step` en backend.
- **T3**: Manejo de errores de Cerebro mid-session. Logging detallado en backend, mensaje claro en chat, HUD vuelve a reposo.
- **T4**: Verificación de compilación y coherencia.

### Archivos modificados
- `src-tauri/Cargo.toml` — deps: tracing, tracing-subscriber, tracing-appender
- `src-tauri/src/lib.rs` — init tracing, graceful exit en run()
- `src-tauri/src/commands/errors.rs` — init_logging() con tracing, macros tracing
- `src-tauri/src/commands/boot.rs` — comando run_boot_step, manejo de spawn
- `src-tauri/src/commands/cerebro.rs` — logging en send_to_cerebro
- `src-tauri/src/cerebro.rs` — LazyLock con Result, tracing::error! detallado
- `src/stores/boot.ts` — retryStep(index)
- `src/stores/chat.ts` — error handler mejorado, HUD reset
- `src/composables/useBootSequence.ts` — retryStep(), fix complete() en catch
- `src/composables/useCerebroClient.ts` — logging en catch
- `src/components/boot/BootCheck.vue` — prop retryable, emit retry
- `src/views/BootSequence.vue` — botón retry por step
- `jarvis.config.toml` — [app] log_level = "info"

### Auditoría
- **Veredicto**: changes_needed → corregido
- **Hallazgos corregidos (alto)**: .expect() en lib.rs y cerebro.rs, spawn() sin manejo en boot.rs, complete() en catch de useBootSequence.ts
- **Compilación**: limpia (0 errores, 0 warnings)

---

## [2026-08-26] Fase 1.5: Lazy processes + Integraciones

### Added
- **Boot redesigned** (J1.5-01): Ollama + Model auto-start, Cerebro no longer blocks boot
- **Lazy process manager** (J1.5-02): Generic module for on-demand process lifecycle
- **Cerebro lazy** (J1.5-03): AI brain starts on first chat message, not at boot
- **SearXNG on-demand** (J1.5-04): Docker container managed via lazy process manager
- **Web search fallback** (J1.5-05): Automatic web search when Cerebro lacks context
- **YouTube integration** (J1.5-06): Search and play videos via SearXNG + browser
- **Spotify integration** (J1.5-07): Search and play tracks via Spotify API + app

### Modified
- Boot sequence: separated Ollama+Model from Cerebro
- Chat flow: intent detection for YouTube/Spotify, web search fallback
- Configuration: added searxng, web_search, spotify, youtube sections

### Files Changed
- src-tauri/src/commands/boot.rs
- src-tauri/src/commands/config.rs
- src-tauri/src/commands/lazy_process.rs (new)
- src-tauri/src/commands/searxng.rs (new)
- src-tauri/src/commands/spotify.rs (new)
- src-tauri/src/commands/cerebro.rs
- src-tauri/src/cerebro.rs
- src-tauri/src/lib.rs
- src-tauri/Cargo.toml
- src/stores/boot.ts
- src/stores/chat.ts
- jarvis.config.toml

### Notes
- Spotify requires client_id and client_secret from Spotify for Developers dashboard
- SearXNG requires Docker installed and running
- All lazy processes auto-stop after 5 minutes of inactivity

---

## [2026-08-26] - Migración Ollama → LlamaCpp (Fase 1.5)

**Plan ejecutado:** 01b-LlamaCppMigration.md

### Tareas completadas
- **Tarea 1:** Reemplazar check_ollama+check_modelo por check_llm en boot.rs
- **Tarea 2:** Reemplazar ask_ollama por ask_llm en cerebro.rs (fallback 3 niveles)
- **Tarea 3:** Actualizar jarvis.config.toml: sección [llm] en vez de [ollama]

### Archivos modificados
- src-tauri/src/commands/boot.rs
- src-tauri/src/cerebro.rs
- src-tauri/src/commands/cerebro.rs
- src-tauri/src/lib.rs
- src-tauri/src/commands/config.rs
- jarvis.config.toml

### Auditoría - Correcciones aplicadas
- config.rs: OllamaConfig → LlmConfig con campos correctos (binary_path, model_path, port, gpu_layers, context_size)
- config.rs: JarvisConfig.ollama → .llm con #[serde(default)]
- config.rs: get_config_value actualizado a claves 'llm.*'
- boot.rs: URLs base corregidas de config.ollama.base_url (11434) a config.llm.base_url() (8081)
- cerebro.rs: ask_llm() URL corregida a llama-server
- boot.rs: dead code eliminado (OLLAMA_CHILD, kill_ollama_sidecar, imports innecesarios)

---

## [Date: 2026-08-26] Boot Redesign - Fase 1.5

### Plan ejecutado
D:\Proyectos\Jarvis\Fase 1.5\01-BootRediseno.md

### Tareas completadas
| ID | Descripción | Status |
|----|-------------|--------|
| T1 | Auto-arrancar Ollama (spawn_ollama + check_llm_internal) | ✅ |
| T2 | Precarga real del modelo (check_modelo separado) | ✅ |
| T3 | Sacar Cerebro del boot obligatorio | ✅ |
| T4 | HUD y pantalla de boot con 2 ítems | ✅ |
| T5 | Verificación manual (pendiente usuario) | ⏳ |

### Archivos modificados
- src-tauri/src/commands/boot.rs (T1, T2, T3 + auditoría)
- src-tauri/src/lib.rs (T2)
- src/stores/boot.ts (T4 + auditoría)
- src/composables/useBootSequence.ts (auditoría)

### Auditoría
- Veredicto: changes_needed → correcciones aplicadas
- Issues corregidos: zombie process leak, off-by-one, run_boot_step cerebro handling, retryStep API mismatch

---

## [Date: 2026-08-26] Cambio de dirección: Ollama → llama.cpp

### Decisión
Se reemplaza Ollama por llama.cpp (llama-server) como motor LLM local.

### Cambios realizados
| Archivo | Cambio |
|---------|--------|
| src-tauri/src/commands/boot.rs | spawn_ollama → spawn_llamacpp, endpoints actualizados |
| src-tauri/src/lib.rs | Removidos check_llm y check_modelo legacy |
| src/stores/boot.ts | step id "ollama" → "llamacpp" |
| src/views/BootSequence.vue | Texto "Ollama" → "llama-server" |

### Configuración llama.cpp
- Binario: `llama-server`
- Puerto: 8080
- Endpoints: `/health` (verificación), `/v1/chat/completions` (generación)
- Formato: OpenAI-compatible

### Notas
- Se eliminaron comandos legacy (check_llm, check_modelo)
- El boot ahora solo tiene 2 pasos: llamacpp y modelo
- Cerebro permanece como conexión on-demand (no parte del boot)

---

## [2026-08-27] Ejecución: 02-LazyProcessManager.md

**Plan ejecutado**: `D:\Proyectos\Jarvis\Fase 1.5\02-LazyProcessManager.md`

### Tareas completadas:
| ID | Descripción | Estado |
|----|-------------|--------|
| T1 | Implementar struct LazyService, estado interno y método ensure_running | ✅ ok |
| T2 | Implementar idle_watcher como tarea tokio::spawn en background | ✅ ok |
| T3 | Registro central de servicios lazy con HashMap y tauri::State | ✅ ok |
| T4 | Caso de prueba trivial (python http.server) para validar mecanismo | ✅ ok |
| T5 | Verificación: ensure_running, idle apagado, y no-duplicado concurrente | ✅ ok |

### Archivos modificados:
- `src-tauri/src/lazy_service.rs` (creado + iteraciones)
- `src-tauri/src/lib.rs` (modularización + manage registry)

### Auditoría:
- **Veredicto inicial**: changes_needed (1 critical, 5 warnings, 4 info)
- **Correcciones aplicadas**: Todas
  - 🔴 TOCTOU race condition en ensure_running() — lock held through entire body
  - 🟡 Lock ordering inversion — standardizado a running→process
  - 🟡 reqwest::get() sin timeout — añadido 5s timeout
  - 🟡 Campos públicos — ahora privados con getters
  - 🟡 URL healthcheck sin validar — rechaza non-http/https
  - 🟡 unwrap_or_default() silenciando errores — ahora con tracing::warn
  - ℹ️ Poll interval documentado
  - ℹ️ Registry thread-safety documentado
  - ℹ️ Dead code marcado con TODO
  - ℹ️ eprintln! reemplazado por tracing::warn
- **Veredicto final**: approved (post-correcciones)

### Tests:
- 5 tests creados, todos pasan (`cargo test`)
- `cargo check` limpio (solo warnings pre-existentes)

---

## 2026-08-28 — Fase 1.5/03: Cerebro a discreción

**Plan ejecutado**: `Fase 1.5/03-CerebroLazy.md`

### Tareas completadas
- **T3**: Agregados `binary_path` e `idle_timeout_secs` a CerebroConfig (Rust, TypeScript, TOML)
- **T1**: Cerebro registrado como lazy service en lib.rs
- **T2**: `ask_cerebro` llama `ensure_running` antes del request HTTP; fallback a Ollama preservado
- **T4**: Card CEREBRO MCP en HUD refleja estado real (Conectado/Apagado/Iniciando)

### Auditoría
- **Finding crítico**: External service mode broken (start creaba dummy process) → Corregido
- **Finding mayor**: Falta `touch()` después de `start()` en ask_cerebro → Corregido
- Otros findings eran pre-existentes, no relacionados con este cambio

### Archivos modificados
- `jarvis.config.toml`
- `src-tauri/src/commands/config.rs`
- `src/types/config.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/cerebro.rs`
- `src-tauri/src/commands/cerebro.rs`
- `src-tauri/src/commands/lazy_process.rs`
- `src/types/hud.ts`
- `src/views/HUD.vue`

---

## 2026-08-28 — Sesión de desarrollo: Fixes y mejoras post-Fase 1.5/03

### Problemas corregidos

| # | Problema | Solución |
|---|----------|----------|
| 1 | Tokio runtime panic en `lib.rs:56` | `tokio::spawn` → `tauri::async_runtime::spawn` en setup closure |
| 2 | Dead code warnings en `lazy_process.rs` | `#[allow(dead_code)]` en campos/métodos no usados |
| 3 | Config no se encontraba al correr desde `target/debug/` | `find_config_path()` mejorado: env var → junto al exe → cwd → project root |
| 4 | Modelo no cargaba (400 Bad Request) | `spawn_llamacpp` ahora pasa `--model`, `--n-gpu-layers`, `--ctx-size`, `--alias` |
| 5 | Boot en loop (doble spawn de llama-server) | `kill_existing_llamacpp()` antes de spawn + health check timeout de 8→90 intentos |
| 6 | Spotify se activa aunque no esté instalado | Nuevo comando `is_spotify_available` + check en chat.ts antes de invoke |
| 7 | Spotify no reemplaza canción en reproducción | OAuth flow con `authorize_spotify_user` + Web API `PUT /v1/me/player/play` |
| 8 | Sin memoria entre mensajes del chat | Pipeline de historial: frontend `messages[]` → backend `history` → LLM `messages[]` |
| 9 | SearXNG retorna 403 Forbidden | Nuevo `searxng-settings.yml` con `formats: [html, json]` montado en contenedor |
| 10 | SearXNG no auto-inicia cuando se necesita | `is_container_running()` via `docker inspect` + health check en `ensure_running()` |
| 11 | Búsqueda web no activa fallback | `search_web_for_context` ahora llama `ensure_running()` antes de buscar |

### Archivos modificados

**Rust (src-tauri/):**
- `src/lib.rs` — `tauri::async_runtime::spawn`, registro de comandos
- `src/cerebro.rs` — `touch()` después de `start()`, historial en `ask_llm` y `ask_cerebro`
- `src/commands/cerebro.rs` — parámetro `history` en `send_to_cerebro_with_fallback`
- `src/commands/boot.rs` — `spawn_llamacpp` con flags de modelo, `kill_existing_llamacpp`, health check extendido
- `src/commands/config.rs` — `find_path()` mejorado, `LlmConfig` con `binary_path`/`idle_timeout_secs`
- `src/commands/lazy_process.rs` — `#[allow(dead_code)]`, fix external service mode
- `src/commands/searxng.rs` — `is_container_running()`, health check en `ensure_running`, mount de settings
- `src/commands/spotify.rs` — `authorize_spotify_user`, `is_spotify_available`, Web API playback

**Vue/TypeScript (src/):**
- `src/stores/chat.ts` — envío de `history`, patrón "autorizar spotify"
- `src/types/config.ts` — `binary_path`, `idle_timeout_secs` en `CerebroConfig`
- `src/types/hud.ts` — métricas dinámicas de Cerebro
- `src/views/HUD.vue` — card estado real del LazyService

**Config:**
- `jarvis.config.toml` — paths corregidos, tokens Spotify, SearXNG config
- `searxng-settings.yml` (nuevo) — habilita formato JSON en SearXNG
- `src-tauri/target/debug/jarvis.config.toml` — sincronizado con raíz

### Estado actual

| Componente | Estado |
|------------|--------|
| llama-server (Qwen2.5 3B) | ✅ Arranca con modelo, funciona en RAM |
| Cerebro | ⏳ Pendiente desarrollo externo |
| SearXNG | ⏳ Contenedor crea pero search falla intermitentemente |
| Spotify | ✅ Funciona (pendiente OAuth para forzar playback) |
| Chat memory | ✅ Historial se envía al LLM |
| YouTube | ⏳ Depende de SearXNG |

### Pendiente
- [ ] Spotify OAuth: registrar `redirect_uri` en Spotify Developer Dashboard
- [ ] SearXNG: estabilizar contenedor y verificar búsqueda web
- [ ] Cerebro: esperar que esté listo para probar flujo completo
