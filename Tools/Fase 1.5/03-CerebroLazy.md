# Micro-fase J1.5-03 — Cerebro a discreción

## Objetivo

Enchufar a Cerebro en el manejador genérico de J1.5-02: se levanta solo
cuando una consulta lo necesita, y se apaga solo tras inactividad.

## Prerrequisitos

- J1.5-01 (Cerebro ya fuera del boot obligatorio)
- J1.5-02 (manejador genérico probado)

## Tareas

### T1 — Registrar a Cerebro como `LazyService`

```rust
LazyService {
    name: "cerebro".into(),
    healthcheck_url: format!("{}/api/health", cfg.cerebro.base_url),
    start: Box::new(|| {
        Command::new(&cfg.cerebro.binary_path) // nueva clave de config, ver T3
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }),
    idle_timeout: Duration::from_secs(cfg.cerebro.idle_timeout_secs),
}
```

### T2 — `send_to_cerebro` pide `ensure_running` antes de consultar

En `cerebro.rs`, antes del request HTTP a `/api/query`:

```rust
lazy_registry.ensure_running("cerebro").await?;
```

Si `ensure_running` falla (Cerebro no pudo levantarse), se sigue
exactamente el mismo camino de fallback a Ollama que ya existe desde
Fase 1 — no hace falta un manejo de error nuevo, es el mismo.

### T3 — Config nueva para Cerebro

```toml
[cerebro]
base_url = "http://localhost:8765"
timeout_secs = 15
binary_path = "C:/ruta/a/cerebro.exe"
idle_timeout_secs = 600
```

### T4 — Card "CEREBRO · MCP" refleja el estado real

La card del HUD pasa de mostrar un estado fijo a reflejar si está
`Conectado` / `Apagado (a discreción)` / `Iniciando...`, leyendo del
mismo registro de `LazyService` (vía un comando `get_lazy_status` o
similar) en vez de un valor hardcodeado.

### T5 — Verificación

- Con Cerebro apagado, hacer una consulta que sí tenga contexto en su
  base de conocimiento → confirmar que JARVIS lo levanta solo, espera, y
  responde con datos reales de Cerebro (no con el fallback de Ollama).
- Dejar pasar el `idle_timeout_secs` sin consultas → confirmar que
  Cerebro se apaga solo y la RAM se libera.

## Entregable

Cerebro completamente "a discreción": nunca hace falta abrirlo a mano,
nunca queda corriendo sin necesidad.
