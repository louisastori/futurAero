# Design: Robot Cell Structure MVP

## Context

Le depot possede deja un flux desktop `entity.create.robot_cell`, mais la cellule produite est surtout un payload de demonstration avec `targets`, `sequenceValidation`, `control` et `safety` embarques. Cette forme permet de piloter la simulation MVP, mais elle ne suit pas encore le modele white-box attendu par `ST-401`: `RobotCell` doit devenir un agregat lisible pointant vers une scene assembly, un robot principal, des equipements, des zones de securite, des sequences et des controleurs.

Le change doit rester strictement MVP:
- pas d editeur 3D complet de layout,
- pas de multirobot ou d equipements parametriques riches,
- pas de nouveau solveur de cellule,
- mais une structure explicite, persistante et inspectable par le shell desktop, le stockage et l IA locale.

## Goals / Non-Goals

**Goals:**
- definir un modele partage cote robotique pour la structure minimale d une `RobotCell`,
- creer explicitement les entites support `RobotModel`, `EquipmentModel`, `RobotSequence` et les zones de securite associees,
- persister dans la `RobotCell` les references requises par le backlog (`sceneAssemblyId`, `robotIds`, `equipmentIds`, `safetyZoneIds`, `sequenceIds`, `controllerModelIds`),
- garder la compatibilite avec le pipeline simulation/controle existant en conservant les resumes utiles,
- exposer dans les snapshots desktop une cellule lisible avec compte d equipements et details structurels.

**Non-Goals:**
- livrer un moteur de layout robotique avancé,
- modeliser plusieurs robots ou plusieurs sequences concurrentes,
- remplacer la logique de simulation ou de safety deja MVP,
- introduire une UI d edition graphique des equipements.

## Decisions

### 1. Porter le contrat structurel dans `faero-robotics`
La structure MVP de cellule robotique est plus proche du domaine robotique que du noyau generique d entites. On ajoute donc dans `faero-robotics` des structs serialisables pour `RobotCellModel`, `RobotModel`, `EquipmentModel` et `RobotSequenceModel`, ainsi qu une validation simple de coherence des references.

Alternative consideree:
- placer ces types dans `faero-types`.
- Rejetee car ils restent specialises robotique et n ont pas besoin d etre partagees par tout le graphe a ce stade.

### 2. Garder `RobotCell` comme agregat racine et les details comme entites support
La `RobotCell` persiste les ids des entites support et un resume exploitable directement par le shell. Les robots, equipements, sequences, controleurs et zones de securite sont crees comme entites top-level distinctes pour garder un graphe white-box et facilement inspectable.

Alternative consideree:
- laisser tous les sous-elements inline dans `RobotCell`.
- Rejetee car cela masque la structure demandee par `ST-401` et rend les references futures plus fragiles.

### 3. Reutiliser une scene assembly explicite pour le layout
Le flux desktop cree une assembly de scene rattachee a la cellule, ainsi que des occurrences lisibles pour le robot et les equipements MVP. Les `EquipmentModel` referencent ces occurrences via `assemblyOccurrenceId`.

Alternative consideree:
- ne stocker qu un `sceneAssemblyId` vide sans occurrences.
- Rejetee car cela ne donne pas de support lisible aux equipements ni a leurs references.

### 4. Conserver les resumes existants pour la compatibilite MVP
Les sections `sequenceValidation`, `control` et `safety` restent presentes dans `RobotCell` pour ne pas casser les parcours simulation, safety et shell existants. La nouveaute est l ajout d une couche structurelle explicite par ids et entites support.

Alternative consideree:
- remplacer integralement l ancien payload par le nouveau modele.
- Rejetee pour limiter le risque de regression sur la simulation MVP deja en place.

## Risks / Trade-offs

- [Structure plus verbeuse qu avant] -> Mitiger en gardant des builders centralises et des ids deterministes.
- [Chevauchement entre donnees inline et entites support] -> Mitiger en traitant les blocs inline comme resumes de compatibilite derives du graphe structurel.
- [Dependance desktop forte pour creer toute la cellule] -> Mitiger en encapsulant le modele et les validations dans `faero-robotics` plutot que dans la seule couche UI.

## Migration Plan

- Changement additif: les `RobotCell` existantes restent lisibles.
- Les nouvelles cellules desktop utiliseront le nouveau graphe structurel.
- Aucun rollback special au dela d un revert de commit n est requis pour ce MVP.

## Open Questions

- Le prochain increment doit-il sortir les zones de securite dans un type partage dedie hors du shell desktop ?
- Faut-il brancher les futures sequences edition/commande sur `RobotSequenceModel` avant de deplacer toute la validation hors de `RobotCell` ?
