# FutureAero Local AI

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/03-Architecture-Desktop-IA-Locale.md`, `OpenSpec/specs/reference/11-Spec-IA-Locale.md`, `OpenSpec/specs/reference/17-Spec-IA-Ultra-Locale.md`

Cette spec canonique capture les exigences stables du runtime IA locale. Le detail des profils, du rationnel et des variantes avancees reste dans [../reference/README.md](../reference/README.md).

## Requirements

### Requirement: Local Contextual Assistance

- **GIVEN** un projet `.faero` charge et un utilisateur qui interroge l assistant
- **WHEN** le runtime IA local traite la demande
- **THEN** la reponse doit etre basee sur le graphe projet, les artefacts persistants et les references locales, sans dependance cloud obligatoire.

### Requirement: Structured Explain Output

- **GIVEN** une demande d explication sur un run, un blocage safety ou un contexte projet
- **WHEN** l assistant repond en mode structure
- **THEN** la sortie doit exposer `summary`, `contextRefs`, `confidence`, `riskLevel`, `limitations`, `proposedCommands` et `explanation`.

### Requirement: Explicit Suggestion Application

- **GIVEN** une suggestion IA contenant des `proposedCommands`
- **WHEN** l utilisateur la previsualise, l applique ou la rejette
- **THEN** aucune mutation ne doit etre silencieuse et toute decision doit laisser une trace explicite dans le projet.

## Supporting References

- [03-Architecture-Desktop-IA-Locale.md](../reference/03-Architecture-Desktop-IA-Locale.md)
- [11-Spec-IA-Locale.md](../reference/11-Spec-IA-Locale.md)
- [17-Spec-IA-Ultra-Locale.md](../reference/17-Spec-IA-Ultra-Locale.md)
- [14-Schemas-Commandes-Evenements.md](../reference/14-Schemas-Commandes-Evenements.md)
