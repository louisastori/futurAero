# Robot Cell Target Sequence MVP

## Requirements

### Requirement: Robot Cell Targets Stay Versioned
FutureAero MUST persist MVP robot-cell target points as explicit support entities instead of opaque embedded-only arrays.

#### Scenario: Robot cell creation materializes target entities
- **WHEN** the desktop shell creates a new robot cell
- **THEN** the project MUST persist readable `RobotTarget` entities linked to the owning `RobotCell` and `RobotSequence`.

#### Scenario: Robot cell keeps explicit target references
- **WHEN** a project snapshot inspects a robot cell or its sequence
- **THEN** the payload MUST expose ordered `targetIds` that reference persisted target entities.

### Requirement: Simple Sequence Order Stays Editable
FutureAero MUST allow MVP sequence order changes through explicit target data that can be edited and validated.

#### Scenario: Reordering a target updates the sequence
- **WHEN** an operator edits the order of a persisted robot target
- **THEN** the owning `RobotSequence` MUST recompute its ordered `targetIds` and derived validation metrics.

#### Scenario: Invalid target order is rejected
- **WHEN** a robot target update introduces duplicate, missing or non-positive order indexes
- **THEN** the backend MUST reject the edit with a readable validation error.

### Requirement: Desktop Shell Exposes Minimal Target Preview
FutureAero MUST expose a minimal readable preview of ordered robot-cell targets in snapshots and fallback views.

#### Scenario: Robot cell summary exposes ordered target preview
- **WHEN** the desktop shell displays a robot cell
- **THEN** its properties MUST expose the target count together with a readable ordered preview of the target sequence.

#### Scenario: Web fallback mirrors the structured target sequence
- **WHEN** the shell runs in fallback mode
- **THEN** the generated snapshot MUST include explicit robot-target support entities and the same ordered preview semantics.
