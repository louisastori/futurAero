# Spec Safety Zones Interlocks LiDAR Securite

## Objectif

Definir une couche safety explicable reliant zones, capteurs, interlocks, permissifs et inhibitions.

## Finalite produit

Cette couche doit permettre:

- de modeliser les zones de surete,
- de comprendre pourquoi un mouvement est autorise ou bloque,
- d'integrer des LiDAR de securite,
- de rejouer et auditer une decision safety.

## Concepts centraux

- zone safe,
- zone warning,
- zone slowdown,
- zone forbidden,
- interlock,
- permissif,
- inhibition,
- safety state,
- ack / reset.

## Entrees

- geometrie cellule,
- robots et equipements,
- capteurs de securite,
- LiDAR securite,
- logique safety,
- etats externes PLC/robot.

## Sorties

- etat safety courant,
- causes d'inhibition,
- transitions safety,
- rapport de validation safety,
- trace des activations LiDAR securite.

## LiDAR securite

Le systeme doit permettre de declarer:

- zones surveillees,
- champs de protection,
- comportements slowdown/stop,
- latence,
- seuils et profils de reaction.

## Interlocks

Une regle d'interlock doit declarer:

- ses entrees,
- sa condition,
- ses actions,
- sa priorite,
- son domaine d'effet.

## Validation

Le moteur safety doit pouvoir:

- verifier la coherence des interlocks,
- detecter conflits et boucles,
- tester des scenarios nominal / warning / arret,
- prouver pourquoi une action a ete inhibee.

## Position produit

Le MVP modele et rejoue la logique safety, mais ne revendique pas a lui seul une certification reglementaire de terrain.

## Regles white-box

- aucune inhibition sans cause listable,
- chaque zone doit etre visualisable,
- chaque trigger LiDAR securite doit etre horodate,
- chaque interlock doit etre relie a ses entrees et sorties.

## Criteres d'acceptation

- une cellule peut definir plusieurs zones de surete,
- un interlock peut inhiber un mouvement avec justification,
- un LiDAR securite peut declencher slowdown ou stop dans la simulation,
- un rapport safety peut lister les causes de blocage et les transitions critiques.
