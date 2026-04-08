# FutureAero Data Model

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/06-Modele-De-Donnees.md`, `OpenSpec/specs/reference/07-Format-De-Projet.md`, `OpenSpec/specs/reference/09-Contrats-Internes.md`

Cette spec canonique capture les exigences stables du modele de donnees partage. Le detail exhaustif des entites, artefacts et contrats reste dans [../reference/README.md](../reference/README.md).

## Requirements

### Requirement: Single Shared Project Model

- **GIVEN** un projet FutureAero
- **WHEN** le backend, l UI, la simulation, le stockage et l IA le manipulent
- **THEN** ils doivent partager le meme modele de donnees sans conversion ad hoc opaque.

### Requirement: Readable Project Format

- **GIVEN** un projet `.faero` et ses documents OpenSpec
- **WHEN** l utilisateur ou les outils ouvrent le depot projet
- **THEN** les informations de scene, d artefacts et de design intent doivent rester lisibles en clair, diffables et versionnables.

### Requirement: Explicit Contracts And References

- **GIVEN** des commandes, evenements, streams ou suggestions
- **WHEN** ils referencent des schemas, des artefacts ou des entites
- **THEN** ces references doivent etre explicites dans les donnees et jamais implicites dans un conteneur cache.

### Requirement: Domain Artifact Families Stay Readable

- **GIVEN** des runs de simulation, perception, commissioning, optimisation ou IA
- **WHEN** le projet `.faero` est persiste
- **THEN** chaque famille d artefacts doit rester rangee dans un dossier lisible et chargeable sans migration opaque.

### Requirement: Plugin Metadata Stays Declarative

- **GIVEN** un plugin installe dans le projet
- **WHEN** son manifest est charge, audite ou affiche
- **THEN** `releaseChannel`, `permissions`, `contributions`, `compatibility` et `signature` doivent rester des donnees explicites dans le modele partage.

## Supporting References

- [06-Modele-De-Donnees.md](../reference/06-Modele-De-Donnees.md)
- [07-Format-De-Projet.md](../reference/07-Format-De-Projet.md)
- [09-Contrats-Internes.md](../reference/09-Contrats-Internes.md)
- [14-Schemas-Commandes-Evenements.md](../reference/14-Schemas-Commandes-Evenements.md)
