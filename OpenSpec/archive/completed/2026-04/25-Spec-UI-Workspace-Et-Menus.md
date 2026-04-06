# Spec UI Workspace Et Menus

Statut: implemente-et-teste puis archive le 2026-04-06

Implementation de reference:

- `apps/desktop/src/App.jsx`
- `apps/desktop/src/styles.css`
- `packages/ui/src/menu-model.mjs`
- `packages/ui/src/i18n.mjs`
- `packages/ui/src/menu-model.test.mjs`

## Objectif

Definir un workspace desktop avec une barre de menus proche de Visual Studio, tout en restant adaptee a un logiciel FutureAero de CAO, simulation, robotique et jumeau numerique.

## Direction UX

Le produit ne doit pas copier Visual Studio a l'identique, mais reprendre ses reperes forts:

- barre de menus dense et stable,
- commandes accessibles par familles metier claires,
- panneaux dockables,
- raccourcis clavier memorisables,
- distinction nette entre edition, build, debug, test et analyse.

## Menus top-level cibles

Ordre recommande:

- `File`
- `Edit`
- `View`
- `Git`
- `Project`
- `Build`
- `Debug`
- `Test`
- `Analyze`
- `Tools`
- `Window`
- `Help`

## Mapping FutureAero des menus

### File

- nouveau projet,
- ouverture,
- recents,
- sauvegarde,
- import/export,
- preferences applicatives.

### Edit

- undo/redo,
- presse-papiers,
- suppression,
- recherche,
- command palette.

### View

- project explorer,
- properties,
- output,
- problems,
- jobs,
- AI assistant,
- viewport 3D,
- timeline simulation,
- monitor telemetrie.

### Git

- commit,
- push,
- pull,
- branches,
- statut de repo.

### Project

- ajout de part,
- ajout d'assembly,
- ajout de robot cell,
- ajout de sensor rig,
- ajout d'endpoint externe,
- proprietes projet.

### Build

- regeneration geometrique,
- rebuild assembly,
- build robot cell,
- preparation package commissioning.

### Debug

- start simulation,
- start without debugging,
- stop,
- step timeline,
- step into logic,
- breakpoints simulation.

### Test

- run all tests,
- run fixture courante,
- rapport coverage,
- replay scenario.

### Analyze

- validation report,
- as-built vs as-designed,
- safety analysis,
- optimization study,
- AI deep explain.

### Tools

- extensions et plugins,
- device manager,
- telemetry streams,
- options.

### Window

- nouvelle fenetre,
- split view,
- reset layout,
- close all documents.

### Help

- documentation,
- OpenSpec,
- raccourcis clavier,
- about.

## Regles de conception

- le nom des menus top-level doit rester stable entre versions mineures,
- chaque item de menu doit pointer vers une commande interne explicite,
- aucun item de menu ne doit executer une mutation silencieuse sans passer par le pipeline de commandes,
- la barre de menus native desktop reste la reference, meme si une toolbar secondaire existe,
- les panneaux `Project Explorer`, `Properties`, `Output` et `Problems` sont consideres comme equivalents aux reperes Visual Studio dans FutureAero.

## Raccourcis recommandes

- `Ctrl+Shift+N`: nouveau projet
- `Ctrl+O`: ouvrir projet
- `Ctrl+S`: sauvegarder
- `Ctrl+Shift+S`: sauvegarder tout
- `Ctrl+Z`: undo
- `Ctrl+Y`: redo
- `Ctrl+Shift+P`: command palette
- `F4`: properties
- `F5`: start simulation
- `Ctrl+F5`: start without debugging
- `Shift+F5`: stop
- `F10`: step timeline
- `F11`: step into logic

## Criteres d'acceptation

- le shell desktop expose les menus top-level dans l'ordre defini,
- chaque item de menu est relie a un `command id`,
- les panneaux principaux sont pilotables depuis `View`,
- les commandes `Build`, `Debug`, `Test` et `Analyze` sont distinctes et non melangees,
- l'application privilegie le francais par defaut et expose l'anglais et l'espagnol comme langues secondaires du shell.
