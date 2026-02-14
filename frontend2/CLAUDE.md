# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development

All cargo commands must be run through the Nix dev shell:

```bash
nix develop --command cargo leptos watch    # Dev server with HMR at http://127.0.0.1:3000
nix develop --command cargo leptos build    # Build (dev)
nix develop --command cargo leptos build --release  # Production build
```

End-to-end tests (Playwright, runs from `end2end/` directory):
```bash
npx playwright test
```

Formatter for Leptos view macros: `leptosfmt`

## Architecture

This is a **Leptos 0.8 SSR + Hydration** app with **Three.js** for 3D rendering, served by **Actix-web**.

### Dual compilation targets

The crate compiles twice — once as a server binary (`ssr` feature) and once as a WASM library (`hydrate` feature). Code must be gated appropriately:

- `#[cfg(feature = "ssr")]` — server-only code (HTTP responses, file serving)
- `#[cfg(not(feature = "ssr"))]` — client-only code (wasm-bindgen imports, DOM effects)

### Three.js integration pattern

`three.js` (project root) is bound via `wasm_bindgen(module = "/three.js")`. wasm-bindgen copies it into `target/site/pkg/snippets/`. Key constraints:

- **No ES module imports** in `three.js` — CDN `import` statements break inside wasm-bindgen snippets. Load libraries via `<script>` tags in `main.rs` and access as `window.THREE`, `window.gsap`, etc.
- **Use `NodeRef` + `Effect::new`** to call JS init functions — a bare `Effect::new` without a reactive dependency fires before DOM hydration completes. Subscribe to a `NodeRef` to ensure the element exists.
- **Guard against double init** — effects may fire more than once; use an `initialized` flag in JS.

### HTML shell

`src/main.rs` contains the server HTML shell (`<head>`, CDN script tags, `<body>`). Add external CDN scripts here.

### Styling

`style/main.css` is the single stylesheet, referenced in `Cargo.toml` under `[package.metadata.leptos]` as `style-file`.
