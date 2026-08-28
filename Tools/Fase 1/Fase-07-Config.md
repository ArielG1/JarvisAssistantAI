# Micro-fase J1.07 — Configuración persistente

## Objetivo

Sacar del código los valores que hoy están hardcodeados (URL de Cerebro,
nombre del modelo de Ollama, timeouts) hacia un `jarvis.config.toml`, mismo
espíritu que la Fase 01 de Cerebro (Config y Paths).

## Prerrequisitos

- J1.06 completa

## Tareas

### T1 — Definir el archivo de config

```toml
[cerebro]
base_url = "http://localhost:8080"
timeout_secs = 15

[ollama]
base_url = "http://localhost:11434"
model = "llama3:8b"

[ui]
panels = ["sistema", "cerebro", "modulos"]
```

### T2 — Carga en Rust

Struct `JarvisConfig` tipada (serde), resuelta al boot antes de
`run_boot_sequence()`, con path por defecto junto al ejecutable y override
por variable de entorno (`JARVIS_CONFIG_PATH`), igual criterio que
`CEREBRO_DATA_DIR` en Cerebro.

### T3 — Reemplazar hardcodeos

`ask_cerebro`, los checks de Ollama en la boot sequence, y la lista de
paneles del HUD pasan a leer de `JarvisConfig` en vez de valores fijos.

### T4 — Verificación

Cambiar `model` o `base_url` en el `.toml` y confirmar que JARVIS usa el
nuevo valor sin recompilar.

## Entregable

Todo lo configurable vive en `jarvis.config.toml`, no en el código fuente.
