# Specs Stables

Ce dossier contient les specs stables de `FutureAero`.

- `system/spec.md`: spec canonique systeme promue depuis les changements actifs.
- `simulation/spec.md`: exigences stables du moteur de simulation et des artefacts de run.
- `ai/spec.md`: exigences stables du runtime IA locale et des suggestions structurees.
- `data-model/spec.md`: exigences stables du modele de donnees partage et du format lisible.
- `safety/spec.md`: exigences stables des zones, interlocks et validations safety.
- `perception/spec.md`: exigences stables des rigs capteurs, calibrations, runs perception et comparaisons observe/nominal.
- `integration/spec.md`: exigences stables des endpoints externes, bindings industriels et replays de liaisons degradees.
- `commissioning/spec.md`: exigences stables des sessions terrain, captures et comparaisons as-built vs as-designed.
- `optimization/spec.md`: exigences stables des etudes multi-objectifs, candidats classes et application explicite.
- `plugins/spec.md`: exigences stables du Plugin SDK, des permissions auditables et des contributions declaratives.
- `reference/`: corpus long-form migre depuis l'ancienne racine `OpenSpec/`.

Les nouvelles exigences doivent d'abord etre proposees dans `OpenSpec/changes/<change-id>/`, puis promues ici une fois stabilisees.

## Workflow De Promotion

1. Definir ou mettre a jour une exigence dans `OpenSpec/changes/<change-id>/`.
2. Implementer le code et fermer les taches associees dans `tasks.md`.
3. Verifier par tests, lints et artefacts persistants que le comportement est stable.
4. Promouvoir les exigences stabilisees vers `OpenSpec/specs/<domaine>/spec.md`.
5. Laisser le rationnel exhaustif, les alternatives et l'historique dans `OpenSpec/specs/reference/`.
