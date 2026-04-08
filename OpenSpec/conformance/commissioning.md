# Conformance - commissioning

Statut: verified

## Requirements Coverage

### Requirement: Field Session Data Is Captured Explicitly
- Evidence: `crates/faero-commissioning/src/lib.rs` gere sessions terrain, captures et ajustements.
- Evidence: entites `CommissioningSession` conservent progression et traces.
- Verification: `cargo test -p faero-commissioning`

### Requirement: As-Built Vs As-Designed Remains Readable
- Evidence: `crates/faero-storage/src/lib.rs` persiste `commissioning/sessions/*` et `commissioning/reports/*`.
- Evidence: sorties `summary.json`, `captures.jsonl`, `adjustments.jsonl`, `measurements.jsonl`.
- Verification: `cargo test -p faero-storage save_project_writes_simulation_perception_commissioning_optimization_and_ai_artifact_files`

### Requirement: Commissioning Evidence Feeds Explanations
- Evidence: `crates/faero-ai/src/lib.rs` detecte contexte commissioning et adapte limitations/critique passes.
- Evidence: references de contexte traçables dans la sortie structuree.
- Verification: `cargo test -p faero-ai`
