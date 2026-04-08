# Vision Produit

## Probleme vise

Les outils CAD/CAE classiques couvrent bien la geometrie et parfois la simulation, mais la boucle complete entre conception mecanique, robotisation, commande, validation realiste et preparation a l'implementation terrain reste souvent fragmentee.

Le projet FutureAero vise a reunir dans une seule application desktop:

- la conception parametrique de pieces et d'assemblages,
- la definition de cellules et systemes de robotisation,
- une simulation proche du reel,
- une perception avancee pour lier le jumeau numerique au terrain,
- une IA locale haut de gamme capable d'assister sans devenir une boite noire.

## Vision

Construire un studio d'ingenierie "white-box" permettant de passer d'une idee mecanique ou robotique a un systeme virtuel testable, explicable et industrialisable.

## Utilisateurs cibles

- Ingenieur mecanique
- Ingenieur robotique
- Integrateur industriel
- Ingenieur simulation
- Responsable methode / industrialisation

## Valeur principale

- Unifier CAD, cinematique, simulation et commande.
- Relier le modele numerique a des capteurs reellement deployables comme le LiDAR.
- Reduire l'ecart entre prototype numerique et mise en service reelle.
- Garder la maitrise des hypotheses, parametres et decisions de l'IA.
- Permettre un fonctionnement local pour proteger la propriete intellectuelle.

## Objectifs produit

- Fournir un environnement desktop operable hors ligne.
- Permettre la creation de pieces, assemblages et cellules robotiques.
- Simuler le comportement mecanique et operationnel d'un systeme.
- Lier chaque objet du projet a des proprietes physiques et des contraintes.
- Integrer une IA locale pour assister la modelisation, l'analyse et la documentation.
- Integrer une couche perception capable de reconstruire et comparer l'environnement reel.
- Permettre une validation progressive allant du concept a la pre-implementation.
- Exploiter au maximum la puissance locale disponible quand l'utilisateur privilegie la qualite de raisonnement.

## Non-objectifs du MVP

- Reproduire tout le perimetre fonctionnel de CATIA ou SOLIDWORKS des la premiere version.
- Supporter d'emblee tous les procedes industriels complexes.
- Garantir une certification reglementaire secteur des le MVP.
- Remplacer les automates, robots ou logiciels usine existants sans couche d'integration.

## Positionnement white-box

Dans FutureAero, "white-box" signifie:

- les relations geometriques sont inspectables,
- les modeles physiques exposes et parametrables,
- les chaines cinematiques visibles,
- les decisions IA justifiees,
- les exports et transformations tracables.

## Definition de succes initial

- Un utilisateur peut concevoir une piece simple et un assemblage.
- Un utilisateur peut modeliser une cellule robotique de base.
- Un scenario peut etre simule avec collisions, gravite, inertie et trajectoires.
- L'IA locale peut expliquer une contrainte, proposer une correction et generer une note technique.
- Les hypotheses du projet sont enregistrables et rejouables.
- Un scan LiDAR ou une carte d'occupation peuvent etre relies au meme graphe projet que la CAO.
