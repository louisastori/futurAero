## Context

The desktop shell already materializes `Signal` and `Controller` support entities when a robot cell is created, the property inspector can edit signal fields, and `faero-sim` can evaluate a small `ControllerStateMachine` with scheduled signal changes. That is enough to demo the concept, but the control graph is still effectively shell-owned: `faero-robotics` has no shared validation model, blocked-state semantics are derived only inside simulation code, and the web fallback duplicates a large amount of seeded controller data.

## Goals / Non-Goals

**Goals:**

- define a minimal robot-cell control contract in `faero-robotics` that validates signals, controller states, transition conditions, and summary counts
- keep persisted `Signal` and `Controller` entities as the editable white-box source of truth in the project graph
- make blocked-sequence detection deterministic and reusable from the same persisted control graph before `simulation.run.start` exists
- keep Tauri and fallback snapshots aligned on the same readable control summary

**Non-Goals:**

- executing a long-running simulation job or persisting run artifacts
- implementing external controller connectivity or live PLC synchronization
- building a graphical state-machine authoring surface

## Decisions

### Represent MVP control as a robotics-level aggregate over existing typed control structs

`faero-types` already carries wire/domain structs like `SignalDefinition`, `ControllerStateMachine`, `SignalCondition`, and samples. `faero-robotics` should add the robot-cell specific aggregate and validation layer instead of duplicating those types in the shell.

Alternative considered:

- keep assembling control data directly in `apps/desktop/src-tauri`
  Rejected because the desktop shell would remain the only place that knows whether a robot-cell control graph is valid.

### Keep `Signal` and `Controller` entities as the editable source of truth

The entity graph already exposes readable white-box support entities and the property update flow can mutate them. `ST-501` should preserve that model and recompute derived control summaries from those persisted entities rather than introducing hidden control blobs.

Alternative considered:

- move the control graph into embedded-only `RobotCell.data.controller`
  Rejected because it would reduce traceability and make entity-level edits less transparent.

### Reuse deterministic control evaluation for blocked-state summaries

Blocked-sequence detection should be computed from the same explicit state machine and signal set that `faero-sim` consumes, even if `ST-501` still stays short of a full async runner. The simplest path is to factor a reusable validation/evaluation bridge and surface its result in robot-cell summaries.

Alternative considered:

- keep blocked detection as a simulation-only concern until `ST-502`
  Rejected because the backlog requires blocked sequences to be detectable already in the minimal control increment.

### Centralize fallback seeding and summary formatting around the same control contract

The fallback already mirrors the Tauri bundle for robot cells. `ST-501` should keep one shared control-building helper per frontend/backend path and align the derived summary fields so tests assert identical semantics.

Alternative considered:

- let fallback continue carrying a hand-maintained copy of the controller graph
  Rejected because drift already becomes costly once targets, signals, and controller previews all evolve together.

## Risks / Trade-offs

- [Validation scope grows across modules] A stricter control model touches `faero-robotics`, `faero-sim`, and the shell. -> Mitigation: keep the shared contract minimal and cover it with Rust tests first.
- [Blocked detection may be duplicated accidentally] Backend and fallback can diverge if each keeps its own heuristics. -> Mitigation: define one summary shape and derive it from the same ordered signal/controller model.
- [Editable controller graphs are fragile] Broken signal references or unreachable states can appear quickly. -> Mitigation: reject invalid transitions and missing signal references at validation time.

## Migration Plan

1. Add robotics-level signal/controller models and validation around the existing typed control structs.
2. Rebuild robot-cell control summaries in the Tauri backend from persisted `Signal` and `Controller` entities.
3. Align fallback control seeding and summary exposure with the backend contract.
4. Extend `faero-sim` or the shell bridge to consume the same validated control graph for blocked-state detection.
5. Validate with Rust, desktop UI, and local CI preflight.

## Open Questions

- Should the blocked-state preview in `ST-501` use a lightweight control evaluation helper or call into `faero-sim` directly?
- Do we want controller transition editing in the property inspector during this increment, or only validated persistence and read-only exposure?
