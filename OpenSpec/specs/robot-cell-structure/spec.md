# FutureAero Robot Cell Structure

Statut: stable

Source promue depuis: `OpenSpec/archive/completed/2026-04/feat-robot-cell-structure-mvp/specs/robot-cell-structure/spec.md`

Cette spec canonique capture les exigences stables de structure lisible des cellules robotiques MVP. Les details de mise en oeuvre et la trace de changement restent archives avec le change source.

## Requirements

### Requirement: Robot Cells Stay Structurally Explicit

FutureAero MUST persist each MVP `RobotCell` as a readable aggregate with explicit references to its scene assembly, robots, equipments, safety zones, sequences and controller models.

#### Scenario: Create a structured robot cell

- **WHEN** the desktop shell creates a new robot cell
- **THEN** the persisted `RobotCell` MUST expose `sceneAssemblyId`, `robotIds`, `equipmentIds`, `safetyZoneIds`, `sequenceIds` and `controllerModelIds`.

#### Scenario: Structured robot cell remains white-box

- **WHEN** a project snapshot inspects a robot cell entity
- **THEN** the snapshot MUST expose those ids directly in the `RobotCell` payload without relying on hidden runtime state.

### Requirement: Equipment Models Stay Readable

FutureAero MUST persist MVP robot-cell equipments explicitly with their type, scene occurrence reference, parameter set and I/O ports.

#### Scenario: Create MVP equipment models

- **WHEN** a robot cell is created with a conveyor, a workstation and a gripper
- **THEN** the project MUST persist readable `EquipmentModel` entities for those equipments with `equipmentType`, `assemblyOccurrenceId`, `parameterSet` and `ioPortIds`.

#### Scenario: Equipment references stay linked to the scene assembly

- **WHEN** an equipment model is persisted
- **THEN** its `assemblyOccurrenceId` MUST reference an occurrence belonging to the robot-cell scene assembly.

### Requirement: Support Entities Stay Auditable

FutureAero MUST create the support entities required by the MVP robot-cell structure through explicit entity creation so desktop tools and local AI can inspect them directly.

#### Scenario: Robot cell creates support entities

- **WHEN** the desktop shell creates a robot cell
- **THEN** the project MUST also persist readable support entities for the robot model, sequence, controller and safety zones.

#### Scenario: Recent activity exposes robot-cell creation flow

- **WHEN** a robot cell and its support graph are created
- **THEN** the recent project activity MUST expose the corresponding entity creation history.

### Requirement: Desktop Summary Exposes Structured Counts

FutureAero MUST expose a readable robot-cell summary in the desktop shell that reflects the structured support graph.

#### Scenario: Snapshot exposes equipment and sequence counts

- **WHEN** the desktop shell inspects a robot cell
- **THEN** the summary and detail text MUST expose the robot target count together with the persisted equipment and safety-zone counts.
