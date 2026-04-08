# Proposal: Kinematic Joints MVP

## Why

L assemblage FutureAero sait maintenant persister des occurrences, des mates et un solve report explicite, mais il manque encore la notion de joint cinematique demandee par le backlog MVP. Sans joints `fixed|revolute|prismatic`, le projet ne peut pas decrire proprement une chaine mecanique pilotable ni exposer les degres de liberte attendus par la robotique et la simulation.

## What Changes

- Introduire une capacite OpenSpec `kinematic-joints` pour formaliser le contrat MVP des joints.
- Ajouter un modele partage pour les joints, leurs limites, leur etat courant et leur nombre de degres de liberte.
- Etendre le pipeline coeur afin de creer et mettre a jour des joints sur un assemblage de maniere explicite et auditable.
- Exposer un premier etat de joint lisible par le shell desktop, les snapshots projet et les tests.
- Preparer le branchement futur vers `faero-robotics` et `faero-sim` sans essayer de livrer toute la cinematique avancee dans le meme increment.

## Capabilities

### New Capabilities
- `kinematic-joints`: joints MVP `fixed|revolute|prismatic` avec limites, etat pilotable et degres de liberte explicites.

### Modified Capabilities

## Impact

- `crates/faero-types`
- `crates/faero-core`
- `crates/faero-assembly`
- `crates/faero-robotics`
- `apps/desktop/src-tauri/src/main.rs`
- tests Rust et desktop relies aux assemblages et a la robotique
