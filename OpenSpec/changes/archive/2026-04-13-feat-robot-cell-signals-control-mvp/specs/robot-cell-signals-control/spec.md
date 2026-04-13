# Specs Delta: Robot Cell Signals Control MVP

## ADDED Requirements

### Requirement: Robot Cell Signals Stay Explicit And Typed
FutureAero MUST persist MVP robot-cell control signals as explicit support entities with stable ids, declared kinds, and typed values.

#### Scenario: Robot cell creation materializes typed signals
- **WHEN** the desktop shell creates a robot cell
- **THEN** the project MUST persist readable `Signal` entities linked to the cell with stable ids, declared `kind`, and initial typed values.

#### Scenario: Invalid signal edits are rejected
- **WHEN** an operator edits a persisted robot-cell signal with a value incompatible with its declared kind
- **THEN** the backend MUST reject the change with a readable validation error.

### Requirement: Controller State Machine Uses Explicit Transition Conditions
FutureAero MUST represent MVP robot-cell control flow as an explicit controller state machine that references persisted signals through named conditions and assignments.

#### Scenario: Robot cell snapshot exposes controller transitions
- **WHEN** a project snapshot inspects a robot cell or its controller entity
- **THEN** the payload MUST expose explicit states, transitions, conditions, and assignments instead of opaque derived labels.

#### Scenario: Invalid controller references are rejected
- **WHEN** a controller transition references a missing signal or a missing source or target state
- **THEN** the system MUST reject the controller graph as invalid.

### Requirement: Minimal Control Graph Detects Blocked Sequences
FutureAero MUST detect when the MVP robot-cell control graph cannot reach a terminal state from its persisted signal values and transitions.

#### Scenario: Blocked sequence is reported
- **WHEN** the controller evaluation ends in a non-terminal state because no valid transition can fire
- **THEN** the robot-cell control summary MUST expose `blockedSequenceDetected` together with the readable blocked state id.

#### Scenario: Shell and fallback expose the same control summary
- **WHEN** the shell runs with the Tauri backend or in web fallback mode
- **THEN** both snapshots MUST expose the same signal counts, controller transition counts, and blocked-state semantics for the robot cell.
