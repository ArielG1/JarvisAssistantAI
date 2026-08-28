# Micro-fase J1.01 — Bootstrap del proyecto Tauri

## Objetivo

Crear el esqueleto del proyecto `jarvis-shell`, independiente del crate de
Cerebro, con Tauri configurado y una ventana que abre en negro con el tema
base. Sin lógica todavía: esta fase solo prueba que la cadena de build
funciona de punta a punta.

## Prerrequisitos

- Ninguno (primera micro-fase)

## Tareas

### T1 — Crear el proyecto

```bash
cargo install tauri-cli --locked
cargo tauri init --app-name jarvis-shell --window-title "J.A.R.V.I.S."
```

Estructura esperada:

```
jarvis-shell/
├── src/              # frontend (HTML/CSS/JS, sin framework)
│   └── index.html
├── src-tauri/
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
```

### T2 — Config base de la ventana

En `tauri.conf.json`: ventana sin decoración de sistema estándar (o con ella,
a decidir en J1.03 al portar el HUD), tamaño mínimo 1024x640, fondo
`#030a12` para evitar flash blanco al cargar.

### T3 — `index.html` mínimo

Placeholder oscuro con el texto `J.A.R.V.I.S. — booting` centrado, solo para
confirmar que el webview renderiza.

### T4 — Verificación

```bash
cargo tauri dev
```

Debe abrir una ventana con fondo oscuro y el texto placeholder. Sin errores
en consola.

## Entregable

Repo `jarvis-shell` compilando y abriendo ventana. Nada más — el HUD real se
porta en J1.03.
