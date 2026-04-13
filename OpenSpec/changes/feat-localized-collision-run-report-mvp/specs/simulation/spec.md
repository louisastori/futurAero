## ADDED Requirements

### Requirement: Localized Collision Contacts Stay Readable
FutureAero MUST persist simulation contacts with enough localized context to identify where the critical interaction happened.

#### Scenario: Collided run exposes localized contact context
- **WHEN** a simulation run persists one or more contact or collision events
- **THEN** each persisted contact MUST expose the involved pair ids, entity ids, a readable location label, and the associated simulation phase or controller state when available.

### Requirement: Simulation Run Persists A Readable Report
FutureAero MUST persist a readable run report directly inside `SimulationRun`.

#### Scenario: Run report highlights the critical instant
- **WHEN** a simulation run completes
- **THEN** the persisted `SimulationRun` MUST expose a `report` block containing a readable headline, ordered findings, critical event ids, and recommended actions derived from the run artifacts.

#### Scenario: Nominal run still exposes a report
- **WHEN** a simulation run completes without collisions
- **THEN** the persisted `report` MUST remain present and summarize the nominal outcome without requiring the UI or IA layer to infer it from raw metrics alone.
