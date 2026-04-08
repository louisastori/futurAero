# Design: GitHub Delivery Governance

## Architecture Overview
La gouvernance GitHub repose sur trois couches explicites et versionnees:

1. **Remote et branche canonique :** `origin/main` reste la reference pour l'integration.
2. **Checks CI obligatoires :** le workflow GitHub Actions couvre Rust, frontend, shell desktop et publication de l'installateur.
3. **Politique de merge et de release :** les PR sont mergees seulement apres revue et checks verts; les artefacts produits en CI servent de base aux releases.

## Implementation Notes
- La gate de couverture Rust doit rester versionnee dans `config/coverage-gate.json` pour eviter les divergences entre poste local et CI.
- L'artefact installateur Windows doit apparaitre a la fin du workflow GitHub et rester telechargeable sans etape manuelle supplementaire.
- Les references documentaires doivent pointer vers un change archive, pas vers un ancien gros fichier mono-spec.
