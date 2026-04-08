# Proposal: GitHub PR et Releases

## Intent
L'objectif est de definir un workflow GitHub clair pour FutureAero: branche `main` protegee, checks CI obligatoires, politique de merge lisible et publication d'artefacts desktop a la fin du workflow.

## Scope
- **Inclus :** gouvernance du remote canonique `origin`, checks obligatoires de PR, gate de couverture versionnee, artefact installateur Windows en fin de workflow, regles de revue et de merge.
- **Exclus :** automatisation complete des tags semver et publication de notes de release riches; cela pourra faire l'objet d'un change dedie si le processus evolue encore.
