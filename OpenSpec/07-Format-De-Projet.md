# Format de Projet

## Objectif

Definir un format de projet local, diffable, robuste et compatible avec le fonctionnement hors ligne.

## Decision de format

Le format de reference du MVP est un dossier projet avec extension `.faero`.

Exemple:

- `Cellule-Demo.faero/`

Ce format dossier est choisi pour:

- faciliter le diff Git,
- permettre la lecture partielle,
- isoler les caches du coeur metier,
- simplifier la reprise apres crash.

## Arborescence canonique

```text
Cellule-Demo.faero/
  project.yaml
  graph/
    nodes/
      ent_*.json
    edges.jsonl
  scenes/
    scene_*.json
  simulations/
    scenarios/
      ent_*.json
    runs/
      run_*.json
  integration/
    endpoints/
      ext_*.json
    streams/
      str_*.json
    bindings/
      bind_*.json
    traces/
      trace_*.json
  safety/
    zones/
      safe_*.json
    interlocks/
      lock_*.json
    validations/
      sval_*.json
  perception/
    calibrations/
      cal_*.json
    pipelines/
      pipe_*.json
    datasets/
      pcd_*.json
    maps/
      map_*.json
  commissioning/
    sessions/
      com_*.json
    captures/
      cap_*.json
    comparisons/
      cmp_*.json
  optimization/
    studies/
      opt_*.json
    runs/
      opr_*.json
  plugins/
    manifests/
      plg_*.json
    state/
      plugins.json
  openspec/
    docs/
      ops_*.faerospec
  validations/
    report_*.json
  ai/
    sessions/
      session_*.json
    suggestions/
      ais_*.json
  assets/
    blobs/
      ast_*
    manifests/
      ast_*.json
  events/
    commands.jsonl
    events.jsonl
  cache/
    meshes/
    thumbnails/
    indexes/
  logs/
    app.log
```

## Fichiers obligatoires

### `project.yaml`

Contient:

- `projectId`
- `name`
- `formatVersion`
- `createdAt`
- `updatedAt`
- `appVersion`
- `displayUnits`
- `defaultFrame`
- `rootSceneId`
- `activeConfigurationId`

### `graph/nodes/*.json`

- un fichier par entite persistable,
- nom du fichier egal a l'identifiant,
- contenu JSON canonique.

### `graph/edges.jsonl`

- une relation par ligne,
- utile pour charger le graphe sans reparser tous les noeuds.

### `events/commands.jsonl`

- journal append-only des commandes utilisateur et systeme.

### `events/events.jsonl`

- journal append-only des evenements derives des commandes.

## Fichiers optionnels

- `simulations/runs/*.json`
- `integration/endpoints/*.json`
- `integration/streams/*.json`
- `integration/bindings/*.json`
- `safety/zones/*.json`
- `safety/interlocks/*.json`
- `commissioning/sessions/*.json`
- `commissioning/captures/*.json`
- `commissioning/comparisons/*.json`
- `optimization/studies/*.json`
- `optimization/runs/*.json`
- `plugins/manifests/*.json`
- `perception/calibrations/*.json`
- `perception/pipelines/*.json`
- `perception/datasets/*.json`
- `perception/maps/*.json`
- `validations/*.json`
- `ai/sessions/*.json`
- `ai/suggestions/*.json`
- `assets/*`
- `logs/*`
- `openspec/docs/*.faerospec`

## Documents OpenSpec lisibles

Les informations OpenSpec ne doivent pas repliquer les conteneurs binaires internes de CATIA ou d autres CAO vendor quand ces conteneurs ne sont pas lisibles en clair.

FutureAero introduit donc un format texte natif `*.faerospec` stocke dans `openspec/docs/`.

Chaque fichier `*.faerospec` contient:

- un front matter YAML delimite par `---`,
- un corps Markdown UTF-8,
- des references explicites vers les entites ou endpoints concernes,
- un contenu diffable, mergeable et inspectable sans outil proprietaire.

Champs de front matter requis:

- `id`
- `title`
- `kind`
- `status`
- `bodyFormat`
- `entityRefs`
- `externalRefs`
- `tags`
- `updatedAt`

Exemple:

```text
---
id: ops_pick_layout
title: Intentions d implantation de cellule
kind: design_intent
status: active
bodyFormat: markdown
entityRefs:
  - ent_cell_001
externalRefs:
  - ext_robot_001
tags:
  - openspec
  - mvp
updatedAt: "2026-04-08T08:00:00Z"
---
## Intent

La cellule doit rester lisible sans dependre d un format binaire vendor.

## Decisions

- Les hypotheses d implantation sont decrites ici.
- Les liens vers le graphe projet restent explicites via `entityRefs`.
```

Regles:

- Un document `*.faerospec` ne stocke jamais de maillage, B-Rep ou donnees de calcul massives.
- Un import vendor peut conserver son binaire dans `assets/`, mais les decisions utiles a l ingenierie doivent etre miroirisees dans un `*.faerospec`.
- Le corps Markdown doit rester exploitable meme sans rendu riche.

## Regles de serialisation

- Encodage: UTF-8
- Separateur de ligne: LF
- Horodatage: ISO 8601 UTC
- Nombres: notation JSON standard, sans format local
- Ordre des cles JSON: stable et trie lexicalement
- Unites stockees: SI pour les valeurs resolues
- Unites d'affichage: stockees dans `project.yaml`

## Regles de nommage

- Les identifiants sont les noms de fichier pour les entites.
- Les espaces sont interdits dans les noms de fichier techniques.
- Les noms utilisateur peuvent contenir des espaces mais ne pilotent jamais les chemins de persistance.

## Donnees diffables contre donnees derivees

Donnees versionnables:

- `project.yaml`
- `graph/nodes/*.json`
- `graph/edges.jsonl`
- `events/*.jsonl`
- `simulations/scenarios/*.json`
- `integration/endpoints/*.json`
- `integration/streams/*.json`
- `integration/bindings/*.json`
- `safety/zones/*.json`
- `safety/interlocks/*.json`
- `commissioning/sessions/*.json`
- `commissioning/comparisons/*.json`
- `optimization/studies/*.json`
- `optimization/runs/*.json`
- `plugins/manifests/*.json`
- `openspec/docs/*.faerospec`
- `perception/calibrations/*.json`
- `perception/pipelines/*.json`
- `perception/datasets/*.json`
- `perception/maps/*.json`
- `validations/*.json`
- `ai/suggestions/*.json`

Donnees derivees non critiques:

- `cache/**`
- miniatures,
- maillages d'affichage temporaires,
- index de recherche reconstruisibles.

## Exemple de `project.yaml`

```yaml
projectId: prj_01JFA_PROJ_001
name: Cellule Demo
formatVersion: 0.1.0
createdAt: "2026-04-06T10:00:00Z"
updatedAt: "2026-04-06T12:30:00Z"
appVersion: 0.1.0-alpha
displayUnits:
  length: mm
  angle: deg
  mass: kg
defaultFrame: world
rootSceneId: ent_01JFA_CELL_001
activeConfigurationId: cfg_default
```

## Exemple de ligne `edges.jsonl`

```json
{"edgeId":"edg_01","from":"ent_01JFA_ASM_001","to":"ent_01JFA_OCC_001","type":"contains","createdAt":"2026-04-06T10:05:00Z"}
```

## Exemple de ligne `commands.jsonl`

```json
{"commandId":"cmd_01","kind":"part.parameter.set","projectId":"prj_01JFA_PROJ_001","targetId":"ent_01JFA_PART_001","timestamp":"2026-04-06T10:10:00Z","payload":{"parameterId":"width","value":130,"unit":"mm"}}
```

## Strategie de sauvegarde

- Ecriture atomique par fichier quand c'est possible.
- Ecriture dans un fichier temporaire puis renommage.
- Flush du journal de commande avant ecriture des vues materialisees.
- L'autosave ne doit jamais ecraser silencieusement une version saine sans snapshot precedent.

## Strategie de reprise

Au chargement:

- lire `project.yaml`,
- reconstruire l'index des noeuds,
- relire `events/*.jsonl`,
- detecter les journaux orphelins,
- reconstruire les caches si necessaire.

## Regles de compatibilite

- `formatVersion` suit semver.
- Une version mineure peut ajouter des champs facultatifs.
- Une version majeure peut exiger une migration.
- Toute migration doit produire un journal d'actions.

## Regles d'integrite

- Aucun fichier de `cache/` ne doit etre requis pour ouvrir un projet.
- Aucun chemin absolu local ne doit etre necessaire au fonctionnement du projet.
- Les assets binaires doivent etre references par hash et manifeste.
- Les informations d intention, de mapping et de revue ne doivent jamais vivre uniquement dans un binaire vendor opaque si elles pilotent le projet.
- Les donnees perception volumineuses doivent etre stockees comme assets references et non inline dans les noeuds metier.
- L'etat d'activation des plugins doit etre separable des manifests pour faciliter audit et rollback.

## Archive portable optionnelle

Le dossier `.faero` peut etre archive en `.faeropkg` pour transfert, mais l'archive n'est pas le format de travail du MVP.

## Criteres d'acceptation

- Un projet peut etre versionne dans Git sans binaire obligatoire pour le coeur metier.
- Un projet corrompu dans `cache/` reste ouvrable.
- Une simulation et une suggestion IA peuvent etre rattachees au meme graphe sans format parallele ad hoc.
- Une calibration ou un dataset LiDAR peuvent etre attaches au projet sans casser la diffabilite du coeur metier.
- Un flux Bluetooth, Wi-Fi ou telemetrique peut etre attache au projet via `integration/streams/` sans schema cache hors projet.
- Un projet peut stocker des notes OpenSpec lisibles en `*.faerospec` sans dependre d un conteneur binaire type CATIA pour comprendre l intention d ingenierie.
