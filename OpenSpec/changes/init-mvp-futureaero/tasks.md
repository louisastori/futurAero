# Implementation Tasks

## 1. Fondations Et Depot
- [x] 1.1 Le monorepo JS et Rust est en place (`package.json`, `Cargo.toml`, `apps/*`, `packages/*`, `crates/*`).
- [x] 1.2 Le pipeline GitHub Actions couvre Rust, frontend, browser E2E, shell desktop et artefact installateur Windows.
- [x] 1.3 Les crates coeur (`faero-geometry`, `faero-sim`, `faero-ai`, `faero-safety`, `faero-storage`, etc.) ne sont plus vides et portent un premier code exploitable.

## 2. Shell Desktop
- [x] 2.1 L'application desktop `apps/desktop` est initialisee en Tauri + React.
- [x] 2.2 Le layout principal est present (explorateur, proprietes, surface de commandes, viewport, IA locale, sortie, problemes).
- [x] 2.3 Le shell React est relie aux commandes Tauri, aux snapshots projet et aux fixtures de workspace.

## 3. Backend Et Format Projet
- [x] 3.1 Le flux CAO parametrique de base `rectangle -> extrusion -> volume -> masse` est implemente.
- [x] 3.2 Les fixtures `.faero` et les documents OpenSpec lisibles `*.faerospec` sont implementes.
- [x] 3.3 `faero-ai` est relie a un runtime Ollama local avec fallback explicite et sortie structuree.

## 4. Simulation Controle Et Safety
- [x] 4.1 Le modele minimal cellule robotique, signaux et controle attendu par le MVP est implemente.
- [x] 4.2 Les runs de simulation deterministes avec timeline, traces de controle et artefacts persistants sont implementes.
- [x] 4.3 L'analyse safety et les rapports associes sont exposes dans le shell desktop.

## 5. Alignement OpenSpec Actuel
- [x] 5.1 Les specs stables de la racine ont ete migrees vers `OpenSpec/specs/reference/`.
- [x] 5.2 Une premiere spec canonique systeme a ete promue dans `OpenSpec/specs/system/spec.md`.
- [x] 5.3 `OpenSpec/changes/` est reserve aux changements actifs et le corpus long-form est traite comme reference.
- [ ] 5.4 Il reste a decouper d'autres specs canoniques de domaine (`simulation`, `ai`, `data-model`, `safety`) a partir du corpus de reference.

## 6. Travail Produit Restant
- [ ] 6.1 Approfondir l'implementation perception et LiDAR par rapport aux specs de reference.
- [ ] 6.2 Approfondir l'integration industrielle et la connectivite terrain par rapport aux specs de reference.
- [ ] 6.3 Approfondir les flux commissioning, as-built et optimisation au-dela du slice MVP courant.
- [ ] 6.4 Durcir le Plugin SDK, l'automatisation des releases et le workflow de promotion des specs canoniques.
