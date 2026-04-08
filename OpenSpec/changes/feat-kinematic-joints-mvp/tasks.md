# Implementation Tasks

## 1. Shared Joint Model

- [x] 1.1 Introduire dans `crates/faero-types` les types partages de joints MVP, limites, axe et etat courant.
- [x] 1.2 Etendre le payload `Assembly` pour persister une liste de joints lisibles et leurs degres de liberte.

## 2. Core And Assembly Pipeline

- [x] 2.1 Ajouter dans `crates/faero-core` les commandes explicites `joint.create` et `joint.state.set` avec validations des occurrences et des limites.
- [x] 2.2 Etendre `crates/faero-assembly` pour calculer ou exposer les degres de liberte MVP attendus par type de joint.

## 3. Desktop Exposure

- [x] 3.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour exposer les joints et leur etat dans les snapshots assembly.
- [x] 3.2 Ajouter un premier flux desktop lisible pour creer ou visualiser un joint MVP sans sortir du pipeline de commandes.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust couvrant creation de joint, limites invalides, etat courant et degres de liberte.
- [x] 4.2 Ajouter des tests desktop ou UI couvrant la presence des joints dans les snapshots et l activite recente.
