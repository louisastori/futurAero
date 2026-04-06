# FutureAero

Scaffold monorepo initial pour FutureAero, aligne sur les `OpenSpec/`.

## Contenu courant

- `OpenSpec/`: specifications produit et techniques
- `crates/`: noyau Rust minimal, stockage `.faero`, stubs integration et plugin host
- `schemas/`: schemas JSON de base pour `Command`, `Event`, `Job` et telemetrie
- `examples/projects/`: fixtures officielles `.faero`
- `apps/desktop/` et `packages/`: shell desktop `Tauri + React` initial et menu workspace

## GitHub

- remote canonique: `origin -> https://github.com/louisastori/futurAero.git`
- branche locale initiale: `main`
- CI cible: GitHub Actions via `.github/workflows/ci.yml`

## Commandes utiles

- `cargo test`
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `npm install`
- `npm run dev:web`
- `npm run dev`
- `powershell -ExecutionPolicy Bypass -File .\\scripts\\test.ps1`

## Statut

Le depot est bootstrappe pour le noyau Rust, les fixtures et un premier shell desktop `Tauri + React`. Le viewport, les panneaux et les menus sont reels cote UI, mais les workflows metier restent encore des premieres integrations.
