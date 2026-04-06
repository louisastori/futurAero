# Outil 13 - Optimization Engine

## But

Chercher automatiquement de meilleures configurations sous contraintes.

## Portee MVP etendu

- etudes multi-objectifs
- classement de candidats
- contraintes safety et integration
- front de Pareto
- application guidee des resultats

## Entrees

- scene et sequences
- objectifs
- contraintes
- variables de decision

## Sorties

- candidats
- scores
- recommandations
- resultats applicables

## Regles white-box

- chaque score doit citer ses metriques
- chaque contrainte violee doit etre explicite
- aucun resultat n'est applique automatiquement

## Criteres d'acceptation

- une etude d'optimisation peut etre definie
- plusieurs candidats peuvent etre classes
- un resultat peut etre applique via commandes normales
