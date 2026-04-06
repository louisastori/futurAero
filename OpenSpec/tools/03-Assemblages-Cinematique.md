# Outil 03 - Assemblages et Cinematique

## But

Composer plusieurs pieces ou sous-ensembles et decrire leurs liaisons mecaniques.

## Portee MVP

- Placement de composants
- Contraintes de coincidence, distance, angle, axe
- Sous-assemblages
- Liaisons pivot, glissiere et fixe
- Evaluation des degres de liberte

## Entrees

- Pieces 3D
- Repere global
- Contraintes d'assemblage

## Sorties

- Assemblage resolu
- Graphe de liaisons
- Etat des degres de liberte

## Regles white-box

- Chaque liaison doit afficher ses composants et axes de reference.
- Le systeme doit expliquer pourquoi un assemblage reste mobile ou se bloque.
- Les dependances de mouvement doivent etre inspectables.

## Criteres d'acceptation

- Un assemblage simple peut etre entierement contraint.
- Une chaine cinematique basique peut etre animee.
- Les collisions de base entre composants peuvent etre detectees.
