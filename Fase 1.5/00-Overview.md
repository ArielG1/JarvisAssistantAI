# JARVIS — Fase 1.5: Búsqueda web + medios + boot bajo demanda

## Objetivo de la fase

Sobre la Fase 1 ya terminada, agregar:

1. Un rediseño del boot: **Ollama + Modelo siguen siendo obligatorios y se
   auto-arrancan** si no están corriendo; **Cerebro deja de bloquear el
   arranque** y pasa a levantarse a discreción, solo cuando una consulta
   lo necesita.
2. Un mecanismo genérico de **procesos "a discreción"** (lazy): arrancar
   bajo demanda y apagar solos tras un rato sin uso — usado primero para
   Cerebro y después para SearXNG, así no queda RAM ocupada por servicios
   que no se están usando en ese momento.
3. **Búsqueda web (SearXNG autohospedado, bajo demanda)** como fallback
   cuando Cerebro no tiene contexto y la consulta parece necesitar
   información actual.
4. Dos capacidades de medios que reutilizan esa misma búsqueda:
   **buscar y reproducir un video de YouTube** en el navegador, y
   **buscar y reproducir una canción en Spotify** (delegando la
   reproducción a la app de Spotify ya instalada).

## Por qué en este orden

El manejador de procesos "a discreción" (micro-fase 02) es la pieza que
reutilizan tanto Cerebro (03) como SearXNG (04) — por eso va primero,
sola, sin mezclarse con la lógica específica de cada servicio.

YouTube (06) y Spotify (07) van después de que la búsqueda web (05) ya
esté funcionando, porque YouTube reutiliza literalmente el mismo
SearXNG (categoría de videos) — no es una integración nueva desde cero.

## Micro-fases

| # | Nombre | Qué deja funcionando |
|---|--------|----------------------|
| J1.5-01 | Boot rediseñado | Ollama+Modelo obligatorios y auto-arrancados; Cerebro ya no bloquea el arranque |
| J1.5-02 | Manejador de procesos a discreción | Módulo genérico: lanzar un proceso bajo demanda, healthcheck, apagar tras inactividad |
| J1.5-03 | Cerebro a discreción | Cerebro usa el manejador genérico en vez de ser parte del boot |
| J1.5-04 | SearXNG bajo demanda | Contenedor Docker de SearXNG, levantado/apagado con el mismo manejador |
| J1.5-05 | Fallback de búsqueda web en el chat | Cuándo buscar, cómo se arma el contexto, aviso visible en el chat |
| J1.5-06 | Buscar y reproducir en YouTube | Pedido en chat → SearXNG (videos) → abrir el resultado en el navegador |
| J1.5-07 | Buscar y reproducir en Spotify | Client Credentials de Spotify → buscar track → abrir `spotify:track:<id>` |
| J1.5-08 | Config final + verificación end-to-end | Todo consolidado en `jarvis.config.toml`, checklist de pruebas |

## Prerrequisitos generales

- Fase 1 completa (J1.01–J1.08)
- Docker instalado y funcionando (para SearXNG)
- Una app registrada en el dashboard de Spotify for Developers (Client ID
  + Secret, gratis, sin necesidad de que el usuario final haga login) —
  se pide recién en J1.5-07
