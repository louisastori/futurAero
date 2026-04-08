# Conformance - optimization

Statut: verified

## Requirements Coverage

### Requirement: Optimization Study Is Explicit
- Evidence: `crates/faero-optimization/src/lib.rs` modele objectifs, contraintes, variables et candidats.
- Evidence: resultat classe expose score et meilleur candidat.
- Verification: `cargo test -p faero-optimization`

### Requirement: Ranked Candidates And Inputs Stay Readable
- Evidence: `crates/faero-storage/src/lib.rs` persiste `optimization/studies/<id>/definition.json` et `ranked-candidates.json`.
- Evidence: test artefact global valide la presence des fichiers optimisation.
- Verification: `cargo test -p faero-storage save_project_writes_simulation_perception_commissioning_optimization_and_ai_artifact_files`

### Requirement: Decisions Stay Traceable
- Evidence: commandes/evenements persistes en JSONL permettent de tracer application/rejet des choix.
- Evidence: IA locale peut referencer les artefacts optimisation depuis le modele partage.
- Verification: `cargo test -p faero-storage helper_readers_and_writers_cover_all_supported_artifact_types`
