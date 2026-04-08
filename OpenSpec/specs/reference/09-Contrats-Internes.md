# Contrats Internes

## Objectif

Definir les contrats de communication entre UI, coeur, moteurs et outils afin d'eviter les integrations implicites.

## Principe central

Toute mutation passe par une commande. Toute consequence observable est publiee en evenement. Les traitements longs sont encapsules dans des jobs.

## Enveloppe de commande

```json
{
  "commandId": "cmd_<ulid>",
  "kind": "part.parameter.set",
  "projectId": "prj_<ulid>",
  "targetId": "ent_<ulid>",
  "actorId": "user.local",
  "timestamp": "2026-04-06T10:10:00Z",
  "baseRevision": "rev_<ulid>",
  "payload": {}
}
```

## Enveloppe d'evenement

```json
{
  "eventId": "evt_<ulid>",
  "kind": "part.regenerated",
  "projectId": "prj_<ulid>",
  "targetId": "ent_<ulid>",
  "causedByCommandId": "cmd_<ulid>",
  "timestamp": "2026-04-06T10:10:01Z",
  "revision": "rev_<ulid>",
  "payload": {}
}
```

## Enveloppe de job

```json
{
  "jobId": "job_<ulid>",
  "kind": "simulation.run",
  "projectId": "prj_<ulid>",
  "targetId": "ent_<ulid>",
  "status": "queued|running|completed|failed|cancelled",
  "progress": 0.0,
  "startedAt": "2026-04-06T10:11:00Z",
  "updatedAt": "2026-04-06T10:11:05Z",
  "resultRef": null,
  "error": null
}
```

## Familles de commandes MVP

### Projet

- `project.create`
- `project.open`
- `project.save`
- `project.close`
- `project.set_preferences`

### Graphe

- `entity.create`
- `entity.patch`
- `entity.rename`
- `entity.tag`
- `entity.delete`
- `relation.create`
- `relation.delete`

### Esquisse et piece

- `sketch.element.add`
- `sketch.constraint.add`
- `sketch.dimension.set`
- `feature.add`
- `feature.patch`
- `part.parameter.set`
- `part.material.set`

### Assemblage et robotique

- `assembly.occurrence.add`
- `assembly.occurrence.transform`
- `assembly.mate.add`
- `assembly.mate.remove`
- `joint.create`
- `joint.state.set`
- `robot.target.add`
- `robot.sequence.patch`

### Controle et simulation

- `signal.set`
- `controller.patch`
- `scenario.create`
- `scenario.patch`
- `simulation.run.start`
- `simulation.run.cancel`

### Capteurs, perception et calibration

- `sensor.rig.create`
- `sensor.mount`
- `lidar.configure`
- `camera.configure`
- `calibration.capture`
- `calibration.solve`
- `perception.pipeline.create`
- `perception.pipeline.patch`
- `perception.run.start`
- `perception.run.cancel`
- `map.rebuild`

### Integration industrielle

- `ros2.bridge.configure`
- `ros2.binding.patch`
- `opcua.endpoint.create`
- `wireless.endpoint.create`
- `telemetry.stream.create`
- `telemetry.binding.patch`
- `opcua.binding.patch`
- `plc.model.create`
- `plc.binding.patch`
- `robot.controller.bind`
- `integration.discovery.refresh`
- `integration.trace.import`
- `integration.link.simulate`

### Safety

- `safety.zone.create`
- `safety.zone.patch`
- `safety.interlock.create`
- `safety.interlock.patch`
- `safety.validation.run`
- `safety.validation.acknowledge`

### Commissioning et as-built

- `commissioning.session.start`
- `commissioning.capture.import`
- `commissioning.capture.attach`
- `asbuilt.compare.run`
- `commissioning.session.complete`

### Optimisation

- `optimization.study.create`
- `optimization.study.patch`
- `optimization.run.start`
- `optimization.run.cancel`
- `optimization.result.apply`

### Plugins

- `plugin.install`
- `plugin.enable`
- `plugin.disable`
- `plugin.uninstall`

### IA locale

- `ai.session.start`
- `ai.runtime.profile.set`
- `ai.context.refresh`
- `ai.suggestion.request`
- `ai.deep_explain.request`
- `ai.design_critic.request`
- `ai.suggestion.apply`
- `ai.suggestion.reject`

## Familles d'evenements MVP

- `project.saved`
- `entity.created`
- `entity.updated`
- `entity.deleted`
- `relation.created`
- `relation.deleted`
- `sketch.solved`
- `sketch.overconstrained`
- `part.regenerated`
- `assembly.solved`
- `assembly.unsolved`
- `joint.state.changed`
- `signal.changed`
- `scenario.created`
- `simulation.started`
- `simulation.progress`
- `simulation.completed`
- `simulation.failed`
- `calibration.completed`
- `calibration.failed`
- `perception.started`
- `perception.progress`
- `perception.completed`
- `perception.failed`
- `map.updated`
- `integration.connected`
- `integration.disconnected`
- `integration.discovery.updated`
- `integration.link.quality.changed`
- `integration.trace.imported`
- `safety.validation.completed`
- `safety.validation.failed`
- `commissioning.session.started`
- `commissioning.capture.imported`
- `asbuilt.comparison.completed`
- `optimization.started`
- `optimization.progress`
- `optimization.completed`
- `optimization.failed`
- `plugin.installed`
- `plugin.enabled`
- `plugin.disabled`
- `validation.report.created`
- `ai.session.started`
- `ai.runtime.profile.changed`
- `ai.suggestion.created`
- `ai.suggestion.applied`
- `ai.suggestion.rejected`
- `ai.critic.completed`

## Contrat de validation

Toute commande mutante doit verifier:

- existence de la cible,
- compatibilite de type,
- revision de base,
- integrite des references,
- compatibilite des unites,
- droits de mutation du module.

## Contrat de resultat de commande

Une commande reussie retourne:

- `accepted: true`
- `commandId`
- `newRevision` ou `jobId`
- `affectedEntityIds`
- `warnings`

Une commande rejetee retourne:

- `accepted: false`
- `error.code`
- `error.message`
- `error.targetId`
- `error.details`

## Contrat undo/redo

- Toute commande mutante doit declarer si elle est inversible.
- Une commande inversible doit publier les donnees minimales de retour arriere.
- Les commandes longues non deterministes peuvent etre marquees `nonUndoable`.
- `ai.suggestion.apply` est annulable seulement si toutes les sous-commandes le sont.

## Contrat de progression de job

Champs minimaux:

- `jobId`
- `phase`
- `progress`
- `message`
- `estimatedRemainingMs`

Phases recommandees pour `simulation.run`:

- `prepare`
- `load_scene`
- `solve_initial_state`
- `execute`
- `write_results`
- `finalize`

Phases recommandees pour `perception.run`:

- `load_inputs`
- `time_sync`
- `apply_calibration`
- `filter`
- `fuse`
- `map`
- `write_results`
- `finalize`

Phases recommandees pour `optimization.run`:

- `load_study`
- `generate_candidates`
- `evaluate`
- `rank`
- `write_results`
- `finalize`

## Contrat de notification UI

L'UI s'abonne a:

- flux des jobs,
- flux des evenements projet,
- flux de selection,
- flux des suggestions IA.

L'UI ne deduit pas une mutation reussie uniquement a partir d'un clic; elle attend un resultat de commande ou un evenement associe.

## Contrat de suggestion IA

Une suggestion IA valide doit contenir:

- `summary`
- `contextRefs`
- `proposedCommands`
- `confidence`
- `riskLevel`
- `explanation`

Une suggestion sans `proposedCommands` reste informative et ne peut pas etre appliquee.

## Exemple de chaine causale

1. `part.parameter.set`
2. `entity.updated`
3. `part.regenerated`
4. `assembly.solved`
5. `simulation.completed` sur relance explicite uniquement

La simulation n'est pas relancee automatiquement tant qu'aucune regle produit ne l'impose.

## Codes d'erreur minimaux

- `E_REVISION_MISMATCH`
- `E_INVALID_REFERENCE`
- `E_UNIT_MISMATCH`
- `E_SOLVER_FAILURE`
- `E_JOB_RUNNING`
- `E_CALIBRATION_INVALID`
- `E_SENSOR_SYNC_FAILURE`
- `E_PERCEPTION_PIPELINE_FAILURE`
- `E_INTEGRATION_BINDING_INVALID`
- `E_INTEGRATION_SECURITY_PROFILE_INVALID`
- `E_INTEGRATION_LINK_DEGRADED`
- `E_SAFETY_GRAPH_INVALID`
- `E_COMMISSIONING_MISSING_CAPTURE`
- `E_OPTIMIZATION_CONSTRAINT_CONFLICT`
- `E_PLUGIN_PERMISSION_DENIED`
- `E_AI_RESOURCE_EXHAUSTED`
- `E_AI_PROFILE_UNAVAILABLE`
- `E_AI_UNPARSABLE_RESPONSE`
- `E_AI_UNSAFE_ACTION`

## Criteres d'acceptation

- Une commande peut etre journalisee, rejouee et diagnostiquee.
- Un job long peut etre observe sans coupler l'UI a son implementation.
- Une suggestion IA appliquee produit la meme trace que des commandes utilisateur normales.
