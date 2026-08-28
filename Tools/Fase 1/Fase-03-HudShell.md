# Micro-fase J1.03 — Shell visual (HUD)

> **Revisión 2** — reemplaza la versión anterior de este documento. El
> diseño del HUD pasó por varias iteraciones de maquetas (ver
> `jarvis-hud-main.html`, `jarvis-boot-ok.html`, `jarvis-boot-error.html`)
> hasta llegar a la especificación final de abajo. Si ya se portó una
> versión anterior a Vue (dots 2D, cards fijas a la derecha), hay que
> actualizarla contra esta revisión antes de seguir con J1.04 en adelante.

## Objetivo

Portar el HUD final —cerebro 3D de partículas, cards flotantes
configurables y ocultables, panel de chat— como la pantalla principal de
`jarvis-shell` (Vue 3 + Pinia), mostrada después de que la boot sequence
(J1.02) termina en `Ok`.

## Prerrequisitos

- J1.02 completa

## Especificación visual final

### Cerebro 3D de partículas

- Renderizado con **Three.js** (no canvas 2D ni SVG estático): nube de
  puntos con volumen real, no un dibujo plano. Silueta generada
  matemáticamente (lóbulo temporal, cerebelo, tallo cerebral, fisura
  longitudinal) — sin modelo 3D importado.
- Tres capas: puntos "núcleo" conectados por líneas (red neuronal),
  "polvo" ambiental de menor tamaño alrededor y dentro, y unas pocas
  "chispas" grandes con resplandor (sprites con textura radial) simulando
  actividad puntual.
- Bloom real vía `UnrealBloomPass` de Three.js, con fallback silencioso si
  esas librerías no cargan (el HUD debe seguir siendo usable sin bloom).
- Rotación automática lenta + arrastre manual con el mouse (pointer
  events), con inercia al soltar.
- **Color ligado al estado activo** (ver más abajo) en puntos, líneas,
  chispas y en el bloom.

### Movimiento real de partículas por estado (no solo color)

Esto es una diferencia clave respecto a versiones anteriores: las
partículas deben **moverse de verdad**, no solo cambiar de tono.

- **`pensando`** y **`trabajando`**: turbulencia — cada partícula (núcleo
  y polvo) oscila alrededor de su posición base con una fase aleatoria
  propia, dando un efecto de agitación real mientras "algo está
  ocurriendo". `trabajando` tiene mayor amplitud que `pensando`. Las
  líneas de conexión siguen a los puntos núcleo ya desplazados (no quedan
  fijas mientras el punto se mueve).
- La turbulencia se activa/desactiva con una transición suave (ease, no
  corte abrupto) al entrar o salir de esos estados.
- **Cambio de estado (cualquiera → cualquiera)**: una "ráfaga" corta
  (~0.7s) de energía — el cerebro se expande levemente, el bloom se
  intensifica, y luego decae — para que el cambio de estado se sienta
  como un evento puntual además del cambio continuo de color/turbulencia.
- `escuchando` y `respondiendo` quedan sin turbulencia (quietos, solo con
  la respiración/pulso sutil de opacidad que ya tenía el diseño base).

### Cards flotantes, alineadas y ocultables

- Las cards (`ESTADO`, `SISTEMA`, `CEREBRO · MCP`, `MÓDULOS`) van dentro
  de **un contenedor flex en columna**, no posicionadas cada una por
  separado con coordenadas fijas — así quedan siempre alineadas y
  espaciadas parejo sin importar el contenido de cada una.
- Estilo vidrio: fondo semitransparente + `backdrop-filter: blur(...)` +
  borde/glow según el color de estado — no paneles opacos de borde recto.
- Una leve animación de "levitar" (traslación vertical de pocos píxeles,
  desfasada entre cards) para que se sientan flotantes sin verse
  desordenadas.
- Cada card tiene un botón para **ocultarla**; al ocultarla colapsa su
  espacio (las demás se reacomodan solas, no queda un hueco) y aparece un
  chip pequeño en una bandeja de "restaurar" para volver a mostrarla.
- La lista de cards y su contenido siguen viniendo de una configuración
  editable (equivalente al `PANEL_CONFIG` de la maqueta), no hardcodeadas
  en el layout.

### Panel de chat

- Ubicado abajo, centrado, con margen respecto al borde inferior de la
  ventana (el cerebro debe seguir siendo el centro visual, el chat va
  "un poco más abajo del centro", no pegado al borde).
- Historial con scroll interno, pero **sin barra de scroll visible** — el
  contenido debe poder scrollearse igual (`scrollbar-width: none` +
  `::-webkit-scrollbar { display: none }`), no ocultar el overflow.
- Burbujas de usuario y de JARVIS con estilo diferenciado; un tercer
  estilo (borde ámbar) para avisos no bloqueantes, como cuando Cerebro no
  responde pero el modelo igual contesta (ver `OneFixed.md`, Fix 2).

### Estados

Sin cambios respecto a la especificación original: `escuchando` (cian),
`pensando` (violeta), `trabajando` (ámbar), `respondiendo` (verde). En
esta micro-fase `escuchando` sigue sin dispararse desde ningún lado real
(no hay voz todavía) — queda declarado y con toda su lógica visual lista
(color, ausencia de turbulencia) para cuando J1.06 en adelante lo
empiece a usar.

## Tareas

### T1 — Componente `ParticleBrain` (Three.js dentro de Vue)

Encapsular toda la lógica de Three.js (escena, cámara, geometría del
cerebro, líneas, polvo, chispas, bloom, turbulencia, ráfaga de cambio de
estado) en un componente Vue dedicado que expone el color/estado activo
como prop o vía el store, y se limpia correctamente (`dispose()` de
geometrías/materiales/renderer) al desmontarse.

### T2 — Componente `FloatingCard` + configuración

Componente genérico de card (header con título + botón ocultar, cuerpo
con slot) reutilizado por las 4 cards actuales. La lista de cards
visibles/ocultas vive en el store de HUD (Pinia), no en estado local de
cada componente, para que sobreviva a re-renders.

### T3 — Store de estado (`useHudStore` o similar)

Estado activo (`escuchando|pensando|trabajando|respondiendo`),
turbulencia derivada del estado, y timestamp del último cambio de estado
(para la ráfaga) centralizados en un store — tanto `ParticleBrain` como
el `state-label` y el dock lo leen de ahí, no cada uno por su cuenta.

### T4 — Transición boot → HUD

Igual que la revisión anterior: fade simple al resolver `Ok` la boot
sequence.

### T5 — Verificación

- Cambiar de estado (vía dock o al enviar un mensaje en J1.06) y
  confirmar visualmente: cambio de color, ráfaga breve, y turbulencia
  real en `pensando`/`trabajando` (no solo tinte).
- Ocultar y restaurar cada card, confirmar que el resto se reacomoda sin
  huecos.
- Verificar que el chat scrollea sin mostrar barra de scroll en
  Windows/Chromium (el motor real de la webview de Tauri).

## Entregable

HUD completo funcionando dentro de la app Tauri: cerebro 3D con
movimiento real ligado al estado, cards flotantes alineadas y
ocultables, y panel de chat con scroll oculto — todo alcanzado después
del arranque.
