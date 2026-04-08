# FutureAero Core System

Statut: stable

Source promue depuis: `OpenSpec/changes/init-mvp-futureaero/specs/system/spec.md`

Cette spec canonique capture les exigences systeme minimales stabilisees. Les details longs, le rationnel et la traceabilite complementaire restent dans [../reference/README.md](../reference/README.md).

## Requirements

### Requirement: Desktop Architecture (Local First)

- **GIVEN** un utilisateur d'ingenierie travaillant hors ligne
- **WHEN** il lance FutureAero
- **THEN** l'application doit fonctionner integralement en local, sans dependance cloud obligatoire, afin de proteger la propriete intellectuelle.

### Requirement: White-Box Transparency

- **GIVEN** une contrainte mecanique generee, un calcul ou une suggestion IA
- **WHEN** l'utilisateur inspecte le projet, les proprietes ou les artefacts de simulation
- **THEN** les relations mathematiques, chaines cinematiques, sources internes et causes doivent etre visibles, editables et explicables.

### Requirement: Local AI Assistance

- **GIVEN** une tache de modelisation, de simulation ou de revue OpenSpec
- **WHEN** l'utilisateur interroge le panneau IA
- **THEN** le LLM local via Ollama doit produire une reponse contextuelle basee sur l'etat courant du projet `.faero`, sans modification silencieuse du projet.

## Supporting References

- [01-Vision-Produit.md](../reference/01-Vision-Produit.md)
- [02-Exigences-Systeme.md](../reference/02-Exigences-Systeme.md)
- [03-Architecture-Desktop-IA-Locale.md](../reference/03-Architecture-Desktop-IA-Locale.md)
- [11-Spec-IA-Locale.md](../reference/11-Spec-IA-Locale.md)
- [14-Schemas-Commandes-Evenements.md](../reference/14-Schemas-Commandes-Evenements.md)
