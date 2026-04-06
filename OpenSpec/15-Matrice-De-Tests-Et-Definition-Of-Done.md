# Matrice De Tests Et Definition Of Done

## Objectif

Definir les tests minimaux et la definition of done pour que le MVP reste fiable, rejouable et explicable.

## Role de GitHub pour les tests

GitHub sert de point de controle officiel pour:

- executer les suites de tests de reference via GitHub Actions,
- exposer l'etat du pipeline sur chaque PR,
- bloquer le merge si un test critique, un lint ou le coverage gate echoue,
- conserver un historique visible des runs de validation.

## Niveaux de tests

### T1 - Tests unitaires

Ciblent:

- types,
- validateurs,
- solveurs locaux,
- calculs de masse,
- conversions d'unites,
- parseurs de payloads.

### T2 - Tests d'integration

Ciblent:

- flux commande -> evenement,
- lecture/ecriture `.faero`,
- regen d'une piece,
- solve d'assemblage,
- creation d'un scenario.

### T3 - Tests scenario

Ciblent:

- cellule pick-and-place complete,
- simulation d'un run,
- replay perception LiDAR,
- replay telemetrie sans fil,
- collisions,
- logique de sequence,
- IA `explain`.

### T4 - Tests non fonctionnels

Ciblent:

- performance,
- determinisme,
- stabilite de calibration,
- pertes et reconnexions de liens,
- reprise apres crash,
- absence de dependance internet,
- resistance a projet partiellement corrompu.

## Matrice minimale par module

### `faero-types`

- parse des envelopes,
- validation des ids,
- compatibilite JSON round-trip.

### `faero-core`

- creation/suppression entites,
- revisioning,
- journal commande/evenement,
- undo/redo.

### `faero-storage`

- sauvegarde projet vide,
- reouverture projet,
- migration de version mineure,
- projet avec `cache/` manquant.

### `faero-geometry`

- esquisse sous/sur/juste contrainte,
- extrusion simple,
- regeneration apres changement de parametre,
- calcul masse bornes.

### `faero-assembly`

- ajout occurrence,
- mate de coincidence,
- joint revolute avec limites,
- etat `solved|conflicting`.

### `faero-robotics`

- creation RobotCell,
- ajout d'equipements,
- ajout de points cibles,
- sequence simple valide/invalide.

### `faero-sim`

- lancement de run,
- collisions detectees,
- timeline ecrite,
- run deterministe a seed egale.

### `faero-perception`

- creation de rig capteurs,
- calibration valide/invalide,
- replay dataset,
- generation carte d'occupation,
- comparaison scene observee / scene nominale.

### `faero-integration`

- endpoints ROS2, OPC UA, Bluetooth, Wi-Fi, MQTT, WebSocket, TCP/UDP et transports telemetriques,
- bindings PLC, robots et streams,
- replay de traces,
- simulation de lien degrade,
- mapping signaux.

### `faero-safety`

- zones,
- interlocks,
- LiDAR securite,
- validation safety.

### `faero-commissioning`

- creation de session,
- captures terrain,
- comparaison as-built,
- sign-off.

### `faero-optimization`

- etudes,
- runs,
- contraintes,
- classement de candidats.

### `faero-plugin-host`

- validation manifest,
- permissions,
- enable/disable,
- isolation.

### `faero-ai`

- contexte construit a partir du graphe,
- reponse JSON parseable,
- rejet des sorties non conformes,
- persistance session/suggestion,
- profils runtime `max|furnace`,
- critique interne multi-passes,
- discussion shell locale via runtime Ollama ou fallback local explicite.

### `faero-ui`

- rendu project tree,
- inspecteur de proprietes,
- affichage jobs,
- affichage suggestion IA,
- structure de menus desktop,
- localisation FR par defaut avec EN/ES disponibles,
- chargement des fixtures desktop via backend,
- panneaux retractables et rouvrables sans reset de session,
- colonnes laterales redimensionnables a la souris avec bornes stables,
- panneau de discussion IA locale avec etats vide, charge, reponse et fallback.

## Fixtures officielles MVP

### FX-001 - Projet vide

Usage:

- ouverture/sauvegarde,
- smoke tests UI.

### FX-002 - Piece parametrique simple

Usage:

- regeneration,
- masse,
- persistance.

### FX-003 - Assemblage charniere

Usage:

- mates,
- joint revolute,
- degres de liberte.

### FX-004 - Cellule pick-and-place

Usage:

- robot abstrait,
- convoyeur,
- capteur presence,
- sequence,
- scenario S1.

### FX-005 - Collision volontaire

Usage:

- validation de detection collision,
- IA mode `explain`.

### FX-006 - Cellule scannee au LiDAR

Usage:

- replay perception,
- calibration,
- comparaison au modele,
- obstacle inconnu.

### FX-007 - Cellule safety interlocks

Usage:

- zones,
- permissifs,
- LiDAR securite,
- blocage de mouvement.

### FX-008 - Session commissioning mixte

Usage:

- traces integration filaires et sans fil,
- capture perception,
- rapport as-built.

### FX-009 - Etude optimisation multi-objectifs

Usage:

- candidats,
- contraintes,
- recommandations.

### FX-010 - Plugin sandbox

Usage:

- installation,
- permissions,
- blocage plugin incompatible.

### FX-011 - Telemetrie sans fil degradee

Usage:

- endpoint Bluetooth ou Wi-Fi,
- stream MQTT ou UDP,
- pertes de paquets,
- reconnexion,
- diagnostic de liaison.

## Cas de tests critiques

### CT-001 - Replay de commandes

Etapes:

1. creer un projet,
2. creer une piece,
3. changer un parametre,
4. sauvegarder,
5. reconstruire depuis `commands.jsonl`.

Attendu:

- meme etat final,
- memes ids d'entites,
- meme revision finale ou revision equivalente selon politique.

### CT-002 - Determinisme simulation

Etapes:

1. charger `FX-004`,
2. lancer deux runs avec meme seed et meme version moteur.

Attendu:

- meme `collision_count`,
- meme `cycle_time_ms`,
- ordre stable des anomalies critiques.

### CT-003 - Rejet d'une suggestion IA invalide

Etapes:

1. injecter une sortie IA sans `contextRefs`,
2. tenter creation de suggestion.

Attendu:

- suggestion rejetee,
- erreur `E_AI_UNPARSABLE_RESPONSE` ou `E_AI_UNSAFE_ACTION`,
- aucun effet sur le graphe projet.

### CT-004 - Projet ouvrable avec cache absent

Etapes:

1. sauvegarder un projet,
2. supprimer `cache/`,
3. reouvrir.

Attendu:

- projet ouvrable,
- cache reconstruit ou regenere a la demande.

### CT-005 - Replay perception LiDAR

Etapes:

1. charger `FX-006`,
2. lancer deux runs perception en mode replay,
3. comparer les cartes et metriques.

Attendu:

- meme `lidar_coverage_ratio`,
- meme `mapping_deviation_mm` a tolerance definie,
- memes zones d'ecart critiques.

### CT-006 - IA mode `furnace`

Etapes:

1. activer le profil `furnace`,
2. lancer une demande `ai.deep_explain.request`,
3. verifier la passe critic.

Attendu:

- la session journalise plusieurs passes,
- la sortie reste parseable,
- les refs critiques sont conservees,
- aucun contournement du pipeline de commandes.

### CT-007 - Replay integration industrielle

Etapes:

1. charger une trace industrielle,
2. rejouer endpoint et mappings,
3. comparer les etats projet.

Attendu:

- replay stable,
- mappings resolus,
- erreurs de binding explicites si presentes.

### CT-008 - Safety interlock et LiDAR securite

Etapes:

1. charger `FX-007`,
2. activer une zone LiDAR stop,
3. verifier l'inhibition.

Attendu:

- transition safety horodatee,
- cause de blocage explicite,
- action inhibee comme prevu.

### CT-009 - Commissioning as-built

Etapes:

1. ouvrir `FX-008`,
2. importer capture terrain,
3. lancer comparaison as-built.

Attendu:

- rapport d'ecarts genere,
- tolerances appliquees,
- session rejouable.

### CT-010 - Optimisation multi-objectifs

Etapes:

1. charger `FX-009`,
2. lancer optimisation,
3. verifier le classement des candidats.

Attendu:

- contraintes appliquees,
- plusieurs candidats classes,
- meilleur candidat journalise.

### CT-011 - Plugin permissions

Etapes:

1. charger `FX-010`,
2. tenter installation plugin,
3. activer puis desactiver.

Attendu:

- permissions visibles,
- plugin incompatible bloque proprement,
- aucune mutation coeur hors commandes.

### CT-012 - Flux Bluetooth et Wi-Fi degrades

Etapes:

1. charger `FX-011`,
2. connecter l'endpoint en mode live ou replay,
3. injecter pertes, jitter et reconnexion.

Attendu:

- les transitions de lien sont horodatees,
- les streams mappes restent explicites,
- le diagnostic de degradation est rejouable,
- aucune ecriture implicite n'est emise pendant une deconnexion.

### CT-013 - Menus style Visual Studio

Etapes:

1. charger le menu model desktop,
2. verifier l'ordre des menus top-level,
3. verifier que chaque item actionnable expose une commande,
4. verifier la localisation FR par defaut puis EN/ES en secondaire.

Attendu:

- ordre stable des menus,
- aucune commande vide,
- panneaux principaux accessibles depuis `View`,
- la version francaise est la valeur par defaut du shell.

### CT-014 - Snapshot desktop de fixture

Etapes:

1. charger `FX-004` dans le shell desktop,
2. recuperer le snapshot backend correspondant,
3. verifier l affichage de l arbre projet, des endpoints, des flux et des plugins,
4. verifier l affichage de l activite recente.

Attendu:

- les compteurs du shell refletent le contenu `.faero`,
- l explorateur desktop n utilise pas de liste statique,
- les derniers `commands/events` de la fixture sont visibles dans la sortie,
- le changement de fixture recharge un snapshot coherent.

### CT-015 - Execution de commandes dans le shell desktop

Etapes:

1. ouvrir une session desktop sur une fixture ou un projet vide,
2. executer `entity.create.part`, `entity.create.external_endpoint` et `plugin.manage`,
3. verifier la mise a jour du snapshot et de la sortie desktop,
4. executer une commande non mutante comme `simulation.run.start`.

Attendu:

- les commandes mutantes modifient le graphe via `faero-core`,
- les compteurs entites/endpoints/flux/plugins evoluent dans le shell,
- le dernier resultat de commande est visible cote UI,
- une activite `system` est ajoutee pour les commandes simulees ou non mutantes.

### CT-016 - Discussion IA locale dans le shell desktop

Etapes:

1. ouvrir le shell desktop sur une fixture,
2. verifier l etat du runtime IA local,
3. envoyer une question sur le projet courant,
4. verifier la reponse, les references et le comportement en fallback si Ollama est indisponible.

Attendu:

- la discussion reste locale,
- le contexte du projet courant est visible dans la reponse ou ses references,
- le modele actif ou le fallback local est explicite,
- aucune mutation du projet n est appliquee silencieusement.

### CT-017 - Repli et reouverture des panneaux workspace

Etapes:

1. ouvrir le shell desktop sur une fixture,
2. replier `Explorateur de projet`, `Surface de commandes` et `Sortie`,
3. verifier que le layout reste utilisable,
4. rouvrir les panneaux depuis l entete du panneau ou via `View`.

Attendu:

- le corps des panneaux se replie sans crash,
- les colonnes laterales peuvent rester en mode compact,
- la session chargee, les compteurs et le contexte courant sont conserves,
- la reouverture remet le contenu precedent sans rechargement force.

### CT-018 - Redimensionnement des colonnes laterales

Etapes:

1. ouvrir le shell desktop sur une fixture,
2. faire glisser la poignee entre la colonne gauche et la zone centrale,
3. faire glisser la poignee entre la zone centrale et la colonne droite,
4. verifier le comportement apres repli puis reouverture d un panneau lateral.

Attendu:

- les colonnes laterales changent bien de largeur a la souris,
- des bornes minimales et maximales evitent de casser la zone centrale,
- le viewport et la surface de commandes restent utilisables pendant le resize,
- la largeur precedente est conservee quand on rouvre une colonne repliee.

## Seuils techniques MVP

- ouverture projet vide: < 2 s sur machine dev cible
- sauvegarde projet vide: < 1 s
- lancement d'un run simple: < 3 s avant premiere progression visible
- freeze UI perceptible: aucun blocage > 100 ms sur interaction courante hors job lourd

## Couverture 100 pourcent

La politique detaillee est definie dans [23-Politique-De-Tests-Et-Couverture-100.md](./23-Politique-De-Tests-Et-Couverture-100.md).

Exigence produit cible:

- 100 pourcent line coverage
- 100 pourcent region coverage
- 100 pourcent function coverage
- 100 pourcent command/event/schema coverage

Gate GitHub actuellement imposee sur le workspace Rust:

- seuils lus depuis `config/coverage-gate.json`
- 99.5 pourcent line coverage minimum
- 97.5 pourcent region coverage minimum
- 100 pourcent function coverage minimum

## Definition Of Done globale

Une story est done si:

- le code compile,
- les tests associes existent et passent,
- les erreurs previsibles sont gerees,
- les logs utiles sont presents,
- les formats persistants sont respectes,
- aucune mutation contourne le pipeline de commandes,
- la story est raccord avec les OpenSpec.

## Definition Of Done pour une commande

- schema de payload defini,
- validation d'entree ecrite,
- au moins un test succes,
- au moins un test erreur,
- evenement(s) de sortie documente(s),
- persistance verifiee si mutation.

## Definition Of Done pour une fonctionnalite UI

- etat charge/vide/erreur gere,
- navigation clavier minimale,
- texte d'erreur comprehensible,
- aucune hypothese sur succes silencieux backend,
- si une barre de menus existe, chaque item actionnable a un `command id` traceable.

## Definition Of Done pour IA locale

- contexte journalise,
- sortie parsee,
- refs citees,
- niveau de confiance present,
- suggestion non appliquee sans confirmation.
- si mode `furnace`, le profil actif et les passes de critique sont journalises.

## Definition Of Done pour simulation

- seed stockee,
- version moteur stockee,
- artefacts ecrits,
- anomalies critiques remontees,
- resultat rejouable sur fixture officielle.

## Definition Of Done pour perception

- calibration ou hypothese de calibration stockee,
- repere et horodatage presents,
- artefacts perception ecrits,
- metriques de qualite exposees,
- comparaison au nominal ou justification d'absence.

## Definition Of Done pour integration industrielle

- endpoint et mappings journalises,
- mode replay teste,
- pairing, discovery, securite et qualite de liaison visibles si applicables,
- mode degrade teste pour les transports qui le supportent,
- erreurs de liaison explicites,
- aucune ecriture externe implicite.

## Definition Of Done pour safety

- zones et interlocks visualisables,
- causes d'inhibition inspectables,
- scenario de validation safety teste,
- traces safety horodatees.

## Definition Of Done pour commissioning

- session versionnee,
- captures rattachees,
- rapport as-built genere,
- ecarts et ajustements journalises.

## Definition Of Done pour optimisation

- objectifs et contraintes persistants,
- run asynchrone trace,
- candidats classes,
- resultat applicable seulement via commandes.

## Definition Of Done pour plugins

- manifest valide,
- permissions auditables,
- plugin activable/desactivable,
- isolation testee.

## Criteres d'acceptation

- chaque epic `P0` du backlog a ses tests de reference,
- les fixtures officielles sont versionnees dans le repo,
- la CI GitHub Actions peut valider le coeur du MVP sans intervention manuelle.
