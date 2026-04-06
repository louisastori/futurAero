# Outil 06 - Simulation et Jumeau Numerique

## But

Executer des scenarios proches du reel sur un modele numerique unifie.

## Portee MVP

- Lancement de scenarios temporels
- Collisions et contacts de base
- Masse, inertie, gravite, friction simple
- Latence et bruit capteurs simples
- LiDAR virtuel et sorties perception de base
- Journal de resultats et anomalies

## Entrees

- Assemblage ou cellule robotique
- Proprietes physiques
- Modele de commande
- Parametres de simulation

## Sorties

- Chronologie de simulation
- Etats des objets dans le temps
- Datasets perception et ecarts scene observee / scene attendue
- Rapports d'erreurs et d'ecarts

## Regles white-box

- Le pas de temps et le niveau de fidelite doivent etre visibles.
- Les collisions doivent indiquer les objets concernes.
- Les rapports doivent distinguer observation, interpretation et recommandation.

## Criteres d'acceptation

- Un scenario est rejouable a configuration egale.
- Les anomalies majeures sont listees dans un rapport.
- Deux versions d'un scenario peuvent etre comparees.
