# Implementation Tasks

## 1. Shared Assembly Model

- [x] 1.1 Introduire dans `crates/faero-types` des types partages pour les occurrences, mates et solve reports d assemblage avec tests de serialisation.
- [x] 1.2 Harmoniser le payload `Assembly` persiste autour de ces types pour que le backend, le stockage et l UI lisent le meme contrat.

## 2. Core Command Pipeline

- [x] 2.1 Etendre `crates/faero-core` avec des commandes assembly explicites pour creation, ajout ou transformation d occurrence, ajout de mate et suppression de mate.
- [x] 2.2 Produire les kinds de commandes et evenements assembly auditables, puis recalculer automatiquement le solve report apres chaque mutation supportee.

## 3. Desktop Integration

- [x] 3.1 Mettre a jour `apps/desktop/src-tauri/src/main.rs` pour que le flux `entity.create.assembly` et les mutations assembly utilisent le pipeline assembly dedie au lieu d un payload ad hoc.
- [x] 3.2 Exposer dans les snapshots desktop et le panneau proprietes les details assembly persistants: occurrences, mates, statut de solve et warnings.

## 4. Validation

- [x] 4.1 Ajouter des tests Rust couvrant les validations assembly, le solve report et les erreurs de references invalides.
- [x] 4.2 Ajouter des tests desktop/UI couvrant la creation d assemblage, la visibilite de l activite assembly et la lecture des details persistants.
