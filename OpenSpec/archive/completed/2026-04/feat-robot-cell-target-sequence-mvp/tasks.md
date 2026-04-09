# Implementation Tasks

## 1. Robotics Model

- [x] 1.1 Introduire dans `crates/faero-robotics` un type MVP `RobotTargetModel` et la validation de base associee.
- [x] 1.2 Aligner `RobotSequenceModel.targetIds` sur des ids d entites cibles reels et valider l ordre editable.

## 2. Desktop Backend

- [x] 2.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour creer des entites support `RobotTarget` lors de `entity.create.robot_cell`.
- [x] 2.2 Recalculer `RobotSequence` et `RobotCell` quand une entite `RobotTarget` est modifiee via `entity.properties.update`.

## 3. Desktop Exposure

- [x] 3.1 Exposer dans les snapshots/details une preview ordonnee lisible des cibles de sequence.
- [x] 3.2 Mettre a jour le fallback web et les helpers de test pour inclure les entites `RobotTarget` et leur ordre.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust couvrant la validation des cibles et la synchronisation sequence/cellule.
- [x] 4.2 Ajouter des tests desktop ou UI couvrant l affichage et l edition de l ordre des cibles.
