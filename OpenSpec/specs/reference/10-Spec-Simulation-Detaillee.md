# Spec Simulation Detaillee

## Objectif

Preciser le comportement du moteur de simulation du MVP afin d'obtenir des scenarios rejouables, explicables et utiles pour la preparation terrain.

## Perimetre MVP

- corps rigides,
- collisions discretes,
- joints et limites,
- gravite,
- friction simple,
- signaux, capteurs, actionneurs,
- LiDAR et sorties perception synthese,
- logique de sequence,
- rapports de simulation.

## Niveaux de fidelite

### `S0`

- verification spatiale,
- animation de sequences,
- collisions grossieres,
- pas de dynamique detaillee requise.

### `S1`

- dynamique rigide,
- inertie,
- vitesses et accelerations,
- latence capteurs,
- signaux et temporisations.

### `S2`

- bruit capteurs,
- tolerances geometriques simplifiees,
- derives et decalages parametrables,
- marges de securite terrain.

## Convention de temps

- Horloge simulation basee sur un pas fixe.
- Unite de calcul: seconde.
- Pas MVP recommande: `0.005s` par defaut.
- Les jobs peuvent sous-diviser le pas pour la stabilite physique sans changer la timeline externe.

## Entrees d'un scenario

- scene source,
- etat initial des joints,
- etat initial des signaux,
- configuration solveur,
- overrides materiaux,
- profils de calibration capteurs,
- pipelines perception actives,
- niveau de fidelite,
- seed aleatoire.

## Pipeline de simulation

1. Charger la scene resolue.
2. Materialiser les corps, joints, capteurs et signaux.
3. Verifier les unites et references.
4. Resoudre l'etat initial.
5. Executer la boucle de simulation.
6. Echantillonner metriques et evenements.
7. Produire resume et artefacts.

## Etat interne minimal

### RigidBodyState

- `entityId`
- `position`
- `rotation`
- `linearVelocity`
- `angularVelocity`
- `sleeping`

### JointState

- `jointId`
- `position`
- `velocity`
- `effort`
- `atLimit`

### SignalState

- `signalId`
- `value`
- `timestamp`
- `quality`

### SensorSample

- `sensorId`
- `rawValue`
- `filteredValue`
- `latencyMs`
- `noiseApplied`

### LidarScanSample

- `sensorId`
- `timestamp`
- `pointCount`
- `frameId`
- `assetRef`
- `coverageMetrics`

## Boucle de simulation

A chaque tick:

1. lire les signaux d'entree,
2. evaluer la logique de commande,
3. mettre a jour les commandes actionneurs,
4. integrer la physique,
5. resoudre contacts et limites,
6. echantillonner les capteurs,
7. produire si besoin les sorties perception du tick ou de la fenetre,
8. publier les evenements du tick,
9. verifier les conditions d'arret ou d'erreur.

## Collisions

MVP:

- broad phase a base d'enveloppes,
- narrow phase sur colliders derives,
- detection discrete,
- classification par severite.

Chaque collision doit enregistrer:

- `time`
- `entityA`
- `entityB`
- `contactPoint`
- `normal`
- `penetrationDepth`
- `severity`

## Joints et actionneurs

Types MVP:

- `fixed`
- `revolute`
- `prismatic`

Un actionneur peut:

- imposer une consigne de position,
- imposer une vitesse cible,
- etre borne par limites et rampes simples.

## Capteurs MVP

Types initiaux:

- presence binaire,
- fin de course,
- position scalaire,
- detecteur de zone.

Parametres communs:

- frequence d'echantillonnage,
- latence,
- bruit,
- seuils.

## LiDAR et perception synthetique

Le MVP etendu doit supporter:

- LiDAR 2D de securite,
- LiDAR 3D de reconstruction,
- cameras et profondeur en support de fusion,
- IMU pour synchronisation et localisation.

Pour le LiDAR, le moteur doit pouvoir parametrer:

- champ de vue,
- portee mini/maxi,
- frequence de balayage,
- resolution angulaire,
- bruit et pertes de retour,
- occultations et angle d'incidence.

## Sorties perception du run

- scans LiDAR horodates,
- nuages de points rejouables,
- cartes d'occupation derivees,
- ecarts scene observee / scene attendue,
- objets ou obstacles detectes.

## Metrices de sortie

- `cycle_time_ms`
- `collision_count`
- `max_joint_error`
- `max_speed`
- `max_acceleration`
- `sensor_missed_events`
- `safety_margin_min`
- `lidar_coverage_ratio`
- `mapping_deviation_mm`
- `false_obstacle_count`

## Conditions d'echec

- instabilite solveur,
- reference invalide,
- joint hors limite dure,
- collision bloquante,
- sequence impossible,
- watchdog temps depasse.

## Artefacts de sortie

- `summary.json`
- `timeline.jsonl`
- `metrics.json`
- `contacts.jsonl`
- `signals.jsonl`
- `lidar_scans.jsonl`
- `pointclouds/`
- `occupancy_maps/`

## Determinisme

Le moteur est considere deterministe si, a scene, config, version moteur et seed egales:

- les metriques principales sont identiques,
- le nombre d'evenements critiques est identique,
- l'ordre des collisions critiques est stable.

## Validation produit

Une simulation doit separer:

- observation brute,
- indicateurs derives,
- interpretation,
- recommandation eventuelle.

L'interpretation et la recommandation peuvent etre assistees par l'IA, mais l'observation brute reste issue du moteur.

## Exemple de resume de run

```json
{
  "runId": "run_01JFA_SIM_001",
  "scenarioId": "ent_01JFA_SCN_001",
  "status": "completed",
  "durationMs": 30000,
  "engineVersion": "0.1.0",
  "seed": 42,
  "metrics": {
    "cycle_time_ms": 27480,
    "collision_count": 0,
    "safety_margin_min": 0.031
  },
  "anomalies": []
}
```

## Criteres d'acceptation

- Un scenario simple de cellule pick-and-place est rejouable.
- Le moteur produit une timeline exploitable par l'UI et l'IA.
- Les collisions, limites et erreurs de sequence sont localisables dans le temps.
- Un LiDAR virtuel peut produire un jeu de donnees rejouable et comparable a la scene.
