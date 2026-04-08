# Conformance - data-model

Statut: verified

## Requirements Coverage

### Requirement: Single Shared Project Model
- Evidence: `crates/faero-types/src/lib.rs` definit `ProjectDocument` et familles de types communes.
- Evidence: modules core, ai, sim, integration et storage echangent ce modele sans conversion opaque.
- Verification: `cargo test -p faero-types -p faero-storage`

### Requirement: Readable Project Format
- Evidence: `crates/faero-storage/src/lib.rs` persiste en YAML, JSON et JSONL lisibles.
- Evidence: test `saves_and_loads_project_document_round_trip`.
- Verification: `cargo test -p faero-storage saves_and_loads_project_document_round_trip`

### Requirement: Explicit Contracts And Artifact Families
- Evidence: commandes/evenements JSONL (`events/commands.jsonl`, `events/events.jsonl`) avec references explicites.
- Evidence: familles d artefacts dediees: `simulations`, `perception`, `commissioning`, `optimization`, `ai`, `plugins`.
- Verification: `cargo test -p faero-storage helper_readers_and_writers_cover_all_supported_artifact_types`
