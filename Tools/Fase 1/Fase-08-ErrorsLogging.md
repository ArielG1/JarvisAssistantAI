# Micro-fase J1.08 — Manejo de errores y logging

## Objetivo

Endurecer la Fase 1 para uso diario: que ningún fallo (Ollama caído, Cerebro
caído a mitad de sesión, timeout) deje a JARVIS en un estado roto o
silencioso, y que quede un log en disco para diagnosticar problemas.

## Prerrequisitos

- J1.07 completa

## Tareas

### T1 — Logging a archivo

Usar `tracing` + `tracing-appender` (mismo criterio que la Fase 14 de
Cerebro) escribiendo a `<data_dir>/logs/jarvis.log`, con nivel configurable
desde `jarvis.config.toml`.

### T2 — Reintento manual desde la UI

Si un step de la boot sequence falla, mostrar un botón "Reintentar" en esa
misma pantalla en vez de forzar a reiniciar toda la app.

### T3 — Cerebro caído durante una conversación

Si `send_to_cerebro` falla a mitad de sesión (no en el boot, sino después),
el mensaje de error aparece en el historial de chat y el HUD vuelve a
reposo — no se cuelga en `pensando`.

### T4 — Verificación

Apagar Cerebro manualmente en medio de una conversación activa y confirmar
que JARVIS lo reporta en el chat sin trabarse ni crashear. Revisar que el
log en disco registró el error con detalle suficiente para diagnosticarlo.

## Entregable — cierre de Fase 1

JARVIS arranca mostrando el progreso de sus dependencias, muestra el HUD
completo, y sostiene una conversación de texto estable con Cerebro,
recuperándose de forma prolija de los fallos más comunes (Ollama caído,
Cerebro caído, timeouts).
