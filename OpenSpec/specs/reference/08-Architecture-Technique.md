# Architecture Technique

## Objectif

Traduire l'architecture produit en une architecture d'implementation de reference pour le MVP.

## Cible technique MVP

- Shell desktop: Tauri
- UI: React + TypeScript
- Viewport 3D MVP: Three.js
- Coeur metier: Rust
- Persistance locale: systeme de fichiers + SQLite pour index locaux optionnels
- Maths: nalgebra
- Simulation rigide MVP: Rapier
- Perception et capteurs avances: crate dediee `faero-perception`
- Inference IA locale: runtime abstrait compatible `llama.cpp` et `ONNX Runtime`
- Orchestration IA haut de gamme: multi-modeles, multi-processus, multi-GPU si disponible
- Integration industrielle: adaptateurs ROS2, OPC UA, PLC, robots, Bluetooth/BLE, Wi-Fi, MQTT, WebSocket, TCP/UDP et serial via crates dediees
- Plugins: host local avec isolation de permissions

## Pourquoi cette cible

- Tauri limite la charge du shell desktop et garde un backend Rust local.
- React accelere la construction de panneaux et d'outils non critiques.
- Three.js suffit pour la visualisation MVP, tant qu'il n'est pas la source de verite geometrique.
- Rust fournit de bonnes garanties pour le coeur, les jobs et la perf.

## Decision structurante

Le rendu 3D n'est pas le modele geometrique. Le viewport consomme des representations derivees du graphe projet; la verite reste dans le coeur metier et ses objets.

## Modules applicatifs

### `faero-app`

Responsabilites:

- cycle de vie desktop,
- menus,
- fenetres,
- preferences utilisateur,
- ouverture et sauvegarde de projet,
- pont UI <-> backend.

### `faero-ui`

Responsabilites:

- arborescence projet,
- inspecteurs,
- timeline de simulation,
- panneau IA,
- commandes utilisateur,
- viewport host.

### `faero-core`

Responsabilites:

- graphe projet,
- validation des commandes,
- orchestration des moteurs,
- undo/redo,
- journaux commande/evenement.

### `faero-geometry`

Responsabilites:

- esquisses,
- solveur de contraintes 2D,
- features parametriques,
- calcul de volumes et masses,
- export de maillages d'affichage.

Note:

- le noyau geometrique final peut passer par un adaptateur FFI,
- le contrat interne doit rester stable meme si l'implementation change.

### `faero-assembly`

Responsabilites:

- occurrences,
- mates,
- joints,
- evaluation des degres de liberte,
- propagation des transformations.

### `faero-robotics`

Responsabilites:

- chaines cinematiques,
- modeles de robots,
- enveloppes de travail,
- sequences de cellule,
- points cibles et trajectoires simples.

### `faero-sim`

Responsabilites:

- simulation temporelle,
- collisions,
- dynamique rigide,
- capteurs,
- execution logique de commande,
- journal de resultats.

### `faero-perception`

Responsabilites:

- modeles LiDAR/camera/IMU,
- calibration intra/extrinseque et temporelle,
- replay de donnees capteurs,
- fusion capteurs,
- reconstruction nuage de points et cartes d'occupation,
- comparaison scene numerique / observation.

### `faero-integration`

Responsabilites:

- endpoints ROS2, OPC UA, PLC, robots, Bluetooth/BLE, Wi-Fi et transports telemetriques,
- mapping signaux/variables/topics,
- discovery, pairing et securisation des endpoints quand applicable,
- flux MQTT, WebSocket, TCP/UDP et serial mappables,
- replay de traces industrielles,
- simulation de lien degrade et journal de qualite,
- adaptation protocoles <-> graphe projet.

### `faero-safety`

Responsabilites:

- zones,
- interlocks,
- permissifs,
- inhibitions,
- LiDAR securite,
- validation de surete rejouable.

### `faero-commissioning`

Responsabilites:

- sessions de mise en service,
- captures terrain,
- comparaison nominal / observe,
- journal d'ajustements et sign-off.

### `faero-optimization`

Responsabilites:

- etudes multi-objectifs,
- evaluation de candidats,
- contraintes,
- pareto front,
- rapports de recommandations.

### `faero-ai`

Responsabilites:

- indexation du projet,
- retrieval local,
- orchestration de l'inference,
- generation de suggestions structurees,
- journalisation IA.

Sous-modules recommandes:

- `reasoner`: raisonnement long et planification
- `coder`: generation et transformation structuree
- `vision`: interpretation de rendu, cartes et perception
- `embedder`: indexation dense locale
- `reranker`: selection de contexte hautement pertinente
- `compressor`: condensation de contexte pour fenetres longues
- `scheduler`: allocation des ressources GPU/CPU/RAM

### `faero-plugin-host`

Responsabilites:

- chargement de plugins,
- negotiation de permissions,
- isolation des extensions,
- contribution a l'UI et aux outils,
- compatibilite de version.

### `faero-storage`

Responsabilites:

- lecture/ecriture `.faero`,
- verifications d'integrite,
- migrations de format,
- gestion d'assets et de cache.

## Topologie de processus

MVP:

- un processus desktop principal,
- un runtime Rust pour les commandes et jobs,
- des workers de fond pour simulation, perception, regen et IA,
- des workers dedies pour integration, commissioning, optimisation et safety validation,
- possibilite d'un sidecar IA si le runtime local l'exige.

Mode IA haut de gamme recommande:

- un worker par famille de modele,
- tensors ou couches distribues sur plusieurs GPU si presents,
- cache KV et index sur NVMe local,
- pipelines distincts pour generation, embeddings et vision.

## Flux de donnees

1. L'UI emet une commande.
2. `faero-core` valide la commande contre le graphe.
3. Le module domaine concerne execute.
4. Le resultat produit des evenements.
5. `faero-storage` persiste commande, evenement et vues materialisees.
6. L'UI recupere l'etat mis a jour.

## Regles d'execution

- Toute mutation passe par une commande.
- Les calculs longs s'executent en job asynchrone.
- Un job ne modifie pas directement l'UI.
- Les erreurs remontent avec code, message, cible et contexte.

## Regles de concurrence

- Une seule commande mutante appliquee a la fois par projet actif.
- Les lectures sont concurrentes si elles ne lisent pas d'etat partiellement regenere.
- Les jobs publient des snapshots de progression et non des etats non valides.

## API interne entre UI et coeur

Transport MVP:

- invocation Tauri command pour les commandes courtes,
- canal evenementiel pour notifications,
- fichiers/SQLite pour certains index reconstruisibles.

Objets exposes a l'UI:

- vue projet,
- vue selection,
- vue scene,
- vue job,
- vue suggestion IA.

## Observabilite locale

- journal applicatif local,
- trace de commande,
- trace de job,
- trace IA redigee pour lecture humaine,
- trace de qualite de liaison pour les endpoints externes,
- option de niveau de log par module.

## Gestion des erreurs

Codes minimaux:

- `E_VALIDATION`
- `E_NOT_FOUND`
- `E_CONFLICT`
- `E_UNSUPPORTED`
- `E_SIMULATION`
- `E_INTEGRATION`
- `E_SAFETY`
- `E_COMMISSIONING`
- `E_OPTIMIZATION`
- `E_CALIBRATION`
- `E_PERCEPTION`
- `E_AI_RUNTIME`
- `E_PLUGIN`
- `E_STORAGE_IO`
- `E_STORAGE_FORMAT`

## Frontiere des plugins

Les outils futurs n'accedent pas directement au stockage ni aux fichiers du projet. Ils parlent au coeur via les contrats internes.

## Criteres d'acceptation

- L'UI peut rester reactive pendant une simulation ou une regeneration.
- Le coeur peut etre teste sans demarrer l'application desktop.
- Le remplacement d'un moteur interne ne casse pas le format projet ni les commandes publiques.
- Un pipeline perception peut etre remplace sans changer le graphe projet ni les envelopes de commandes.
