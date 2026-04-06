# Moteur de Simulation Reel

## Objectif

Le moteur de simulation doit rapprocher le modele numerique du comportement terrain sans promettre une copie parfaite du monde reel. Il doit permettre une validation progressive, explicable et parametree.

## Couches de realisme

- Niveau S0 Concept: validation spatiale, volumes, collisions grossieres, sequences simples.
- Niveau S1 Ingenierie: inertie, gravite, friction, limites de course, vitesses, accelerations, latence capteurs.
- Niveau S2 Pre-implementation: bruit, tolerances, decalages d'assemblage, marges de securite, derives operationnelles, reprises sur erreur.

## Domaines a couvrir

- Geometrie et enveloppes spatiales
- Cinematique des chaines et effecteurs
- Dynamique des corps rigides
- Contacts et collisions
- Materiaux et masses
- Actionneurs et transmissions
- Capteurs et latences
- Perception active: LiDAR, camera, profondeur, fusion
- Evenements temporels et sequences
- Environnement de travail et zones interdites

## Hypotheses de modele

Chaque scenario doit expliciter:

- les unites,
- le pas de temps,
- le solveur selectionne,
- le niveau de fidelite,
- les materiaux,
- les hypotheses de frottement,
- les modeles de bruit et de latence actifs,
- les profils de calibration capteurs actifs,
- les simplifications connues.

## Regles white-box de simulation

- Aucune variable physique critique ne doit etre cachee.
- Les collisions et contraintes actives doivent pouvoir etre inspectees.
- Les observations capteurs synthetiques doivent pouvoir etre comparees a la scene source.
- Les ecarts entre mesure cible et resultat doivent etre quantifies.
- Un rapport doit separer donnees d'entree, calculs et conclusions.

## Cas d'usage prioritaires

- Verifier qu'un robot atteint sa zone de travail sans collision.
- Evaluer si un assemblage mecanique supporte un cycle de mouvement simple.
- Detecter un conflit entre convoyeur, pince et piece.
- Valider qu'un capteur virtuel voit bien la piece au bon instant.
- Evaluer si un LiDAR couvre correctement une zone et detecte un obstacle inattendu.
- Comparer un nuage de points issu du terrain a une implantation CAO attendue.
- Mesurer la marge entre trajectoire theorique et enveloppe securisee.

## Limites assumees du MVP

- Pas de simulation multiphysique avancee.
- Pas de modele materiau non lineaire detaille.
- Pas de calibration automatique terrain dans le premier increment.

## Criteres d'acceptation

- Un scenario simple peut etre rejoue a l'identique.
- Un rapport indique clairement les hypotheses et simplifications.
- Les collisions, depassements de course et erreurs de sequence sont remontes.
- Les resultats peuvent etre compares entre deux versions d'un meme projet.
