# Plan Repo Et Scaffold

## Objectif

Definir la structure initiale du depot pour lancer l'implementation sans ambiguite organisationnelle.

## Type de depot

Monorepo unique.

Raisons:

- partage fort des types et contrats,
- versionning synchronise,
- CI plus simple au MVP,
- evolution coordonnee des modules coeur/UI.

## Role de GitHub

GitHub est la plateforme de reference pour:

- le remote canonique `origin`,
- l'hebergement du monorepo,
- les pull requests et la revue,
- les checks obligatoires avant merge,
- l'execution du pipeline GitHub Actions,
- l'archivage des artefacts de CI et des rapports de couverture.

## Arborescence cible

```text
FutureAero/
  OpenSpec/
  apps/
    desktop/
      src/
      src-tauri/
  packages/
    ui/
      src/
    viewport/
      src/
  crates/
    faero-core/
      src/
    faero-storage/
      src/
    faero-geometry/
      src/
    faero-assembly/
      src/
    faero-robotics/
      src/
    faero-sim/
      src/
    faero-perception/
      src/
    faero-integration/
      src/
    faero-safety/
      src/
    faero-commissioning/
      src/
    faero-optimization/
      src/
    faero-plugin-host/
      src/
    faero-ai/
      src/
    faero-types/
      src/
    faero-testkit/
      src/
  schemas/
    command.schema.json
    event.schema.json
    job.schema.json
    telemetry/
      *.schema.json
  examples/
    projects/
      pick-and-place-demo.faero/
  scripts/
    bootstrap.ps1
    fmt.ps1
    lint.ps1
    test.ps1
  .github/
    workflows/
      ci.yml
  Cargo.toml
  package.json
  pnpm-workspace.yaml
  rust-toolchain.toml
  README.md
```

## Responsabilites par dossier

### `apps/desktop`

- shell desktop Tauri,
- bridge UI <-> backend,
- menus et cycle de vie application,
- barre de menus native style Visual Studio.

### `packages/ui`

- composants React generiques,
- layout workspace,
- panneaux, arbres, listes, badges de statut,
- menu model et mapping `menu item -> command id`.

### `packages/viewport`

- integration Three.js,
- camera, gizmos, overlays,
- affichage de la scene derivee du coeur.

### `crates/faero-types`

- types partages Rust,
- contrats metier,
- envelopes `Command/Event/Job`,
- enums de base.

### `crates/faero-core`

- registre d'entites et de relations,
- bus de commandes,
- bus d'evenements,
- revisioning,
- undo/redo.

### `crates/faero-storage`

- format `.faero`,
- lecture/ecriture,
- migration,
- integrity checks.

### `crates/faero-geometry`

- esquisse,
- features,
- masse,
- export mesh d'affichage.

### `crates/faero-assembly`

- occurrences,
- mates,
- joints,
- solve d'assemblage.

### `crates/faero-robotics`

- scene robotique,
- cibles,
- sequences,
- enveloppes.

### `crates/faero-sim`

- simulation,
- timeline,
- metriques,
- artefacts de run.

### `crates/faero-perception`

- capteurs avances,
- LiDAR, cameras et IMU,
- calibration,
- replay,
- reconstruction,
- comparaison au modele nominal.

### `crates/faero-integration`

- ROS2,
- OPC UA,
- PLC,
- controleurs robots,
- Bluetooth/BLE, Wi-Fi et transports telemetriques,
- discovery, pairing et qualite de liaison,
- traces et replays d'integration.

### `crates/faero-safety`

- zones,
- interlocks,
- permissifs,
- LiDAR securite,
- validation safety.

### `crates/faero-commissioning`

- sessions commissioning,
- captures terrain,
- as-built vs as-designed,
- sign-off et historique d'ajustements.

### `crates/faero-optimization`

- etudes multi-objectifs,
- evaluation de candidats,
- contraintes,
- classement et rapports.

### `crates/faero-plugin-host`

- manifests,
- permissions,
- chargement,
- isolation,
- contributions plugins.

### `crates/faero-ai`

- retrieval local,
- orchestration inference,
- validation des sorties structurees,
- profils runtime `eco|standard|max|furnace`,
- critique interne multi-passes,
- scheduler de ressources locales.

### `crates/faero-testkit`

- fixtures de projets,
- scenes de test,
- helpers de replay.

## Frontieres de dependances

- `faero-types` ne depend de personne.
- `faero-core` depend de `faero-types`.
- `faero-storage` depend de `faero-types` et `faero-core`.
- `faero-geometry`, `faero-assembly`, `faero-robotics`, `faero-sim`, `faero-perception`, `faero-integration`, `faero-safety`, `faero-commissioning`, `faero-optimization`, `faero-plugin-host`, `faero-ai` dependent de `faero-types` et exposent des services a `faero-core`.
- l'UI ne depend pas directement des details internes des crates domaine.

## Regles de ownership

- `faero-core` possede la verite transactionnelle.
- les crates domaine ne persistent pas directement.
- `faero-storage` n'interprete pas la logique metier.
- `packages/viewport` ne calcule pas de verite physique ou geometrique.

## Bootstrap technique recommande

### Etape 1

- initialiser le monorepo JS + Rust,
- configurer Tauri,
- configurer pnpm,
- configurer workspace Cargo.

### Etape 2

- creer `faero-types`,
- creer `faero-core`,
- creer `faero-storage`,
- relier un `project.create` minimal.

### Etape 3

- afficher un projet vide dans l'UI,
- afficher la project tree,
- afficher une console jobs vide.

## Fichiers de configuration attendus

### `package.json`

Scripts minimaux:

- `dev`
- `build`
- `lint`
- `test`
- `typecheck`

### `Cargo.toml`

Doit declarer le workspace et les crates MVP.

### `rust-toolchain.toml`

- version Rust figee pour la CI

### `.github/workflows/ci.yml`

Jobs minimaux:

- install
- lint JS
- lint Rust
- test JS
- test Rust
- coverage gate 100
- build desktop

Role GitHub attendu:

- declenchement sur `push` et `pull_request`,
- statut bloqueur sur les checks critiques,
- publication des rapports de pipeline visibles depuis la PR.

## Convention de nommage

- crates Rust prefixees `faero-`
- commandes au format `domain.action`
- fichiers examples en kebab-case
- types metier en PascalCase

## Convention de branchement fonctionnel

- une story `ST-xxx` mappe sur un ou plusieurs PRs,
- toute PR cite au moins une story,
- pas de PR melangeant bootstrap infra et feature metier lourde sans raison explicite.
- la branche `main` est protegee cote GitHub et n'accepte que des merges via PR avec pipeline vert.

## Map spec -> modules

- `06-Modele-De-Donnees`: `faero-types`, `faero-core`, `faero-storage`
- `07-Format-De-Projet`: `faero-storage`
- `08-Architecture-Technique`: repo entier
- `09-Contrats-Internes`: `faero-types`, `faero-core`
- `10-Spec-Simulation-Detaillee`: `faero-sim`
- `11-Spec-IA-Locale`: `faero-ai`
- `16-Spec-Perception-Lidar-Et-Fusion-Capteurs`: `faero-perception`
- `17-Spec-IA-Ultra-Locale`: `faero-ai`
- `18-Spec-Integration-Industrielle`: `faero-integration`
- `19-Spec-Safety-Zones-Interlocks-Lidar-Securite`: `faero-safety`
- `20-Spec-As-Built-Vs-As-Designed-Et-Commissioning`: `faero-commissioning`
- `21-Spec-Optimization-Engine`: `faero-optimization`
- `22-Spec-Plugin-SDK`: `faero-plugin-host`
- `23-Politique-De-Tests-Et-Couverture-100`: repo entier
- `24-Spec-Connectivite-Sans-Fil-Et-Telemetrie`: `faero-integration`
- `archive/completed/2026-04/25-Spec-UI-Workspace-Et-Menus`: `apps/desktop`, `packages/ui`
- `archive/completed/2026-04/26-Spec-GitHub-PR-Et-Releases`: repo entier

## Livrable scaffold accepte

Le scaffold initial est accepte si:

- le depot clone et installe proprement,
- `pnpm dev` ouvre le shell desktop,
- `cargo test` passe pour `faero-types` et `faero-core`,
- un projet vide peut etre cree et sauvegarde.
