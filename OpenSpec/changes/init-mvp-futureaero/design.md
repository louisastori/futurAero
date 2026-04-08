# Design: Architecture FutureAero

## Architecture Overview
Le projet suit une architecture Frontend/Backend lourde optimisée pour le Desktop :
- **Frontend :** Shell desktop React, propulsé par Vite et Tauri. Gestion de l'état global et communication IPC (Inter-Process Communication) avec Rust.
- **Backend (Core) :** Noyau minimal Rust divisé en crates (`crates/`). Il héberge la logique métier (calcul vectoriel, collision, cinématique).
- **Persistance :** Un format de projet diffable et textuel (`*.faero`) permettant de versionner les scènes, la géométrie, et l'historique de simulation.

## Key Technical Decisions
1. **Tauri plutôt qu'Electron :** Pour réduire l'empreinte mémoire et bénéficier des performances natives de Rust pour les calculs physiques lourds.
2. **Ollama local :** Pour le moteur d'IA. Le backend s'interfacera avec le service Ollama localisé sur la workstation pour des inférences sans latence réseau.
3. **OpenSpec Textuel (`*.faerospec`) :** Pour capturer les intentions d'ingénierie sans conteneur binaire opaque.