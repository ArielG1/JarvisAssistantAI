# Micro-fase J1.5-08 — Config final + verificación end-to-end

## Objetivo

Consolidar toda la configuración nueva de la Fase 1.5 en un solo
`jarvis.config.toml` coherente, y correr una checklist completa antes de
dar la fase por cerrada.

## Prerrequisitos

- J1.5-01 a J1.5-07 completas

## `jarvis.config.toml` consolidado

> **Nota:** el bloque `[ollama]` de abajo queda reemplazado por `[llm]`
> si se aplicó J1.5-01b (migración a llama.cpp) — ver esa micro-fase
> para el detalle. Se deja el ejemplo con `[llm]` directamente, que es
> la recomendación vigente.

```toml
[cerebro]
base_url = "http://localhost:8765"
timeout_secs = 15
binary_path = "C:/ruta/a/cerebro.exe"
idle_timeout_secs = 600

[llm]
binary_path = "C:/llama.cpp/llama-server.exe"
model_path = "C:/modelos/Qwen2.5-3B-Instruct-Q4_K_M.gguf"
port = 8081
gpu_layers = 15
context_size = 4096

[searxng]
base_url = "http://localhost:8080"
# idle_timeout_secs eliminado: SearXNG no tiene idle timeout (siempre activo, ver FIXED.md FIX 1)

[spotify]
client_id = "..."
client_secret = "..."

[search]
trigger_words = ["hoy", "ahora", "último", "reciente", "buscá", "googleá"]

[ui]
cards = ["estado", "sistema", "cerebro", "modulos"]
```

## Checklist de verificación end-to-end

- [ ] Abrir JARVIS con Ollama y Cerebro apagados → boot solo pide Ollama
      y Modelo, auto-arranca Ollama, llega al HUD con la card de Cerebro
      en "Desconectado".
- [ ] Preguntar algo que sí está en la base de Cerebro → se levanta solo,
      responde con datos reales, y tras el `idle_timeout_secs` se apaga.
- [ ] Preguntar algo de actualidad con Cerebro sin contexto → pasa por
      SearXNG (se levanta solo si no estaba), responde citando la fuente.
- [ ] Preguntar algo conversacional trivial ("hola") → no dispara ninguna
      búsqueda ni levanta SearXNG innecesariamente.
- [ ] "Poneme <canción> en YouTube" → abre el navegador con el video
      correcto.
- [ ] "Poneme <canción> en Spotify" → abre/enfoca la app de Spotify
      reproduciendo el tema correcto.
- [ ] Dejar la app abierta sin uso el tiempo suficiente → confirmar en el
      administrador de tareas que Cerebro se apagó por idle timeout
      (SearXNG permanece activo, ver FIXED.md FIX 1).
- [ ] Verificar que SearXNG está corriendo al inicio (`docker ps` muestra
      `jarvis-searxng`).

## Entregable — cierre de Fase 1.5

JARVIS con boot liviano y confiable, tres niveles de fallback de
información (Cerebro → web → modelo a ciegas), y dos capacidades de
medios (YouTube, Spotify), todo con Cerebro y SearXNG corriendo
únicamente cuando hacen falta.

## Cambios respecto a FIXED.md

| Cambio | Fuente | Descripción |
|--------|--------|-------------|
| `[searxng] idle_timeout_secs` eliminado | FIX 1 | SearXNG ya no tiene idle timeout; siempre está corriendo como servicio Docker persistente. |
| Checklist "se apagaron solos" actualizado | FIX 1 | Solo Cerebro se apaga por idle timeout. SearXNG permanece activo. |
| Checklist "SearXNG corriendo al inicio" agregado | FIX 1 | `docker ps` debe mostrar `jarvis-searxng` al iniciar el sistema. |
