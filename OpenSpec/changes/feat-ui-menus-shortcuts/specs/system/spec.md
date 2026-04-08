# Specs Delta: UI Menus & Shortcuts

## ADDED Requirements

### Requirement: Standard Menu Bar
- **GIVEN** the FutureAero desktop application is running
- **WHEN** the user interacts with the top OS-level window
- **THEN** they must see a standard native menu bar containing: File, Edit, View, Insert, Simulation, AI, Help.

### Requirement: Global Keyboard Shortcuts
- **GIVEN** the application has focus
- **WHEN** the user presses a registered key combination (e.g., `Ctrl + S` or `Cmd + S`)
- **THEN** the corresponding application command is executed (e.g., saving the `.faero` project), regardless of the currently active React component (unless an input field prevents default).

### Requirement: Crucial Engineering Shortcuts Mapping
- `Ctrl+S` / `Cmd+S` : Sauvegarder le projet `.faero`
- `Ctrl+Z` / `Ctrl+Y` : Undo / Redo metier (relie au pipeline interne)
- `F5` : Lancer / mettre en pause la simulation
- `F10` : Avancer la simulation d'un pas (step)
- `Ctrl+Space` : Focus sur le chat de l'IA locale
- `Ctrl+B` : Compiler / reconstruire l'arbre geometrique (regeneration backend)
