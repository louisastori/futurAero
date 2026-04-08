# Conformance - ai

Statut: verified

## Requirements Coverage

### Requirement: Local Contextual Assistance
- Evidence: `crates/faero-ai/src/lib.rs` utilise contexte projet (`ProjectDocument`) et endpoint Ollama local configurable.
- Evidence: fallback local explicite quand runtime indisponible.
- Verification: `cargo test -p faero-ai chat_with_project_falls_back_when_tag_lookup_fails`

### Requirement: Structured Explain Output
- Evidence: `AiStructuredExplain` expose `summary`, `contextRefs`, `confidence`, `riskLevel`, `limitations`, `proposedCommands`, `explanation`.
- Evidence: generation dans `build_structured_explain`.
- Verification: `cargo test -p faero-ai structured_explain_proposes_commands_for_blocked_runs`

### Requirement: Explicit Runtime Profiles And Critique
- Evidence: profils `balanced`, `max`, `furnace` resolus explicitement et degrades journalises.
- Evidence: `critique_passes` produits par `critique_structured_explain`.
- Verification: `cargo test -p faero-ai resolve_profile_selection_degrades_when_local_resources_are_insufficient`
