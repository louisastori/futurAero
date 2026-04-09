## Context

The repo already persists one `RobotSequence` entity per robot cell and keeps a sample ordered target list in `RobotCell.data.targets`. That is enough to run validation and summarize the cell, but not enough to treat the targets as first-class editable project entities. The property inspector already supports editing `parameterSet.*` values generically, so the simplest path is to represent editable target fields there and keep derived sequence aggregates synchronized in the backend.

## Goals / Non-Goals

**Goals:**

- introduce explicit target entities that can be versioned and surfaced in project snapshots
- let the existing inspector edit target order and target kinematic inputs without adding a new editor surface
- recompute the simple robot sequence and robot-cell validation summaries whenever the ordered targets change
- expose an ordered, human-readable target preview in the shell and fallback data

**Non-Goals:**

- new command families for advanced sequence authoring
- multi-robot sequencing
- geometric previews in the viewport

## Decisions

### Persist MVP targets as `RobotTarget` entities

The target point needs to be versioned and inspectable like the rest of the robot-cell support graph. A dedicated support entity is more transparent than mutating hidden arrays inside `RobotCell`.

Alternative considered:

- keep targets embedded in `RobotCell` only
  Rejected because it still fails the versioned-target requirement.

### Make `parameterSet` the editable source of truth for target tuning

The existing inspector can already edit `parameterSet.*` fields. Storing editable `orderIndex`, `xMm`, `yMm`, `zMm`, `nominalSpeedMmS` and `dwellTimeMs` there avoids widening the inspector framework just for one entity type.

Alternative considered:

- extend the inspector to edit arbitrary nested non-parameter fields
  Rejected because that is broader than `ST-402` and unnecessary for this MVP.

### Recompute the sequence from ordered target entities on write

When a `RobotTarget` changes, the backend should gather the ordered targets for its cell and sequence, validate them, rewrite the owning `RobotSequence`, and refresh the owning `RobotCell` summary fields. This keeps the read model coherent without adding background jobs.

Alternative considered:

- tolerate stale embedded sequence aggregates until the next cell rebuild
  Rejected because it would make the snapshot misleading immediately after edits.

### Use a shell preview instead of a dedicated target timeline

For the MVP, a simple ordered target preview in the properties panel is sufficient to satisfy the “visualisation minimale des cibles” requirement.

Alternative considered:

- build a new timeline or graph widget
  Rejected because it expands scope into UI product work not required for the backlog item.

## Risks / Trade-offs

- [Write-path coupling] Updating a target now fans out to the sequence and the cell. → Mitigation: keep one backend synchronization helper and cover it with Rust tests.
- [Editable order collisions] Two targets could be assigned the same `orderIndex`. → Mitigation: validation rejects duplicate or non-positive order indexes.
- [Fallback drift] The web fallback can diverge from Tauri again. → Mitigation: centralize fallback robot-cell support data in one shared helper.

## Migration Plan

1. Add the target model and validation in `faero-robotics`.
2. Materialize target entities and real `targetIds` during robot-cell creation.
3. Recompute sequence and cell aggregates after target property edits.
4. Expose ordered target previews in desktop snapshots and fallback data.
5. Validate with Rust, desktop UI, and local CI preflight.

## Open Questions

- Should the canonical data model later promote target entities into `faero-types` once multi-sequence cells arrive?
- Do we want a future command dedicated to reorder targets instead of relying on `orderIndex` editing?
