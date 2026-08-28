# Micro-fase J1.04 — Panel de chat (sin conectar)

## Objetivo

Agregar al HUD un input de texto y un historial de mensajes, con eco local
(lo que el usuario escribe aparece como mensaje, y una respuesta simulada
aparece del otro lado) para validar la UI de conversación antes de cablear
la llamada real a Cerebro en J1.06.

## Prerrequisitos

- J1.03 completa

## Tareas

### T1 — Contenedor de chat en el HUD

Un panel nuevo (respeta el mismo sistema visual que `PANEL_CONFIG`, pero con
su propio tipo `type:'chat'` ya que necesita layout distinto: lista de
mensajes con scroll + input fijo abajo). Ubicación sugerida: franja inferior
central, sin tapar el cerebro de partículas.

### T2 — Estructura de mensaje

```js
{ role: 'user' | 'jarvis', text: string, ts: number }
```

Historial en memoria (array), sin persistencia todavía (llega en J1.07 si
hace falta).

### T3 — Input y envío

Campo de texto + Enter para enviar. Al enviar: se agrega el mensaje del
usuario al historial, se limpia el input, y (por ahora) se agrega una
respuesta de eco fija tipo `"[mock] recibí: <texto>"` después de un delay
corto simulado.

### T4 — Disparo de estados en el eco

Mientras "espera" el eco simulado: estado `pensando`. Al mostrar la
respuesta: estado `respondiendo` por unos segundos y vuelta a reposo (sin
estado activo, o el color por defecto cian de fondo).

### T5 — Verificación

Escribir varios mensajes seguidos, confirmar que el historial se acumula
correctamente, el input no se traba, y los cambios de color de estado se
ven en el HUD en sincronía con el eco.

## Entregable

Panel de chat visualmente terminado y funcional con datos simulados —
listo para reemplazar el eco por la llamada real a Cerebro en J1.06.
