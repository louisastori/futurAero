# Design: Menu Model & Keybindings

## Architecture Overview
La gestion des menus et raccourcis necessite une communication bidirectionnelle entre le backend Tauri (Rust) et le frontend React (JavaScript).

1. **Native Menu (Tauri) :** Les menus natifs de l'OS sont configures dans `apps/desktop/src-tauri/src/main.rs`. Lorsqu'un menu est clique, Tauri emet un evenement IPC au frontend.
2. **Menu Model (`packages/ui/src/menu-model.mjs`) :** Unifie la definition des commandes. Il sert de source de verite pour generer a la fois les menus contextuels dans l'UI web et faire le pont avec Tauri.
3. **Shortcut Manager (`packages/ui/src/keyboard-shortcuts.mjs`) :** Un hook global React (ou un listener `window` global) ecoute les frappes. Il intercepte les combinaisons et declenche les fonctions du store d'etat.

## UX Details
- L'interface white-box exige de la transparence : chaque action declenchee par un raccourci clavier doit afficher une breve notification non intrusive (ex: "Projet sauvegarde", "Simulation en cours...").
- Les tooltips des boutons de l'interface graphique doivent automatiquement afficher le raccourci associe (ex: "Lancer la simulation (F5)").
