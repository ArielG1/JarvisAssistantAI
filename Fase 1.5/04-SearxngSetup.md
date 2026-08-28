# Micro-fase J1.5-04 — SearXNG bajo demanda

## Objetivo

Levantar un contenedor Docker de SearXNG usando el mismo manejador
genérico de J1.5-02, y exponer un comando Rust simple de búsqueda que
las micro-fases siguientes (05 fallback web, 06 YouTube, 07 Spotify) van
a reutilizar.

## Prerrequisitos

- J1.5-02 (manejador genérico)
- Docker instalado y corriendo en la máquina

## Tareas

### T1 — Preparar la imagen/contenedor de SearXNG

Instrucciones para el usuario (no automatizable de forma segura desde el
propio JARVIS en esta fase): `docker pull searxng/searxng` y una config
mínima de SearXNG habilitando salida en JSON (`formats: - json` en su
`settings.yml`), ya que por defecto solo sirve HTML.

### T2 — Registrar SearXNG como `LazyService`

```rust
LazyService {
    name: "searxng".into(),
    healthcheck_url: format!("{}/healthz", cfg.searxng.base_url),
    start: Box::new(|| {
        Command::new("docker")
            .args(["start", "searxng"]) // contenedor ya creado, solo lo arranca
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }),
    idle_timeout: Duration::from_secs(cfg.searxng.idle_timeout_secs),
}
```

(`docker start` de un contenedor existente es más rápido y simple que
`docker run` desde cero cada vez — el contenedor se crea una sola vez de
forma manual siguiendo T1.)

### T3 — Comando genérico de búsqueda

```rust
#[derive(Deserialize)]
struct SearxResult { title: String, url: String, content: Option<String> }

pub async fn search_web(query: &str, category: &str) -> Result<Vec<SearxResult>, String> {
    lazy_registry.ensure_running("searxng").await?;
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
idle_timeout_secs = 300
```

(Puerto de ejemplo — ajustar al que se le asigne al contenedor para no
chocar con el 8080 que ya se sacó de uso en Fase 1.)

### T5 — Verificación

- Con el contenedor apagado, llamar a `search_web("clima Buenos Aires",
  "general")` desde una consola de dev tools o un comando de prueba →
  confirmar que Docker lo levanta solo y devuelve resultados.
- Confirmar que tras `idle_timeout_secs` sin uso, `docker ps` ya no lo
  muestra corriendo.

## Entregable

`search_web(query, category)` funcionando de punta a punta, con SearXNG
completamente a discreción — listo para engancharse al chat (05) y a los
comandos de medios (06, 07).
