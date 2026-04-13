## 1. Simulation Report Contract

- [x] 1.1 Etendre `crates/faero-types` et `crates/faero-sim` pour produire des contacts localises et un bloc `report` persiste dans le resultat de run.
- [x] 1.2 Deriver de maniere deterministe le headline, les findings, les critical event ids et les recommended actions a partir des artefacts du run.

## 2. Desktop And Storage Integration

- [x] 2.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour persister `report` et la localisation des collisions dans `SimulationRun`.
- [x] 2.2 Mettre a jour `crates/faero-storage` pour serialiser et recharger ce shape complet sans cache parallele.

## 3. Fallback, UI And AI

- [x] 3.1 Aligner le fallback web, `apps/desktop/src/App.test-helpers.jsx` et l auto-prompt IA sur le nouveau bloc `report`.
- [x] 3.2 Exposer dans le shell desktop les collisions localisees, le rapport de run et les evenements critiques associes.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust sur les collisions localisees, le rapport nominal et le round-trip `.faero`.
- [x] 4.2 Ajouter des tests UI ou desktop sur l affichage du rapport de run et la focalisation sur les evenements critiques.
