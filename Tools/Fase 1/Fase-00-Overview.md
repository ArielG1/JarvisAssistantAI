# JARVIS — Fase 1: Interfaz + Chat con Cerebro

## Objetivo de la fase

Tener una app de escritorio (Tauri) funcional con la estética de JARVIS, que al
iniciar levante sus dependencias mostrando el progreso, y que permita chatear
por texto con Cerebro a través de su HTTP API. Sin voz todavía (STT/TTS
llegan en Fase 2 y 3).

Esta fase se divide en micro-fases pensadas para ejecutarse en un contexto
chico cada una — cada micro-fase es independiente, compila y es verificable
antes de pasar a la siguiente, igual que el esquema de `construct/` de
Cerebro.

## Micro-fases

| # | Nombre | Qué deja funcionando |
|---|--------|----------------------|
| J1.01 | Bootstrap del proyecto Tauri | `cargo tauri dev` abre una ventana vacía con el tema oscuro base |
| J1.02 | Secuencia de arranque (boot sequence) | Pantalla de arranque que chequea/levanta Ollama + modelo y el servidor de Cerebro, mostrando cada paso |
| J1.03 | Shell visual (HUD) | El HUD de partículas + paneles + dock, portado del mockup, con el estado `escuchando` definido pero sin uso |
| J1.04 | Panel de chat | Input de texto + historial de mensajes, integrado al HUD, sin conectar todavía (eco local) |
| J1.05 | Cliente HTTP de Cerebro | Comando Tauri (Rust) que llama a la HTTP API de Cerebro |
| J1.06 | Conexión chat ↔ Cerebro | El input real dispara la consulta a Cerebro y muestra la respuesta, con estados `pensando`/`respondiendo` |
| J1.07 | Configuración persistente | `jarvis.config.toml`: URL de Cerebro, modelo de Ollama, paneles activos |
| J1.08 | Manejo de errores y logging | Qué pasa si Ollama no está, si Cerebro no responde, timeouts, reintentos, log a archivo |

## Prerrequisitos generales de la fase

- Rust estable (1.75+) y `cargo tauri` instalados
- Cerebro (fases 00-16) corriendo localmente, HTTP API accesible
- Ollama instalado, con al menos un modelo descargado (a definir cuál en J1.02)

## Convención de estado visual

Los cuatro estados ya están definidos en el HUD, pero en esta fase **solo se
usan `pensando` y `respondiendo`**. `escuchando` y `trabajando` quedan
declarados y con su color asignado, listos para engancharse en fases
posteriores (voz y orquestación) sin tener que tocar el sistema de estados.

| Estado | Color | Uso en Fase 1 |
|---|---|---|
| escuchando | cian | Declarado, sin uso (llega en Fase 2 con STT) |
| pensando | violeta | Mientras espera respuesta de Cerebro |
| trabajando | ámbar | Declarado, sin uso (llega en Fase 4 con orquestación) |
| respondiendo | verde | Mientras se muestra/lee la respuesta |
