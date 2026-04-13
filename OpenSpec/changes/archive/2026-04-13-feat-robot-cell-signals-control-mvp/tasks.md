# Implementation Tasks

## 1. Robotics Control Model

- [x] 1.1 Introduire dans `crates/faero-robotics` un modele MVP de controle cellule robotique et la validation des `Signal` et `ControllerStateMachine`.
- [x] 1.2 Exposer un resume de controle deterministe incluant `signalCount`, `controllerTransitionCount` et `blockedSequenceDetected`.

## 2. Desktop Backend And Simulation Bridge

- [x] 2.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour reconstruire les entites support `Signal` et `Controller` a partir du contrat partage et recalculer le resume de controle.
- [x] 2.2 Brancher `crates/faero-sim` ou le bridge desktop sur le meme graphe de controle valide pour detecter une sequence bloquee.

## 3. Fallback And Exposure

- [x] 3.1 Aligner `apps/desktop/src/robotCellFallback.js` et les helpers de test sur la meme structure de signaux et de controle.
- [x] 3.2 Exposer dans les snapshots et details une vue minimale lisible du controle cellule robotique.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust couvrant la validation des signaux et transitions ainsi que la detection de sequence bloquee.
- [x] 4.2 Ajouter des tests desktop ou UI couvrant l edition de signaux et l affichage du resume de controle.
