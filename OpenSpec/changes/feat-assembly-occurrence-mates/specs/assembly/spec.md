# Specs Delta: Assembly Occurrences And Mates

## ADDED Requirements

### Requirement: Explicit Assembly Occurrences
FutureAero MUST persist an assembly as an explicit composition of occurrences that reference part or sub-assembly definitions, each with its own local transform and stable identifier.

#### Scenario: Add occurrence to an assembly
- **WHEN** a user or desktop command adds an occurrence to an `Assembly`
- **THEN** the project MUST persist a readable occurrence record containing the target definition, the parent assembly and the local transform.

#### Scenario: Transform an existing occurrence
- **WHEN** a user or desktop command updates an existing occurrence placement
- **THEN** the mutation MUST remain auditable through an explicit assembly command and the persisted assembly data MUST reflect the new transform.

### Requirement: Explicit Mate Constraints
FutureAero MUST persist mate constraints as explicit assembly data rather than deriving them from opaque UI state.

#### Scenario: Add mate between occurrences
- **WHEN** a mate is created between two occurrences of the same assembly
- **THEN** the project MUST persist a readable mate record with a stable id, the occurrence references, the mate type and any MVP offset parameters.

#### Scenario: Remove mate from an assembly
- **WHEN** a mate is removed from an assembly
- **THEN** the persisted assembly data MUST no longer reference that mate and the mutation MUST be visible in the assembly command history.

### Requirement: Persisted Solve Report After Assembly Mutations
FutureAero MUST recompute and persist an assembly solve report after each supported occurrence or mate mutation so the UI and local AI can inspect the latest assembly state.

#### Scenario: Solve after nominal mutation
- **WHEN** an occurrence or mate mutation leaves the assembly in a valid connected state
- **THEN** the system MUST persist a solve report containing the status, remaining degrees of freedom estimate, solved occurrences and warnings, and publish an `assembly.solved` event.

#### Scenario: Report under-constrained or conflicting state
- **WHEN** an occurrence or mate mutation leaves the assembly under-connected or conflicting
- **THEN** the system MUST persist the warnings and publish an explicit unsolved assembly outcome that remains readable from the project snapshot.

### Requirement: Desktop Assembly Commands Stay White-Box
FutureAero MUST expose the assembly MVP in the desktop shell through explicit commands and readable project snapshots rather than hidden imperative mutations.

#### Scenario: Desktop creates an assembly
- **WHEN** the desktop shell creates a new assembly from available parts
- **THEN** the resulting activity log and snapshot MUST show an assembly command flow, explicit occurrence/mate data and the current solve summary.

#### Scenario: Desktop inspects assembly details
- **WHEN** the properties panel or project explorer inspects an assembly
- **THEN** it MUST be able to read the occurrence count, mate count, solve status and warning count directly from the persisted assembly data.
