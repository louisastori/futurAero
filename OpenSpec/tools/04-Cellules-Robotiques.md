# Outil 04 - Cellules Robotiques

## But

Modeliser l'environnement de robotisation a partir des composants mecaniques et des equipements terrain.

## Portee MVP

- Placement d'un ou plusieurs robots abstraits
- Definition des outillages, convoyeurs, postes et zones
- Definition des enveloppes de travail
- Definition de points cibles et sequences de base

## Entrees

- Assemblages
- Modeles de robots
- Elements de cellule
- Repere usine ou repere scene

## Sorties

- Scene robotique
- Zones de travail et d'exclusion
- Sequence operationnelle de base

## Regles white-box

- Les limites de course et enveloppes doivent etre visibles.
- Chaque sequence doit citer les objets touches.
- Les hypotheses de simplification d'un robot doivent etre explicites.

## Criteres d'acceptation

- Une cellule simple pick-and-place peut etre montee.
- Les intersections de zones sont detectees.
- Les points de passage et cibles sont versionnes.
