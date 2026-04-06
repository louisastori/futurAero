# Backlog Dev-Ready

## Objectif

Transformer les specs en un backlog MVP directement executable par une equipe produit/plateforme.

## Regles de priorisation

- `P0`: indispensable au MVP.
- `P1`: utile juste apres le MVP.
- `P2`: optionnel ou differe.

## Definition d'une story dev-ready

Une story est consideree prete si elle possede:

- un objectif clair,
- un module responsable,
- des dependances identifiees,
- des criteres d'acceptation testables,
- une sortie observable par l'utilisateur ou par le systeme.

## Epic E0 - Fondation projet

### ST-001 - Initialiser le monorepo

- Priorite: `P0`
- Modules: `faero-app`, `faero-ui`, `faero-core`
- Dependances: aucune
- Sortie: workspace compilable avec application desktop vide

Criteres:

- le workspace installe ses dependances,
- un build debug passe,
- une fenetre desktop s'ouvre,
- la structure des modules suit le plan repo.

### ST-002 - Implementer le loader/saver `.faero`

- Priorite: `P0`
- Modules: `faero-storage`, `faero-core`
- Dependances: `ST-001`
- Sortie: ouverture et sauvegarde d'un projet vide

Criteres:

- `project.yaml` est cree,
- le dossier projet est relisible,
- un projet corrompu dans `cache/` reste ouvrable,
- les erreurs de format remontent avec code explicite.

### ST-003 - Implementer le journal `commands/events`

- Priorite: `P0`
- Modules: `faero-core`, `faero-storage`
- Dependances: `ST-002`
- Sortie: toute commande mutante genere une trace persistante

Criteres:

- une commande est append dans `commands.jsonl`,
- au moins un evenement associe est append dans `events.jsonl`,
- la replay de base fonctionne sur un projet minimal.

## Epic E1 - Graphe projet et objets coeur

### ST-101 - Implementer le registre d'entites

- Priorite: `P0`
- Module: `faero-core`
- Dependances: `ST-003`
- Sortie: CRUD sur entites persistables

Criteres:

- creation, lecture, patch et suppression fonctionnent,
- les identifiants sont stables,
- les revisions sont mises a jour,
- les references invalides sont rejetees.

### ST-102 - Implementer le registre de relations

- Priorite: `P0`
- Module: `faero-core`
- Dependances: `ST-101`
- Sortie: graphe navigable

Criteres:

- `contains`, `references`, `instantiates`, `drives` fonctionnent,
- les relations orphelines sont detectees,
- les voisins d'un noeud sont interrogeables.

### ST-103 - Implementer les vues projet pour l'UI

- Priorite: `P0`
- Modules: `faero-core`, `faero-ui`
- Dependances: `ST-102`
- Sortie: project tree et selection de base

Criteres:

- l'UI peut afficher l'arborescence,
- une selection retourne les proprietes essentielles,
- la mise a jour est reactive apres commande.

## Epic E2 - CAD minimal

### ST-201 - Esquisse 2D minimale

- Priorite: `P0`
- Modules: `faero-geometry`, `faero-ui`
- Dependances: `ST-103`
- Sortie: lignes, arcs, cercles, dimensions

Criteres:

- ajout d'elements 2D,
- ajout de contraintes simples,
- solve state expose: `under|well|over`,
- erreurs de contraintes identifiables.

### ST-202 - Feature extrusion MVP

- Priorite: `P0`
- Modules: `faero-geometry`, `faero-core`
- Dependances: `ST-201`
- Sortie: piece 3D extrudee a partir d'une esquisse

Criteres:

- une extrusion cree un `Part`,
- regeneration sur changement de dimension,
- bounds et massProperties sont calcules.

### ST-203 - Material profile et masse

- Priorite: `P0`
- Modules: `faero-geometry`, `faero-core`
- Dependances: `ST-202`
- Sortie: piece avec materiau et masse coherente

Criteres:

- affectation de materiau persistee,
- masse recalculee,
- centre de masse expose a l'UI.

## Epic E3 - Assemblage et cinematique

### ST-301 - Occurrences d'assemblage

- Priorite: `P0`
- Modules: `faero-assembly`, `faero-ui`
- Dependances: `ST-202`
- Sortie: placement de pieces dans un assemblage

Criteres:

- ajout d'occurrences,
- transformation locale persistee,
- arbre d'assemblage affichable.

### ST-302 - Mates de base

- Priorite: `P0`
- Modules: `faero-assembly`
- Dependances: `ST-301`
- Sortie: coincidence, distance, angle

Criteres:

- solve d'assemblage basique,
- etat `solved|unsolved|conflicting`,
- references geometriques invalides bloquees.

### ST-303 - Joints cinematiques MVP

- Priorite: `P0`
- Modules: `faero-assembly`, `faero-robotics`
- Dependances: `ST-302`
- Sortie: joints `fixed|revolute|prismatic`

Criteres:

- position et limites pilotables,
- etat de joint lisible,
- degres de liberte exposes.

## Epic E4 - Cellule robotique

### ST-401 - RobotCell et equipements

- Priorite: `P0`
- Modules: `faero-robotics`, `faero-core`
- Dependances: `ST-303`
- Sortie: scene robotique structuree

Criteres:

- ajout d'un robot abstrait,
- ajout d'equipements type convoyeur/poste/pince,
- zones de securite persistees.

### ST-402 - Points cibles et sequence simple

- Priorite: `P0`
- Modules: `faero-robotics`, `faero-ui`
- Dependances: `ST-401`
- Sortie: sequence pick-and-place simple

Criteres:

- points cibles versionnes,
- ordre de sequence modifiable,
- visualisation minimale des cibles.

## Epic E5 - Controle et simulation

### ST-501 - Signaux et controle minimal

- Priorite: `P0`
- Modules: `faero-sim`, `faero-robotics`
- Dependances: `ST-402`
- Sortie: signaux et machine a etats simple

Criteres:

- signaux booleens et scalaires,
- transitions a conditions explicites,
- sequence bloquee detectee.

### ST-502 - Runner de simulation MVP

- Priorite: `P0`
- Modules: `faero-sim`, `faero-core`
- Dependances: `ST-501`
- Sortie: job `simulation.run.start`

Criteres:

- lancement asynchrone,
- progression publiee,
- artefacts `summary/metrics/timeline`,
- seed et version moteur stockees.

### ST-503 - Detection de collision et rapport

- Priorite: `P0`
- Modules: `faero-sim`
- Dependances: `ST-502`
- Sortie: collisions localisees et rapport de run

Criteres:

- collisions listees avec temps et objets,
- `collision_count` calcule,
- run rejouable a config egale.

## Epic E5B - Perception avancee

### ST-551 - Rigs capteurs et modeles LiDAR

- Priorite: `P1`
- Modules: `faero-perception`, `faero-core`, `faero-ui`
- Dependances: `ST-401`
- Sortie: capteurs montes sur une cellule avec repere et profil

Criteres:

- creation d'un `SensorRig`,
- ajout d'un LiDAR 2D ou 3D,
- montage et frame persistes,
- configuration de base editable.

### ST-552 - Calibration et synchronisation

- Priorite: `P1`
- Modules: `faero-perception`
- Dependances: `ST-551`
- Sortie: profil de calibration exploitable

Criteres:

- calibration extrinseque stockee,
- metriques de qualite exposees,
- erreurs de synchronisation detectees,
- replay avec calibration stable.

### ST-553 - Pipeline perception et run

- Priorite: `P1`
- Modules: `faero-perception`, `faero-core`
- Dependances: `ST-552`, `ST-502`
- Sortie: job `perception.run.start`

Criteres:

- run asynchrone,
- scans ou datasets ecrits,
- progression publiee,
- artefacts perception persistants.

### ST-554 - Cartes et comparaison scene observee / scene nominale

- Priorite: `P1`
- Modules: `faero-perception`, `faero-ui`
- Dependances: `ST-553`
- Sortie: carte d'occupation et rapport d'ecarts

Criteres:

- generation d'une carte ou d'un nuage de points,
- comparaison a la scene numerique,
- ecarts quantifies,
- obstacle inconnu remonte.

## Epic E5C - Integration industrielle et connectivite

### ST-561 - Endpoints ROS2 et OPC UA

- Priorite: `P1`
- Modules: `faero-integration`, `faero-core`, `faero-ui`
- Dependances: `ST-103`
- Sortie: endpoints industriels declarables dans le projet

Criteres:

- endpoint ROS2 declare,
- endpoint OPC UA declare,
- etat de connexion visible,
- mode replay supporte.

### ST-562 - PLC, robots et traces industrielles

- Priorite: `P1`
- Modules: `faero-integration`
- Dependances: `ST-561`, `ST-401`
- Sortie: bindings PLC/robot et import de traces

Criteres:

- tags ou variables mappables,
- binding robot explicite,
- trace importable et rejouable,
- erreurs de mapping detectees.

### ST-563 - Bluetooth, Wi-Fi et streams telemetriques

- Priorite: `P1`
- Modules: `faero-integration`, `faero-core`, `faero-ui`
- Dependances: `ST-561`
- Sortie: endpoints sans fil et streams telemetriques declarables

Criteres:

- endpoint Bluetooth LE, Bluetooth Classic ou Wi-Fi declare,
- discovery et pairing visibles,
- stream MQTT, WebSocket, TCP/UDP ou serial mappable,
- etat de liaison expose a l'UI.

### ST-564 - Replay et degradation de liaison

- Priorite: `P1`
- Modules: `faero-integration`, `faero-testkit`
- Dependances: `ST-563`
- Sortie: simulation et replay de liens radio ou reseau

Criteres:

- trace sans fil ou reseau importable,
- pertes, jitter et deconnexions injectables,
- diagnostic de liaison explicite,
- replay deterministe sur fixture officielle.

## Epic E5D - Safety

### ST-571 - Zones et interlocks

- Priorite: `P1`
- Modules: `faero-safety`, `faero-ui`
- Dependances: `ST-401`
- Sortie: graphe safety inspectable

Criteres:

- creation de zones,
- creation d'interlocks,
- causes de blocage lisibles,
- validation de coherence.

### ST-572 - LiDAR securite et validation safety

- Priorite: `P1`
- Modules: `faero-safety`, `faero-perception`
- Dependances: `ST-571`, `ST-551`
- Sortie: safety validation run

Criteres:

- LiDAR securite configure,
- slowdown/stop modelises,
- run safety rejouable,
- rapport safety genere.

## Epic E5E - Commissioning et as-built

### ST-581 - Session commissioning

- Priorite: `P1`
- Modules: `faero-commissioning`, `faero-ui`
- Dependances: `ST-562`, `ST-553`
- Sortie: session de mise en service structuree

Criteres:

- session ouverte,
- captures terrain attachees,
- statut de progression visible,
- journal d'ajustements actif.

### ST-582 - Comparaison as-built vs as-designed

- Priorite: `P1`
- Modules: `faero-commissioning`, `faero-perception`
- Dependances: `ST-581`
- Sortie: rapport d'ecarts terrain vs nominal

Criteres:

- comparaison geometrique ou logique,
- ecarts quantifies,
- tolerances appliquees,
- rapport rejouable.

## Epic E5F - Optimisation

### ST-591 - Etudes multi-objectifs

- Priorite: `P1`
- Modules: `faero-optimization`, `faero-core`
- Dependances: `ST-503`, `ST-554`, `ST-572`
- Sortie: definition d'etude et variables de decision

Criteres:

- objectifs visibles,
- contraintes visibles,
- variables modifiables,
- etude persistable.

### ST-592 - Runs optimisation et recommandations

- Priorite: `P1`
- Modules: `faero-optimization`, `faero-ui`
- Dependances: `ST-591`
- Sortie: candidats classes et resultats applicables

Criteres:

- run asynchrone,
- plusieurs candidats classes,
- front de Pareto ou equivalent,
- application via commandes normales.

## Epic E6 - IA locale

### ST-601 - Session IA et retrieval local

- Priorite: `P0`
- Modules: `faero-ai`, `faero-core`
- Dependances: `ST-103`, `ST-502`
- Sortie: session IA lisant le graphe projet

Criteres:

- contexte construit depuis graphe + runs,
- aucune dependance internet,
- journal IA persiste.

### ST-602 - Reponse structuree mode `explain`

- Priorite: `P0`
- Modules: `faero-ai`
- Dependances: `ST-601`
- Sortie: explication d'une collision ou d'un blocage

Criteres:

- reponse JSON parseable,
- `contextRefs` obligatoires,
- `confidence` et `riskLevel` presents.

### ST-603 - Suggestions applicables

- Priorite: `P1`
- Modules: `faero-ai`, `faero-core`, `faero-ui`
- Dependances: `ST-602`
- Sortie: suggestion avec `proposedCommands`

Criteres:

- visualisation avant application,
- application via pipeline normal de commandes,
- suggestion rejetable sans effet de bord.

### ST-604 - Profils IA `max` et `furnace`

- Priorite: `P1`
- Modules: `faero-ai`, `faero-core`
- Dependances: `ST-602`
- Sortie: profils runtime IA haute performance

Criteres:

- choix explicite du profil runtime,
- journalisation du profil actif,
- bascule sans casser les schemas de sortie,
- degradation propre si la machine n'a pas les ressources.

### ST-605 - Critique interne multi-passes

- Priorite: `P1`
- Modules: `faero-ai`
- Dependances: `ST-604`
- Sortie: double passe reasoner + critic

Criteres:

- premiere passe de generation,
- seconde passe de critique,
- trace des contradictions detectees,
- sortie finale toujours structuree.

### ST-606 - Discussion IA locale dans le shell desktop

- Priorite: `P1`
- Modules: `faero-ai`, `faero-app`, `faero-ui`
- Dependances: `ST-601`, `ST-704`, `ST-705`
- Sortie: panneau de discussion IA locale disponible dans le shell desktop

Criteres:

- le shell desktop expose un panneau `Assistant IA local`,
- la discussion reste locale et ne depend d aucun service cloud,
- le backend Tauri injecte le contexte du projet courant dans la demande,
- le shell affiche explicitement le runtime, le modele actif ou le fallback local.

## Epic E8 - Plugin SDK

### ST-801 - Host plugins et manifests

- Priorite: `P1`
- Modules: `faero-plugin-host`, `faero-core`
- Dependances: `ST-003`
- Sortie: installation et activation de plugins

Criteres:

- manifest valide,
- plugin installable,
- plugin activable/desactivable,
- etat plugin auditables.

### ST-802 - Permissions et contributions

- Priorite: `P1`
- Modules: `faero-plugin-host`, `faero-ui`
- Dependances: `ST-801`
- Sortie: capabilities chargees avec permissions explicites

Criteres:

- permissions visibles,
- contribution UI ou outil chargeable,
- plugin incompatible bloque proprement,
- aucune ecriture directe hors commandes coeur.

## Epic E9 - Qualite et couverture

### ST-901 - Coverage gate 100 pourcent

- Priorite: `P1`
- Modules: repo entier
- Dependances: `ST-001`
- Sortie: politique CI bloquante de couverture 100 pourcent

Criteres:

- line, branch et function coverage a 100 pourcent sur scope defini,
- exclusions explicitement versionnees,
- CI bloquante sous le seuil,
- rapport de couverture publie.

## Epic E7 - UI workspace

### ST-701 - Layout principal

- Priorite: `P0`
- Modules: `faero-ui`
- Dependances: `ST-001`
- Sortie: layout tree + viewport + inspector + jobs

Criteres:

- responsive desktop,
- persistance du layout,
- navigation clavier minimale.

### ST-702 - Inspecteur de proprietes

- Priorite: `P0`
- Modules: `faero-ui`
- Dependances: `ST-103`
- Sortie: edition de proprietes de base

Criteres:

- edition de nom, tags, parametres,
- validation de saisie,
- erreurs visibles sans crash.

### ST-703 - Timeline de simulation

- Priorite: `P1`
- Modules: `faero-ui`, `faero-sim`
- Dependances: `ST-502`
- Sortie: lecture des evenements de run

Criteres:

- affichage des collisions,
- affichage des changements de signaux,
- saut a un instant critique.

### ST-704 - Barre de menus style Visual Studio

- Priorite: `P1`
- Modules: `faero-ui`, `faero-app`
- Dependances: `ST-701`, `ST-702`
- Sortie: menus desktop proches de Visual Studio

Criteres:

- menus `File/Edit/View/Git/Project/Build/Debug/Test/Analyze/Tools/Window/Help`,
- chaque item relie a une commande interne,
- panneaux principaux accessibles depuis `View`,
- aucune action mutante ne contourne le pipeline de commandes.

### ST-705 - Workspace desktop pilote par snapshots `.faero`

- Priorite: `P1`
- Modules: `faero-ui`, `faero-app`, `faero-storage`
- Dependances: `ST-701`, `ST-704`
- Sortie: arbre projet, proprietes et sortie relies aux fixtures backend

Criteres:

- le shell charge un snapshot projet depuis le backend Tauri,
- l explorateur de projet affiche entites, endpoints, flux et plugins reels,
- le panneau proprietes expose les metadonnees utiles du projet charge,
- le panneau sortie affiche une activite recente commande/evenement issue de la fixture.

### ST-706 - Routage interactif des commandes desktop

- Priorite: `P1`
- Modules: `faero-ui`, `faero-app`, `faero-core`
- Dependances: `ST-704`, `ST-705`
- Sortie: commandes du shell executees depuis la surface centrale avec retour d etat

Criteres:

- une session workspace en memoire existe cote backend desktop,
- le shell peut executer un sous-ensemble de commandes depuis les menus,
- les mutations supportees passent par `faero-core`,
- la sortie desktop affiche le dernier resultat et l activite systeme associee.

## Ordre de livraison recommande

1. `E0`
2. `E1`
3. `E2`
4. `E3`
5. `E4`
6. `E5`
7. `E5B`
8. `E5C`
9. `E5D`
10. `E5E`
11. `E5F`
12. `E7`
13. `E6`
14. `E8`
15. `E9`

## MVP strict

Le MVP strict correspond au minimum:

- `ST-001` a `ST-003`
- `ST-101` a `ST-103`
- `ST-201` a `ST-203`
- `ST-301` a `ST-303`
- `ST-401` a `ST-402`
- `ST-501` a `ST-503`
- `ST-701` et `ST-702`
- `ST-601` et `ST-602`

## Definition de pre-demarrage implementation

Le projet est considere pret a coder si:

- les stories `P0` ci-dessus sont validees au niveau spec,
- les schemas de commandes sont figes a 80 pourcent,
- la structure du repo est acceptee,
- les conventions de tests sont posees.
