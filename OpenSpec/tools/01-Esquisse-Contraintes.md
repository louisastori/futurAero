# Outil 01 - Esquisse et Contraintes

## But

Creer le socle 2D de la modelisation parametrique.

## Portee MVP

- Creation de lignes, arcs, cercles, rectangles et points.
- Application de contraintes geometriques de base.
- Application de cotes et dimensions pilotantes.
- Detection des esquisses sous-contraintes et sur-contraintes.

## Entrees

- Plan de travail
- Unites
- Elements 2D
- Contraintes utilisateur

## Sorties

- Esquisse resolue
- Jeu de dimensions nommees
- Diagnostic de contraintes

## Regles white-box

- Chaque contrainte doit etre visible, nommee et supprimable.
- Le solveur doit signaler l'origine d'un conflit.
- Une cote pilotante doit indiquer quels elements elle impacte.

## Criteres d'acceptation

- Une esquisse peut etre entierement definie et verrouillee.
- Un conflit de contraintes remonte au moins les elements impliques.
- Les dimensions peuvent etre reutilisees par l'outil de piece 3D.
