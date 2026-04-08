# Specs Delta: Kinematic Joints MVP

## ADDED Requirements

### Requirement: Explicit Joint Types In Assemblies
FutureAero MUST persist kinematic joints explicitly inside an assembly with support for the MVP joint types `fixed`, `revolute` and `prismatic`.

#### Scenario: Create a revolute or prismatic joint
- **WHEN** a user or backend command creates a supported joint between two occurrences of the same assembly
- **THEN** the project MUST persist a readable joint record with its type, source occurrence, target occurrence and axis.

#### Scenario: Reject unsupported or invalid joint references
- **WHEN** a joint command targets missing occurrences or an unsupported joint type
- **THEN** the mutation MUST fail explicitly and the project MUST remain unchanged.

### Requirement: Joint Limits And State Stay Readable
FutureAero MUST persist the current state and optional limits of each MVP joint in a white-box representation that desktop tools and local AI can inspect directly.

#### Scenario: Joint created with limits
- **WHEN** a revolute or prismatic joint is created with min and max limits
- **THEN** the project MUST persist those limits in the joint data without relying on hidden runtime state.

#### Scenario: Joint state updated
- **WHEN** a joint state changes through a supported command
- **THEN** the persisted assembly data MUST expose the new position and retain the previous structural joint definition.

### Requirement: Degrees Of Freedom Stay Explicit
FutureAero MUST expose the degrees of freedom contributed by each MVP joint in the persisted model.

#### Scenario: Fixed joint
- **WHEN** a fixed joint is persisted
- **THEN** its readable state MUST expose zero degrees of freedom.

#### Scenario: Revolute or prismatic joint
- **WHEN** a revolute or prismatic joint is persisted
- **THEN** its readable state MUST expose one controllable degree of freedom.

### Requirement: Joint Commands Stay Auditable
FutureAero MUST route MVP joint mutations through explicit command and event kinds rather than hidden payload rewrites.

#### Scenario: Joint command recorded
- **WHEN** a joint is created or its state is updated
- **THEN** the recent project activity MUST expose the corresponding joint command and event history.

#### Scenario: Desktop snapshot exposes joints
- **WHEN** the desktop shell inspects an assembly containing joints
- **THEN** the snapshot and property detail MUST expose the joint count and a readable joint state summary.
