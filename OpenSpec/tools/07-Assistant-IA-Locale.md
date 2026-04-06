# Outil 07 - Assistant IA Locale

## But

Assister l'utilisateur sans sortir les donnees du poste et sans masquer la logique des propositions.

## Portee MVP

- Question/reponse sur le projet courant
- Resume d'assemblage, de scenario ou de conflit
- Proposition de correction de parametres
- Generation de notes techniques et checklists
- Explication des impacts d'une modification

## Entrees

- Graphe projet
- Journaux de simulation
- Selection utilisateur
- Historique de session

## Sorties

- Reponse explicative
- Liste d'actions proposees
- Brouillon de documentation

## Regles white-box

- Les objets utilises pour raisonner doivent etre cites.
- Une proposition de changement doit etre diffable avant application.
- L'outil doit indiquer ses limites ou zones d'incertitude.

## Criteres d'acceptation

- L'utilisateur peut demander pourquoi une collision apparait.
- L'assistant peut proposer une correction de sequence ou de position.
- Une reponse peut etre convertie en note projet rattachee a la scene.
