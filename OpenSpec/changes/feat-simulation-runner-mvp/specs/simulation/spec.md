## ADDED Requirements

### Requirement: Simulation Run Job Publishes Progress
FutureAero MUST expose `simulation.run.start` as a readable simulation job that publishes deterministic MVP progress phases.

#### Scenario: Starting a simulation creates a job envelope
- **WHEN** an operator launches `simulation.run.start` from the desktop shell
- **THEN** the project MUST persist a readable `SimulationRun` artifact exposing `jobId`, `status`, `phase`, `progress` and ordered `progressSamples`.

## MODIFIED Requirements

### Requirement: Deterministic Fixed-Step Runs
FutureAero MUST keep MVP simulation runs deterministic and replayable for identical inputs.

#### Scenario: Persisted run metadata keeps replay inputs explicit
- **WHEN** an operator launches `simulation.run.start` with a seeded fixed-step scenario
- **THEN** the persisted `SimulationRun` MUST expose `seed`, `engineVersion` and `stepCount` together with the resulting run artifacts.

### Requirement: Persisted Run Artifacts
FutureAero MUST persist the MVP simulation runner outputs directly in the project graph.

#### Scenario: Completed simulation run stays fully readable
- **WHEN** a simulation run completes and the `.faero` project is saved
- **THEN** the `SimulationRun` artifact MUST expose `job`, `summary`, `metrics`, `timelineSamples`, `signalSamples`, `controllerStateSamples` and `contacts` without any parallel hidden cache.
