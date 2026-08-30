# Micro-fase J1.5-01 — Boot rediseñado (Ollama auto-arrancado, Cerebro ya no bloquea)

> **Nota:** si se aplica J1.5-01b (migración de Ollama a `llama.cpp` /
> `llama-server`), los pasos "Ollama" y "Modelo" de esta micro-fase se
> **fusionan en un solo paso `check_llm`** — ver esa micro-fase para el
> reemplazo exacto de `check_ollama`/`check_modelo`. El resto de este
> documento (sacar a Cerebro del boot obligatorio) no cambia.


## Objetivo

Cambiar el contrato del boot definido en J1.02: de "3 ítems obligatorios"
a "2 obligatorios (Ollama + Modelo, auto-arrancados) + Cerebro fuera del
boot". El HUD debe poder mostrarse aunque Cerebro no esté corriendo.

## Prerrequisitos

- J1.02 completa (la base de `run_boot_sequence` y los eventos `boot-step`)

## Cambios sobre `check_ollama` (auto-arranque real)

Hasta ahora `check_ollama` solo esperaba/pollaba. Ahora, si tras el primer
chequeo no responde, JARVIS **lanza el proceso él mismo**:

### T1 — Lanzar `ollama serve` como proceso hijo

```rust
use std::process::{Command, Stdio};

fn spawn_ollama() -> Result<(), String> {
    Command::new("ollama")
        .arg("serve")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("No se pudo iniciar Ollama automáticamente: {e}"))
}
```

`check_ollama` pasa a ser: probar `/api/version` → si falla, `spawn_ollama()`
→ pollear unos segundos más → si sigue sin responder, ahí sí es error (con
mensaje que distingue "no se pudo lanzar" de "se lanzó pero no arrancó a
tiempo").

### T2 — Precarga real del modelo

`check_modelo` se mantiene igual que en J1.02 (un `/api/generate` de
prueba con el modelo configurado, que de paso lo deja cargado en memoria
para la primera consulta real).

### T3 — Sacar a Cerebro del boot obligatorio

`run_boot_sequence` pasa a chequear **solo** Ollama y Modelo. Cerebro deja
de tener un paso en la pantalla de arranque — su verificación/arranque se
mueve completamente a la micro-fase 03 (manejador a discreción).

```rust
#[tauri::command]
pub async fn run_boot_sequence(app: AppHandle) -> Result<(), String> {
    // paso 1: ollama (con auto-arranque)
    // paso 2: modelo
    // (cerebro ya NO va acá)
    Ok(())
}
```

### T4 — HUD y pantalla de boot con 2 ítems, no 3

Actualizar `BootSequence.vue` (o el componente que corresponda) para
mostrar solo `OLLAMA` y `MODELO`. El indicador de estado de Cerebro se
muestra en el HUD principal (card "CEREBRO · MCP"), no en el boot — ahí
puede decir "Desconectado" tranquilamente sin que eso sea un error de
arranque.

### T5 — Verificación

- Con Ollama apagado antes de abrir JARVIS: confirmar que el log/consola
  muestra que se lanzó el proceso, y que el boot avanza solo sin
  intervención manual.
- Con Cerebro apagado: confirmar que JARVIS igual llega al HUD normal
  (con la card de Cerebro en "Desconectado"), sin quedarse trabado en el
  boot.

## Entregable

Boot de 2 pasos (Ollama auto-arrancado + Modelo), llegando siempre al HUD
aunque Cerebro esté apagado.
