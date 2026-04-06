# Outil 11 - Safety Zones Interlocks LiDAR Securite

## But

Modeliser la surete de cellule et expliquer les inhibitions de mouvement.

## Portee MVP etendu

- zones de surete
- interlocks et permissifs
- LiDAR securite
- ralentissement et stop
- validation safety

## Entrees

- geometrie cellule
- capteurs safety
- regles interlocks
- etats externes

## Sorties

- etat safety
- causes de blocage
- trace de transitions
- rapport de validation

## Regles white-box

- aucune inhibition sans cause visible
- chaque zone doit etre visualisable
- chaque trigger LiDAR doit etre horodate

## Criteres d'acceptation

- une zone safety peut etre creee
- un interlock peut bloquer un mouvement
- un LiDAR securite peut declencher slowdown ou stop
