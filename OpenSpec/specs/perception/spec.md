# FutureAero Perception

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md`, `OpenSpec/specs/reference/20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md`, `OpenSpec/specs/reference/12-Backlog-Dev-Ready.md`

Cette spec canonique capture les exigences stables des rigs capteurs, des calibrations et des artefacts perception exploitables localement.

## Requirements

### Requirement: Sensor Rigs Stay Explicit

- **GIVEN** une cellule ou une scene equipee de capteurs
- **WHEN** un rig perception est defini dans le projet
- **THEN** chaque capteur, son type, son montage, son repere et son statut de calibration doivent etre inspectables dans le modele partage.

### Requirement: Calibration Metrics Stay Traceable

- **GIVEN** une calibration de rig ou de capteur
- **WHEN** elle est executee ou rejouee
- **THEN** les metriques minimales comme l erreur, la couverture et le dataset source doivent rester persistantes et lisibles dans le projet.

### Requirement: Perception Runs Produce Reusable Artifacts

- **GIVEN** un run perception local
- **WHEN** il termine ou progresse
- **THEN** il doit produire des artefacts explicites tels que `occupancyMap`, `pointCloudFrames`, `progressSamples` et `observedSceneComparison`, reutilisables par le shell, l IA et le commissioning.

### Requirement: Observed Vs Nominal Comparison Stays Measurable

- **GIVEN** une scene observee et une scene nominale
- **WHEN** le moteur perception compare les deux
- **THEN** les ecarts doivent citer les capteurs sources, la tolerance appliquee, les zones non observees et des valeurs mesurables comme l ecart moyen ou maximum.

## Supporting References

- [16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md](../reference/16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md)
- [20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md](../reference/20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md)
- [12-Backlog-Dev-Ready.md](../reference/12-Backlog-Dev-Ready.md)
