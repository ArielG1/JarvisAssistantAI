# Micro-fase J1.06 — Conexión chat ↔ Cerebro

## Objetivo

Reemplazar el eco simulado de J1.04 por la llamada real a `send_to_cerebro`
(J1.05). Este es el hito de la Fase 1: JARVIS conversando de verdad con
Cerebro por texto.

## Prerrequisitos

- J1.04 completa (panel de chat funcional con eco)
- J1.05 completa (comando `send_to_cerebro` probado)

## Tareas

### T1 — Reemplazar el eco por la llamada real

En el handler de envío del panel de chat: en vez del delay simulado, se
llama a `invoke('send_to_cerebro', { message: texto })`, se agrega el
resultado al historial como mensaje de `jarvis`.

### T2 — Estados reales

- Al enviar → estado `pensando` (se mantiene mientras la promesa de
  `invoke` está pendiente).
- Al recibir respuesta → estado `respondiendo` durante unos segundos
  (tiempo estimado de lectura o fijo, ya que todavía no hay TTS que marque
  el fin).
- Si hay error → mostrar el mensaje de error en el historial como mensaje
  de sistema (visualmente distinto, ej. texto ámbar), sin romper el estado
  del HUD.

### T3 — Verificación end-to-end

Con Ollama, el modelo y Cerebro corriendo: abrir JARVIS, esperar el boot,
escribir una pregunta real sobre algo indexado en Cerebro, y confirmar que
la respuesta que llega es coherente con el contenido real de la base de
conocimiento (no un mock).

## Entregable

Fase 1 completa: JARVIS arranca mostrando sus dependencias, muestra el HUD,
y permite chatear por texto con Cerebro usando datos reales.
