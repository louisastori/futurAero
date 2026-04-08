# Proposal: Assembly Occurrences And Mates

## Why

FutureAero expose deja un premier solveur d assemblage et une commande desktop `entity.create.assembly`, mais l assemblage reste aujourd hui un jalon de demonstration: les occurrences et les mates sont syntheses d un bloc, sans pipeline explicite de commandes ni contrat OpenSpec propre au domaine. Le backlog MVP demande pourtant un assemblage navigable, persistant et explicable avant de pousser plus loin la robotique et la simulation.

## What Changes

- Introduire une capacite OpenSpec `assembly` pour formaliser les exigences MVP autour des occurrences, des mates et du solve report.
- Rendre les occurrences et contraintes d assemblage explicites dans le modele partage au lieu de ne conserver qu un resume ad hoc construit lors de la creation.
- Faire passer les mutations d assemblage du shell desktop par des commandes assemblees dediees et auditables.
- Persister et exposer un solve report lisible apres ajout, transformation ou suppression d occurrences et de mates.
- Ajouter des tests Rust et desktop couvrant le flux d assemblage nominal et les erreurs de validation.

## Capabilities

### New Capabilities
- `assembly`: composition explicite de pieces et sous-ensembles via occurrences, mates MVP et solve reports lisibles.

### Modified Capabilities

## Impact

- `crates/faero-types`
- `crates/faero-core`
- `crates/faero-assembly`
- `apps/desktop/src-tauri/src/main.rs`
- `apps/desktop/src/App.jsx`
- tests Rust et desktop relies a l assemblage
