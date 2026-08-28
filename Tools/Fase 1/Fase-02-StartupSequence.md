# Micro-fase J1.02 — Secuencia de arranque (boot sequence)

## Objetivo

Antes de mostrar el HUD principal, JARVIS debe verificar y levantar sus
dependencias externas, mostrando en pantalla cada paso a medida que ocurre
(no una barra de carga genérica). Si algo falla, se muestra el error ahí
mismo en vez de arrancar a medias.

Esto reemplaza el placeholder de J1.01 con una pantalla de arranque real.

## Prerrequisitos

- J1.01 completa

## Pasos a chequear (orden fijo)

1. **Ollama** — ¿el servicio está corriendo? (`GET http://localhost:11434/api/version`)
   Si no responde, se intenta levantar (`ollama serve` como proceso hijo).
2. **Modelo cargado** — ¿el modelo configurado ya está en memoria? (`GET /api/ps`
   o un `POST /api/generate` de prueba con `keep_alive`). Si no está cargado,
   se dispara la carga y se muestra "cargando modelo… puede tardar".
3. **Cerebro (servidor MCP/HTTP)** — ¿responde su endpoint de salud?
   Si no está corriendo y JARVIS tiene permiso/ruta configurada para
   levantarlo, lo inicia como proceso hijo; si no, muestra error claro
   ("Cerebro no está corriendo, inicialo manualmente o configurá la ruta").
4. **Listo** — transición al HUD principal (J1.03).

## Tareas

### T1 — Modelo de estado del boot (Rust)

```rust
#[derive(Clone, serde::Serialize)]
enum BootStepStatus { Pending, Running, Ok, Error(String) }

#[derive(Clone, serde::Serialize)]
struct BootStep { id: String, label: String, status: BootStepStatus }
```

Lista fija de steps: `ollama`, `modelo`, `cerebro`.

### T2 — Emisión de eventos al frontend

Cada cambio de estado de un step se emite como evento Tauri
(`window.emit("boot-step", step)`) para que la UI lo pinte en tiempo real,
en vez de esperar a que termine todo el proceso.

### T3 — Lógica de arranque (Rust, async)

Función `run_boot_sequence()` que ejecuta los 3 checks en orden, actualiza y
emite cada `BootStep`, y devuelve `Ok(())` o el primer error bloqueante.
Los checks de red usan un timeout corto (ej. 3s) para no colgar la UI si un
proceso no va a responder.

### T4 — Pantalla de arranque (frontend) — Revisión 2

La pantalla de boot muestra el **mismo componente `ParticleBrain`** que el
HUD principal (ver J1.03 Revisión 2), no un texto suelto sin cerebro — así
la identidad visual es consistente desde el primer segundo. Debajo del
cerebro van los 3 ítems en una lista simple, estética JARVIS
(monoespaciada, cian):

```
[ ✓ ] OLLAMA............... en línea
[ ●●● ] MODELO (llama3:8b).. cargando
[   ] CEREBRO............... pendiente
```

Se escucha el evento `boot-step` y se actualiza el renglón correspondiente.

**Manejo de error (ver `jarvis-boot-error.html` como referencia):**
- El o los steps en error se pintan en rojo, con un botón **individual**
  para iniciar ese proceso puntual cuando aplica (ej. "Iniciar" junto a
  Cerebro si no está corriendo).
- Además hay un botón **"Reintentar todo"** que vuelve a chequear los 3
  ítems por igual — no solo el que falló. Esto es intencional: si el
  usuario abrió Ollama manualmente mientras miraba la pantalla de error,
  o cargó el modelo por su cuenta, el reintento global tiene que
  detectarlo sin que haga falta reiniciar la app.
- No se avanza al HUD principal hasta que los 3 ítems queden en `Ok`.

### T5 — Verificación

- Con Ollama y Cerebro apagados: los tres steps deben mostrar el intento y,
  si no logran levantar, el error específico de cada uno (no un error
  genérico).
- Con todo corriendo: la secuencia pasa por los tres steps en `Ok` y
  transiciona sola al HUD.

## Entregable

Pantalla de arranque funcional que verifica/levanta Ollama, el modelo y
Cerebro, mostrando el progreso real paso a paso antes de mostrar el HUD.
