# Outil 08 - Validation et Industrialisation

## But

Transformer les resultats de conception et simulation en elements exploitables pour une implementation reelle.

## Portee MVP

- Rapports de validation
- Comparaison objectif/resultat
- Checklists de pre-implementation
- Exports cibles a definir
- Historique des hypotheses retenues

## Entrees

- Pieces, assemblages et cellule
- Resultats de simulation
- Resultats perception et mesures terrain
- Objectifs de performance

## Sorties

- Rapport de validation
- Liste d'ecarts
- Paquet d'export minimal
- Rapport de comparaison nominal / observe

## Regles white-box

- Chaque conclusion doit citer ses donnees d'origine.
- Un ecart doit etre rattache a une mesure ou une hypothese.
- Les rapports doivent etre regenerables.
- Les mesures capteurs utilisees doivent citer calibration, repere et horodatage.

## Criteres d'acceptation

- Un rapport peut lister les points de risque avant implementation.
- Deux iterations peuvent etre comparees selon des criteres definis.
- Les hypotheses de validation sont conservees avec le projet.
