# Proposal: Initialisation du MVP FutureAero

## Intent
L'objectif est de construire un studio d'ingénierie "white-box" unifié sous forme d'application desktop (hors-ligne). Ce logiciel doit rassembler la CAO (CAD/CAE), la robotisation, la simulation physique et une intégration d'IA locale, afin de réduire la fragmentation des outils industriels traditionnels.

## Scope
- **Inclus :** Application desktop avec un backend en Rust et une interface en Tauri + React. Intégration d'un assistant IA local (Ollama). Implémentation des modules de base (géométrie, assemblage, simulation, perception, robotique).
- **Exclus :** Certification réglementaire des secteurs critiques pour cette version, remplacement complet des usines existantes sans intégration, reproduction immédiate de toutes les fonctions de CATIA/SolidWorks.

## Approach
Nous utiliserons une architecture en monorepo avec des "crates" Rust isolés pour le backend (`faero-geometry`, `faero-sim`, etc.) afin de garantir la performance et la transparence (white-box). Le frontend offrira un espace de travail inspiré des IDE professionnels avec un graphe de projet, un viewport 3D et un chat IA contextuel.