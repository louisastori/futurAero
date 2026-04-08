# Conformance - perception

Statut: verified

## Requirements Coverage

### Requirement: Sensor Runs Stay Reproducible And Inspectable
- Evidence: `crates/faero-perception/src/lib.rs` produit des runs structures (resume, frames, comparaison).
- Evidence: structure `PerceptionRun` persistee comme entite lisible.
- Verification: `cargo test -p faero-perception`

### Requirement: Occupancy And Comparison Artifacts Stay Readable
- Evidence: `crates/faero-storage/src/lib.rs` persiste `occupancy-map.json`, `comparison.json`, `frames.jsonl`.
- Evidence: test artifact global couvre la famille perception.
- Verification: `cargo test -p faero-storage save_project_writes_simulation_perception_commissioning_optimization_and_ai_artifact_files`

### Requirement: Perception Context Is Usable By AI
- Evidence: `crates/faero-ai/src/lib.rs` detecte les demandes perception et adapte limites/confiance si artefacts absents.
- Evidence: references contexte explicites dans `AiStructuredExplain`.
- Verification: `cargo test -p faero-ai`
