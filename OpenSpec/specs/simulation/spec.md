# FutureAero Simulation

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/04-Moteur-Simulation-Reel.md`, `OpenSpec/specs/reference/10-Spec-Simulation-Detaillee.md`, `OpenSpec/specs/reference/12-Backlog-Dev-Ready.md`

Cette spec canonique capture les exigences stables du moteur de simulation MVP. Le detail du rationnel, des hypotheses et des scenarios reste dans [../reference/README.md](../reference/README.md).

## Requirements

### Requirement: Deterministic Fixed-Step Runs

- **GIVEN** un scenario, une seed, une version moteur et un pas fixe
- **WHEN** l utilisateur lance un run de simulation
- **THEN** le moteur doit produire un resultat deterministe et rejouable pour les memes entrees.

### Requirement: Persisted Run Artifacts

- **GIVEN** un run de simulation termine
- **WHEN** le projet `.faero` est persiste
- **THEN** les artefacts `summary`, `metrics`, `timeline`, `signalSamples`, `controllerStateSamples` et `contacts` doivent etre lisibles depuis le projet sans format cache parallelle.

### Requirement: Timeline Readable By UI And AI

- **GIVEN** un run de simulation persiste
- **WHEN** l UI desktop ou l IA locale inspecte le run
- **THEN** la timeline, les collisions, les changements de signaux et les etats de controle doivent etre exploitables directement pour expliquer un instant critique.

## Supporting References

- [04-Moteur-Simulation-Reel.md](../reference/04-Moteur-Simulation-Reel.md)
- [10-Spec-Simulation-Detaillee.md](../reference/10-Spec-Simulation-Detaillee.md)
- [12-Backlog-Dev-Ready.md](../reference/12-Backlog-Dev-Ready.md)
- [14-Schemas-Commandes-Evenements.md](../reference/14-Schemas-Commandes-Evenements.md)
