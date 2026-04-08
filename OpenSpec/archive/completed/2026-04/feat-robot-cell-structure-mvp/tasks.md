# Implementation Tasks

## 1. Structured Robot Cell Model

- [x] 1.1 Introduire dans `crates/faero-robotics` les types MVP `RobotCellModel`, `RobotModel`, `EquipmentModel` et `RobotSequenceModel`.
- [x] 1.2 Ajouter une validation simple des references et des comptes structurels attendus pour une cellule robotique MVP.

## 2. Desktop Support Graph

- [x] 2.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour creer une scene assembly et des entites support explicites lors de `entity.create.robot_cell`.
- [x] 2.2 Persister dans `RobotCell` les ids `sceneAssemblyId`, `robotIds`, `equipmentIds`, `safetyZoneIds`, `sequenceIds` et `controllerModelIds` tout en gardant les resumes utiles au shell et a la simulation.

## 3. Desktop Exposure

- [x] 3.1 Etendre les snapshots et details desktop pour exposer les comptes d equipements et la structure lisible de la cellule.
- [x] 3.2 Mettre a jour le fallback web et les helpers de test pour refleter la nouvelle structure `RobotCell`.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust couvrant la validation du modele et la creation des entites support de cellule robotique.
- [x] 4.2 Ajouter des tests desktop ou UI couvrant la presence de `sceneAssemblyId`, des equipments et des entites support dans les snapshots et l activite recente.
