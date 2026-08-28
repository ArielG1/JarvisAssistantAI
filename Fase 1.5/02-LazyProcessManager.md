# Micro-fase J1.5-02 — Manejador de procesos "a discreción"

## Objetivo

Construir un módulo Rust genérico y reutilizable que sepa: arrancar un
proceso bajo demanda si no está corriendo, esperar a que responda su
healthcheck, y apagarlo solo tras un rato sin uso. Esta micro-fase NO
integra todavía a Cerebro ni a SearXNG — solo deja el mecanismo listo y
probado con un caso trivial.

## Prerrequisitos

- Ninguno específico de esta fase (es independiente de J1.5-01, aunque
  lógicamente van juntas)

## Diseño

```rust
pub struct LazyService {
    pub name: String,
    pub healthcheck_url: String,
    pub start: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
    pub idle_timeout: Duration,
}
```

Estado interno por servicio: `last_used: Instant`, `starting: bool` (para
no lanzar dos arranques en paralelo si llegan dos consultas casi
simultáneas).

### T1 — `ensure_running(&self) -> Result<(), String>`

1. Si el healthcheck responde OK → actualizar `last_used` y devolver `Ok`.
2. Si no responde → si ya hay un arranque en curso, esperar a que termine
   (no lanzar dos veces); si no, llamar a `start()` y pollear el
   healthcheck unos segundos.
3. Actualizar `last_used` al confirmar que ya responde.

### T2 — Vigilante de inactividad (`idle_watcher`)

Una tarea en background (`tokio::spawn`, corre mientras la app viva) que
cada N segundos revisa cada `LazyService`: si `now - last_used >
idle_timeout` y el proceso sigue corriendo, lo apaga (matar el proceso
hijo por PID, guardado al momento de lanzarlo).

### T3 — Registro central de servicios lazy

Un `HashMap<String, LazyService>` (o `Vec`) inicializado una vez al
arrancar la app, con acceso vía `tauri::State` para que cualquier comando
pueda pedir `ensure_running("cerebro")` antes de usarlo.

### T4 — Caso de prueba trivial

Antes de integrar Cerebro o SearXNG (que son más complejos), probar el
mecanismo con algo simple — por ejemplo, un `python -m http.server` de
juguete — para confirmar arranque, healthcheck, uso, e inactividad→apagado
sin depender todavía de Docker ni del binario de Cerebro.

### T5 — Verificación

- Pedir `ensure_running` con el servicio de prueba apagado → confirmar
  que lo levanta y responde.
- Dejarlo inactivo más que `idle_timeout` → confirmar que el vigilante lo
  apaga solo (revisar que el proceso ya no está en la lista de tareas).
- Dos llamadas a `ensure_running` casi simultáneas mientras arranca →
  confirmar que no se lanzan dos procesos duplicados.

## Entregable

Módulo `lazy_service` genérico, probado con un servicio de juguete, listo
para que J1.5-03 y J1.5-04 lo usen con Cerebro y SearXNG respectivamente.
