# Conformance - plugins

Statut: verified

## Requirements Coverage

### Requirement: Plugin Metadata Stays Declarative
- Evidence: `crates/faero-types/src/lib.rs` et `PluginManifest` exposent `releaseChannel`, `permissions`, `contributions`, `compatibility`, `signature`.
- Evidence: `ProjectDocument` maintient `plugin_manifests` et `plugin_states` explicites.
- Verification: `cargo test -p faero-types`

### Requirement: Plugin State Is Persisted Readably
- Evidence: `crates/faero-storage/src/lib.rs` ecrit manifests sous `plugins/manifests/*.json` et etat sous `plugins/state/plugins.json`.
- Evidence: helpers de lecture/ecriture couvrent les formats plugin.
- Verification: `cargo test -p faero-storage helper_readers_and_writers_cover_all_supported_artifact_types`

### Requirement: Plugin Capabilities Remain Auditable
- Evidence: `crates/faero-plugin-host/src/lib.rs` s appuie sur manifest declaratif pour chargement/controle.
- Evidence: tests storage garantissent round-trip lisible des manifests.
- Verification: `cargo test -p faero-plugin-host && cargo test -p faero-storage`
