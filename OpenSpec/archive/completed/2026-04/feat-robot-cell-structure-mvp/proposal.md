# Proposal: Robot Cell Structure MVP

## Why

FutureAero sait deja creer une `RobotCell` de demonstration, mais son payload reste surtout oriente resume de sequence et simulation. Le backlog `ST-401` demande maintenant une scene robotique structuree avec references explicites vers l assemblage de scene, le robot, les equipements, les zones de securite, les sequences et le controleur.

## What Changes

- Introduire une capacite OpenSpec `robot-cell-structure` pour formaliser le contrat MVP d une `RobotCell` structuree.
- Ajouter un modele partage cote robotique pour une `RobotCell` lisible, ses robots, ses equipements et ses sequences.
- Etendre le flux desktop `entity.create.robot_cell` pour creer une scene assembly et les entites support explicites associees.
- Persister dans `RobotCell` les references `sceneAssemblyId`, `robotIds`, `equipmentIds`, `safetyZoneIds`, `sequenceIds` et `controllerModelIds`.
- Exposer dans les snapshots projet et les tests desktop une lecture white-box des equipements et de la structure de cellule.

## Capabilities

### New Capabilities
- `robot-cell-structure`: `RobotCell` MVP structuree avec scene assembly, robot principal, equipements explicites, zones de securite, sequences et references de controle.

### Modified Capabilities

## Impact

- `crates/faero-robotics`
- `apps/desktop/src-tauri/src/main.rs`
- `apps/desktop/src/App.jsx`
- `apps/desktop/src/App.test-helpers.jsx`
- tests Rust et desktop lies aux cellules robotiques, a la simulation et au stockage white-box
