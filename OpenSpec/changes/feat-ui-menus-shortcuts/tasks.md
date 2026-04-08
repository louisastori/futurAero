# Implementation Tasks

## 1. Modelisation et Base (Packages UI)
- [ ] 1.1 Definir la structure des menus et des commandes dans `packages/ui/src/menu-model.mjs` avec leurs raccourcis associes.
- [ ] 1.2 Creer un systeme d'enregistrement d'evenements clavier global dans `packages/ui/src/keyboard-shortcuts.mjs` (supportant macOS `Cmd` et Windows/Linux `Ctrl`).

## 2. Integration Native Tauri (Rust)
- [ ] 2.1 Configurer la `Menu` API dans `src-tauri/src/main.rs` pour construire la barre de menus native (File, Edit, View, Insert, Simulation, AI).
- [ ] 2.2 Ajouter les emetteurs d'evenements dans Rust pour envoyer un signal au frontend lorsqu'un item du menu natif est clique.

## 3. Integration Frontend (React)
- [ ] 3.1 Dans `apps/desktop/src/App.jsx`, ajouter les listeners IPC pour ecouter les evenements provenant du menu natif Tauri.
- [ ] 3.2 Relier les listeners IPC et le gestionnaire `keyboard-shortcuts.mjs` aux actions de l'application (sauvegarde backend, trigger Ollama, lancement de la boucle Rust de simulation).
- [ ] 3.3 Mettre a jour les composants UI (boutons, tooltips) pour qu'ils affichent dynamiquement la touche raccourci lue depuis `menu-model.mjs`.

## 4. Tests
- [ ] 4.1 Ecrire des tests unitaires pour `keyboard-shortcuts.test.mjs` verifiant la bonne capture des combos de touches.
- [ ] 4.2 Ecrire des tests unitaires pour `menu-model.test.mjs` verifiant la validite de l'arbre de menu.
