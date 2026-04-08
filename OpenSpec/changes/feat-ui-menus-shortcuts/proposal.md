# Proposal: Menus Standards et Raccourcis Clavier

## Intent
L'objectif est de doter FutureAero d'une navigation conforme aux standards des logiciels de CAO/Ingenierie (ex: CATIA, SolidWorks, Visual Studio). Cela permet aux utilisateurs de retrouver leurs automatismes (Ctrl+S, Ctrl+Z, F5) et d'acceder rapidement aux fonctionnalites metier sans avoir a chercher dans des panneaux graphiques complexes.

## Scope
- **Inclus :** Creation d'une barre de menus native via Tauri (Fichier, Edition, Vue, Insertion, Simulation, IA, Aide). Implementation d'un gestionnaire de raccourcis clavier global dans le frontend React (`keyboard-shortcuts.mjs`). Affichage des raccourcis dans les tooltips de l'interface.
- **Exclus :** Personnalisation dynamique des raccourcis par l'utilisateur (sera fait dans une iteration ulterieure de type "Preferences"). Raccourcis specifiques a la manipulation 3D (pan, zoom, orbit) qui relevent du module Viewport.
