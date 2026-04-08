# OpenSpec Conformance Matrix

Ce dossier centralise les preuves de conformite des specs stables FutureAero.

Chaque fiche domaine contient:
- le mapping exigences -> implementation,
- les evidences code,
- les commandes de verification recommandees.

## Coverage Matrix

| Domain | Requirements Focus | Primary Evidence | Verification Commands |
| --- | --- | --- | --- |
| `system` | noyau canonique partage, runtime local-first, explicabilite transversale | `crates/faero-types`, `crates/faero-core`, `crates/faero-ai`, `crates/faero-storage` | `cargo test -p faero-types -p faero-core -p faero-ai -p faero-storage` |
| `simulation` | runs deterministes, artefacts persistes lisibles, timeline exploitable | `crates/faero-sim`, `crates/faero-storage` | `cargo test -p faero-sim && cargo test -p faero-storage` |
| `ai` | assistance locale contextuelle, sortie structuree, profils/critique explicites | `crates/faero-ai` | `cargo test -p faero-ai` |
| `data-model` | modele partage unique, format lisible/diffable, contrats explicites | `crates/faero-types`, `crates/faero-storage` | `cargo test -p faero-types -p faero-storage` |
| `safety` | zones/interlocks explicites, statut explicable, impact simulation/operations | `crates/faero-safety`, `crates/faero-sim`, `crates/faero-ai` | `cargo test -p faero-safety -p faero-sim -p faero-ai` |
| `perception` | runs capteurs inspectables, occupancy/comparison lisibles, contexte IA | `crates/faero-perception`, `crates/faero-storage`, `crates/faero-ai` | `cargo test -p faero-perception -p faero-storage -p faero-ai` |
| `integration` | endpoints types, bindings auditables, replay/lien degrade | `crates/faero-integration` | `cargo test -p faero-integration` |
| `commissioning` | sessions terrain explicites, as-built lisible, traces exploitables | `crates/faero-commissioning`, `crates/faero-storage`, `crates/faero-ai` | `cargo test -p faero-commissioning -p faero-storage -p faero-ai` |
| `optimization` | etudes explicites, classement lisible, decisions tracables | `crates/faero-optimization`, `crates/faero-storage` | `cargo test -p faero-optimization -p faero-storage` |
| `plugins` | metadata declarative, persistence lisible, capacites auditables | `crates/faero-plugin-host`, `crates/faero-types`, `crates/faero-storage` | `cargo test -p faero-plugin-host -p faero-types -p faero-storage` |

## Domain Reports

- [system](./system.md)
- [simulation](./simulation.md)
- [ai](./ai.md)
- [data-model](./data-model.md)
- [safety](./safety.md)
- [perception](./perception.md)
- [integration](./integration.md)
- [commissioning](./commissioning.md)
- [optimization](./optimization.md)
- [plugins](./plugins.md)

## Suggested Full Validation

Pour une verification complete locale:

`cargo test --workspace`
