# FutureAero

Scaffold monorepo initial pour FutureAero, aligne sur les `OpenSpec/`.

## Contenu courant

- `OpenSpec/`: specifications produit et techniques
- `crates/`: noyau Rust minimal, stockage `.faero`, stubs integration et plugin host
- `schemas/`: schemas JSON de base pour `Command`, `Event`, `Job` et telemetrie
- `examples/projects/`: fixtures officielles `.faero`
- `apps/desktop/` et `packages/`: scaffold UI/desktop minimal

## GitHub

- remote canonique: `origin -> https://github.com/louisastori/futurAero.git`
- branche locale initiale: `main`
- CI cible: GitHub Actions via `.github/workflows/ci.yml`

## Commandes utiles

- `cargo test`
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `powershell -ExecutionPolicy Bypass -File .\\scripts\\test.ps1`

## Statut

Le depot est bootstrappe pour le noyau Rust et les fixtures. Le shell desktop et l'UI sont encore des placeholders de scaffold.
