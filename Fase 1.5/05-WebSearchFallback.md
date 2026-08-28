# Micro-fase J1.5-05 — Fallback de búsqueda web en el chat

> **Nota:** si se aplicó J1.5-01b (migración a `llama.cpp`), todas las
> referencias a `ask_ollama` de este documento pasan a ser `ask_llm`
> (mismo rol, distinto backend) — ver esa micro-fase para la firma
> exacta.


## Objetivo

Cuando Cerebro no tiene contexto relevante, en vez de caer directo al
modelo local "a ciegas" (como en Fase 1), evaluar si conviene buscar en
la web primero y usar esos resultados como contexto adicional.

## Prerrequisitos

- J1.5-04 (`search_web` funcionando)
- Fase 1 completa (fallback Cerebro→Ollama ya existente en `cerebro.rs`)

## Criterio de cuándo buscar

Con un modelo de 3B no conviene delegar la decisión a tool-calling nativo
del modelo (poco confiable a ese tamaño). Se usa un criterio explícito:

- Buscar SI: Cerebro no tuvo contexto relevante, Y (la consulta contiene
  palabras que sugieren actualidad — "hoy", "ahora", "último", una fecha
  reciente, un año — O el usuario lo pide explícitamente — "buscá en
  internet", "googleá" — O la consulta menciona algo que claramente no es
  del dominio de Cerebro, como una pregunta general de cultura/noticias).
- NO buscar SI: la consulta es conversacional simple ("hola", "gracias",
  "¿cómo estás?") — evita gastar el arranque de SearXNG en charla trivial.

Esta heurística puede vivir como una función simple en Rust
(`should_search(query: &str) -> bool`) con la lista de palabras clave en
config para poder ajustarla sin recompilar.

## Tareas

### T1 — `should_search`

Función con la heurística de arriba, configurable vía
`jarvis.config.toml` (lista de palabras gatillo).

### T2 — Encadenar el flujo en `send_to_cerebro`

```
1. ask_cerebro(query)
   -> si responde con contexto: listo (source: "cerebro")
2. si no responde o sin contexto Y should_search(query):
   -> search_web(query, "general")
   -> armar prompt: pregunta + top 3-4 resultados (título+snippet)
   -> ask_ollama(prompt) (source: "web")
3. si no aplica buscar, o la búsqueda falla:
   -> ask_ollama(query) directo (source: "ollama"), igual que en Fase 1
```

### T3 — Aviso visible en el chat + estado `trabajando`

Mientras busca en la web, el estado pasa a `trabajando` (no `pensando`,
que queda para cuando solo está esperando al modelo) — así la turbulencia
más intensa de `trabajando` (definida en el HUD de Fase 1) comunica que
está haciendo algo más "externo". El mensaje de aviso en el chat pasa a
poder tener 3 variantes de `source`: `cerebro` (sin aviso), `web` ("Sin
resultados en Cerebro — buscando en internet..."), `ollama` (igual que
Fase 1, sin resultados de ningún lado).

### T4 — Citar la fuente cuando la respuesta viene de la web

El mensaje final debería poder incluir de dónde salió la info (al menos
el dominio de la fuente principal), para que el usuario pueda verificar.

### T5 — Verificación

- Pregunta claramente factual/de actualidad con Cerebro apagado y sin
  contexto → confirmar que pasa por SearXNG y la respuesta refleja info
  real y reciente.
- Pregunta conversacional trivial ("hola") → confirmar que NO dispara una
  búsqueda (revisar que SearXNG no se levanta para esto).

## Entregable

El chat ahora tiene 3 niveles de fallback: Cerebro → búsqueda web → modelo
local a ciegas, con aviso claro de cuál se usó en cada respuesta.
