# Micro-fase J1.5-01b — Migrar de Ollama a llama.cpp (`llama-server`)

> Esta micro-fase reemplaza todas las referencias a Ollama en J1.5-01
> (boot) y en el fallback de `cerebro.rs` (Fase 1 / J1.5-05). Aplicarla
> junto con J1.5-01, antes de seguir con J1.5-02 en adelante.

## Objetivo

Dejar de depender de Ollama (capa de gestión que no se usa, dado que solo
corre como servidor) y hablar directo con `llama-server`, el servidor
HTTP que ya viene incluido en `llama.cpp`. Menos overhead, control real
sobre cuántas capas van a la GPU (clave con los 2GB de la GT1030), y a
futuro se puede embeber el binario junto con JARVIS sin pedirle al
usuario que instale nada aparte.

## Prerrequisitos

- Ninguno técnico — es un cambio de endpoints, no depende de otras
  micro-fases de 1.5. Se recomienda aplicarla junto con J1.5-01 porque
  toca las mismas funciones de arranque.

## Qué cambia

### Modelo: de `ollama pull` a un archivo `.gguf` propio

Bajar manualmente el modelo cuantizado (ej. `Qwen2.5-3B-Instruct-Q4_K_M.gguf`
desde HuggingFace) a una carpeta local. Ya no hay comando de descarga
automática — es un paso manual único, se documenta en el `README` del
proyecto, no en JARVIS.

### Arranque: de `ollama serve` a `llama-server`

```rust
fn spawn_llm() -> Result<(), String> {
    Command::new(&cfg.llm.binary_path) // ej. "C:/llama.cpp/llama-server.exe"
        .args([
            "-m", &cfg.llm.model_path,       // ruta al .gguf
            "--port", &cfg.llm.port.to_string(),
            "-ngl", &cfg.llm.gpu_layers.to_string(), // capas en la GT1030
            "-c", &cfg.llm.context_size.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("No se pudo iniciar llama-server: {e}"))
}
```

`-ngl` (n_gpu_layers) es exactamente el control fino que Ollama no
exponía cómodamente — con 2GB de VRAM probablemente convenga offloadear
solo una parte de las capas y dejar el resto en CPU; esto se ajusta a
mano probando valores (empezar con `-ngl 15` y subir/bajar según uso de
VRAM y velocidad).

### Healthcheck y chequeo de modelo (`boot.rs`)

`llama-server` expone un único endpoint que sirve para ambas cosas —
cuando responde, el modelo ya está cargado (a diferencia de Ollama, no
hay un paso separado de "cargar modelo" después de que el servidor
arranca):

```rust
async fn check_llm() -> Result<(), String> {
    let cfg = config::get();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;
    for _ in 0..8 { // la carga inicial del modelo puede tardar más que un ping de Ollama
        let ok = client
            .get(format!("http://localhost:{}/health", cfg.llm.port))
            .send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        if ok { return Ok(()); }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    Err(format!("llama-server no responde en el puerto {}", cfg.llm.port))
}
```

Esto **reemplaza los dos pasos separados** `check_ollama` +
`check_modelo` de J1.5-01 por un solo paso `check_llm` — el boot queda
con un ítem menos, no dos.

### Generación de texto (`cerebro.rs`, fallback y `ask_ollama`)

`llama-server` expone la misma API que OpenAI (`/v1/chat/completions`),
así que en vez de armar un prompt plano hay que mandar la lista de
mensajes con roles:

```rust
async fn ask_llm(query: &str) -> Result<String, String> {
    let cfg = config::get();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(format!("http://localhost:{}/v1/chat/completions", cfg.llm.port))
        .json(&serde_json::json!({
            "messages": [{ "role": "user", "content": query }],
            "stream": false
        }))
        .send().await
        .map_err(|e| format!("llama-server no responde: {e}"))?;
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    body["choices"][0]["message"]["content"].as_str()
        .map(String::from)
        .ok_or_else(|| "Respuesta inválida de llama-server".into())
}
```

Esta función **reemplaza a `ask_ollama`** en todos los lugares donde se
usa (fallback de Fase 1, y el fallback de 3 niveles de J1.5-05). Se
recomienda renombrarla a `ask_llm` en todo el código para que quede claro
que ya no es específico de Ollama.

### Config

Reemplaza la sección `[ollama]` por `[llm]` en `jarvis.config.toml` (ver
también J1.5-08, que queda actualizado con este bloque):

```toml
[llm]
binary_path = "C:/llama.cpp/llama-server.exe"
model_path = "C:/modelos/Qwen2.5-3B-Instruct-Q4_K_M.gguf"
port = 8081
gpu_layers = 15
context_size = 4096
```

> El puerto se fija explícitamente en **8081**, distinto tanto del 8765
> de Cerebro como del 8080 que ya causó confusión en Fase 1 (Fix 3 de
> `OneFixed.md`) — para no repetir ese error con un puerto por defecto
> ambiguo.

## Tareas

### T1 — Bajar `llama-server` y el modelo `.gguf`

Compilar o descargar el binario de `llama.cpp` (`llama-server`), y bajar
el `.gguf` de Qwen2.5-3B-Instruct cuantizado.

### T2 — Reemplazar `check_ollama`+`check_modelo` por `check_llm` en `boot.rs`

Un solo paso en la secuencia de arranque en vez de dos.

### T3 — Reemplazar `ask_ollama` por `ask_llm` en `cerebro.rs`

Actualizar tanto el fallback simple de Fase 1 como el flujo de 3 niveles
de J1.5-05 para llamar a esta función.

### T4 — Actualizar `jarvis.config.toml`

Sección `[llm]` en vez de `[ollama]`, con `binary_path`, `model_path`,
`port`, `gpu_layers`, `context_size`.

### T5 — Ajustar `gpu_layers` a mano

Probar con `-ngl 15` como punto de partida, subir si sobra VRAM libre,
bajar si `llama-server` falla al cargar o el sistema empieza a swappear.

### T6 — Verificación

- Boot con `llama-server` apagado → confirmar que JARVIS lo lanza solo
  con los argumentos correctos (`-m`, `--port`, `-ngl`), y que el boot
  avanza a "Ok" recién cuando `/health` responde.
- Una consulta de chat sin Cerebro → confirmar que `ask_llm` devuelve una
  respuesta coherente vía `/v1/chat/completions`.
- Confirmar en el administrador de tareas que ya no aparece ningún
  proceso de Ollama corriendo.

## Entregable

JARVIS corriendo su modelo local vía `llama-server` directo, sin Ollama
de por medio, con control explícito de cuántas capas van a la GT1030.
