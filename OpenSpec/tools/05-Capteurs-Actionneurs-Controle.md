# Outil 05 - Capteurs Actionneurs Controle

## But

Ajouter la couche operationnelle entre mecanique, robotisation et comportement logique.

## Portee MVP

- Capteurs de presence, position et fin de course
- Actionneurs simples
- Etats logiques
- Sequences et transitions
- Delais, temporisations et conditions elementaires

## Entrees

- Scene robotique
- Bibliotheque de capteurs/actionneurs
- Regles de sequence

## Sorties

- Modele logique de commande
- Table d'etats
- Chronologie evenementielle

## Regles white-box

- Toute transition doit avoir une condition explicite.
- Les temporisations et filtres doivent etre visibles.
- Le systeme doit expliquer pourquoi une sequence est bloquee.

## Criteres d'acceptation

- Un cycle simple peut etre decrit sous forme de sequence.
- Les entrees capteurs influencent la simulation.
- Les erreurs de logique bloquante sont detectables.
