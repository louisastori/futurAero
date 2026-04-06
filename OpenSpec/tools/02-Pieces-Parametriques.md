# Outil 02 - Pieces Parametriques

## But

Transformer les esquisses en geometrie 3D exploitable par les autres outils.

## Portee MVP

- Extrusion
- Revolution
- Enlevement de matiere
- Conges et chanfreins simples
- Proprietes de masse calculees

## Entrees

- Esquisses resolues
- Parametres numeriques
- Materiau de base

## Sorties

- Piece 3D
- Historique d'operations
- Proprietes de masse et enveloppe

## Regles white-box

- L'ordre des operations doit etre lisible.
- Chaque operation doit exposer ses dependances.
- Les erreurs de regeneration doivent pointer l'operation fautive.

## Criteres d'acceptation

- Une modification de cote regenera la piece de facon predictable.
- Les proprietes de masse sont recalculables a tout moment.
- La piece peut etre instanciee dans un assemblage.
