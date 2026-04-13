## 1. Simulation Runner Contract

- [x] 1.1 Etendre `crates/faero-sim` avec un resultat de runner MVP qui expose `summary`, `metrics`, `timelineSamples`, `signalSamples`, `controllerStateSamples`, `contacts` et `progressSamples`.
- [x] 1.2 Persister explicitement dans le contrat de run les entrees deterministes `seed`, `engineVersion`, `stepCount` et le scenario source de `simulation.run.start`.

## 2. Desktop Backend Integration

- [x] 2.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour creer une entite `SimulationRun` portant un bloc `job` lisible avec `jobId`, `status`, `phase`, `progress` et `progressSamples`.
- [x] 2.2 Publier une progression MVP coherente (`queued`, `running`, `trace_persisted`, `completed`) pendant `simulation.run.start` et persister le resultat final dans le projet courant.

## 3. Fallback And UI Exposure

- [x] 3.1 Aligner le fallback web et `apps/desktop/src/App.test-helpers.jsx` sur le meme shape `SimulationRun` persiste, y compris `job`, `summary`, `metrics` et timeline.
- [x] 3.2 Exposer dans le shell desktop une lecture minimale de la progression et des artefacts du run sans contourner le contrat persiste.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust couvrant le contrat du runner MVP, les metadonnees de determinisme et la persistance des artefacts attendus.
- [x] 4.2 Ajouter des tests desktop ou UI couvrant `simulation.run.start`, l affichage de la progression et la lecture des artefacts persistes.
