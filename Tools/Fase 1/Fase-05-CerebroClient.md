# Micro-fase J1.05 — Cliente HTTP de Cerebro

## Objetivo

Exponer un comando Tauri en Rust que llame a la HTTP API de Cerebro (fase 06
de Cerebro, ya terminada) y devuelva la respuesta al frontend. Sin cablear
todavía al panel de chat — se prueba de forma aislada.

## Prerrequisitos

- J1.02 completa (ya existe el check de salud de Cerebro, este comando
  reutiliza esa misma URL base)
- Cerebro corriendo localmente con su HTTP API accesible

## Tareas

### T1 — Cliente HTTP (Rust)

Agregar `reqwest` al `Cargo.toml` de `src-tauri`. Función async
`ask_cerebro(query: String) -> Result<String, String>` que hace el request
al endpoint de consulta de Cerebro (confirmar el path exacto contra la Fase
06 del plan de Cerebro) y parsea la respuesta.

### T2 — Comando Tauri

```rust
#[tauri::command]
async fn send_to_cerebro(message: String) -> Result<String, String> {
    ask_cerebro(message).await
}
```

Registrado en el `invoke_handler` de `main.rs`.

### T3 — Timeout y manejo de error

Timeout corto (ej. 15s, las consultas con embeddings/LLM pueden tardar más
que un healthcheck). Si falla: devolver un mensaje de error legible, no un
panic ni un string crudo de `reqwest`.

### T4 — Verificación aislada

Desde la consola de dev tools del webview (o un botón de prueba temporal),
invocar `send_to_cerebro("test")` y confirmar que devuelve una respuesta
real de Cerebro. Probar también con Cerebro apagado para confirmar que el
error se devuelve limpio.

## Entregable

Comando Tauri `send_to_cerebro` probado y funcionando de forma aislada,
listo para conectarse al panel de chat en J1.06.
