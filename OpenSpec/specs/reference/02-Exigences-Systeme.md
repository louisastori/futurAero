# Exigences Systeme

## Exigences fonctionnelles

- FR-01: L'application doit fonctionner en mode desktop avec stockage local du projet.
- FR-02: L'application doit pouvoir operer sans connexion internet pour les fonctions coeur.
- FR-03: Le systeme doit fournir un espace de modelisation 2D pour les esquisses et contraintes.
- FR-04: Le systeme doit permettre la generation de pieces 3D parametriques a partir d'esquisses et d'operations.
- FR-05: Le systeme doit supporter les assemblages mecaniques et les liaisons entre composants.
- FR-06: Le systeme doit decrire une chaine cinematique et calculer ses degres de liberte.
- FR-07: Le systeme doit permettre la creation d'une cellule de robotisation comprenant robots, convoyeurs, outillages, capteurs et zones de securite.
- FR-08: Le systeme doit attacher a chaque composant des proprietes physiques et operationnelles.
- FR-09: Le systeme doit simuler le comportement d'un scenario dans un environnement numerique rejouable.
- FR-10: Le systeme doit proposer une IA locale capable d'assister la conception, l'analyse de conflit, la documentation et la generation de parametres.
- FR-11: Toute suggestion IA doit etre accompagnee d'une justification lisible et d'un niveau de confiance.
- FR-12: Le systeme doit historiser les hypotheses, changements majeurs et resultats de validation.
- FR-13: Le systeme doit exporter un sous-ensemble de donnees projet vers des formats d'echange cibles a definir.
- FR-14: Le systeme doit permettre la creation d'outils modulaires relies au meme graphe projet.
- FR-15: Le systeme doit fournir un mode de revue pour comparer modele, simulation et objectifs.
- FR-16: Le systeme doit permettre la definition et la simulation de capteurs avances incluant au minimum le LiDAR 2D/3D, les cameras et l'IMU.
- FR-17: Le systeme doit permettre la calibration intrinseque, extrinseque et temporelle d'un ensemble de capteurs.
- FR-18: Le systeme doit produire des sorties de perception exploitables comme nuages de points, cartes d'occupation, objets detectes et ecarts par rapport au modele CAO.
- FR-19: Le systeme doit pouvoir comparer une observation capteur a la scene numerique pour detecter derives, obstacles ou ecarts d'implantation.
- FR-20: Le systeme doit fournir une chaine de fusion capteurs explicable et configurable.
- FR-21: Le systeme doit permettre une integration industrielle locale avec ROS2, OPC UA, PLC, controleurs robots, Bluetooth, Wi-Fi et autres transports locaux via des adaptateurs explicites.
- FR-22: Le systeme doit modeliser les endpoints externes, signaux terrain, topics, tags, noeuds, variables et flux filaires/sans fil relies au meme graphe projet.
- FR-23: Le systeme doit permettre de definir des zones de securite, interlocks, permissifs et inhibitions de mouvement.
- FR-24: Le systeme doit supporter la configuration de LiDAR de securite et leur interaction avec les etats de surete de la cellule.
- FR-25: Le systeme doit fournir un workflow de commissioning allant de la scene nominale a la verification terrain.
- FR-26: Le systeme doit pouvoir comparer un as-built observe au modele as-designed et produire un rapport d'ecarts exploitable.
- FR-27: Le systeme doit fournir un moteur d'optimisation multi-objectifs sur implantation, sequence, couverture, marge de securite et temps de cycle.
- FR-28: Le systeme doit exposer un SDK plugins stable pour etendre outils, connecteurs, solveurs, imports/exports et panneaux UI.
- FR-29: Le systeme doit permettre l'installation, l'activation et la desactivation de plugins avec permissions explicites.
- FR-30: Le systeme doit fournir une strategie de tests imposant une couverture a 100 pourcent sur le perimetre code first-party du MVP.
- FR-31: Le systeme doit pouvoir declarer des flux Bluetooth Low Energy, Bluetooth Classic, Wi-Fi, MQTT, WebSocket, TCP/UDP, serial et autres flux reseau locaux en modes live, replay et emulated.
- FR-32: Le systeme doit exposer pour chaque flux les parametres de discovery, pairing, securite, latence, jitter, pertes, debit et etat de liaison quand ces donnees sont disponibles.

## Exigences non fonctionnelles

- NFR-01: Les calculs critiques doivent etre deterministes a configuration egale.
- NFR-02: Les composants coeur doivent etre executes localement.
- NFR-03: L'interface doit rester exploitable sur station sans GPU haut de gamme, avec degradation controlee.
- NFR-04: Le moteur de projet doit supporter de gros assemblages par chargement progressif.
- NFR-05: Les donnees projet doivent etre versionnables et diffables autant que possible.
- NFR-06: Les modules de simulation doivent exposer clairement leurs hypotheses.
- NFR-07: Le systeme doit journaliser les actions IA et les mutations majeures du projet.
- NFR-08: Les outils doivent partager un langage commun de scene, unite, repere et temps.
- NFR-09: Les donnees capteurs horodatees doivent rester synchronisables entre elles et avec la timeline de simulation.
- NFR-10: Les sorties de perception doivent etre rejouables a partir des memes entrees et profils de calibration.
- NFR-11: Les connecteurs industriels doivent pouvoir etre rejoues en mode hors ligne a partir de traces capturees.
- NFR-12: Les decisions de safety, inhibitions et permissifs doivent etre auditables et rejouables.
- NFR-13: Les plugins doivent etre isoles par permissions et ne pas pouvoir corrompre silencieusement le coeur projet.
- NFR-14: La CI du MVP doit refuser toute regression de couverture en dessous de 100 pourcent sur le scope defini.
- NFR-15: Les flux filaires et sans fil doivent pouvoir etre testes en mode degrade, avec injection ou replay de pertes, jitter, deconnexions et reconnexions.

## Objets metier centraux

- Project: conteneur principal, metadonnees, unites, versions.
- Part: piece parametrique ou importee.
- Assembly: ensemble de composants et de contraintes.
- RobotCell: scene de robotisation avec enveloppe spatiale et logique operationnelle.
- SimulationScenario: conditions initiales, horizon temporel, niveau de fidelite, resultats.
- ControllerModel: logique de commande abstraite ou executable.
- SensorModel: capteurs, bruit, latence, plages.
- PerceptionPipeline: chaine configurable d'acquisition, calibration, fusion et interpretation.
- CalibrationProfile: parametres de calibration geometrique et temporelle.
- PointCloudDataset: jeu de donnees de nuages de points ou mesures assimilables.
- ExternalEndpoint: representation d'un endpoint ROS2, OPC UA, PLC, robot controller, Bluetooth, Wi-Fi ou autre transport local.
- TelemetryStream: description d'un flux de messages ou trames mappable au graphe projet.
- SafetyInterlock: regle de surete reliant zones, capteurs, etats et inhibitions.
- SafetyControllerModel: automate de surete logique ou abstraction equivalente.
- CommissioningSession: session structuree de mise en service et de validation terrain.
- FieldCaptureDataset: capture terrain structuree pour replay, as-built et diagnostic.
- NetworkCaptureDataset: capture rejouable de paquets, trames ou messages reseau/sans fil.
- AsBuiltComparisonReport: comparaison du reel observe au nominal.
- OptimizationStudy: definition d'une etude d'optimisation.
- PluginManifest: declaration d'un plugin et de ses permissions.
- MaterialProfile: masse volumique, rigidite, friction, limites.
- ValidationReport: resultats, ecarts, hypotheses, statut.

## Contraintes de conception

- L'IA ne doit pas modifier un modele sans action explicite de l'utilisateur.
- Une simulation ne doit jamais melanger silencieusement des unites incompatibles.
- Les objets metier doivent etre adressables par identifiant stable.
- Le projet doit pouvoir etre ouvert meme si certains outils optionnels sont absents.
- Une donnee capteur ne doit jamais etre interpretee sans repere, horodatage et profil de calibration associes.
- Une logique de safety ne doit jamais etre appliquee sans graphe de dependances inspectable.
- Une liaison sans fil non certifiee ne doit pas etre consideree comme unique chaine de safety sans profil de validation explicite.
- Un plugin ne doit jamais recevoir plus de permissions que celles explicitement accordees.

## Criteres d'acceptation globaux du premier increment

- Un projet simple peut etre cree, sauvegarde, rouvert et versionne.
- Les objets de base part, assembly et robotCell peuvent coexister dans une meme scene.
- Une simulation simple produit un rapport lisible avec hypotheses et resultats.
- Une suggestion IA peut etre inspectee avant application.
- Une cellule simple peut recevoir un LiDAR virtuel et produire une sortie perception comparee au modele.
- Une cellule peut etre reliee a un endpoint industriel externe dans une spec de projet.
- Un flux Bluetooth, Wi-Fi ou telemetrique peut etre relie au projet, observe et rejoue avec son etat de liaison.
- Une session commissioning simple peut produire un rapport as-built vs as-designed.
