# Schemas Commandes Evenements

## Objectif

Figer un premier contrat structurel pour les envelopes et les payloads critiques du MVP.

## Convention generale

- transport en JSON,
- `kind` comme discriminant principal,
- horodatages ISO 8601 UTC,
- identifiants ULID prefixes,
- payloads stricts sans champs libres silencieux.

## Envelope `Command`

```json
{
  "commandId": "cmd_01J...",
  "kind": "project.create",
  "projectId": "prj_01J...",
  "targetId": null,
  "actorId": "user.local",
  "timestamp": "2026-04-06T10:00:00Z",
  "baseRevision": null,
  "payload": {}
}
```

Champs:

- `commandId`: requis
- `kind`: requis
- `projectId`: requis sauf bootstrap avant creation effective
- `targetId`: requis si la commande cible une entite
- `actorId`: requis
- `timestamp`: requis
- `baseRevision`: requis pour les mutations d'entite
- `payload`: requis

## Envelope `Event`

```json
{
  "eventId": "evt_01J...",
  "kind": "entity.created",
  "projectId": "prj_01J...",
  "targetId": "ent_01J...",
  "causedByCommandId": "cmd_01J...",
  "timestamp": "2026-04-06T10:00:00Z",
  "revision": "rev_01J...",
  "payload": {}
}
```

## Envelope `Job`

```json
{
  "jobId": "job_01J...",
  "kind": "simulation.run",
  "projectId": "prj_01J...",
  "targetId": "ent_01J...",
  "status": "queued",
  "progress": 0.0,
  "phase": "prepare",
  "message": "Preparing simulation",
  "estimatedRemainingMs": null,
  "startedAt": null,
  "updatedAt": "2026-04-06T10:00:00Z",
  "resultRef": null,
  "error": null
}
```

## Payloads de commandes MVP

### `project.create`

```json
{
  "name": "Cellule Demo",
  "displayUnits": {
    "length": "mm",
    "angle": "deg",
    "mass": "kg"
  },
  "defaultFrame": "world"
}
```

### `entity.create`

```json
{
  "entityType": "Part",
  "name": "Bracket-A",
  "initialData": {
    "geometrySource": "parametric",
    "parameterSet": {}
  },
  "parentId": "ent_01J..."
}
```

### `entity.patch`

```json
{
  "ops": [
    { "op": "replace", "path": "/name", "value": "Bracket-B" }
  ]
}
```

### `sketch.element.add`

```json
{
  "sketchId": "ent_01J...",
  "element": {
    "elementId": "skel_01J...",
    "kind": "line",
    "start": { "x": 0.0, "y": 0.0 },
    "end": { "x": 0.1, "y": 0.0 }
  }
}
```

### `sketch.constraint.add`

```json
{
  "sketchId": "ent_01J...",
  "constraint": {
    "constraintId": "skc_01J...",
    "kind": "horizontal",
    "refs": ["skel_01J..."]
  }
}
```

### `part.parameter.set`

```json
{
  "parameterId": "width",
  "value": 130,
  "unit": "mm",
  "expression": null
}
```

### `assembly.occurrence.add`

```json
{
  "assemblyId": "ent_01J...",
  "definitionId": "ent_01J...",
  "name": "Bracket-A:1",
  "transform": {
    "translation": { "x": 0.0, "y": 0.0, "z": 0.0 },
    "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
    "scale": { "x": 1.0, "y": 1.0, "z": 1.0 }
  }
}
```

### `assembly.mate.add`

```json
{
  "assemblyId": "ent_01J...",
  "constraintType": "coincident",
  "sourceRef": { "entityId": "ent_01J...", "path": "faces[0]", "role": "source" },
  "targetRef": { "entityId": "ent_01J...", "path": "faces[3]", "role": "target" }
}
```

### `joint.create`

```json
{
  "assemblyId": "ent_01J...",
  "jointType": "revolute",
  "sourceOccurrenceId": "ent_01J...",
  "targetOccurrenceId": "ent_01J...",
  "axis": { "x": 0.0, "y": 0.0, "z": 1.0 },
  "limits": { "min": -1.57, "max": 1.57 }
}
```

### `scenario.create`

```json
{
  "name": "PickAndPlaceCycle",
  "sceneRef": { "entityId": "ent_01J...", "role": "source" },
  "fidelityLevel": "S1",
  "timeStepMs": 5,
  "durationMs": 30000,
  "solverConfig": {
    "physicsMode": "rigid_body",
    "collisionMode": "discrete",
    "randomSeed": 42
  }
}
```

### `simulation.run.start`

```json
{
  "scenarioId": "ent_01J...",
  "overrideConfig": null
}
```

### `sensor.rig.create`

```json
{
  "name": "Rig-Cellule-01",
  "mountRef": { "entityId": "ent_01J...", "role": "target" },
  "sensorIds": []
}
```

### `lidar.configure`

```json
{
  "lidarId": "ent_01J...",
  "sensorType": "lidar_3d",
  "channels": 32,
  "horizontalFovDeg": 360.0,
  "verticalFovDeg": 40.0,
  "minRangeM": 0.2,
  "maxRangeM": 80.0,
  "angularResolutionDeg": 0.2,
  "scanRateHz": 10.0
}
```

### `camera.configure`

```json
{
  "cameraId": "ent_01J...",
  "sensorType": "camera_depth",
  "resolution": { "width": 1280, "height": 720 },
  "fovDeg": 78.0,
  "frameRateHz": 30.0,
  "latencyMs": 25
}
```

### `calibration.solve`

```json
{
  "targetIds": ["ent_01J...", "ent_01J..."],
  "calibrationType": "multi_sensor",
  "datasetRefs": ["pcd_01J..."],
  "referenceFrame": "cell.world"
}
```

### `perception.pipeline.create`

```json
{
  "name": "Pipeline-Lidar-Cellule",
  "inputSensorIds": ["ent_01J...", "ent_01J..."],
  "fusionMode": "lidar_camera_imu",
  "stageConfigs": [
    { "stage": "time_sync", "enabled": true },
    { "stage": "filter", "enabled": true },
    { "stage": "map", "enabled": true }
  ]
}
```

### `perception.run.start`

```json
{
  "pipelineId": "ent_01J...",
  "inputMode": "simulated|replay|hybrid",
  "relatedScenarioId": "ent_01J...",
  "datasetRefs": []
}
```

### `ros2.bridge.configure`

```json
{
  "endpointId": "ent_01J...",
  "domainId": 10,
  "nodeNames": ["faero_bridge"],
  "topicMaps": []
}
```

### `opcua.endpoint.create`

```json
{
  "name": "Cellule-OPCUA",
  "endpointUrl": "opc.tcp://127.0.0.1:4840",
  "securityMode": "none"
}
```

### `wireless.endpoint.create`

```json
{
  "name": "AMR-Lidar-WiFi-01",
  "endpointType": "wifi_device",
  "mode": "live",
  "transportProfile": {
    "transportKind": "wifi",
    "adapterId": "wlan0",
    "discoveryMode": "mdns",
    "credentialPolicy": "runtime_prompt",
    "securityMode": "wpa3"
  },
  "addressing": {
    "host": "amr-lidar-01.local",
    "port": 9001,
    "path": "/telemetry"
  }
}
```

### `telemetry.stream.create`

```json
{
  "name": "BumperStatus",
  "endpointId": "ent_01J...",
  "streamType": "mqtt_topic",
  "direction": "inbound",
  "schemaRef": "schemas/telemetry/bumper-status.schema.json",
  "codecProfile": {
    "encoding": "json"
  },
  "timingProfile": {
    "expectedRateHz": 20,
    "maxLatencyMs": 80
  },
  "qosProfile": {
    "delivery": "at_least_once",
    "ordering": "best_effort"
  }
}
```

### `integration.discovery.refresh`

```json
{
  "endpointId": "ent_01J...",
  "timeoutMs": 5000
}
```

### `integration.link.simulate`

```json
{
  "endpointId": "ent_01J...",
  "degradationProfile": {
    "latencyMs": 120,
    "jitterMs": 40,
    "dropRate": 0.05,
    "disconnectAfterMs": null
  }
}
```

### `safety.interlock.create`

```json
{
  "name": "StopIfLidarTriggered",
  "inputRefs": [
    { "entityId": "ent_01J...", "role": "source" }
  ],
  "conditionTree": {
    "op": "eq",
    "left": "lidar_zone_stop",
    "right": true
  },
  "outputActions": [
    { "kind": "robot.stop", "targetId": "ent_01J..." }
  ],
  "priority": 100
}
```

### `safety.validation.run`

```json
{
  "targetSceneId": "ent_01J...",
  "scenarioId": "ent_01J..."
}
```

### `commissioning.session.start`

```json
{
  "name": "Commissioning-Cellule-01",
  "targetSceneRef": { "entityId": "ent_01J...", "role": "target" },
  "objective": "Validate nominal cell against field installation"
}
```

### `asbuilt.compare.run`

```json
{
  "sessionId": "ent_01J...",
  "nominalRef": { "entityId": "ent_01J...", "role": "source" },
  "observedRef": { "entityId": "ent_01J...", "role": "target" },
  "toleranceProfileId": "ent_01J..."
}
```

### `optimization.study.create`

```json
{
  "name": "Optimize-Cycle-And-Safety",
  "targetRef": { "entityId": "ent_01J...", "role": "target" },
  "objectiveFunctions": [
    { "metricId": "cycle_time_ms", "goal": "minimize" },
    { "metricId": "safety_margin_min", "goal": "maximize" }
  ],
  "constraints": [
    { "metricId": "collision_count", "op": "==", "value": 0 }
  ]
}
```

### `optimization.run.start`

```json
{
  "studyId": "ent_01J...",
  "budget": {
    "maxIterations": 500,
    "maxDurationMs": 300000
  }
}
```

### `plugin.install`

```json
{
  "manifestId": "ent_01J...",
  "sourceType": "local",
  "sourceRef": "plugins/manifests/plg_01J....json"
}
```

### `plugin.enable`

```json
{
  "pluginId": "plg.optimization.local",
  "version": "0.1.0"
}
```

### `ai.runtime.profile.set`

```json
{
  "profileId": "ent_01J...",
  "mode": "furnace",
  "allowGpuSaturation": true,
  "allowLongContext": true,
  "allowMultiPassCritic": true
}
```

### `ai.suggestion.request`

```json
{
  "mode": "explain",
  "prompt": "Explique pourquoi le robot entre en collision.",
  "selectionRefs": [
    { "entityId": "ent_01J...", "role": "source" }
  ],
  "relatedRunId": "run_01J..."
}
```

### `ai.deep_explain.request`

```json
{
  "prompt": "Analyse complete de la collision et des derives associees.",
  "selectionRefs": [
    { "entityId": "ent_01J...", "role": "source" }
  ],
  "relatedRunId": "run_01J...",
  "runtimeMode": "furnace"
}
```

### `ai.suggestion.apply`

```json
{
  "suggestionId": "ais_01J...",
  "applyMode": "all"
}
```

## Payloads d'evenements MVP

### `entity.created`

```json
{
  "entityType": "Part",
  "parentId": "ent_01J..."
}
```

### `part.regenerated`

```json
{
  "partId": "ent_01J...",
  "featureCount": 2,
  "massKg": 1.42,
  "bounds": {
    "min": { "x": 0.0, "y": 0.0, "z": 0.0 },
    "max": { "x": 0.12, "y": 0.06, "z": 0.04 }
  }
}
```

### `assembly.solved`

```json
{
  "assemblyId": "ent_01J...",
  "status": "solved",
  "remainingDegreesOfFreedom": 1
}
```

### `simulation.completed`

```json
{
  "runId": "run_01J...",
  "scenarioId": "ent_01J...",
  "artifactRefs": [
    "simulations/runs/run_01J.../summary.json"
  ],
  "metrics": {
    "cycle_time_ms": 27480,
    "collision_count": 0
  }
}
```

### `calibration.completed`

```json
{
  "calibrationProfileId": "ent_01J...",
  "qualityMetrics": {
    "rmsError": 0.004,
    "coverageRatio": 0.93
  }
}
```

### `perception.completed`

```json
{
  "runId": "run_01J...",
  "pipelineId": "ent_01J...",
  "artifactRefs": [
    "perception/datasets/pcd_01J....json",
    "perception/maps/map_01J....json"
  ],
  "metrics": {
    "lidar_coverage_ratio": 0.91,
    "mapping_deviation_mm": 7.5
  }
}
```

### `integration.connected`

```json
{
  "endpointId": "ent_01J...",
  "endpointType": "wifi_device",
  "transportKind": "wifi",
  "mode": "live"
}
```

### `integration.link.quality.changed`

```json
{
  "endpointId": "ent_01J...",
  "transportKind": "wifi",
  "status": "degraded",
  "rssiDbm": -64,
  "latencyMs": 42,
  "jitterMs": 8,
  "dropRate": 0.01
}
```

### `safety.validation.completed`

```json
{
  "validationId": "ent_01J...",
  "criticalFindings": 0,
  "status": "passed"
}
```

### `asbuilt.comparison.completed`

```json
{
  "reportId": "ent_01J...",
  "maxDeviationMm": 4.2,
  "unknownObstacleCount": 0
}
```

### `optimization.completed`

```json
{
  "runId": "ent_01J...",
  "bestCandidateRef": "ent_01J...",
  "candidateCount": 48
}
```

### `plugin.enabled`

```json
{
  "pluginId": "plg.optimization.local",
  "version": "0.1.0"
}
```

### `ai.runtime.profile.changed`

```json
{
  "profileId": "ent_01J...",
  "mode": "furnace"
}
```

### `ai.critic.completed`

```json
{
  "sessionId": "ent_01J...",
  "contradictionCount": 1,
  "finalConfidence": 0.87
}
```

### `ai.suggestion.created`

```json
{
  "suggestionId": "ais_01J...",
  "mode": "explain",
  "confidence": 0.82,
  "riskLevel": "low"
}
```

## Resultat standard de commande

Succes:

```json
{
  "accepted": true,
  "commandId": "cmd_01J...",
  "newRevision": "rev_01J...",
  "jobId": null,
  "affectedEntityIds": ["ent_01J..."],
  "warnings": []
}
```

Erreur:

```json
{
  "accepted": false,
  "commandId": "cmd_01J...",
  "error": {
    "code": "E_INVALID_REFERENCE",
    "message": "Source reference not found",
    "targetId": "ent_01J...",
    "details": {
      "path": "faces[99]"
    }
  }
}
```

## Contraintes de validation JSON

- aucun champ inconnu par defaut dans les payloads critiques,
- `kind` doit correspondre au schema de `payload`,
- `baseRevision` obligatoire sur `entity.patch`, `part.parameter.set`, `assembly.mate.add`, `joint.create`,
- `baseRevision` obligatoire sur `lidar.configure`, `camera.configure`, `perception.pipeline.patch`,
- `baseRevision` obligatoire sur `safety.interlock.create`, `optimization.study.patch`, `plugin.enable`, `telemetry.binding.patch`,
- `ai.runtime.profile.set` doit etre refuse si le profil demande des ressources indisponibles sans mode degrade,
- `wireless.endpoint.create` doit etre refuse en mode `live` si le profil de transport ou de securite est absent sauf declaration explicite de lien non securise,
- `telemetry.stream.create` doit etre refuse si aucun codec ou schema n'est declare,
- `relatedRunId` obligatoire pour une demande IA axee simulation si `mode=explain` et que le prompt cite une collision,
- `simulation.run.start` refuse si un run actif existe deja pour le meme scenario,
- `perception.run.start` refuse si la calibration requise est absente dans le mode concerne,
- `integration.link.simulate` refuse si l'endpoint ne supporte ni replay ni injection de degradation,
- `plugin.install` refuse si le manifest demande une permission inconnue,
- `optimization.run.start` refuse si aucune contrainte ou aucun objectif n'est defini.

## Mapping recommande code

- Rust: enum taggee `CommandPayload`
- TypeScript: union discriminee `CommandEnvelope`
- JSON Schema: source d'interoperabilite et de validation IO

## Criteres d'acceptation

- les schemas couvrent au moins tout le flux MVP strict,
- un payload invalide est rejete avant execution metier,
- les memes envelopes peuvent etre consommees par l'UI, le stockage et les tests.
