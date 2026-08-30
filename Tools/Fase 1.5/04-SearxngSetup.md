# Micro-fase J1.5-04 — SearXNG always-on

## Objetivo

Levantar un contenedor Docker de SearXNG **arrancado al boot y siempre
activo durante toda la sesión**, usando `LazyProcessManager` para
gestionarlo, y exponer un comando Rust simple de búsqueda que las
micro-fases siguientes (05 fallback web, 06 YouTube, 07 Spotify) van a
reutilizar.

## Prerrequisitos

- J1.5-02 (manejador genérico)
- Docker instalado y corriendo en la máquina

## Tareas

### T1 — Preparar la imagen/contenedor de SearXNG

Instrucciones para el usuario (no automatizable de forma segura desde el
propio JARVIS en esta fase): `docker pull searxng/searxng` y una config
mínima de SearXNG habilitando salida en JSON (`formats: - json` en su
`settings.yml`), ya que por defecto solo sirve HTML.

**Bug conocido de Docker:** `docker run -d` se desacopla del proceso
llamador. Si el manager registra el PID del `docker run`, pierde
visibilidad sobre el contenedor real. Si el contenedor muere (crash,
`docker kill`, etc.), el manager nunca lo detecta. Por eso se usa
`LazyProcessManager` con healthcheck HTTP, que valida que SearXNG
realmente responde en `http://localhost:8080/healthz` en cada intento.

### T2 — Arranque eagerly al boot

```rust
// main.rs o lib.rs — durante el setup inicial
searxng::ensure_running(&app_handle).await?;
```

`ensure_running` ejecuta `docker start searxng` (contenedor ya creado
manualmente en T1) y confirma que el healthcheck responde. Se llama
**una vez al inicio** de la aplicación, no bajo demanda.

> **Nota:** el módulo `LazyService` se reemplazó por `LazyProcessManager`
> (FIX 4 de FIXED.md) para corregir la gestión de procesos desacoplados
> de Docker.

### T3 — Comando genérico de búsqueda

```rust
#[derive(Deserialize)]
struct SearxResult { title: String, url: String, content: Option<String> }

pub async fn search_web(query: &str, category: &str) -> Result<Vec<SearxResult>, String> {
    searxng::ensure_running(&app_handle).await?;
    let cfg = config::get();
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/search", cfg.searxng.base_url))
        .query(&[("q", query), ("format", "json"), ("categories", category)])
        .send().await
        .map_err(|e| format!("SearXNG no responde: {e}"))?;
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    // parsear body["results"] a Vec<SearxResult>
    todo!()
}
```

`category` es lo que reutilizan las fases siguientes: `"general"` para
búsqueda de texto, `"videos"` para YouTube.

### T4 — Config nueva

```toml
[searxng]
base_url = "http://localhost:8080"
```

(Puerto de ejemplo — ajustar al que se le asigne al contenedor para no
chocar con el 8080 que ya se sacó de uso en Fase 1.)

### T5 — Verificación

- Confirmar que al iniciar JARVIS, `docker ps` muestra el contenedor
  `searxng` corriendo.
- Llamar a `search_web("clima Buenos Aires", "general")` y confirmar
  que devuelve resultados.
- Confirmar que SearXNG **no** se apaga por idle timeout — permanece
  activo hasta que JARVIS se cierre.

## Entregable

`search_web(query, category)` funcionando de punta a punta, con SearXNG
arrancado al boot y permaneciendo activo toda la sesión, listo para
engancharse al chat (05) y a los comandos de medios (06, 07).

---

**Cambio respecto a FIXED.md:** Este documento actualiza el diseño
original que trataba SearXNG como servicio lazy (bajo demanda). El
FIX 1 de FIXED.md convierte SearXNG en always-on porque el timeout
de Docker desacoplaba la salud del contenedor del manager. Ver
FIXED.md para detalles.
