# FutureAero Commissioning

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md`, `OpenSpec/specs/reference/16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md`, `OpenSpec/specs/reference/12-Backlog-Dev-Ready.md`

Cette spec canonique capture les exigences stables des sessions terrain, des captures et des comparaisons as-built vs as-designed.

## Requirements

### Requirement: Commissioning Sessions Stay Reusable

- **GIVEN** une mise en service terrain
- **WHEN** une session de commissioning est ouverte
- **THEN** le nominal cible, les captures, le journal d ajustements et le statut courant doivent rester persistants et rejouables localement.

### Requirement: As-Built Comparison Produces An Explicit Report

- **GIVEN** des mesures terrain comparees au nominal
- **WHEN** le systeme construit un rapport as-built vs as-designed
- **THEN** les deviations, tolerances, mesures conformes et causes principales doivent etre quantifiees dans un artefact lisible.

### Requirement: Commissioning Uses Shared Local Artifacts

- **GIVEN** des captures perception, des endpoints terrain et des comparaisons as-built
- **WHEN** le shell ou l IA inspecte une session
- **THEN** ils doivent reutiliser les memes artefacts persistants plutot que reconstruire un etat cache.

## Supporting References

- [20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md](../reference/20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md)
- [16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md](../reference/16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md)
- [12-Backlog-Dev-Ready.md](../reference/12-Backlog-Dev-Ready.md)
