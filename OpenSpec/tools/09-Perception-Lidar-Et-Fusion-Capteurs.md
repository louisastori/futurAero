# Outil 09 - Perception LiDAR Et Fusion Capteurs

## But

Ajouter une couche perception avancee pour connecter la cellule numerique a des observations capteurs exploitables et comparables au reel.

## Portee MVP etendu

- LiDAR 2D et 3D
- camera RGB et profondeur
- IMU
- calibration intra/extrinseque
- replay de donnees capteurs
- nuages de points et cartes d'occupation
- comparaison scene observee / scene nominale

## Entrees

- scene robotique
- modeles de capteurs
- profils de calibration
- donnees capteurs ou donnees synthetiques
- pipeline perception

## Sorties

- scans horodates
- nuages de points
- cartes d'occupation
- estimation de pose
- rapport d'ecarts

## Regles white-box

- chaque capteur doit exposer son montage, son repere et sa latence
- chaque pipeline doit exposer ses etapes
- chaque sortie doit etre reliee a ses sources et a sa calibration
- les ecarts detectes doivent etre mesurables et exportables

## Criteres d'acceptation

- un LiDAR peut etre monte sur une cellule et configure
- une calibration peut etre creee et verifiee
- un run perception produit un dataset rejouable
- une carte observee peut etre comparee a la cellule nominale
