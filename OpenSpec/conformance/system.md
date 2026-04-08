# Conformance - system

Statut: verified

## Requirements Coverage

### Requirement: Canonical Shared Core
- Evidence: `crates/faero-types/src/lib.rs` centralise les types partages et structures d'echange.
- Evidence: `crates/faero-core/src/lib.rs` orchestre le graphe projet avec invariants de base.
- Verification: `cargo test -p faero-types && cargo test -p faero-core`

### Requirement: Local-First Runtime
- Evidence: `crates/faero-ai/src/lib.rs` utilise Ollama local (`127.0.0.1`) avec fallback local explicite.
- Evidence: `apps/desktop/src-tauri/src/main.rs` expose les commandes runtime sans dependance cloud obligatoire.
- Verification: `cargo test -p faero-ai`

### Requirement: Explainability Across Domains
- Evidence: `faero-ai` produit `AiStructuredExplain` avec `contextRefs`, `limitations`, `confidence`, `riskLevel`.
- Evidence: les artefacts lisibles de simulation/perception/commissioning sont persistes par `faero-storage`.
- Verification: `cargo test -p faero-ai -p faero-storage`
