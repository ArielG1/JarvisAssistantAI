# Micro-fase J1.5-06 — Buscar y reproducir en YouTube

## Objetivo

Que JARVIS entienda pedidos tipo "buscame/poneme <canción/video> en
YouTube" y lo abra directamente en el navegador por defecto.

## Prerrequisitos

- J1.5-04 (`search_web` ya soporta `category: "videos"` vía SearXNG)

## Tareas

### T1 — Detección de intención

Reconocer el pedido antes de mandarlo al flujo normal de chat/Cerebro —
un patrón simple (regex o palabras clave: "youtube", "poneme el video
de", "reproducime") alcanza para esta fase; no hace falta NLU sofisticado
todavía.

### T2 — Búsqueda filtrada a YouTube

```rust
let results = search_web(&query_sin_comando, "videos").await?;
let first = results.iter().find(|r| r.url.contains("youtube.com"));
```

### T3 — Abrir en el navegador

```rust
use std::process::Command;

#[cfg(target_os = "windows")]
fn open_url(url: &str) -> Result<(), String> {
    Command::new("cmd").args(["/C", "start", "", url]).spawn()
        .map(|_| ()).map_err(|e| e.to_string())
}
```

(Ajustar por plataforma si en algún momento se compila para algo además
de Windows; por ahora, dado el hardware/entorno actual, alcanza con la
rama de Windows.)

### T4 — Respuesta en el chat

Confirmación breve en el chat ("Abriendo *<título del video>* en
YouTube") en vez de devolver un bloque largo de resultados — el objetivo
es la acción (abrir y reproducir), no una lista para elegir, salvo que no
haya un resultado claro.

### T5 — Verificación

Pedir "poneme la canción X en YouTube" con Cerebro apagado → confirmar
que SearXNG se levanta (si no estaba), encuentra un resultado de YouTube,
y se abre en el navegador por defecto reproduciendo.

## Entregable

Comando de voz/texto "reproducir X en YouTube" funcionando de punta a
punta.
