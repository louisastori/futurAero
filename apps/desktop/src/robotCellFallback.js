function createEntity(entityType, id, name, detail, data, extra = {}) {
  return {
    id,
    entityType,
    name,
    revision: "rev_seed",
    status: "active",
    detail,
    data,
    ...extra,
  };
}

function pad3(value) {
  return String(value).padStart(3, "0");
}

function buildCellId(index) {
  return `ent_cell_${pad3(index)}`;
}

function robotCellToken(cellId) {
  return cellId.replace(/^ent_cell_/, "");
}

function sceneAssemblyId(cellId) {
  return `ent_asm_cell_${robotCellToken(cellId)}`;
}

function robotId(cellId) {
  return `ent_robot_${robotCellToken(cellId)}`;
}

function equipmentId(cellId, kind) {
  return `ent_${kind}_${robotCellToken(cellId)}`;
}

function sequenceId(cellId) {
  return `ent_seq_${robotCellToken(cellId)}`;
}

function targetId(cellId, targetKey) {
  return `ent_target_${robotCellToken(cellId)}_${targetKey}`;
}

function safetyZoneId(cellId, kind) {
  return `ent_zone_${robotCellToken(cellId)}_${kind}`;
}

function controllerId(cellId) {
  return `ent_ctrl_${robotCellToken(cellId)}`;
}

function signalId(cellId, kind) {
  return `ent_sig_${robotCellToken(cellId)}_${kind}`;
}

function partId(cellId, kind) {
  return `ent_part_${kind}_${robotCellToken(cellId)}`;
}

function occurrenceId(kind) {
  return `occ_${kind}_001`;
}

function distanceBetween(left, right) {
  const dx = right.xMm - left.xMm;
  const dy = right.yMm - left.yMm;
  const dz = right.zMm - left.zMm;
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
}

function computeSequenceValidation(targets) {
  let pathLengthMm = 0;
  let maxSegmentMm = 0;
  let motionTimeMs = 0;
  for (let index = 1; index < targets.length; index += 1) {
    const previous = targets[index - 1];
    const next = targets[index];
    const distance = distanceBetween(previous.pose, next.pose);
    const averageSpeed = Math.max(
      1,
      Math.floor((previous.nominalSpeedMmS + next.nominalSpeedMmS) / 2),
    );
    pathLengthMm += distance;
    maxSegmentMm = Math.max(maxSegmentMm, distance);
    motionTimeMs += Math.ceil((distance / averageSpeed) * 1000);
  }
  const dwellTimeMs = targets.reduce(
    (total, target) => total + target.dwellTimeMs,
    0,
  );
  return {
    targetCount: targets.length,
    pathLengthMm,
    maxSegmentMm,
    estimatedCycleTimeMs: motionTimeMs + dwellTimeMs,
    warningCount:
      Number(maxSegmentMm > 1000) + Number(pathLengthMm > 1800),
  };
}

function targetPreview(targets) {
  return targets.map((target) => target.id).join(" -> ");
}

function sampleTargets() {
  return [
    {
      id: "pick",
      pose: { xMm: 0, yMm: 0, zMm: 120 },
      nominalSpeedMmS: 250,
      dwellTimeMs: 120,
    },
    {
      id: "transfer",
      pose: { xMm: 450, yMm: 60, zMm: 240 },
      nominalSpeedMmS: 320,
      dwellTimeMs: 40,
    },
    {
      id: "place",
      pose: { xMm: 860, yMm: 120, zMm: 140 },
      nominalSpeedMmS: 240,
      dwellTimeMs: 160,
    },
  ];
}

function sampleSafetyZones() {
  return [
    {
      id: "zone_warning_perimeter",
      kind: "warning",
      active: true,
    },
    {
      id: "zone_lidar_protective",
      kind: "lidar_protective",
      active: false,
    },
  ];
}

function sampleSafetyInterlocks() {
  return [
    {
      id: "int_warning_reduce_speed",
      sourceZoneId: "zone_warning_perimeter",
      inhibitedAction: "robot.speed.up",
      requiresManualReset: false,
    },
    {
      id: "int_lidar_stop_move",
      sourceZoneId: "zone_lidar_protective",
      inhibitedAction: "robot.move",
      requiresManualReset: true,
    },
  ];
}

function sampleSignals(cellId) {
  return [
    createEntity(
      "Signal",
      signalId(cellId, "progress_gate"),
      "Progress Gate",
      "sig_progress_gate | 0.62",
      {
        cellId,
        signalId: "sig_progress_gate",
        kind: "scalar",
        initialValue: 0.62,
        currentValue: 0.62,
        tags: ["control", "scalar"],
        parameterSet: {
          unit: "ratio",
          checkpoints: [0.25, 0.62, 1.0],
        },
      },
    ),
    createEntity(
      "Signal",
      signalId(cellId, "cycle_start"),
      "Cycle Start",
      "sig_cycle_start | false",
      {
        cellId,
        signalId: "sig_cycle_start",
        kind: "boolean",
        initialValue: false,
        currentValue: false,
        tags: ["control", "boolean"],
        parameterSet: {},
      },
    ),
    createEntity(
      "Signal",
      signalId(cellId, "safety_clear"),
      "Safety Clear",
      "sig_safety_clear | true",
      {
        cellId,
        signalId: "sig_safety_clear",
        kind: "boolean",
        initialValue: true,
        currentValue: true,
        tags: ["safety", "boolean"],
        parameterSet: {},
      },
    ),
    createEntity(
      "Signal",
      signalId(cellId, "payload_released"),
      "Payload Released",
      "sig_payload_released | false",
      {
        cellId,
        signalId: "sig_payload_released",
        kind: "boolean",
        initialValue: false,
        currentValue: false,
        tags: ["process", "boolean"],
        parameterSet: {},
      },
    ),
  ];
}

function buildTargetEntities(cellId, sequenceModelId, targets) {
  return targets.map((target, index) =>
    createEntity(
      "RobotTarget",
      targetId(cellId, target.id),
      `Target ${target.id}`,
      `#${index + 1} ${target.id} | ${target.pose.xMm}, ${target.pose.yMm}, ${target.pose.zMm} | ${target.nominalSpeedMmS} mm/s`,
      {
        id: targetId(cellId, target.id),
        cellId,
        sequenceId: sequenceModelId,
        targetKey: target.id,
        orderIndex: index + 1,
        pose: target.pose,
        nominalSpeedMmS: target.nominalSpeedMmS,
        dwellTimeMs: target.dwellTimeMs,
        tags: ["robotics", "target", "sequence"],
        parameterSet: {
          orderIndex: index + 1,
          xMm: target.pose.xMm,
          yMm: target.pose.yMm,
          zMm: target.pose.zMm,
          nominalSpeedMmS: target.nominalSpeedMmS,
          dwellTimeMs: target.dwellTimeMs,
        },
      },
    ),
  );
}

function normalizeTargetEntity(entity) {
  if (entity?.entityType !== "RobotTarget") {
    return entity;
  }
  const parameterSet = entity.data?.parameterSet ?? {};
  const orderIndex = Number(parameterSet.orderIndex ?? entity.data?.orderIndex ?? 1);
  const xMm = Number(parameterSet.xMm ?? entity.data?.pose?.xMm ?? 0);
  const yMm = Number(parameterSet.yMm ?? entity.data?.pose?.yMm ?? 0);
  const zMm = Number(parameterSet.zMm ?? entity.data?.pose?.zMm ?? 0);
  const nominalSpeedMmS = Number(
    parameterSet.nominalSpeedMmS ?? entity.data?.nominalSpeedMmS ?? 1,
  );
  const dwellTimeMs = Number(
    parameterSet.dwellTimeMs ?? entity.data?.dwellTimeMs ?? 0,
  );
  return {
    ...entity,
    detail: `#${orderIndex} ${entity.data?.targetKey ?? "target"} | ${xMm}, ${yMm}, ${zMm} | ${nominalSpeedMmS} mm/s`,
    data: {
      ...entity.data,
      orderIndex,
      pose: { xMm, yMm, zMm },
      nominalSpeedMmS,
      dwellTimeMs,
      parameterSet: {
        ...entity.data?.parameterSet,
        orderIndex,
        xMm,
        yMm,
        zMm,
        nominalSpeedMmS,
        dwellTimeMs,
      },
    },
  };
}

function orderedTargetEntities(entities, cellId, sequenceModelId) {
  return entities
    .filter(
      (entity) =>
        entity.entityType === "RobotTarget" &&
        entity.data?.cellId === cellId &&
        entity.data?.sequenceId === sequenceModelId,
    )
    .map(normalizeTargetEntity)
    .sort(
      (left, right) =>
        (left.data?.parameterSet?.orderIndex ?? 0) -
        (right.data?.parameterSet?.orderIndex ?? 0),
    );
}

function rawTargetsFromEntities(targetEntities) {
  return targetEntities.map((entity) => ({
    id: entity.data.targetKey,
    pose: entity.data.pose,
    nominalSpeedMmS: entity.data.nominalSpeedMmS,
    dwellTimeMs: entity.data.dwellTimeMs,
  }));
}

export function syncFallbackRobotCellTargets(entities, targetEntityId) {
  const nextEntities = structuredClone(entities);
  const targetIndex = nextEntities.findIndex((entity) => entity.id === targetEntityId);
  if (targetIndex === -1 || nextEntities[targetIndex]?.entityType !== "RobotTarget") {
    return nextEntities;
  }

  nextEntities[targetIndex] = normalizeTargetEntity(nextEntities[targetIndex]);
  const targetEntity = nextEntities[targetIndex];
  const cellId = targetEntity.data.cellId;
  const sequenceModelId = targetEntity.data.sequenceId;
  const targetEntities = orderedTargetEntities(nextEntities, cellId, sequenceModelId);
  const rawTargets = rawTargetsFromEntities(targetEntities);
  const validation = computeSequenceValidation(rawTargets);
  const preview = targetPreview(rawTargets);
  const orderedTargetIds = targetEntities.map((entity) => entity.id);

  const sequenceIndex = nextEntities.findIndex((entity) => entity.id === sequenceModelId);
  if (sequenceIndex !== -1) {
    nextEntities[sequenceIndex] = {
      ...nextEntities[sequenceIndex],
      detail: `${validation.targetCount} pts | ${validation.estimatedCycleTimeMs} ms | ${preview}`,
      data: {
        ...nextEntities[sequenceIndex].data,
        targetIds: orderedTargetIds,
        targets: rawTargets,
        pathLengthMm: validation.pathLengthMm,
        estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
        targetCount: validation.targetCount,
        targetPreview: preview,
      },
    };
  }

  const cellIndex = nextEntities.findIndex((entity) => entity.id === cellId);
  if (cellIndex !== -1) {
    const currentCell = nextEntities[cellIndex];
    nextEntities[cellIndex] = {
      ...currentCell,
      detail: `${validation.targetCount} pts | ${currentCell.robotCellSummary?.equipmentCount ?? 0} equip | ${currentCell.robotCellSummary?.signalCount ?? 0} sig | ${currentCell.data?.sceneAssemblyId ?? "scene"} | ${validation.estimatedCycleTimeMs} ms | ${preview}`,
      data: {
        ...currentCell.data,
        targetIds: orderedTargetIds,
        targets: rawTargets,
        targetPreview: preview,
        parameterSet: {
          ...currentCell.data?.parameterSet,
          targetCount: validation.targetCount,
          estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
        },
        sequenceValidation: {
          ...currentCell.data?.sequenceValidation,
          targetCount: validation.targetCount,
          pathLengthMm: validation.pathLengthMm,
          maxSegmentMm: validation.maxSegmentMm,
          estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
          warningCount: validation.warningCount,
        },
      },
      robotCellSummary: {
        ...currentCell.robotCellSummary,
        targetCount: validation.targetCount,
        pathLengthMm: validation.pathLengthMm,
        maxSegmentMm: validation.maxSegmentMm,
        estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
        warningCount: validation.warningCount,
        targetPreview: preview,
      },
    };
  }

  return nextEntities;
}

export function buildFallbackRobotCellBundle(index) {
  const cellId = buildCellId(index);
  const token = robotCellToken(cellId);
  const asmId = sceneAssemblyId(cellId);
  const robotModelId = robotId(cellId);
  const conveyorId = equipmentId(cellId, "conveyor");
  const fixtureId = equipmentId(cellId, "fixture");
  const toolId = equipmentId(cellId, "tool");
  const warningZoneId = safetyZoneId(cellId, "warning");
  const protectiveZoneId = safetyZoneId(cellId, "protective");
  const sequenceModelId = sequenceId(cellId);
  const controllerModelId = controllerId(cellId);
  const signals = sampleSignals(cellId);
  const targets = sampleTargets();
  const validation = computeSequenceValidation(targets);
  const preview = targetPreview(targets);
  const targetEntities = buildTargetEntities(cellId, sequenceModelId, targets);
  const safetyZones = sampleSafetyZones();
  const safetyInterlocks = sampleSafetyInterlocks();
  const structureSummary = {
    robotCount: 1,
    equipmentCount: 3,
    safetyZoneCount: 2,
    sequenceCount: 1,
    controllerCount: 1,
  };
  const robotCellSummary = {
    sceneAssemblyId: asmId,
    targetPreview: preview,
    targetCount: validation.targetCount,
    pathLengthMm: validation.pathLengthMm,
    maxSegmentMm: validation.maxSegmentMm,
    estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
    equipmentCount: 3,
    sequenceCount: 1,
    safetyZoneCount: 2,
    signalCount: 4,
    controllerTransitionCount: 3,
    warningCount: validation.warningCount,
  };

  return [
    createEntity(
      "Assembly",
      asmId,
      `RobotCell-${token} / Scene`,
      "solved | 4 occ | 3 mates | 0 ddl",
      {
        tags: ["robotics", "scene", "mvp"],
        parameterSet: {
          occurrenceCount: 4,
          mateCount: 3,
          jointCount: 0,
        },
        occurrences: [
          {
            id: occurrenceId("robot"),
            definitionEntityId: partId(cellId, "robot"),
            transform: { xMm: 0, yMm: 0, zMm: 0, yawDeg: 0 },
          },
          {
            id: occurrenceId("conveyor"),
            definitionEntityId: partId(cellId, "conveyor"),
            transform: { xMm: 620, yMm: 120, zMm: 0, yawDeg: 0 },
          },
          {
            id: occurrenceId("fixture"),
            definitionEntityId: partId(cellId, "fixture"),
            transform: { xMm: 980, yMm: -140, zMm: 0, yawDeg: 0 },
          },
          {
            id: occurrenceId("tool"),
            definitionEntityId: partId(cellId, "tool"),
            transform: { xMm: 120, yMm: 0, zMm: 260, yawDeg: 0 },
          },
        ],
        mateConstraints: [
          {
            id: "mate_robot_conveyor",
            leftOccurrenceId: occurrenceId("robot"),
            rightOccurrenceId: occurrenceId("conveyor"),
            type: "offset",
            distanceMm: 620,
          },
          {
            id: "mate_conveyor_fixture",
            leftOccurrenceId: occurrenceId("conveyor"),
            rightOccurrenceId: occurrenceId("fixture"),
            type: "offset",
            distanceMm: 360,
          },
          {
            id: "mate_robot_tool",
            leftOccurrenceId: occurrenceId("robot"),
            rightOccurrenceId: occurrenceId("tool"),
            type: "offset",
            distanceMm: 120,
          },
        ],
      },
    ),
    createEntity(
      "Part",
      partId(cellId, "robot"),
      `${token} / RobotBase`,
      "520 x 640 x 420 mm | 377395.2 g",
      {
        tags: ["robotics", "scene", "seed"],
        parameterSet: {
          widthMm: 520,
          heightMm: 640,
          depthMm: 420,
        },
      },
      {
        partGeometry: {
          state: "well_constrained",
          widthMm: 520,
          heightMm: 640,
          depthMm: 420,
          pointCount: 4,
          perimeterMm: 2320,
          areaMm2: 332800,
          volumeMm3: 139776000,
          estimatedMassGrams: 377395.2,
          materialName: "Aluminum 6061",
        },
      },
    ),
    createEntity(
      "Part",
      partId(cellId, "conveyor"),
      `${token} / Conveyor`,
      "850 x 220 x 600 mm | 302940.0 g",
      {
        tags: ["robotics", "scene", "seed"],
        parameterSet: {
          widthMm: 850,
          heightMm: 220,
          depthMm: 600,
        },
      },
      {
        partGeometry: {
          state: "well_constrained",
          widthMm: 850,
          heightMm: 220,
          depthMm: 600,
          pointCount: 4,
          perimeterMm: 2140,
          areaMm2: 187000,
          volumeMm3: 112200000,
          estimatedMassGrams: 302940.0,
          materialName: "Aluminum 6061",
        },
      },
    ),
    createEntity(
      "Part",
      partId(cellId, "fixture"),
      `${token} / Workstation`,
      "640 x 180 x 480 mm | 149299.2 g",
      {
        tags: ["robotics", "scene", "seed"],
        parameterSet: {
          widthMm: 640,
          heightMm: 180,
          depthMm: 480,
        },
      },
      {
        partGeometry: {
          state: "well_constrained",
          widthMm: 640,
          heightMm: 180,
          depthMm: 480,
          pointCount: 4,
          perimeterMm: 1640,
          areaMm2: 115200,
          volumeMm3: 55296000,
          estimatedMassGrams: 149299.2,
          materialName: "Aluminum 6061",
        },
      },
    ),
    createEntity(
      "Part",
      partId(cellId, "tool"),
      `${token} / Gripper`,
      "110 x 80 x 140 mm | 3326.4 g",
      {
        tags: ["robotics", "scene", "seed"],
        parameterSet: {
          widthMm: 110,
          heightMm: 80,
          depthMm: 140,
        },
      },
      {
        partGeometry: {
          state: "well_constrained",
          widthMm: 110,
          heightMm: 80,
          depthMm: 140,
          pointCount: 4,
          perimeterMm: 380,
          areaMm2: 8800,
          volumeMm3: 1232000,
          estimatedMassGrams: 3326.4,
          materialName: "Aluminum 6061",
        },
      },
    ),
    createEntity(
      "RobotModel",
      robotModelId,
      `RobotCell-${token} / Robot`,
      "4 links | 8 kg payload",
      {
        id: robotModelId,
        cellId,
        kinematicChain: ["base", "shoulder", "wrist", "tool"],
        jointIds: ["joint_axis_001"],
        toolMountRef: {
          equipmentId: toolId,
          role: "tool",
        },
        workspaceBounds: {
          reachRadiusMm: 1450,
          verticalSpanMm: 1900,
        },
        payloadLimits: {
          nominalKg: 8,
          maxKg: 12,
        },
        calibrationState: "seeded",
      },
    ),
    createEntity(
      "EquipmentModel",
      conveyorId,
      `RobotCell-${token} / Conveyor`,
      "conveyor | occ_conveyor_001",
      {
        id: conveyorId,
        cellId,
        equipmentType: "conveyor",
        assemblyOccurrenceId: occurrenceId("conveyor"),
        parameterSet: {
          widthMm: 850,
          heightMm: 220,
          depthMm: 600,
          nominalSpeedMmS: 320,
        },
        ioPortIds: [signalId(cellId, "cycle_start")],
      },
    ),
    createEntity(
      "EquipmentModel",
      fixtureId,
      `RobotCell-${token} / Workstation`,
      "workstation | occ_fixture_001",
      {
        id: fixtureId,
        cellId,
        equipmentType: "workstation",
        assemblyOccurrenceId: occurrenceId("fixture"),
        parameterSet: {
          widthMm: 640,
          heightMm: 180,
          depthMm: 480,
          nominalSpeedMmS: null,
        },
        ioPortIds: [signalId(cellId, "progress_gate")],
      },
    ),
    createEntity(
      "EquipmentModel",
      toolId,
      `RobotCell-${token} / Gripper`,
      "gripper | occ_tool_001",
      {
        id: toolId,
        cellId,
        equipmentType: "gripper",
        assemblyOccurrenceId: occurrenceId("tool"),
        parameterSet: {
          widthMm: 110,
          heightMm: 80,
          depthMm: 140,
          nominalSpeedMmS: null,
        },
        ioPortIds: [signalId(cellId, "payload_released")],
      },
    ),
    createEntity(
      "SafetyZoneModel",
      warningZoneId,
      `RobotCell-${token} / zone_warning_perimeter`,
      "warning | active",
      {
        id: warningZoneId,
        cellId,
        zoneId: "zone_warning_perimeter",
        zoneKind: "warning",
        active: true,
        interlockIds: ["int_warning_reduce_speed"],
      },
    ),
    createEntity(
      "SafetyZoneModel",
      protectiveZoneId,
      `RobotCell-${token} / zone_lidar_protective`,
      "lidar_protective | inactive",
      {
        id: protectiveZoneId,
        cellId,
        zoneId: "zone_lidar_protective",
        zoneKind: "lidar_protective",
        active: false,
        interlockIds: ["int_lidar_stop_move"],
      },
    ),
    createEntity(
      "RobotSequence",
      sequenceModelId,
      `RobotCell-${token} / Sequence`,
      `${validation.targetCount} pts | ${validation.estimatedCycleTimeMs} ms | ${preview}`,
      {
        id: sequenceModelId,
        cellId,
        robotId: robotModelId,
        targetIds: targetEntities.map((entity) => entity.id),
        targets,
        pathLengthMm: validation.pathLengthMm,
        estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
        targetCount: validation.targetCount,
        targetPreview: preview,
        structureSummary,
      },
    ),
    ...targetEntities,
    ...signals,
    createEntity(
      "ControllerModel",
      controllerModelId,
      `RobotCell-${token} / Controller`,
      "4 states | 3 transitions",
      {
        cellId,
        tags: ["control", "state_machine"],
        stateMachine: {
          id: `ctrl_${token}`,
          name: `Controller ${token}`,
          initialStateId: "idle",
          states: [
            { id: "idle", name: "Idle", terminal: false },
            { id: "transfer", name: "Transfer", terminal: false },
            { id: "place", name: "Place", terminal: false },
            { id: "done", name: "Done", terminal: true },
          ],
          transitions: [
            {
              id: "tr_start_cycle",
              fromStateId: "idle",
              toStateId: "transfer",
              conditions: [
                {
                  signalId: "sig_cycle_start",
                  comparator: "equal",
                  expectedValue: true,
                },
                {
                  signalId: "sig_safety_clear",
                  comparator: "equal",
                  expectedValue: true,
                },
              ],
              assignments: [],
              description: "cycle_start_confirmed",
            },
            {
              id: "tr_reach_place",
              fromStateId: "transfer",
              toStateId: "place",
              conditions: [
                {
                  signalId: "sig_progress_gate",
                  comparator: "greater_than_or_equal",
                  expectedValue: 0.55,
                },
              ],
              assignments: [],
              description: "progress_gate_reached",
            },
            {
              id: "tr_finish_cycle",
              fromStateId: "place",
              toStateId: "done",
              conditions: [
                {
                  signalId: "sig_progress_gate",
                  comparator: "greater_than_or_equal",
                  expectedValue: 0.95,
                },
              ],
              assignments: [
                {
                  signalId: "sig_payload_released",
                  value: true,
                },
              ],
              description: "placement_complete",
            },
          ],
        },
        parameterSet: {
          stateCount: 4,
          transitionCount: 3,
        },
      },
    ),
    createEntity(
      "RobotCell",
      cellId,
      `RobotCell-${token}`,
      `${validation.targetCount} pts | 3 equip | 4 sig | ${asmId} | ${validation.estimatedCycleTimeMs} ms | ${preview}`,
      {
        controller: {
          robotModel: "FAERO-X90",
          tcpPayloadKg: 8,
        },
        tags: ["robotics", "simulation", "mvp"],
        parameterSet: {
          tcpPayloadKg: 8,
          estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
          targetCount: validation.targetCount,
          equipmentCount: 3,
          sequenceCount: 1,
        },
        id: cellId,
        sceneAssemblyId: asmId,
        robotIds: [robotModelId],
        equipmentIds: [conveyorId, fixtureId, toolId],
        safetyZoneIds: [warningZoneId, protectiveZoneId],
        sequenceIds: [sequenceModelId],
        controllerModelIds: [controllerModelId],
        targetIds: targetEntities.map((entity) => entity.id),
        targetPreview: preview,
        targets,
        sequenceValidation: {
          targetCount: validation.targetCount,
          pathLengthMm: validation.pathLengthMm,
          maxSegmentMm: validation.maxSegmentMm,
          estimatedCycleTimeMs: validation.estimatedCycleTimeMs,
          warningCount: validation.warningCount,
          sequenceEntityId: sequenceModelId,
        },
        control: {
          signalCount: 4,
          controllerTransitionCount: 3,
          controllerId: controllerModelId,
          signalIds: [
            signalId(cellId, "cycle_start"),
            signalId(cellId, "progress_gate"),
            signalId(cellId, "safety_clear"),
            signalId(cellId, "payload_released"),
          ],
          contactPairs: [
            {
              id: `pair_${token}_tool_fixture`,
              leftEntityId: toolId,
              rightEntityId: fixtureId,
              baseClearanceMm: 0.42,
            },
            {
              id: `pair_${token}_tool_conveyor`,
              leftEntityId: toolId,
              rightEntityId: conveyorId,
              baseClearanceMm: 0.36,
            },
          ],
        },
        safety: {
          zoneCount: 2,
          interlockCount: 2,
          zones: safetyZones,
          interlocks: safetyInterlocks,
        },
        structureSummary,
      },
      {
        robotCellSummary,
      },
    ),
  ];
}
