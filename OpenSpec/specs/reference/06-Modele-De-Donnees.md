# Modele de Donnees

## Objectif

Definir un modele de donnees unique pour tout le projet FutureAero. Ce modele doit servir a la fois a la persistance, a l'affichage, aux calculs, a la simulation et a l'IA locale.

## Principes

- Un seul graphe projet fait foi.
- Chaque objet metier possede un identifiant stable.
- Les donnees metier sont separees des caches et des rendus derives.
- Les unites internes sont canoniques et les unites d'affichage sont des preferences.
- Les relations entre objets sont explicites et jamais implicites.

## Types partages

### Identifiants

- `ProjectId`: `prj_<ulid>`
- `EntityId`: `ent_<ulid>`
- `RevisionId`: `rev_<ulid>`
- `AssetId`: `ast_<ulid>`
- `JobId`: `job_<ulid>`
- `AiSuggestionId`: `ais_<ulid>`

## Champs communs a toute entite

Chaque entite persistable doit contenir:

- `id`
- `type`
- `name`
- `revision`
- `createdAt`
- `updatedAt`
- `createdBy`
- `tags`
- `status`

## Primitifs communs

### UnitSystem

- Stockage canonique interne en SI.
- Longueur: metre
- Masse: kilogramme
- Temps: seconde
- Angle: radian
- Force: newton
- Couple: newton metre

### Vector3

```json
{ "x": 0.0, "y": 0.0, "z": 0.0 }
```

### Quaternion

```json
{ "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 }
```

### Transform3D

```json
{
  "translation": { "x": 0.0, "y": 0.0, "z": 0.0 },
  "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
  "scale": { "x": 1.0, "y": 1.0, "z": 1.0 }
}
```

### Bounds3D

```json
{
  "min": { "x": 0.0, "y": 0.0, "z": 0.0 },
  "max": { "x": 1.0, "y": 1.0, "z": 1.0 }
}
```

### ParameterValue

```json
{
  "kind": "number|string|boolean|enum|vector3",
  "value": 12.5,
  "unit": "mm",
  "expression": "base_width / 2",
  "resolvedValue": 0.0125
}
```

### Reference

```json
{
  "entityId": "ent_01J...",
  "path": "features[2].profiles[0]",
  "role": "source|target|driver|result"
}
```

## Structure globale du graphe

Le projet est un graphe type compose de noeuds et de relations:

- les noeuds representent les objets metier,
- les relations representent les dependances,
- les relations ont un type et ne sont jamais deduites a l'ecriture.

## Types de relations autorises

- `contains`: un objet en contient un autre
- `references`: un objet pointe vers un autre
- `instantiates`: un assemblage instancie une piece ou un sous-ensemble
- `drives`: un parametre pilote un autre objet
- `simulates`: un scenario simule une scene
- `observes`: un capteur observe une scene, une zone ou un objet
- `calibrates`: un profil calibre un capteur ou un rig
- `maps`: une sortie perception decrit une zone ou une scene
- `connectsTo`: un endpoint ou un adaptateur relie un objet projet a un systeme externe
- `protects`: une regle de surete protege une zone, un robot ou un mouvement
- `compares`: une capture terrain ou un rapport compare deux etats ou deux scenes
- `optimizes`: une etude d'optimisation cible une scene, une sequence ou une configuration
- `extends`: un plugin etend un domaine, un outil ou une vue
- `reportsOn`: un rapport cible un scenario ou un objet
- `generatedBy`: un objet a ete genere par un job ou une suggestion IA

## Entites coeur

### Project

Role:

- racine logique du projet,
- definition du systeme d'unites d'affichage,
- declaration des scenes, librairies locales et configurations actives.

Champs requis:

- `id`
- `name`
- `formatVersion`
- `displayUnits`
- `defaultFrame`
- `rootSceneId`
- `activeConfigurationId`
- `entityIndex`

### Part

Role:

- definition d'une piece parametrique ou importee.

Champs requis:

- `id`
- `geometrySource`: `parametric|imported|generated`
- `sketchIds`
- `featureIds`
- `parameterSet`
- `materialProfileId`
- `bodyIds`
- `massProperties`
- `bounds`

### Sketch2D

Role:

- support d'esquisse 2D resolue par contraintes.

Champs requis:

- `id`
- `planeRef`
- `elements`
- `constraints`
- `dimensions`
- `solveState`

### Feature3D

Role:

- operation parametrique sur une piece.

Champs requis:

- `id`
- `featureType`
- `inputRefs`
- `parameterSet`
- `suppressed`
- `regenState`
- `resultBodyIds`

### Assembly

Role:

- composition de pieces et de sous-ensembles dans un repere commun.

Champs requis:

- `id`
- `occurrenceIds`
- `mateConstraintIds`
- `jointIds`
- `parameterSet`
- `solveState`

### AssemblyOccurrence

Role:

- instance d'une piece ou d'un sous-ensemble placee dans un assemblage.

Champs requis:

- `id`
- `definitionId`
- `parentAssemblyId`
- `localTransform`
- `suppressed`
- `configurationOverrides`

### MateConstraint

Role:

- contrainte d'assemblage geometrique.

Champs requis:

- `id`
- `constraintType`
- `sourceRef`
- `targetRef`
- `limits`
- `solvePriority`
- `status`

### Joint

Role:

- liaison mecanique etatisee pour cinematique et simulation.

Champs requis:

- `id`
- `jointType`
- `sourceOccurrenceId`
- `targetOccurrenceId`
- `axis`
- `limits`
- `defaultState`
- `driveModel`

### RobotCell

Role:

- scene de robotisation de haut niveau.

Champs requis:

- `id`
- `sceneAssemblyId`
- `robotIds`
- `equipmentIds`
- `safetyZoneIds`
- `sequenceIds`
- `controllerModelIds`

### RobotModel

Role:

- modele de robot ou axe pilotable.

Champs requis:

- `id`
- `kinematicChain`
- `jointIds`
- `toolMountRef`
- `workspaceBounds`
- `payloadLimits`
- `calibrationState`

### EquipmentModel

Role:

- tout equipement non robot principal: convoyeur, gabarit, poste, pince, table, outillage.

Champs requis:

- `id`
- `equipmentType`
- `assemblyOccurrenceId`
- `parameterSet`
- `ioPortIds`

### SensorModel

Role:

- capteur simulable et routable vers la logique.

Champs requis:

- `id`
- `sensorType`
- `mountRef`
- `samplingRateHz`
- `latencyMs`
- `noiseModel`
- `rangeModel`
- `outputSignalId`

### SensorRig

Role:

- ensemble de capteurs synchronises partageant un montage commun.

Champs requis:

- `id`
- `mountRef`
- `sensorIds`
- `timeSyncProfileId`
- `extrinsicCalibrationProfileId`
- `status`

### LidarModel

Role:

- capteur LiDAR 2D ou 3D simulable et replayable.

Champs requis:

- `id`
- `sensorType`: `lidar_2d|lidar_3d|lidar_solid_state`
- `mountRef`
- `channels`
- `horizontalFovDeg`
- `verticalFovDeg`
- `minRangeM`
- `maxRangeM`
- `angularResolutionDeg`
- `scanRateHz`
- `latencyMs`
- `noiseModel`
- `returnModel`
- `outputDatasetId`

### CameraModel

Role:

- camera RGB, monochrome, stereo ou profondeur partageant le meme graphe capteurs.

Champs requis:

- `id`
- `sensorType`: `camera_rgb|camera_depth|camera_stereo|camera_thermal`
- `mountRef`
- `resolution`
- `fovDeg`
- `frameRateHz`
- `latencyMs`
- `intrinsicCalibrationProfileId`
- `outputDatasetId`

### IMUModel

Role:

- unite inertielle utilisee pour synchronisation, localisation et fusion.

Champs requis:

- `id`
- `mountRef`
- `sampleRateHz`
- `gyroNoiseModel`
- `accelNoiseModel`
- `biasModel`
- `outputDatasetId`

### CalibrationProfile

Role:

- profil de calibration intrinseque, extrinseque ou temporelle.

Champs requis:

- `id`
- `calibrationType`: `intrinsic|extrinsic|temporal|multi_sensor`
- `targetIds`
- `referenceFrame`
- `parameters`
- `qualityMetrics`
- `validFrom`

### PerceptionPipeline

Role:

- chaine de traitement capteurs configurable et explicable.

Champs requis:

- `id`
- `inputSensorIds`
- `stageConfigs`
- `fusionMode`
- `outputRefs`
- `runtimeProfile`
- `status`

### PointCloudDataset

Role:

- jeu de donnees de points issu d'un LiDAR, d'une fusion ou d'une reconstruction.

Champs requis:

- `id`
- `sourceSensorIds`
- `frameId`
- `timestampRange`
- `pointFormat`
- `pointCount`
- `assetRef`
- `qualityMetrics`

### OccupancyMap

Role:

- representation volumique ou surfacique des zones occupees/libres/inconnues.

Champs requis:

- `id`
- `mapType`: `grid_2d|voxel_3d|distance_field`
- `frameId`
- `resolution`
- `sourceRefs`
- `assetRef`
- `coverageMetrics`

### LocalizationEstimate

Role:

- estimation de pose issue du pipeline de perception.

Champs requis:

- `id`
- `frameId`
- `pose`
- `covariance`
- `timestamp`
- `sourceRefs`
- `qualityMetrics`

### ExternalEndpoint

Role:

- endpoint externe relie a un protocole industriel, robotique, filaire ou sans fil.

Champs requis:

- `id`
- `endpointType`: `ros2|opcua|plc|robot_controller|bluetooth_le|bluetooth_classic|wifi_device|mqtt_broker|websocket_peer|tcp_stream|udp_stream|serial_device|fieldbus_trace|custom_stream`
- `transportProfile`
- `connectionProfile`
- `addressing`
- `signalMapIds`
- `mode`: `live|replay|emulated|gateway`
- `linkMetrics`
- `status`

### TelemetryStream

Role:

- flux de messages, trames ou echantillons lie a un endpoint externe et au graphe projet.

Champs requis:

- `id`
- `endpointId`
- `streamType`: `ble_gatt|bluetooth_spp|wifi_udp|wifi_tcp|mqtt_topic|websocket|serial|binary_frame|json_frame|custom`
- `direction`: `inbound|outbound|bidirectional`
- `codecProfile`
- `schemaRef`
- `timingProfile`
- `qosProfile`
- `status`

### Ros2GraphBinding

Role:

- liaison entre le graphe projet et un graphe ROS2.

Champs requis:

- `id`
- `domainId`
- `nodeNames`
- `topicMaps`
- `serviceMaps`
- `frameConvention`
- `status`

### OpcUaEndpoint

Role:

- endpoint OPC UA client ou server pour lecture/ecriture de variables structurees.

Champs requis:

- `id`
- `endpointUrl`
- `namespaceMap`
- `nodeMaps`
- `securityMode`
- `status`

### PlcModel

Role:

- abstraction d'un PLC et de ses variables, tags, blocs ou zones memoire.

Champs requis:

- `id`
- `vendorProfile`
- `programMap`
- `tagBindings`
- `cycleTimeMs`
- `status`

### RobotControllerBinding

Role:

- lien entre un robot du modele et un controleur reel ou emule.

Champs requis:

- `id`
- `robotModelId`
- `controllerType`
- `endpointId`
- `programBindings`
- `status`

### SafetyZone

Role:

- zone de surete, de ralentissement, d'avertissement ou d'interdiction.

Champs requis:

- `id`
- `zoneType`: `safe|warning|slowdown|forbidden`
- `geometryRef`
- `monitoredEntityIds`
- `triggerRefs`
- `responsePolicy`

### SafetyInterlock

Role:

- regle de surete reliant etats, capteurs, zones et inhibitions.

Champs requis:

- `id`
- `interlockType`
- `inputRefs`
- `conditionTree`
- `outputActions`
- `priority`
- `status`

### SafetyControllerModel

Role:

- logique de surete abstraite ou calquee sur un dispositif reel.

Champs requis:

- `id`
- `controllerType`
- `safetyFunctionIds`
- `interlockIds`
- `signalIds`
- `status`

### CommissioningSession

Role:

- session de mise en service reliant nominal, terrain, ecarts et actions.

Champs requis:

- `id`
- `targetSceneRef`
- `objective`
- `stepStates`
- `fieldCaptureIds`
- `comparisonReportIds`
- `status`

### FieldCaptureDataset

Role:

- capture terrain de mesures, traces, etats machines ou perception pour replay.

Champs requis:

- `id`
- `captureType`: `perception|plc_trace|robot_trace|network_trace|wireless_trace|mixed`
- `sourceEndpointIds`
- `timestampRange`
- `assetRefs`
- `qualityMetrics`

### NetworkCaptureDataset

Role:

- capture rejouable de paquets, trames ou messages transportes sur un lien filaire ou sans fil.

Champs requis:

- `id`
- `endpointId`
- `captureType`: `pcap|ble_trace|mqtt_log|socket_dump|serial_trace`
- `timestampRange`
- `assetRefs`
- `linkMetrics`
- `status`

### AsBuiltComparisonReport

Role:

- rapport comparant scene observee et scene nominale.

Champs requis:

- `id`
- `nominalRef`
- `observedRef`
- `deviationMetrics`
- `findings`
- `status`

### OptimizationStudy

Role:

- definition d'un probleme d'optimisation multi-objectifs.

Champs requis:

- `id`
- `targetRef`
- `decisionVariables`
- `objectiveFunctions`
- `constraints`
- `searchStrategy`
- `status`

### OptimizationRun

Role:

- execution d'une etude d'optimisation.

Champs requis:

- `id`
- `studyId`
- `startedAt`
- `finishedAt`
- `bestCandidateRef`
- `paretoFrontRef`
- `resultSummary`

### PluginManifest

Role:

- declaration versionnee d'un plugin et de ses permissions.

Champs requis:

- `id`
- `pluginId`
- `version`
- `capabilities`
- `permissions`
- `entrypoints`
- `compatibility`
- `status`

### PluginCapability

Role:

- capacite exposee par un plugin au coeur ou a l'UI.

Champs requis:

- `id`
- `capabilityType`
- `providedContracts`
- `uiContributions`
- `permissionRefs`

### ActuatorModel

Role:

- actionneur relie a un mouvement ou a un etat d'equipement.

Champs requis:

- `id`
- `actuatorType`
- `targetRef`
- `commandSignalId`
- `responseModel`
- `limits`

### ControllerModel

Role:

- logique de commande abstraite pour simulation.

Champs requis:

- `id`
- `controllerType`
- `stateMachine`
- `inputSignalIds`
- `outputSignalIds`
- `timers`
- `faultHandlers`

### Signal

Role:

- signal logique ou analogique partage entre capteurs, controle et actionneurs.

Champs requis:

- `id`
- `signalType`
- `dataType`
- `defaultValue`
- `currentValue`
- `producerIds`
- `consumerIds`

### SimulationScenario

Role:

- configuration executable d'un scenario rejouable.

Champs requis:

- `id`
- `sceneRef`
- `fidelityLevel`
- `timeStepMs`
- `durationMs`
- `solverConfig`
- `initialState`
- `controllerBindings`
- `perceptionBindings`
- `externalBindings`
- `safetyBindings`
- `materialOverrides`
- `expectedMetrics`

### SimulationRun

Role:

- execution horodatee d'un scenario.

Champs requis:

- `id`
- `scenarioId`
- `startedAt`
- `finishedAt`
- `engineVersion`
- `runConfigHash`
- `resultSummary`
- `artifactRefs`

### ValidationReport

Role:

- rapport de verification comparant objectifs, hypothese et resultats.

Champs requis:

- `id`
- `targetRef`
- `metricResults`
- `findings`
- `assumptions`
- `status`

### AiSuggestion

Role:

- proposition inspectable issue de l'IA locale.

Champs requis:

- `id`
- `sessionId`
- `promptHash`
- `contextRefs`
- `summary`
- `proposedCommands`
- `confidence`
- `riskLevel`
- `explanation`
- `reviewState`

### AiRuntimeProfile

Role:

- profil de ressources et d'orchestration pour l'IA locale.

Champs requis:

- `id`
- `profileName`
- `mode`: `eco|standard|max|furnace`
- `resourcePolicy`
- `modelRouting`
- `maxContextBudget`
- `criticPolicy`
- `status`

### AiSession

Role:

- session IA rattachee a une tache, un contexte et un profil runtime.

Champs requis:

- `id`
- `runtimeProfileId`
- `userIntent`
- `mode`
- `contextRefs`
- `modelPasses`
- `resourceSnapshot`
- `status`

### Asset

Role:

- binaire externe stocke dans le projet.

Champs requis:

- `id`
- `assetType`
- `contentHash`
- `mimeType`
- `relativePath`
- `sizeBytes`

### OpenSpecDocument

Role:

- document d intention, de revue ou de mapping stocke en clair dans le projet,
- capture les informations que des outils CAO vendor gardent souvent dans des conteneurs binaires peu diffables,
- relie les decisions humaines aux entites et endpoints du graphe sans les melanger aux donnees transactionnelles coeur.

Champs requis:

- `id`
- `title`
- `kind`: `design_intent|review_note|import_mapping|manufacturing_note|safety_case|custom`
- `status`
- `bodyFormat`: `markdown`
- `entityRefs`
- `externalRefs`
- `tags`
- `updatedAt`
- `content`

## Invariants de donnees

- Toute entite autre que `Project` doit appartenir a un projet unique.
- Toute relation doit pointer vers des identifiants existants.
- Une `AssemblyOccurrence` ne pointe jamais directement sur un `Sketch2D`.
- Une `SimulationRun` est immuable apres cloture.
- Une `AiSuggestion` ne modifie jamais directement une entite; elle propose des commandes.
- Un `LidarModel` ou un `CameraModel` ne peut pas etre exploite sans repere de montage.
- Une sortie `PointCloudDataset` ne peut pas exister sans horodatage et reference de frame.
- Un `ExternalEndpoint` en mode `live` doit declarer un `transportProfile`, un `mode` et des parametres de securite exploitables.
- Un `TelemetryStream` ne peut pas exister sans `endpointId` valide ni codec ou schema explicite.
- Un `SafetyInterlock` ne peut pas referencer des entrees ou actions inexistantes.
- Un `NetworkCaptureDataset` doit declarer un endpoint source et une base temporelle rejouable.
- Un `PluginManifest` doit declarer explicitement chaque permission demandee.
- Un `OpenSpecDocument` reste lisible en clair dans un format texte FutureAero et ne depend jamais d un blob binaire opaque pour etre compris.
- Un `OpenSpecDocument` documente l intention ou la tracabilite et ne remplace pas les noeuds metier transactionnels du graphe.
- Les resultats derives comme maillages d'affichage ne sont pas stockes dans les entites metier.

## Exemple minimal de Part

```json
{
  "id": "ent_01JFA_PART_001",
  "type": "Part",
  "name": "Bracket-A",
  "revision": "rev_01JFA_REV_001",
  "createdAt": "2026-04-06T10:00:00Z",
  "updatedAt": "2026-04-06T10:05:00Z",
  "createdBy": "user.local",
  "tags": ["mvp", "fixture"],
  "status": "active",
  "geometrySource": "parametric",
  "sketchIds": ["ent_01JFA_SKT_001"],
  "featureIds": ["ent_01JFA_FTR_001", "ent_01JFA_FTR_002"],
  "parameterSet": {
    "width": { "kind": "number", "value": 120, "unit": "mm", "resolvedValue": 0.12 }
  },
  "materialProfileId": "ent_01JFA_MAT_001",
  "bodyIds": ["ent_01JFA_BOD_001"],
  "massProperties": {
    "massKg": 1.42,
    "centerOfMass": { "x": 0.03, "y": 0.01, "z": 0.02 }
  },
  "bounds": {
    "min": { "x": 0.0, "y": 0.0, "z": 0.0 },
    "max": { "x": 0.12, "y": 0.06, "z": 0.04 }
  }
}
```

## Exemple minimal de SimulationScenario

```json
{
  "id": "ent_01JFA_SCN_001",
  "type": "SimulationScenario",
  "name": "PickAndPlaceCycle",
  "revision": "rev_01JFA_REV_010",
  "createdAt": "2026-04-06T12:00:00Z",
  "updatedAt": "2026-04-06T12:00:00Z",
  "createdBy": "user.local",
  "tags": ["mvp"],
  "status": "active",
  "sceneRef": { "entityId": "ent_01JFA_CELL_001", "role": "source" },
  "fidelityLevel": "S1",
  "timeStepMs": 5,
  "durationMs": 30000,
  "solverConfig": {
    "physicsMode": "rigid_body",
    "collisionMode": "discrete",
    "randomSeed": 42
  },
  "initialState": {
    "signalValues": { "part_present": true },
    "jointStates": { "robot.j1": 0.0 }
  },
  "controllerBindings": ["ent_01JFA_CTL_001"],
  "perceptionBindings": ["ent_01JFA_PIPE_001"],
  "externalBindings": ["ent_01JFA_OPCUA_001"],
  "safetyBindings": ["ent_01JFA_SAFE_001"],
  "materialOverrides": [],
  "expectedMetrics": [
    { "metricId": "cycle_time_ms", "operator": "<=", "target": 28000 }
  ]
}
```

## Criteres d'acceptation

- Les memes objets peuvent etre consommes par l'UI, la simulation et l'IA sans conversion ad hoc.
- Toute entite persistable a un identifiant stable et une revision.
- Les objets critiques du MVP sont couverts par ce modele sans champ implicite cache.
