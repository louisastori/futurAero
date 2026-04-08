# FutureAero Safety

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/19-Spec-Safety-Zones-Interlocks-Lidar-Securite.md`, `OpenSpec/specs/reference/10-Spec-Simulation-Detaillee.md`, `OpenSpec/specs/reference/12-Backlog-Dev-Ready.md`

Cette spec canonique capture les exigences stables safety du MVP. Le detail des scenarios, capteurs et extensions reste dans [../reference/README.md](../reference/README.md).

## Requirements

### Requirement: Inspectable Safety Evaluation

- **GIVEN** une action robotique candidate
- **WHEN** le moteur safety evalue les zones et interlocks actifs
- **THEN** le statut, les causes, les zones actives et les interlocks bloquants doivent etre inspectables dans le projet et dans l UI.

### Requirement: Blocking Rules Stay Explicit

- **GIVEN** un danger detecte par zone, interlock ou LiDAR securite
- **WHEN** une action doit etre inhibee
- **THEN** l inhibition doit etre explicite, horodatee et rattachee a ses causes au lieu d etre deduite silencieusement.

### Requirement: Safety Artifacts Reused By Simulation And AI

- **GIVEN** un rapport safety persiste
- **WHEN** la simulation ou l IA locale explique un blocage
- **THEN** elles doivent reutiliser le meme artefact safety comme source de verite locale.

## Supporting References

- [10-Spec-Simulation-Detaillee.md](../reference/10-Spec-Simulation-Detaillee.md)
- [12-Backlog-Dev-Ready.md](../reference/12-Backlog-Dev-Ready.md)
- [19-Spec-Safety-Zones-Interlocks-Lidar-Securite.md](../reference/19-Spec-Safety-Zones-Interlocks-Lidar-Securite.md)
