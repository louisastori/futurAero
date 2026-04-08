# Conformance - safety

Statut: verified

## Requirements Coverage

### Requirement: Safety Zones And Interlocks Are Explicit
- Evidence: `crates/faero-safety/src/lib.rs` modele zones, etats interlock et causes explicites.
- Evidence: sorties safety persistables dans le graphe projet via `EntityRecord`.
- Verification: `cargo test -p faero-safety`

### Requirement: Safety Status Is Explainable
- Evidence: `crates/faero-ai/src/lib.rs` consomme `SafetyReport` et produit un resume structure avec causes et niveau de risque.
- Evidence: commandes proposees explicites (`analyze.safety`, mutation previewable).
- Verification: `cargo test -p faero-ai structured_explain_proposes_commands_for_blocked_runs`

### Requirement: Safety Impacts Simulation And Operations
- Evidence: `crates/faero-sim/src/lib.rs` expose collisions/contacts et sequence bloquee.
- Evidence: status warning/collided utilisable par UI et IA pour decision.
- Verification: `cargo test -p faero-sim missing_safety_can_produce_a_collision_state`
