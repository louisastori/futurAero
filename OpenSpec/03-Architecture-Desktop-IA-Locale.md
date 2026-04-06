# Architecture Desktop IA Locale

## Vue d'ensemble

L'architecture cible est celle d'une application desktop modulaire avec coeur natif pour les calculs et couche d'interface separee. Le systeme doit privilegier la robustesse locale, la performance et la traceabilite.

## Blocs majeurs

- App Shell: gestion de fenetres, sessions, preferences, projets recents, securite locale.
- UI Workspace: editeurs 2D/3D, arborescence projet, panneau proprietes, console de simulation, panneau IA.
- Project Graph: representation unifiee des objets metier et de leurs relations.
- Geometry Engine: esquisses, contraintes, operations solides, repere, unites, topologie.
- Assembly Engine: mates, liaisons, arborescence d'assemblage, propagation des contraintes.
- Robotics Engine: chaine cinematique, actionneurs, zones de travail, sequences.
- Simulation Engine: dynamique, collisions, capteurs, temps, journal des resultats.
- Perception Engine: acquisition simulee, calibration, fusion capteurs, reconstruction et comparaison scene-realite.
- Integration Engine: ROS2, OPC UA, PLC, controleurs robots, Bluetooth/BLE, Wi-Fi, MQTT, WebSocket, TCP/UDP, serial, replay de traces et mapping signaux.
- Safety Engine: zones, interlocks, permissifs, inhibitions et etats de surete.
- Commissioning Engine: workflow nominal -> terrain -> comparaison -> ajustement -> validation.
- Optimization Engine: exploration multi-objectifs sous contraintes.
- Plugin Host: chargement, isolation et permissions des extensions.
- Local AI Runtime: inference locale, recherche dans le projet, planification d'actions, explication.
- AI Orchestrator: routage entre modeles de raisonnement, vision, code, retrieval et compression de contexte.
- Persistence Layer: format projet, cache, journal d'evenements, assets.
- Tool SDK: contrat commun pour ajouter des outils sans casser le graphe central.

## Principes d'architecture

- Un seul graphe source de verite pour tout le projet.
- Les outils lisent et ecrivent via des contrats stables.
- Les traitements lourds doivent etre executables en tache de fond.
- Les resultats doivent etre serialisables avec provenance.
- Les couches UI et calcul doivent rester decouplees.
- Les capteurs et sorties perception partagent la meme base de temps que la simulation.
- Les integrations externes sont adaptees par contrats explicites et jamais branchees directement au coeur.
- Les mecanismes de discovery, pairing, securite et qualite de liaison sont des donnees inspectables et rejouables.

## IA locale

Le runtime IA local doit etre concu comme un systeme d'assistance et non comme un moteur opaque de modifications directes.

### Capacites attendues

- Comprendre la structure d'un projet et ses objets.
- Resumer une scene, une contrainte ou un conflit.
- Proposer des corrections de parametres ou d'agencement.
- Generer des notes techniques, checklists et rapports.
- Traduire une intention utilisateur en suite d'actions proposees.

### Regles de securite et explicabilite

- Toute action candidate est decrite avant execution.
- Chaque recommandation cite les objets ou mesures utilises.
- Le systeme garde l'historique des prompts, des reponses et des applications.
- Les modeles locaux sont remplacables sans casser le format projet.
- Le systeme peut basculer vers un mode performance maximale utilisant agressivement GPU, CPU, RAM et NVMe local.

## Flux principal

1. L'utilisateur cree ou modifie un objet.
2. Le Project Graph enregistre l'etat et les dependances.
3. Les moteurs geometrique, cinematique ou simulation recalculent le minimum necessaire.
4. Le runtime IA peut lire le contexte a partir du graphe et des journaux.
5. Le moteur de perception peut produire cartes, detections et ecarts scene-realite.
6. Les moteurs integration, safety, commissioning et optimisation peuvent enrichir l'etat projet.
7. Les propositions IA sont presentees comme des changements inspectables.
8. Les resultats sont stockes avec provenance.

## Decisions structurantes a respecter

- Les unites, reperes et conventions temporelles sont centralises.
- Les resultats de simulation ne sont jamais stockes sans leur configuration.
- Les sorties capteurs et cartes derivees ne sont jamais stockees sans calibration ni reference temporelle.
- Les modules ne communiquent pas par etat cache implicite.
- L'API interne privilegie les commandes explicites et evenements nommes.
- Les plugins et connecteurs n'ecrivent jamais directement dans le stockage projet.
- Les etats de liaison filaires et sans fil ne sont jamais deduits implicitement par l'UI; ils viennent de traces ou evenements explicites.

## Criteres d'acceptation architecture

- Un outil peut etre ajoute sans redefinition des objets coeur.
- Une action IA peut etre annulee comme toute autre commande.
- Une scene identique produit le meme resultat deterministe dans le meme mode.
- Le projet peut etre ouvert sans lancer automatiquement l'inference IA.
