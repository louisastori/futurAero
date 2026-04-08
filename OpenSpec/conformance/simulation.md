# Conformance - simulation

Statut: verified

## Requirements Coverage

### Requirement: Deterministic Fixed-Step Runs
- Evidence: `crates/faero-sim/src/lib.rs` calcule un run deterministe a seed, pas fixe et version moteur donnes.
- Evidence: test `run_simulation_is_deterministic_for_a_seed`.
- Verification: `cargo test -p faero-sim run_simulation_is_deterministic_for_a_seed`

### Requirement: Persisted Run Artifacts
- Evidence: `crates/faero-storage/src/lib.rs` ecrit `summary`, `metrics`, `timeline`, `signals`, `controller`, `contacts`.
- Evidence: test `save_project_writes_simulation_perception_commissioning_optimization_and_ai_artifact_files`.
- Verification: `cargo test -p faero-storage save_project_writes_simulation_perception_commissioning_optimization_and_ai_artifact_files`

### Requirement: Timeline Readable By UI And AI
- Evidence: `SimulationSummary` expose `timeline_samples`, `signal_samples`, `controller_state_samples`, `contacts`.
- Evidence: lecture des artefacts JSON/JSONL par `faero-storage::load_project`.
- Verification: `cargo test -p faero-sim && cargo test -p faero-storage`
